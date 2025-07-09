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
use futures_util::stream::{self, StreamExt};
use std::sync::atomic::{AtomicI64, Ordering};

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
    pub finished_count: Arc<AtomicI64>,
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
            finished_count: Arc::new(AtomicI64::new(0)),
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
        let detail_urls: Vec<String> = match fetcher.fetch_and_parse_list(&list_url, limit).await {
            Ok(urls) => {
                self.total_items = urls.len() as i64;
                self.update_task_status(CrawlerTaskStatus::Running, 0.0, None).await; // Update total_items
                urls.into_iter().collect()
            }
            Err(e) => {
                let error_msg = format!("Failed to fetch list page: {}", e);
                self.update_task_status(CrawlerTaskStatus::Failed, 0.0, Some(error_msg))
                    .await;
                return;
            }
        };

        let max_concurrent = 10;
        let mut processed = 0;
        let mut anime_data_buffer = Vec::new();
        let mut subtitle_group_buffer = Vec::new();
        let mut resource_buffer = Vec::new();

        let mut stream = stream::iter(detail_urls.into_iter().enumerate())
            .map(|(i, url)| {
                let fetcher = &fetcher;
                async move {
                    match fetcher.fetch_and_parse_detail(&url).await {
                        Ok(anime_data) => Some((i, anime_data)),
                        Err(e) => {
                            tracing::warn!("Failed to parse detail page {}: {}", url, e);
                            None
                        }
                    }
                }
            })
            .buffer_unordered(max_concurrent);

        while let Some(result) = stream.next().await {
            if self.is_task_cancelled().await {
                self.update_task_status(CrawlerTaskStatus::Cancelled, (processed as f64) / (self.total_items as f64), Some("Task was cancelled".to_string())).await;
                return;
            }
            if let Some((_, anime_data)) = result {
                // 合并到本地缓冲区
                if let Some(anime) = anime_data.anime {
                    if self.anime_ids.insert(anime.mikan_id) {
                        anime_data_buffer.push(anime);
                    }
                }
                for group in anime_data.subtitle_groups {
                    if let Some(id) = group.id {
                        if self.subtitle_group_ids.insert(id) {
                            subtitle_group_buffer.push(group);
                        }
                    }
                }
                for res in anime_data.resources {
                    if let Some(hash) = &res.magnet_hash {
                        if self.resource_hashes.insert(hash.clone()) {
                            resource_buffer.push(res);
                        }
                    }
                }
                // 每完成一部动画，主线程自增finished_count并更新进度
                let finished = self.finished_count.fetch_add(1, Ordering::SeqCst) + 1;
                let percent = finished as f64 / self.total_items as f64;
                self.update_task_status(CrawlerTaskStatus::Running, percent, None).await;
            }
            processed += 1;
            self.processed_items = processed;
            // 每10条写入一次
            if processed % 10 == 0 || processed as i64 == self.total_items {
                // 将缓冲区数据写入self
                self.anime_buffer.append(&mut anime_data_buffer);
                self.subtitle_group_buffer.append(&mut subtitle_group_buffer);
                self.resource_buffer.append(&mut resource_buffer);
                if let Err(e) = self.flush_buffers().await {
                    let error_msg = format!("Failed to save data to database: {}", e);
                    self.update_task_status(CrawlerTaskStatus::Failed, processed as f64 / self.total_items as f64, Some(error_msg)).await;
                    return;
                }
            }
        }

        self.update_task_status(CrawlerTaskStatus::Completed, 1.0, None)
            .await;
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
        let mut tx = self.pool.begin().await?;
        
        let anime_repo = AnimeRepository::new(&self.pool);
        anime_repo.insert_many_animes(&mut tx, &self.anime_buffer).await?;

        let subtitle_repo = SubtitleGroupRepository::new(&self.pool);
        subtitle_repo.insert_many_subtitle_groups(&mut tx, &self.subtitle_group_buffer).await?;

        let resource_repo = ResourceRepository::new(&self.pool);
        resource_repo.insert_many_resources(&mut tx, &self.resource_buffer).await?;

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

