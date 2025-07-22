use crate::{
    error::{AppError, OpenFileError}, models::DownloadTask, repositories::{base::Repository, download_task::DownloadTaskRepository},
    services::download_service::DownloadService, types::download::StartDownloadTask,
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{command, State};

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
pub async fn list_downloads(
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Vec<DownloadTask>, AppError> {
    let repo = DownloadTaskRepository::new(&pool);
    let tasks = repo.list(0, 0).await?;
    Ok(tasks)
}

#[command(rename_all = "snake_case")]
pub async fn get_download_path(
    download_service: State<'_, Arc<DownloadService>>,
    id: i64,
) -> Result<String, AppError> {
    let path = download_service.get_download_path(id).await?;
    Ok(path)
}

#[command(rename_all = "snake_case")]
pub fn open_file_path(path: String) -> Result<(), AppError> {
    // 使用 open crate 打开路径
    match open::that_detached(&path) {
        Ok(_) => {
            tracing::info!("打开文件成功: {}", path);
            Ok(())
        }
        Err(e) => {
            tracing::error!("打开文件失败: {}: {}", path, e);
            Err(AppError::OpenFile(OpenFileError::Failed(e.to_string())))
        }
    }
}

#[command(rename_all = "snake_case")]
pub async fn get_download_folder(
    download_service: State<'_, Arc<DownloadService>>,
) -> Result<String, AppError> {
    let folder = download_service.get_download_folder().await?;
    Ok(folder)
}