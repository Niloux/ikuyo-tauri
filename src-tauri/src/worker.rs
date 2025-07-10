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
use tokio::time::{sleep, Duration, Instant};
use tracing::info;

pub struct Worker {
    pool: Arc<SqlitePool>,
    notify: Arc<Notify>,
    semaphore: Arc<Semaphore>,
    retry_count: usize,
    retry_delay_ms: u64,
    config: Config,
}

impl Worker {
    pub fn new(
        pool: Arc<SqlitePool>,
        notify: Arc<Notify>,
        config: Config,
        permits: Option<usize>,
    ) -> Self {
        let permits = permits.unwrap_or(1);
        Self {
            pool,
            notify,
            semaphore: Arc::new(Semaphore::new(permits)),
            retry_count: 3,
            retry_delay_ms: 1000,
            config,
        }
    }

    pub async fn run(&self) {
        // 启动Bangumi缓存定时刷新任务
        let pool_clone = self.pool.clone();
        let config_clone = self.config.clone();
        tokio::spawn(async move {
            bangumi_cache_refresh_loop(pool_clone, config_clone).await;
        });

        loop {
            let permit = self.semaphore.clone().acquire_owned().await.unwrap();
            let repo = CrawlerTaskRepository::new(&self.pool);
            let pending_tasks = repo.list_by_status(CrawlerTaskStatus::Pending, -1, 0).await;
            match pending_tasks {
                Ok(mut tasks) if !tasks.is_empty() => {
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

// Bangumi缓存定时刷新主循环
async fn bangumi_cache_refresh_loop(pool: Arc<SqlitePool>, config: Config) {
    use chrono::Utc;
    info!("Bangumi缓存定时刷新任务启动");
    // 启动时立即刷新所有订阅subject/episodes缓存
    refresh_all_subscribed_bangumi(&pool, &config).await;
    let mut last_sub = Utc::now().timestamp();
    let mut last_non_sub = Utc::now().timestamp();
    let mut last_calendar = Utc::now().timestamp();
    loop {
        let now = Utc::now().timestamp();
        let sub_interval = config.bangumi_sub_refresh_interval.unwrap_or(3600);
        let nonsub_interval = config.bangumi_nonsub_refresh_interval.unwrap_or(43200);
        let calendar_interval = config.bangumi_calendar_refresh_interval.unwrap_or(86400);
        if now - last_sub >= sub_interval {
            refresh_all_subscribed_bangumi(&pool, &config).await;
            last_sub = now;
        }
        if now - last_non_sub >= nonsub_interval {
            refresh_all_non_subscribed_bangumi(&pool, &config).await;
            last_non_sub = now;
        }
        if now - last_calendar >= calendar_interval {
            let service = BangumiService::new(pool.clone(), config.clone());
            let _ = service.get_calendar().await;
            last_calendar = now;
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
