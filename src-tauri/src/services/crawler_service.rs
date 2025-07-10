use crate::core::mikan_fetcher::MikanFetcher;
use crate::models::{Anime, CrawlerTaskStatus, Resource, SubtitleGroup};
use crate::repositories::{
    anime::AnimeRepository, base::Repository, crawler_task::CrawlerTaskRepository,
    resource::ResourceRepository, subtitle_group::SubtitleGroupRepository,
};
use crate::types::crawler::{CrawlerMode, CrawlerTaskCreate, SeasonName};
use futures_util::stream::{self, StreamExt};
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::sync::atomic::{AtomicI64, Ordering};
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
        let fetcher = MikanFetcher::new(base_url, proxy);
        let limit = params.limit.map(|v| v as i64);

        // 统一收集所有目标urls
        let mut all_detail_urls = Vec::new();
        let mut total_items = 0;
        let mut error_message = None;
        let mut failed = false;

        match params.mode {
            CrawlerMode::Year => {
                let year = params.year.unwrap_or_default();
                let seasons = [
                    SeasonName::Spring,
                    SeasonName::Summer,
                    SeasonName::Autumn,
                    SeasonName::Winter,
                ];
                for season in seasons.iter() {
                    let list_url = format!(
                        "{}/Home/BangumiCoverFlowByDayOfWeek?year={}&seasonStr={}",
                        base_url,
                        year,
                        season.as_str()
                    );
                    match fetcher.fetch_and_parse_list(&list_url, limit).await {
                        Ok(urls) => {
                            total_items += urls.len();
                            all_detail_urls.extend(urls);
                        }
                        Err(e) => {
                            error_message = Some(format!(
                                "Failed to fetch list page for {}: {}",
                                season.as_str(),
                                e
                            ));
                            failed = true;
                            break;
                        }
                    }
                }
            }
            CrawlerMode::Season => {
                let year = params.year.unwrap_or_default();
                let season = params.season.as_ref().map_or("春", |s| s.as_str());
                let list_url = format!(
                    "{}/Home/BangumiCoverFlowByDayOfWeek?year={}&seasonStr={}",
                    base_url, year, season
                );
                match fetcher.fetch_and_parse_list(&list_url, limit).await {
                    Ok(urls) => {
                        total_items = urls.len();
                        all_detail_urls = urls;
                    }
                    Err(e) => {
                        error_message = Some(format!("Failed to fetch list page: {}", e));
                        failed = true;
                    }
                }
            }
            CrawlerMode::Homepage => {
                let list_url = format!("{}/Home", base_url);
                match fetcher.fetch_and_parse_list(&list_url, limit).await {
                    Ok(urls) => {
                        total_items = urls.len();
                        all_detail_urls = urls;
                    }
                    Err(e) => {
                        error_message = Some(format!("Failed to fetch homepage list: {}", e));
                        failed = true;
                    }
                }
            }
        }

        self.total_items = total_items as i64;
        self.processed_items = 0;
        self.update_task_status(
            if failed {
                CrawlerTaskStatus::Failed
            } else {
                CrawlerTaskStatus::Running
            },
            0.0,
            error_message.clone(),
        )
        .await;
        if failed {
            return;
        }

        // 分批调度爬取详情，边爬边flush
        let max_concurrent = 16;
        let mut processed = 0;
        let mut anime_data_buffer = Vec::new();
        let mut subtitle_group_buffer = Vec::new();
        let mut resource_buffer = Vec::new();

        let mut stream = stream::iter(all_detail_urls.into_iter().enumerate())
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
                self.update_task_status(
                    CrawlerTaskStatus::Cancelled,
                    (processed as f64) / (self.total_items as f64),
                    Some("Task was cancelled".to_string()),
                )
                .await;
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
                self.update_task_status(CrawlerTaskStatus::Running, percent, None)
                    .await;
            }
            processed += 1;
            self.processed_items = processed;
            // 每10条写入一次
            if processed % 10 == 0 || processed as i64 == self.total_items {
                // 将缓冲区数据写入self
                self.anime_buffer.append(&mut anime_data_buffer);
                self.subtitle_group_buffer
                    .append(&mut subtitle_group_buffer);
                self.resource_buffer.append(&mut resource_buffer);
                if let Err(e) = self.flush_buffers().await {
                    let error_msg = format!("Failed to save data to database: {}", e);
                    self.update_task_status(
                        CrawlerTaskStatus::Failed,
                        processed as f64 / self.total_items as f64,
                        Some(error_msg),
                    )
                    .await;
                    return;
                }
            }
        }

        self.update_task_status(CrawlerTaskStatus::Completed, 1.0, None)
            .await;
    }

    async fn flush_buffers(&mut self) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        let anime_repo = AnimeRepository::new(&self.pool);
        anime_repo
            .insert_many_animes(&mut tx, &self.anime_buffer)
            .await?;

        let subtitle_repo = SubtitleGroupRepository::new(&self.pool);
        subtitle_repo
            .insert_many_subtitle_groups(&mut tx, &self.subtitle_group_buffer)
            .await?;

        let resource_repo = ResourceRepository::new(&self.pool);
        resource_repo
            .insert_many_resources(&mut tx, &self.resource_buffer)
            .await?;

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
