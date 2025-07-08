use crate::types::crawler::{CrawlerTaskCreate, TaskResponse, CrawlerTaskType, CrawlerTaskStatus};
use crate::repositories::crawler_task::CrawlerTaskRepository;
use crate::models::CrawlerTask;
use sqlx::SqlitePool;
use tauri::State;

fn convert_to_response(task: CrawlerTask) -> TaskResponse {
    TaskResponse {
        id: task.id.unwrap_or(0),
        task_type: match task.task_type {
            crate::models::CrawlerTaskType::Manual => CrawlerTaskType::Manual,
            crate::models::CrawlerTaskType::Scheduled => CrawlerTaskType::Schedule,
        },
        status: match task.status {
            crate::models::CrawlerTaskStatus::Pending => CrawlerTaskStatus::Pending,
            crate::models::CrawlerTaskStatus::Running => CrawlerTaskStatus::Running,
            crate::models::CrawlerTaskStatus::Completed => CrawlerTaskStatus::Completed,
            crate::models::CrawlerTaskStatus::Failed => CrawlerTaskStatus::Failed,
            crate::models::CrawlerTaskStatus::Cancelled => CrawlerTaskStatus::Cancelled,
        },
        parameters: task.parameters,
        result_summary: task.result_summary,
        created_at: task.created_at,
        started_at: task.started_at,
        completed_at: task.completed_at,
        error_message: task.error_message,
        percentage: task.percentage,
        processed_items: task.processed_items,
        total_items: task.total_items,
        processing_speed: task.processing_speed,
        estimated_remaining: task.estimated_remaining,
    }
}

#[tauri::command]
pub async fn create_crawler_task(
    task: CrawlerTaskCreate,
    pool: State<'_, SqlitePool>,
) -> Result<TaskResponse, String> {
    tracing::info!("Creating crawler task: {:?}", task);
    
    let repo = CrawlerTaskRepository::new(&*pool);
    let parameters = serde_json::to_string(&task).unwrap_or_default();
    let current_time = chrono::Utc::now().timestamp_millis();
    
    let new_task = CrawlerTask {
        id: None,
        task_type: crate::models::CrawlerTaskType::Manual,
        status: crate::models::CrawlerTaskStatus::Pending,
        parameters: Some(parameters),
        result_summary: None,
        created_at: Some(current_time),
        started_at: None,
        completed_at: None,
        error_message: None,
        percentage: Some(0.0),
        processed_items: Some(0),
        total_items: Some(100), // 默认值，实际执行时会更新
        processing_speed: None,
        estimated_remaining: None,
    };
    
    let task_id = repo.create(&new_task).await.map_err(|e| e.to_string())?;
    
    let created_task = repo.get_by_id(task_id).await.map_err(|e| e.to_string())?;
    match created_task {
        Some(task) => Ok(convert_to_response(task)),
        None => Err("任务创建失败".to_string()),
    }
}

#[tauri::command]
pub async fn get_crawler_task_status(
    task_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<TaskResponse, String> {
    tracing::info!("Getting status for crawler task ID: {}", task_id);
    
    let repo = CrawlerTaskRepository::new(&*pool);
    let task = repo.get_by_id(task_id).await.map_err(|e| e.to_string())?;
    
    match task {
        Some(task) => Ok(convert_to_response(task)),
        None => Err("任务不存在".to_string()),
    }
}

#[tauri::command]
pub async fn list_crawler_tasks(
    page: Option<i64>,
    page_size: Option<i64>,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<TaskResponse>, String> {
    tracing::info!("Listing crawler tasks with page: {:?}, page_size: {:?}", page, page_size);
    
    let repo = CrawlerTaskRepository::new(&*pool);
    let current_page = page.unwrap_or(1);
    let current_page_size = page_size.unwrap_or(10);
    let offset = (current_page - 1) * current_page_size;
    
    let tasks = repo.list(current_page_size, offset).await.map_err(|e| e.to_string())?;
    let responses: Vec<TaskResponse> = tasks.into_iter().map(convert_to_response).collect();
    
    Ok(responses)
}

#[tauri::command]
pub async fn get_crawler_task(
    task_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<TaskResponse, String> {
    tracing::info!("Getting crawler task ID: {}", task_id);
    
    let repo = CrawlerTaskRepository::new(&*pool);
    let task = repo.get_by_id(task_id).await.map_err(|e| e.to_string())?;
    
    match task {
        Some(task) => Ok(convert_to_response(task)),
        None => Err("任务不存在".to_string()),
    }
}

#[tauri::command]
pub async fn cancel_crawler_task(
    task_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<TaskResponse, String> {
    tracing::info!("Cancelling crawler task ID: {}", task_id);
    
    let repo = CrawlerTaskRepository::new(&*pool);
    let task = repo.get_by_id(task_id).await.map_err(|e| e.to_string())?;
    
    match task {
        Some(mut task) => {
            // 只有Pending或Running状态的任务可以取消
            match task.status {
                crate::models::CrawlerTaskStatus::Pending | crate::models::CrawlerTaskStatus::Running => {
                    task.status = crate::models::CrawlerTaskStatus::Cancelled;
                    task.completed_at = Some(chrono::Utc::now().timestamp_millis());
                    repo.update(&task).await.map_err(|e| e.to_string())?;
                    Ok(convert_to_response(task))
                }
                _ => Err("任务无法取消，当前状态不允许取消操作".to_string()),
            }
        }
        None => Err("任务不存在".to_string()),
    }
}

#[tauri::command]
pub async fn delete_crawler_task(
    task_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    tracing::info!("Deleting crawler task ID: {}", task_id);
    
    let repo = CrawlerTaskRepository::new(&*pool);
    repo.delete(task_id).await.map_err(|e| e.to_string())?;
    
    Ok(())
}