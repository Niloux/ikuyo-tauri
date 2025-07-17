use crate::{
    error::AppError,
    models::DownloadTask,
    types::download::StartDownloadTask,
    services::download_service::DownloadService,
    repositories::download_task::DownloadTaskRepository,
};
use std::sync::Arc;
use tauri::{command, State};
use sqlx::SqlitePool;

#[command(rename_all = "snake_case")]
pub async fn start_download(
    download_service: State<'_, Arc<DownloadService>>,
    task: StartDownloadTask,
) -> Result<i64, AppError> {
    let id = download_service.start_new_download(task).await?;
    Ok(id)
}

#[command(rename_all = "snake_case")]
pub async fn pause_download(
    download_service: State<'_, Arc<DownloadService>>,
    id: i64,
) -> Result<(), AppError> {
    download_service.pause_download(id).await?;
    Ok(())
}

#[command(rename_all = "snake_case")]
pub async fn resume_download(
    download_service: State<'_, Arc<DownloadService>>,
    id: i64,
) -> Result<(), AppError> {
    download_service.resume_download(id).await?;
    Ok(())
}

#[command(rename_all = "snake_case")]
pub async fn remove_download(
    download_service: State<'_, Arc<DownloadService>>,
    id: i64,
    delete_files: bool,
) -> Result<(), AppError> {
    // 如果delete_files为true，则删除文件，否则只删除torrent
    download_service.remove_download(id, delete_files).await?;
    Ok(())
}

#[command(rename_all = "snake_case")]
pub async fn fetch_all_downloads(
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Vec<DownloadTask>, AppError> {
    // fetch_all_downloads 用于前端fetch_all_downloads, 返回所有未删除的任务
    let repo = DownloadTaskRepository::new(&pool);
    let tasks = repo.fetch_all_downloads().await?;
    Ok(tasks)
}