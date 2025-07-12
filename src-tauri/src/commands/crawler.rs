use crate::{
    error::{AppError, TaskError},
    models::{CrawlerTask},
    repositories::{base::Repository, crawler_task::CrawlerTaskRepository},
    types::crawler::{CrawlerTaskCreate, TaskResponse},
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{command, State};
use tokio::sync::Notify;

fn convert_to_response(task: CrawlerTask) -> TaskResponse {
    TaskResponse {
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
    }
}

#[command(rename_all = "snake_case")]
pub async fn create_crawler_task(
    task: CrawlerTaskCreate,
    pool: State<'_, Arc<SqlitePool>>,
    notify: State<'_, Arc<Notify>>,
) -> Result<TaskResponse, AppError> {
    crate::services::crawler_service::CrawlerService::create_task(
        pool.inner().clone(),
        notify.inner().clone(),
        task,
    ).await
}

#[command(rename_all = "snake_case")]
pub async fn get_crawler_task_status(
    task_id: i64,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<TaskResponse, AppError> {
    let repo = CrawlerTaskRepository::new(&pool);
    let task = repo.get_by_id(task_id).await?;

    match task {
        Some(task) => Ok(convert_to_response(task)),
        None => Err(AppError::Task(TaskError::Failed("任务不存在".to_string()))),
    }
}

#[command(rename_all = "snake_case")]
pub async fn list_crawler_tasks(
    page: Option<i64>,
    page_size: Option<i64>,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Vec<TaskResponse>, AppError> {
    let repo = CrawlerTaskRepository::new(&pool);
    let current_page = page.unwrap_or(1);
    let current_page_size = page_size.unwrap_or(10);
    let offset = (current_page - 1) * current_page_size;

    let tasks = repo
        .list(current_page_size, offset)
        .await?;
    let responses: Vec<TaskResponse> = tasks.into_iter().map(convert_to_response).collect();

    Ok(responses)
}

#[command(rename_all = "snake_case")]
pub async fn get_crawler_task(
    task_id: i64,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<TaskResponse, AppError> {
    tracing::info!("Getting crawler task ID: {}", task_id);

    let repo = CrawlerTaskRepository::new(&pool);
    let task = repo.get_by_id(task_id).await?;

    match task {
        Some(task) => Ok(convert_to_response(task)),
        None => Err(AppError::Task(TaskError::Failed("任务不存在".to_string()))),
    }
}

#[command(rename_all = "snake_case")]
pub async fn cancel_crawler_task(
    task_id: i64,
    pool: State<'_, Arc<SqlitePool>>,
    worker: State<'_, Arc<crate::worker::Worker>>,
) -> Result<TaskResponse, AppError> {
    crate::services::crawler_service::CrawlerService::cancel_task(
        pool.inner().clone(),
        worker.inner().clone(),
        task_id,
    ).await
}

#[command(rename_all = "snake_case")]
pub async fn delete_crawler_task(
    task_id: i64,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<(), AppError> {
    tracing::info!("Deleting crawler task ID: {}", task_id);

    let repo = CrawlerTaskRepository::new(&pool);
    repo.delete(task_id).await?;

    Ok(())
}
