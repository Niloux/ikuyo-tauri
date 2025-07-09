use crate::core::mikan_fetcher::{AnimeData, MikanFetcher};
use crate::models::{Anime, CrawlerTaskStatus, Resource, SubtitleGroup};
use crate::repositories::{
    anime::AnimeRepository, base::Repository, crawler_task::CrawlerTaskRepository,
    resource::ResourceRepository, subtitle_group::SubtitleGroupRepository,
};
use crate::types::crawler::{CrawlerMode, CrawlerTaskCreate};
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::sync::Arc;

pub struct CrawlerService {
    pub pool: Arc<SqlitePool>,
    pub task_id: i64,
    pub anime_buffer: Vec<Anime>,
    pub subtitle_group_buffer: Vec<SubtitleGroup>,
    pub resource_buffer: Vec<Resource>,
    pub anime_ids: HashSet<i64>,
    pub subtitle_group_ids: HashSet<i64>,
    pub resource_hashes: HashSet<String>,
    pub processed_items: i64,
    pub total_items: i64,
}

impl CrawlerService {
    pub fn new(pool: Arc<SqlitePool>, task_id: i64) -> Self {
        Self {
            pool,
            task_id,
            anime_buffer: Vec::new(),
            subtitle_group_buffer: Vec::new(),
            resource_buffer: Vec::new(),
            anime_ids: HashSet::new(),
            subtitle_group_ids: HashSet::new(),
            resource_hashes: HashSet::new(),
            processed_items: 0,
            total_items: 0,
        }
    }

    pub async fn run(&mut self) {
        let repo = CrawlerTaskRepository::new(&self.pool);
        let task = match repo.get_by_id(self.task_id).await {
            Ok(Some(task)) => task,
            _ => {
                tracing::error!("Task {} not found, aborting run.", self.task_id);
                return;
            }
        };

        let params: CrawlerTaskCreate = task
            .parameters
            .as_ref()
            .and_then(|p| serde_json::from_str(p).ok())
            .unwrap_or_default();

        self.update_task_status(CrawlerTaskStatus::Running, 0.0, None)
            .await;

        let base_url = "https://mikanani.me"; // TODO: Get from config
        let proxy = Some("http://127.0.0.1:7890"); // TODO: Get from config

        let list_url = match params.mode {
            CrawlerMode::Year => format!(
                "{}/Home/Bangumi?year={}",
                base_url,
                params.year.unwrap_or_default()
            ),
            CrawlerMode::Season => format!(
                "{}/Home/Bangumi?year={}&season={}",
                base_url,
                params.year.unwrap_or_default(),
                params.season.as_ref().map_or("", |s| s.as_str())
            ),
            CrawlerMode::Homepage => format!("{}/Home", base_url),
        };
        
        let fetcher = MikanFetcher::new(base_url, proxy);
        let limit = params.limit.map(|v| v as i64);
        let detail_urls = match fetcher.fetch_and_parse_list(&list_url, limit).await {
            Ok(urls) => {
                self.total_items = urls.len() as i64;
                self.update_task_status(CrawlerTaskStatus::Running, 0.0, None).await; // Update total_items
                urls
            }
            Err(e) => {
                let error_msg = format!("Failed to fetch list page: {}", e);
                self.update_task_status(CrawlerTaskStatus::Failed, 0.0, Some(error_msg))
                    .await;
                return;
            }
        };

        for (i, url) in detail_urls.iter().enumerate() {
            if self.is_task_cancelled().await {
                self.update_task_status(CrawlerTaskStatus::Cancelled, (i as f64) / (self.total_items as f64), Some("Task was cancelled".to_string())).await;
                return;
            }

            match fetcher.fetch_and_parse_detail(url).await {
                Ok(anime_data) => self.process_anime_data(anime_data),
                Err(e) => {
                    tracing::warn!("Failed to parse detail page {}: {}", url, e);
                    continue; // Skip this item and continue with the next
                }
            }
            self.processed_items += 1;

            if i % 10 == 0 || i + 1 == detail_urls.len() {
                let percent = (i + 1) as f64 / (self.total_items as f64);
                self.update_task_status(CrawlerTaskStatus::Running, percent, None)
                    .await;
            }
        }

        if let Err(e) = self.flush_buffers().await {
            let error_msg = format!("Failed to save data to database: {}", e);
            self.update_task_status(CrawlerTaskStatus::Failed, 1.0, Some(error_msg))
                .await;
        } else {
            self.update_task_status(CrawlerTaskStatus::Completed, 1.0, None)
                .await;
        }
    }

    fn process_anime_data(&mut self, anime_data: AnimeData) {
        if let Some(anime) = anime_data.anime {
            if self.anime_ids.insert(anime.mikan_id) {
                self.anime_buffer.push(anime);
            }
        }
        for group in anime_data.subtitle_groups {
            if let Some(id) = group.id {
                if self.subtitle_group_ids.insert(id) {
                    self.subtitle_group_buffer.push(group);
                }
            }
        }
        for res in anime_data.resources {
            if let Some(hash) = &res.magnet_hash {
                if self.resource_hashes.insert(hash.clone()) {
                    self.resource_buffer.push(res);
                }
            }
        }
    }

    async fn flush_buffers(&mut self) -> anyhow::Result<()> {
        let tx = self.pool.begin().await?;
        
        let anime_repo = AnimeRepository::new(&self.pool);
        for anime in &self.anime_buffer {
            anime_repo.create(anime).await?;
        }

        let subtitle_repo = SubtitleGroupRepository::new(&self.pool);
        for group in &self.subtitle_group_buffer {
            subtitle_repo.create(group).await?;
        }

        let resource_repo = ResourceRepository::new(&self.pool);
        for res in &self.resource_buffer {
            resource_repo.create(res).await?;
        }

        tx.commit().await?;

        self.anime_buffer.clear();
        self.subtitle_group_buffer.clear();
        self.resource_buffer.clear();

        Ok(())
    }

    async fn is_task_cancelled(&self) -> bool {
        let repo = CrawlerTaskRepository::new(&self.pool);
        if let Ok(Some(task)) = repo.get_by_id(self.task_id).await {
            return task.status == CrawlerTaskStatus::Cancelled;
        }
        true // If task not found, treat as cancelled
    }

    async fn update_task_status(
        &self,
        status: CrawlerTaskStatus,
        percentage: f64,
        error_message: Option<String>,
    ) {
        let repo = CrawlerTaskRepository::new(&self.pool);
        if let Ok(Some(mut task)) = repo.get_by_id(self.task_id).await {
            task.status = status;
            task.percentage = Some(percentage * 100.0);
            task.processed_items = Some(self.processed_items);
            task.total_items = Some(self.total_items);
            task.error_message = error_message;

            if task.status == CrawlerTaskStatus::Completed
                || task.status == CrawlerTaskStatus::Failed
                || task.status == CrawlerTaskStatus::Cancelled
            {
                task.completed_at = Some(chrono::Utc::now().timestamp_millis());
            }

            if let Err(e) = repo.update(&task).await {
                tracing::error!("Failed to update task progress: {}", e);
            }
        }
    }
}

