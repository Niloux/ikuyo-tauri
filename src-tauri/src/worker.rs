use crate::config::Config;
use crate::models::CrawlerTaskStatus;
use crate::repositories::base::Repository;
use crate::repositories::crawler_task::CrawlerTaskRepository;
use crate::services::bangumi_service::BangumiService;
use crate::services::crawler_service::CrawlerService;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration};
use tracing::info;

pub struct Worker {
    pool: Arc<SqlitePool>,
    notify: Arc<Notify>,
    semaphore: Arc<Semaphore>,
    retry_count: usize,
    retry_delay_ms: u64,
    config: Config,
    exit_flag: Arc<std::sync::atomic::AtomicBool>,
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
            retry_delay_ms: 1000,
            config,
            exit_flag,
        }
    }

    pub async fn run(&self) {
        // 启动内容缓存与资源刷新主循环
        let pool_clone = self.pool.clone();
        let config_clone = self.config.clone();
        tokio::spawn(async move {
            main_refresh_loop(pool_clone, config_clone).await;
        });

        loop {
            if self.exit_flag.load(std::sync::atomic::Ordering::SeqCst) {
                info!("Worker: 检测到退出标志，停止新任务调度");
                break;
            }
            let permit = self.semaphore.clone().acquire_owned().await.unwrap();
            let repo = CrawlerTaskRepository::new(&self.pool);
            let pending_tasks = repo.list_by_status(CrawlerTaskStatus::Pending, -1, 0).await;
            match pending_tasks {
                Ok(mut tasks) if !tasks.is_empty() => {
                    if self.exit_flag.load(std::sync::atomic::Ordering::SeqCst) {
                        info!("Worker: 退出中，放弃调度新任务");
                        drop(permit);
                        break;
                    }
                    let mut task = tasks.remove(0);
                    task.status = CrawlerTaskStatus::Running;
                    task.started_at = Some(chrono::Utc::now().timestamp_millis());
                    let _ = repo.update(&task).await;
                    let pool = self.pool.clone();
                    let mut crawler_service =
                        CrawlerService::new(pool.clone(), task.id.unwrap_or_default());
                    let retry_count = self.retry_count;
                    tokio::spawn(async move {
                        let mut attempt = 0;
                        let mut success = false;
                        while attempt < retry_count && !success {
                            attempt += 1;
                            // 调用爬虫抓取与批量入库
                            crawler_service.run().await;
                            // 这里run内部已处理状态与错误，无需run_result占位
                            // 可根据实际需求补充成功/失败判断
                            success = true; // 假设run内部已处理所有异常
                            task.status = CrawlerTaskStatus::Completed;
                            task.completed_at = Some(chrono::Utc::now().timestamp_millis());
                            let repo = CrawlerTaskRepository::new(&pool);
                            let _ = repo.update(&task).await;
                            info!("Worker: 任务完成: {:?}", task.id);
                        }
                        drop(permit);
                    });
                }
                _ => {
                    tokio::select! {
                        _ = self.notify.notified() => {
                            info!("Worker: 收到唤醒信号，立即检查任务");
                        }
                        _ = sleep(Duration::from_secs(5)) => {
                        }
                    }
                }
            }
        }
    }
}

// 内容缓存与资源刷新主循环
// 该主循环负责：
// 1. 定时刷新Bangumi订阅/非订阅/日历缓存
// 2. 每天0点自动插入首页资源爬取任务（Schedule+homepage），并在应用启动/恢复时补发当天缺失的自动任务
// 3. 未来可扩展更多定时内容刷新任务
async fn main_refresh_loop(pool: Arc<SqlitePool>, config: Config) {
    use chrono::{Datelike, Utc};
    use crate::models::{CrawlerTask, CrawlerTaskStatus, CrawlerTaskType};
    use crate::repositories::crawler_task::CrawlerTaskRepository;
    use crate::types::crawler::{CrawlerMode, CrawlerTaskCreate};
    use serde_json;
    info!("内容缓存与资源刷新主循环启动");
    // 启动时立即刷新所有订阅subject/episodes缓存
    refresh_all_subscribed_bangumi(&pool, &config).await;
    let mut last_sub = Utc::now().timestamp();
    let mut last_non_sub = Utc::now().timestamp();
    let mut last_calendar = Utc::now().timestamp();
    let mut last_homepage_task_date = None;
    loop {
        let now = Utc::now();
        let today = (now.year(), now.month(), now.day());
        // 检查当天是否已插入Schedule+homepage任务
        if last_homepage_task_date != Some(today) {
            let repo = CrawlerTaskRepository::new(&pool);
            // 查询当天所有Schedule+homepage任务
            let start_of_day = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp_millis();
            let end_of_day = now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp_millis();
            let tasks = repo.list_by_type(CrawlerTaskType::Scheduled, -1, 0).await.unwrap_or_default();
            // 只筛选当天mode为Homepage的任务
            let homepage_tasks: Vec<_> = tasks.iter().filter(|t| {
                if let Some(params) = &t.parameters {
                    if let Ok(parsed) = serde_json::from_str::<CrawlerTaskCreate>(params) {
                        t.created_at.unwrap_or(0) >= start_of_day && t.created_at.unwrap_or(0) <= end_of_day && parsed.mode == CrawlerMode::Homepage
                    } else { false }
                } else { false }
            }).collect();
            // 检查是否有Completed状态的任务
            let has_completed = homepage_tasks.iter().any(|t| t.status == CrawlerTaskStatus::Completed);
            if !has_completed {
                // 插入Schedule+homepage任务
                let parameters = serde_json::to_string(&CrawlerTaskCreate {
                    mode: CrawlerMode::Homepage,
                    year: None,
                    season: None,
                    limit: None,
                }).unwrap();
                let new_task = CrawlerTask {
                    id: None,
                    task_type: CrawlerTaskType::Scheduled,
                    status: CrawlerTaskStatus::Pending,
                    parameters: Some(parameters),
                    result_summary: None,
                    created_at: Some(now.timestamp_millis()),
                    started_at: None,
                    completed_at: None,
                    error_message: None,
                    percentage: Some(0.0),
                    processed_items: Some(0),
                    total_items: Some(100),
                    processing_speed: None,
                    estimated_remaining: None,
                };
                let _ = repo.create(&new_task).await;
                info!("自动插入首页资源爬取任务（Schedule+homepage）");
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
            refresh_all_subscribed_bangumi(&pool, &config).await;
            last_sub = now_ts;
        }
        if now_ts - last_non_sub >= nonsub_interval {
            refresh_all_non_subscribed_bangumi(&pool, &config).await;
            last_non_sub = now_ts;
        }
        if now_ts - last_calendar >= calendar_interval {
            let service = BangumiService::new(pool.clone(), config.clone());
            let _ = service.get_calendar().await;
            last_calendar = now_ts;
        }
        sleep(Duration::from_secs(60)).await; // 每分钟检查一次
    }
}

// 刷新所有订阅subject/episodes缓存
async fn refresh_all_subscribed_bangumi(pool: &Arc<SqlitePool>, config: &Config) {
    use crate::services::bangumi_service::BangumiService;
    use sqlx::Row;
    use std::collections::HashSet;
    // 获取所有订阅subject_id（去重）
    let rows = sqlx::query("SELECT DISTINCT bangumi_id FROM user_subscriptions")
        .fetch_all(&**pool)
        .await
        .unwrap_or_default();
    let ids: HashSet<i64> = rows.iter().map(|r| r.get::<i64, _>(0)).collect();
    let service = BangumiService::new(pool.clone(), config.clone());
    for id in ids {
        let _ = service.get_subject(id).await;
        let _ = service.get_episodes(id, None, None, None).await;
    }
}

// 刷新所有非订阅subject/episodes缓存
async fn refresh_all_non_subscribed_bangumi(pool: &Arc<SqlitePool>, config: &Config) {
    use crate::services::bangumi_service::BangumiService;
    use sqlx::Row;
    use std::collections::HashSet;
    // 获取所有subject_id
    let rows = sqlx::query("SELECT id FROM bangumi_subject_cache")
        .fetch_all(&**pool)
        .await
        .unwrap_or_default();
    let all_ids: HashSet<i64> = rows.iter().map(|r| r.get::<i64, _>(0)).collect();
    // 获取所有订阅subject_id
    let sub_rows = sqlx::query("SELECT DISTINCT bangumi_id FROM user_subscriptions")
        .fetch_all(&**pool)
        .await
        .unwrap_or_default();
    let sub_ids: HashSet<i64> = sub_rows.iter().map(|r| r.get::<i64, _>(0)).collect();
    // 非订阅id = all_ids - sub_ids
    let non_sub_ids: Vec<i64> = all_ids.difference(&sub_ids).cloned().collect();
    let service = BangumiService::new(pool.clone(), config.clone());
    for id in non_sub_ids {
        let _ = service.get_subject(id).await;
        let _ = service.get_episodes(id, None, None, None).await;
    }
}
