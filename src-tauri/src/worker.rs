use crate::repositories::crawler_task::CrawlerTaskRepository;
use crate::repositories::base::Repository;
use crate::models::{CrawlerTaskStatus};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{sleep, Duration};
use tracing::info;
use std::sync::atomic::{AtomicUsize, Ordering};
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
                    let notify = self.notify.clone();
                    let semaphore = self.semaphore.clone();
                    let retry_count = self.retry_count;
                    let retry_delay_ms = self.retry_delay_ms;
                    tokio::spawn(async move {
                        let mut attempt = 0;
                        let mut success = false;
                        while attempt < retry_count && !success {
                            attempt += 1;
                            // todo: 调用爬虫抓取与批量入库
                            // 这里后续调用crawler_service::run
                            // ...
                            // 假设run返回Result
                            let run_result: Result<(), String> = Ok(()); // todo: 替换为真实调用
                            match run_result {
                                Ok(_) => {
                                    success = true;
                                    task.status = CrawlerTaskStatus::Completed;
                                    task.completed_at = Some(chrono::Utc::now().timestamp_millis());
                                    let repo = CrawlerTaskRepository::new(&pool);
                                    let _ = repo.update(&task).await;
                                    info!("Worker: 任务完成: {:?}", task.id);
                                }
                                Err(e) => {
                                    if attempt >= retry_count {
                                        task.status = CrawlerTaskStatus::Failed;
                                        task.completed_at = Some(chrono::Utc::now().timestamp_millis());
                                        task.error_message = Some(format!("重试{}次后失败: {}", attempt, e));
                                        let repo = CrawlerTaskRepository::new(&pool);
                                        let _ = repo.update(&task).await;
                                        info!("Worker: 任务最终失败: {:?}", task.id);
                                    } else {
                                        info!("Worker: 任务失败，准备重试({}/{}): {:?}", attempt, retry_count, task.id);
                                        tokio::time::sleep(std::time::Duration::from_millis(retry_delay_ms)).await;
                                    }
                                }
                            }
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