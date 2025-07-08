use crate::types::crawler::{CrawlerTaskCreate, TaskResponse, CrawlerTaskType, CrawlerTaskStatus};

#[tauri::command]
pub fn create_crawler_task(task: CrawlerTaskCreate) -> Result<TaskResponse, String> {
    tracing::info!("Creating crawler task: {:?}", task);
    // 返回示例数据
    Ok(TaskResponse {
        id: 1,
        task_type: CrawlerTaskType::Manual,
        status: CrawlerTaskStatus::Pending,
        parameters: Some(serde_json::to_string(&task).unwrap_or_default()),
        result_summary: None,
        created_at: Some(chrono::Utc::now().timestamp_millis()),
        started_at: None,
        completed_at: None,
        error_message: None,
        percentage: Some(0.0),
        processed_items: Some(0),
        total_items: Some(100),
        processing_speed: None,
        estimated_remaining: None,
    })
}

#[tauri::command]
pub fn get_crawler_task_status(task_id: i64) -> Result<TaskResponse, String> {
    tracing::info!("Getting status for crawler task ID: {}", task_id);
    // 返回示例数据
    Ok(TaskResponse {
        id: task_id,
        task_type: CrawlerTaskType::Manual,
        status: CrawlerTaskStatus::Completed,
        parameters: Some("{}".to_string()),
        result_summary: Some("Successfully crawled 100 items.".to_string()),
        created_at: Some(chrono::Utc::now().timestamp_millis()),
        started_at: Some(chrono::Utc::now().timestamp_millis()),
        completed_at: Some(chrono::Utc::now().timestamp_millis()),
        error_message: None,
        percentage: Some(100.0),
        processed_items: Some(100),
        total_items: Some(100),
        processing_speed: Some(20.0),
        estimated_remaining: Some(0.0),
    })
}