use crate::core::http_fetcher::HttpFetcher;
use crate::core::mikan_parser::MikanParser;
use crate::core::anime_parser::AnimeParser;
use crate::models::{Anime, CrawlerTaskStatus, Resource, SubtitleGroup};
use crate::repositories::{
    anime::AnimeRepository, base::Repository, crawler_task::CrawlerTaskRepository,
    resource::ResourceRepository, subtitle_group::SubtitleGroupRepository,
};
use crate::types::crawler::{CrawlerMode, CrawlerTaskCreate, SeasonName};
use futures_util::stream::{self, StreamExt};
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use crate::error::{AppError, TaskError, Result};

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
    // 新增：任务取消信号
    cancellation_token: Option<CancellationToken>,
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
            cancellation_token: None,
        }
    }

    pub fn set_cancellation_token(&mut self, token: CancellationToken) {
        self.cancellation_token = Some(token);
    }

    pub async fn run(&mut self) -> Result<()> {
        let repo = CrawlerTaskRepository::new(&self.pool);
        let task = match repo.get_by_id(self.task_id).await {
            Ok(Some(task)) => task,
            Ok(None) => {
                tracing::error!("任务{}不存在，停止运行。", self.task_id);
                return Err(AppError::Task(TaskError::Failed("任务不存在".to_string())));
            },
            Err(e) => {
                tracing::error!("查询任务{}失败: {}", self.task_id, e);
                return Err(AppError::Task(TaskError::Failed(format!("查询任务失败: {}", e))));
            }
        };

        let params: CrawlerTaskCreate = task
            .parameters
            .as_ref()
            .and_then(|p| serde_json::from_str(p).ok())
            .unwrap_or_default();

        let start_time = chrono::Utc::now().timestamp_millis();
        self.update_task_status(CrawlerTaskStatus::Running, None, Some(0.0), None)
            .await;

        let base_url = "https://mikanani.me"; // TODO: Get from config
        let fetcher = HttpFetcher::new();
        let parser = MikanParser::new(base_url);
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
                    match fetcher.fetch(&list_url).await {
                        Ok(html) => match parser.parse_list(&html) {
                            Ok(mut urls) => {
                                if let Some(lim) = limit {
                                    urls.truncate(lim as usize);
                                }
                                total_items += urls.len();
                                all_detail_urls.extend(urls);
                            }
                            Err(e) => {
                                error_message = Some(format!("解析列表页失败: {}", e));
                                failed = true;
                                break;
                            }
                        },
                        Err(e) => {
                            error_message = Some(format!("fetch列表页失败: {}", e));
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
                match fetcher.fetch(&list_url).await {
                    Ok(html) => match parser.parse_list(&html) {
                        Ok(mut urls) => {
                            if let Some(lim) = limit {
                                urls.truncate(lim as usize);
                            }
                            total_items = urls.len();
                            all_detail_urls = urls;
                        }
                        Err(e) => {
                            error_message = Some(format!("解析列表页失败: {}", e));
                            failed = true;
                        }
                    },
                    Err(e) => {
                        error_message = Some(format!("fetch列表页失败: {}", e));
                        failed = true;
                    }
                }
            }
            CrawlerMode::Homepage => {
                let list_url = format!("{}/Home", base_url);
                match fetcher.fetch(&list_url).await {
                    Ok(html) => match parser.parse_list(&html) {
                        Ok(mut urls) => {
                            if let Some(lim) = limit {
                                urls.truncate(lim as usize);
                            }
                            total_items = urls.len();
                            all_detail_urls = urls;
                        }
                        Err(e) => {
                            error_message = Some(format!("解析首页列表页失败: {}", e));
                            failed = true;
                        }
                    },
                    Err(e) => {
                        error_message = Some(format!("fetch首页列表页失败: {}", e));
                        failed = true;
                    }
                }
            }
        }

        self.total_items = total_items as i64;
        tracing::info!("总动画数: {}", self.total_items);
        self.processed_items = 0;
        self.update_task_status(
            if failed {
                CrawlerTaskStatus::Failed
            } else {
                CrawlerTaskStatus::Running
            },
            error_message.clone(),
            Some(0.0),
            None,
        )
        .await;
        if failed {
            return Err(AppError::Task(TaskError::Failed(error_message.unwrap_or_else(|| "未知错误".to_string()))));
        }

        // 分批调度爬取详情，边爬边flush
        let max_concurrent = 8;
        let mut processed = 0;
        let mut anime_data_buffer = Vec::new();
        let mut subtitle_group_buffer = Vec::new();
        let mut resource_buffer = Vec::new();

        let mut stream = stream::iter(all_detail_urls.into_iter().enumerate())
            .map(|(i, url)| {
                let fetcher = &fetcher;
                let parser = &parser;
                let mikan_id = url.split('/').last().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
                async move {
                    use tokio::time::{timeout, Duration};
                    match timeout(
                        Duration::from_secs(30),
                        async {
                            match fetcher.fetch(&url).await {
                                Ok(html) => match parser.parse_detail(&html, mikan_id) {
                                    Ok(anime_data) => Ok(anime_data),
                                    Err(e) => Err(e),
                                },
                                Err(e) => Err(e.into()),
                            }
                        },
                    )
                    .await
                    {
                        Ok(Ok(anime_data)) => Some((i, anime_data)),
                        Ok(Err(e)) => {
                            tracing::warn!("爬取详情页{}失败: {}", url, e);
                            None
                        }
                        Err(_) => {
                            tracing::warn!("爬取详情页{}超时", url);
                            None
                        }
                    }
                }
            })
            .buffer_unordered(max_concurrent);

        while let Some(result) = stream.next().await {
            if let Some(token) = &self.cancellation_token {
                if token.is_cancelled() {
                    let elapsed = chrono::Utc::now().timestamp_millis() - start_time;
                    let speed = if elapsed > 0 {
                        self.processed_items as f64 / (elapsed as f64 / 1000.0)
                    } else {
                        0.0
                    };
                    self.update_task_status(
                        CrawlerTaskStatus::Cancelled,
                        Some("任务被取消".to_string()),
                        Some(speed),
                        None,
                    )
                    .await;
                    return Err(AppError::Task(TaskError::Cancel("任务被取消".to_string())));
                }
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
                let elapsed = chrono::Utc::now().timestamp_millis() - start_time;
                let speed = if elapsed > 0 {
                    self.processed_items as f64 / (elapsed as f64 / 1000.0)
                } else {
                    0.0
                };
                let remaining = if speed > 0.0 {
                    (self.total_items - self.processed_items) as f64 / speed
                } else {
                    None.unwrap_or(0.0)
                };
                self.update_task_status(
                    CrawlerTaskStatus::Running,
                    None,
                    Some(speed),
                    Some(remaining),
                )
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
                    let error_msg = format!("保存数据到数据库失败: {}", e);
                    self.update_task_status(
                        CrawlerTaskStatus::Failed,
                        Some(error_msg.clone()),
                        Some(0.0),
                        None,
                    )
                    .await;
                    return Err(AppError::Task(TaskError::Failed(error_msg)));
                }
            }
        }

        let elapsed = chrono::Utc::now().timestamp_millis() - start_time;
        let speed = if elapsed > 0 {
            self.processed_items as f64 / (elapsed as f64 / 1000.0)
        } else {
            0.0
        };
        self.update_task_status(CrawlerTaskStatus::Completed, None, Some(speed), Some(0.0))
            .await;
        Ok(())
    }

    async fn flush_buffers(&mut self) -> crate::error::Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await?;

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

        tx.commit()
            .await?;

        self.anime_buffer.clear();
        self.subtitle_group_buffer.clear();
        self.resource_buffer.clear();

        Ok(())
    }

    async fn update_task_status(
        &self,
        status: CrawlerTaskStatus,
        error_message: Option<String>,
        processing_speed: Option<f64>,
        estimated_remaining: Option<f64>,
    ) {
        let repo = CrawlerTaskRepository::new(&self.pool);
        if let Ok(Some(mut task)) = repo.get_by_id(self.task_id).await {
            task.status = status;
            // 统一由此处维护进度
            let percentage = if self.total_items > 0 {
                (self.processed_items as f64) / (self.total_items as f64) * 100.0
            } else {
                0.0
            };
            task.percentage = Some(percentage);
            task.processed_items = Some(self.processed_items);
            task.total_items = Some(self.total_items);
            task.error_message = error_message;
            task.processing_speed = processing_speed;
            task.estimated_remaining = estimated_remaining;

            if task.status == CrawlerTaskStatus::Completed
                || task.status == CrawlerTaskStatus::Failed
                || task.status == CrawlerTaskStatus::Cancelled
            {
                task.completed_at = Some(chrono::Utc::now().timestamp_millis());
            }

            if let Err(e) = repo.update(&task).await {
                tracing::error!("更新任务[{}]状态失败: {}", self.task_id, e);
            }
        }
    }

    /// 创建爬虫任务并唤醒worker
    pub async fn create_task(
        pool: Arc<SqlitePool>,
        notify: Arc<tokio::sync::Notify>,
        task: crate::types::crawler::CrawlerTaskCreate,
    ) -> Result<crate::types::crawler::TaskResponse> {
        use crate::models::{CrawlerTask, CrawlerTaskType, CrawlerTaskStatus};
        use crate::repositories::crawler_task::CrawlerTaskRepository;
        use crate::types::crawler::TaskResponse;
        use crate::error::{AppError, TaskError};
        let repo = CrawlerTaskRepository::new(&pool);
        let parameters = serde_json::to_string(&task).unwrap_or_default();
        let current_time = chrono::Utc::now().timestamp_millis();
        let new_task = CrawlerTask {
            id: None,
            task_type: CrawlerTaskType::Manual,
            status: CrawlerTaskStatus::Pending,
            parameters: Some(parameters),
            result_summary: None,
            created_at: Some(current_time),
            started_at: None,
            completed_at: None,
            error_message: None,
            percentage: Some(0.0),
            processed_items: Some(0),
            total_items: Some(0),
            processing_speed: None,
            estimated_remaining: None,
        };
        repo.create(&new_task).await?;
        // 获取最新创建的 pending 任务
        let created_task = repo
            .list_by_status(CrawlerTaskStatus::Pending, 1, 0)
            .await?
            .into_iter()
            .next();
        // 唤醒worker
        notify.notify_one();
        match created_task {
            Some(task) => Ok(TaskResponse {
                id: task.id.unwrap_or_default(),
                task_type: task.task_type.into(),
                status: task.status.into(),
                parameters: task.parameters,
                result_summary: task.result_summary,
                created_at: Some(task.created_at.unwrap_or_default()),
                started_at: task.started_at,
                completed_at: task.completed_at,
                error_message: task.error_message,
                percentage: Some(task.percentage.unwrap_or_default()),
                processed_items: Some(task.processed_items.unwrap_or_default()),
                total_items: Some(task.total_items.unwrap_or_default()),
                processing_speed: task.processing_speed,
                estimated_remaining: task.estimated_remaining,
            }),
            None => Err(AppError::Task(TaskError::Failed("任务创建失败".to_string()))),
        }
    }

    /// 取消爬虫任务并更新状态
    pub async fn cancel_task(
        pool: Arc<SqlitePool>,
        worker: Arc<crate::worker::Worker>,
        task_id: i64,
    ) -> Result<crate::types::crawler::TaskResponse> {
        use crate::repositories::crawler_task::CrawlerTaskRepository;
        use crate::models::{CrawlerTaskStatus};
        use crate::types::crawler::TaskResponse;
        use crate::error::{AppError, TaskError};
        // 作用域内获取 token 并 clone，避免持有 MutexGuard 进入 await
        let token_opt = {
            let token_map = worker.get_token_map();
            let map = token_map.lock().unwrap();
            map.get(&task_id).cloned()
        };
        if let Some(token) = token_opt {
            token.cancel();
        }
        // 仍然更新数据库状态用于 UI 展示
        let repo = CrawlerTaskRepository::new(&pool);
        let task = repo.get_by_id(task_id).await?;
        match task {
            Some(mut task) => {
                match task.status {
                    CrawlerTaskStatus::Pending | CrawlerTaskStatus::Running => {
                        task.status = CrawlerTaskStatus::Cancelled;
                        task.completed_at = Some(chrono::Utc::now().timestamp_millis());
                        repo.update(&task).await?;
                        Ok(TaskResponse {
                            id: task.id.unwrap_or_default(),
                            task_type: task.task_type.into(),
                            status: task.status.into(),
                            parameters: task.parameters,
                            result_summary: task.result_summary,
                            created_at: Some(task.created_at.unwrap_or_default()),
                            started_at: task.started_at,
                            completed_at: task.completed_at,
                            error_message: task.error_message,
                            percentage: Some(task.percentage.unwrap_or_default()),
                            processed_items: Some(task.processed_items.unwrap_or_default()),
                            total_items: Some(task.total_items.unwrap_or_default()),
                            processing_speed: task.processing_speed,
                            estimated_remaining: task.estimated_remaining,
                        })
                    }
                    _ => Err(AppError::Task(TaskError::Cancel("任务无法取消，当前状态不允许取消操作".to_string()))),
                }
            }
            None => Err(AppError::Task(TaskError::Failed("任务不存在".to_string()))),
        }
    }
}
