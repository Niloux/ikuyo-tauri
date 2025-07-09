use crate::repositories::crawler_task::CrawlerTaskRepository;
use crate::repositories::base::Repository;
use crate::models::{CrawlerTaskStatus};
use crate::services::crawler_service::CrawlerService;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{sleep, Duration};
use tracing::info;
use tokio::sync::Semaphore;

pub struct Worker {
    pool: Arc<SqlitePool>,
    notify: Arc<Notify>,
    max_workers: usize,
    semaphore: Arc<Semaphore>,
    retry_count: usize,
    retry_delay_ms: u64,
}

impl Worker {
    pub fn new(pool: Arc<SqlitePool>, notify: Arc<Notify>, max_workers: usize) -> Self {
        Self {
            pool,
            notify,
            max_workers,
            semaphore: Arc::new(Semaphore::new(max_workers)),
            retry_count: 3,
            retry_delay_ms: 1000,
        }
    }

    pub async fn run(&self) {
        loop {
            let permit = self.semaphore.clone().acquire_owned().await.unwrap();
            let repo = CrawlerTaskRepository::new(&self.pool);
            let pending_tasks = repo.list_by_status(CrawlerTaskStatus::Pending, 1, 0).await;
            match pending_tasks {
                Ok(mut tasks) if !tasks.is_empty() => {
                    let mut task = tasks.remove(0);
                    task.status = CrawlerTaskStatus::Running;
                    task.started_at = Some(chrono::Utc::now().timestamp_millis());
                    let _ = repo.update(&task).await;
                    let pool = self.pool.clone();
                    let mut crawler_service = CrawlerService::new(pool.clone(), task.id.unwrap_or_default());
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