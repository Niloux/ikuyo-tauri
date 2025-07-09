use crate::repositories::crawler_task::CrawlerTaskRepository;
use crate::repositories::base::Repository;
use crate::models::{CrawlerTaskStatus};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{sleep, Duration};
use tracing::info;

pub struct Worker {
    pool: Arc<SqlitePool>,
    notify: Arc<Notify>,
}

impl Worker {
    pub fn new(pool: Arc<SqlitePool>, notify: Arc<Notify>) -> Self {
        Self { pool, notify }
    }

    pub async fn run(&self) {
        loop {
            // 1. 检查Pending任务
            let repo = CrawlerTaskRepository::new(&self.pool);
            let pending_tasks = repo.list_by_status(CrawlerTaskStatus::Pending, 1, 0).await;
            match pending_tasks {
                Ok(mut tasks) if !tasks.is_empty() => {
                    let mut task = tasks.remove(0);
                    // 2. 原子更新为Running
                    task.status = CrawlerTaskStatus::Running;
                    task.started_at = Some(chrono::Utc::now().timestamp_millis());
                    let _ = repo.update(&task).await;
                    info!("Worker: 开始执行任务: {:?}", task.id);
                    // 3. 占位：调用爬虫执行逻辑（后续补充）
                    // todo: 调用爬虫抓取与批量入库
                    // 4. 完成后更新状态
                    task.status = CrawlerTaskStatus::Completed;
                    task.completed_at = Some(chrono::Utc::now().timestamp_millis());
                    let _ = repo.update(&task).await;
                    info!("Worker: 任务完成: {:?}", task.id);
                }
                _ => {
                    // 无任务，等待唤醒或定时轮询
                    tokio::select! {
                        _ = self.notify.notified() => {
                            info!("Worker: 收到唤醒信号，立即检查任务");
                        }
                        _ = sleep(Duration::from_secs(5)) => {
                            // 定时轮询
                        }
                    }
                }
            }
        }
    }
} 