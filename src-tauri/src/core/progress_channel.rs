use tokio::sync::mpsc::{self, Sender, Receiver};
use crate::models::CrawlerTaskStatus;
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ProgressMessage {
    pub task_id: i64,
    pub status: CrawlerTaskStatus,
    pub percentage: f64,
    pub error_message: Option<String>,
    pub processed_items: i64,
    pub total_items: i64,
}

pub fn create_progress_channel(buffer: usize) -> (Sender<ProgressMessage>, Receiver<ProgressMessage>) {
    mpsc::channel(buffer)
}

pub async fn progress_consumer(mut rx: Receiver<ProgressMessage>, pool: Arc<SqlitePool>) {
    use crate::repositories::crawler_task::CrawlerTaskRepository;
    while let Some(msg) = rx.recv().await {
        let repo = CrawlerTaskRepository::new(&pool);
        // 简化：只更新进度、状态、错误
        if let Ok(Some(mut task)) = repo.get_by_id(msg.task_id).await {
            task.status = msg.status;
            task.percentage = Some(msg.percentage);
            task.error_message = msg.error_message.clone();
            task.processed_items = Some(msg.processed_items);
            task.total_items = Some(msg.total_items);
            let _ = repo.update(&task).await;
        }
        // todo: 可扩展为批量写入、日志归档等
    }
} 