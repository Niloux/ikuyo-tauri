use crate::config::Config;
use crate::error::{AppError, Result};
use crate::models::CrawlerTaskStatus;
use crate::repositories::base::Repository;
use crate::repositories::crawler_task::CrawlerTaskRepository;
use crate::services::bangumi_service::BangumiService;
use crate::services::crawler_service::CrawlerService;
use futures_util::stream::{StreamExt};
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use tokio::sync::{Notify, RwLock, Semaphore};
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub struct Worker {
    pool: Arc<SqlitePool>,
    notify: Arc<Notify>,
    semaphore: Arc<Semaphore>,
    retry_count: usize,
    config: Config,
    exit_flag: Arc<std::sync::atomic::AtomicBool>,
    cancel_tokens: Arc<Mutex<HashMap<i64, CancellationToken>>>,
}

impl Worker {
    pub fn new(
        pool: Arc<SqlitePool>,
        notify: Arc<Notify>,
        config: Config,
        permits: Option<usize>,
        exit_flag: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        let permits = permits.unwrap_or(1);
        Self {
            pool,
            notify,
            semaphore: Arc::new(Semaphore::new(permits)),
            retry_count: 3,
            config,
            exit_flag,
            cancel_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_token_map(&self) -> Arc<Mutex<HashMap<i64, CancellationToken>>> {
        self.cancel_tokens.clone()
    }

    pub async fn run(&self) {
        // 启动内容缓存与资源刷新主循环
        let pool_clone = self.pool.clone();
        let config_clone = self.config.clone();
        tokio::spawn(async move {
            main_refresh_loop(pool_clone, config_clone).await;
        });

        loop {
            if self.exit_flag.load(Ordering::SeqCst) {
                info!("Worker: 检测到退出标志，停止新任务调度");
                break;
            }
            let permit = self
                .semaphore
                .clone()
                .acquire_owned()
                .await
                .expect("Semaphore closed unexpectedly");
            let repo = CrawlerTaskRepository::new(&self.pool);

            let pending_tasks_result = repo.list_by_status(CrawlerTaskStatus::Pending, -1, 0).await;

            match pending_tasks_result {
                Ok(mut tasks) if !tasks.is_empty() => {
                    if self.exit_flag.load(Ordering::SeqCst) {
                        info!("Worker: 退出中，放弃调度新任务");
                        drop(permit);
                        break;
                    }
                    let mut task = tasks.remove(0);
                    task.status = CrawlerTaskStatus::Running;
                    task.started_at = Some(chrono::Utc::now().timestamp_millis());

                    if let Err(e) = repo.update(&task).await {
                        error!("Worker: 更新任务状态失败: {:?}, 错误: {:?}", task.id, e);
                        continue;
                    }

                    let pool = self.pool.clone();

                    // 如果任务没有ID，则跳过任务
                    let task_id = if let Some(id) = task.id {
                        id
                    } else {
                        error!("Worker: 任务没有ID, 跳过任务: {:?}", task);
                        continue;
                    };

                    let mut crawler_service = CrawlerService::new(pool.clone(), task_id);
                    let retry_count = self.retry_count;
                    let cancel_tokens = self.cancel_tokens.clone();

                    let token = CancellationToken::new();
                    if let Ok(mut map) = cancel_tokens.lock() {
                        map.insert(task_id, token.clone());
                    }

                    let task_for_fail = Arc::new(RwLock::new(task.clone()));
                    tokio::spawn(async move {
                        let mut attempt = 0;
                        let mut success = false;
                        while attempt < retry_count && !success {
                            attempt += 1;
                            crawler_service.set_cancellation_token(token.clone());

                            match crawler_service.run().await {
                                Ok(_) => {
                                    success = true;
                                    info!("Worker: 任务 {} 完成", task_id);
                                }
                                Err(e) => {
                                    warn!(
                                        "Worker: 任务 {} 失败 (尝试 {}/{}), 错误: {:?}",
                                        task_id, attempt, retry_count, e
                                    );
                                    if attempt >= retry_count {
                                        error!("Worker: 任务 {} 在所有重试后最终失败", task_id);
                                        let repo = CrawlerTaskRepository::new(&pool);
                                        let mut final_task = task_for_fail.write().await;
                                        final_task.status = CrawlerTaskStatus::Failed;
                                        final_task.completed_at =
                                            Some(chrono::Utc::now().timestamp_millis());
                                        final_task.error_message = Some(e.to_string());
                                        if let Err(db_err) = repo.update(&*final_task).await {
                                            error!(
                                                "Worker: 更新任务为失败状态时出错: {:?}",
                                                db_err
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        if let Ok(mut map) = cancel_tokens.lock() {
                            map.remove(&task_id);
                        }
                        drop(permit);
                    });
                }
                Err(e) => {
                    error!("Worker: 从数据库获取待处理任务失败: {:?}", e);
                    sleep(Duration::from_secs(5)).await;
                }
                _ => {
                    tokio::select! {
                        _ = self.notify.notified() => {
                            info!("Worker: 收到唤醒信号，立即检查任务");
                        }
                        _ = sleep(Duration::from_secs(5)) => {}
                    }
                }
            }
        }
    }
}

async fn main_refresh_loop(pool: Arc<SqlitePool>, config: Config) {
    use crate::models::{CrawlerTask, CrawlerTaskStatus, CrawlerTaskType};
    use crate::repositories::crawler_task::CrawlerTaskRepository;
    use crate::types::crawler::{CrawlerMode, CrawlerTaskCreate};
    use chrono::{Datelike, Utc};

    info!("内容缓存与资源刷新主循环启动");

    if let Err(e) = refresh_all_subscribed_bangumi(&pool, &config).await {
        error!("启动时刷新订阅缓存失败: {:?}", e);
    }

    let mut last_sub = Utc::now().timestamp();
    let mut last_non_sub = Utc::now().timestamp();
    let mut last_calendar = Utc::now().timestamp();
    let mut last_homepage_task_date = None;

    loop {
        let now = Utc::now();
        let today = (now.year(), now.month(), now.day());

        if last_homepage_task_date != Some(today) {
            let start_of_day = now
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp_millis();
            let end_of_day = now
                .date_naive()
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc()
                .timestamp_millis();

            let tasks = {
                let repo = CrawlerTaskRepository::new(&pool);
                match repo.list_by_type(CrawlerTaskType::Scheduled, -1, 0).await {
                    Ok(t) => t,
                    Err(e) => {
                        error!("获取定时任务列表失败: {:?}, 将在下一分钟重试", e);
                        sleep(Duration::from_secs(60)).await;
                        continue;
                    }
                }
            };

            let homepage_tasks: Vec<_> = tasks
                .iter()
                .filter(|t| {
                    if let Some(params) = &t.parameters {
                        if let Ok(parsed) = serde_json::from_str::<CrawlerTaskCreate>(params) {
                            t.created_at.unwrap_or(0) >= start_of_day
                                && t.created_at.unwrap_or(0) <= end_of_day
                                && parsed.mode == CrawlerMode::Homepage
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                })
                .collect();

            let has_completed = homepage_tasks
                .iter()
                .any(|t| t.status == CrawlerTaskStatus::Completed);

            if !has_completed {
                let parameters = match serde_json::to_string(&CrawlerTaskCreate {
                    mode: CrawlerMode::Homepage,
                    year: None,
                    season: None,
                    limit: None,
                }) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("序列化首页任务参数失败: {:?}, 跳过本次插入", e);
                        last_homepage_task_date = Some(today);
                        continue;
                    }
                };
                let new_task = CrawlerTask {
                    parameters: Some(parameters),
                    id: None,
                    task_type: CrawlerTaskType::Scheduled,
                    status: CrawlerTaskStatus::Pending,
                    result_summary: None,
                    created_at: Some(now.timestamp_millis()),
                    started_at: None,
                    completed_at: None,
                    error_message: None,
                    percentage: Some(0.0),
                    processed_items: Some(0),
                    total_items: Some(0),
                    processing_speed: None,
                    estimated_remaining: None,
                };
                {
                    let repo = CrawlerTaskRepository::new(&pool);
                    if let Err(e) = repo.create(&new_task).await {
                        error!("自动插入首页资源爬取任务失败: {:?}", e);
                    } else {
                        info!("自动插入首页资源爬取任务（Schedule+homepage）");
                    }
                }
            } else {
                info!("当天已存在Completed状态的Schedule+homepage任务，不再补发");
            }
            last_homepage_task_date = Some(today);
        }

        let now_ts = now.timestamp();
        let sub_interval = config.bangumi_sub_refresh_interval.unwrap_or(3600);
        let nonsub_interval = config.bangumi_nonsub_refresh_interval.unwrap_or(43200);
        let calendar_interval = config.bangumi_calendar_refresh_interval.unwrap_or(86400);

        if now_ts - last_sub >= sub_interval {
            if let Err(e) = refresh_all_subscribed_bangumi(&pool, &config).await {
                error!("刷新订阅番剧缓存失败: {:?}", e);
            }
            last_sub = now_ts;
        }
        if now_ts - last_non_sub >= nonsub_interval {
            if let Err(e) = refresh_all_non_subscribed_bangumi(&pool, &config).await {
                error!("刷新非订阅番剧缓存失败: {:?}", e);
            }
            last_non_sub = now_ts;
        }
        if now_ts - last_calendar >= calendar_interval {
            let service = BangumiService::new(pool.clone(), config.clone());
            if let Err(e) = service.get_calendar().await {
                error!("刷新日历缓存失败: {:?}", e);
            }
            last_calendar = now_ts;
        }
        sleep(Duration::from_secs(60)).await;
    }
}

async fn refresh_bangumi_batch(
    ids: &HashSet<i64>,
    interval: i64,
    log_prefix: &str,
    pool: &Arc<SqlitePool>,
    config: &Config,
) -> Result<()> {
    use chrono::Utc;
    use sqlx::Row;

    if ids.is_empty() {
        return Ok(());
    }

    let now = Utc::now().timestamp();
    let mut to_refresh = Vec::new();
    // 1. 先串行查询所有subject
    for &id in ids.iter() {
        let row: Option<i64> = sqlx::query("SELECT updated_at FROM bangumi_subject_cache WHERE id = ?")
            .bind(id)
            .fetch_optional(&**pool)
            .await?
            .map(|r| r.get(0));
        let needs_refresh = row.map_or(true, |updated_at| now - updated_at >= interval);
        if needs_refresh {
            to_refresh.push((id, "subject".to_string()));
        }
    }
    // 2. 再串行查询所有episodes
    for &id in ids.iter() {
        let row: Option<i64> = sqlx::query("SELECT updated_at FROM bangumi_episodes_cache WHERE id = ? AND params_hash = '0'")
            .bind(id)
            .fetch_optional(&**pool)
            .await?
            .map(|r| r.get(0));
        let needs_refresh = row.map_or(true, |updated_at| now - updated_at >= interval);
        if needs_refresh {
            to_refresh.push((id, "episodes".to_string()));
        }
    }
    // 3. 并发网络请求
    let service = BangumiService::new(pool.clone(), config.clone());
    let service_ref = &service;
    let results = futures_util::stream::iter(to_refresh.into_iter().map(move |(id, item_type)| {
        async move {
            // 这里每个future都用同一个service的引用，避免clone
            let service = service_ref;
            info!("[worker] 刷新{}/{}缓存: {}", log_prefix, item_type, id);
            let refresh_result = if item_type == "subject" {
                service.get_subject(id).await.map(|_| ())
            } else {
                service.get_episodes(id, Some(0), Some(1000), Some(0)).await.map(|_| ())
            };
            if let Err(e) = refresh_result {
                warn!("[worker] 刷新{}/{}缓存失败: {}: {:?}", log_prefix, item_type, id, e);
            }
            Ok::<(), AppError>(())
        }
    }))
    .buffer_unordered(8)
    .collect::<Vec<_>>()
    .await;
    // 4. 统一处理错误
    for result in results {
        if let Err(e) = result {
            error!("[worker] 批量刷新时发生内部错误: {:?}", e);
        }
    }
    Ok(())
}

async fn refresh_all_subscribed_bangumi(pool: &Arc<SqlitePool>, config: &Config) -> Result<()> {
    use sqlx::Row;
    let rows = sqlx::query("SELECT DISTINCT bangumi_id FROM user_subscriptions")
        .fetch_all(&**pool)
        .await?;
    let ids: HashSet<i64> = rows.iter().map(|r| r.get::<i64, _>(0)).collect();
    let sub_interval = config.bangumi_sub_refresh_interval.unwrap_or(3600);
    refresh_bangumi_batch(&ids, sub_interval, "订阅", pool, config).await
}

async fn refresh_all_non_subscribed_bangumi(pool: &Arc<SqlitePool>, config: &Config) -> Result<()> {
    use sqlx::Row;
    let rows = sqlx::query(
        "SELECT id FROM bangumi_subject_cache WHERE id NOT IN (SELECT DISTINCT bangumi_id FROM user_subscriptions)"
    ).fetch_all(&**pool).await?;

    let non_sub_ids: HashSet<i64> = rows.iter().map(|r| r.get::<i64, _>(0)).collect();
    let nonsub_interval = config.bangumi_nonsub_refresh_interval.unwrap_or(43200);
    refresh_bangumi_batch(&non_sub_ids, nonsub_interval, "非订阅", pool, config).await
}
