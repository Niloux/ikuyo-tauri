use crate::error::{AppError, DownloadTaskError};
use crate::models::{DownloadStatus, DownloadTask};
use crate::repositories::base::Repository;
use crate::repositories::download_task::DownloadTaskRepository;
use crate::types::download::{ProgressUpdate, StartDownloadTask};
use librqbit::api::TorrentIdOrHash;
use librqbit::{AddTorrent, AddTorrentOptions, Session};
use sqlx::SqlitePool;
use std::sync::Arc;
use futures_util::stream::StreamExt;
use tauri::Emitter;
use tokio::time::{interval, Duration};
use async_trait::async_trait;

#[async_trait]
pub trait ProgressSyncer {
    async fn sync_progress(&self, session: &Arc<Session>, task: &DownloadTask) -> Option<ProgressUpdate>;
}

pub struct DefaultProgressSyncer;
#[async_trait]
impl ProgressSyncer for DefaultProgressSyncer {
    async fn sync_progress(&self, session: &Arc<Session>, task: &DownloadTask) -> Option<ProgressUpdate> {
        DownloadService::sync_task_status_from_session(session, task.id.unwrap())
    }
}

#[async_trait]
pub trait StatusPusher {
    async fn push_status(&self, app_handle: &tauri::AppHandle, progress: &ProgressUpdate);
}

pub struct DefaultStatusPusher;
#[async_trait]
impl StatusPusher for DefaultStatusPusher {
    async fn push_status(&self, app_handle: &tauri::AppHandle, progress: &ProgressUpdate) {
        let _ = app_handle.emit("download_progress", progress);
    }
}

#[async_trait]
pub trait TaskUpdater {
    async fn update_task(&self, pool: &Arc<SqlitePool>, task: &DownloadTask, progress: &ProgressUpdate);
}

pub struct DefaultTaskUpdater;
#[async_trait]
impl TaskUpdater for DefaultTaskUpdater {
    async fn update_task(&self, pool: &Arc<SqlitePool>, task: &DownloadTask, progress: &ProgressUpdate) {
        let repo = DownloadTaskRepository::new(pool);
        if let Ok(Some(mut task_to_update)) = repo.get_by_id(task.id.unwrap()).await {
            if task_to_update.status != progress.status.clone() {
                task_to_update.status = progress.status.clone();
                task_to_update.total_size = progress.total_bytes as i64;
                task_to_update.error_msg = progress.error_msg.clone();
                task_to_update.updated_at = DownloadService::get_current_timestamp();
                match repo.update(&task_to_update).await {
                    Ok(_) => tracing::debug!(
                        "数据库状态更新成功: task_id={}, status={:?}",
                        task.id.unwrap(),
                        task_to_update.status
                    ),
                    Err(e) => tracing::error!(
                        "数据库状态更新失败: task_id={}, error={}",
                        task.id.unwrap(),
                        e
                    ),
                }
            }
        }
    }
}

pub struct DownloadService {
    pub pool: Arc<SqlitePool>,
    pub session: Arc<Session>,
}

impl DownloadService {
    /// 构造函数，便于统一初始化
    pub fn new(pool: Arc<SqlitePool>, session: Arc<Session>) -> Self {
        Self { pool, session }
    }

    pub async fn start_new_download(&self, task: StartDownloadTask) -> Result<i64, AppError> {
        let add = AddTorrent::Url(task.magnet_url.clone().into());
        // 如果save_path不为空，则设置output_folder
        // output表示该下载任务的保存路径,会覆盖session(path)的设置
        let opts = AddTorrentOptions {
            output_folder: task.save_path.clone(),
            // 性能优化配置
            paused: false,    // 立即开始下载
            overwrite: false, // 不覆盖已存在的文件
            ..Default::default()
        };
        let resp = self
            .session
            .add_torrent(add, Some(opts))
            .await
            .map_err(|e| AppError::DownloadTask(DownloadTaskError::Failed(e.to_string())))?;
        let handle = match resp.into_handle() {
            Some(h) => h,
            None => return Err(AppError::Unknown("添加下载任务失败".to_string())),
        };
        // 数据库插入 download_task，保存 handle.id() 作为任务id
        let now = Self::get_current_timestamp();
        let task = DownloadTask {
            id: Some(handle.id() as i64),
            magnet_url: task.magnet_url,
            save_path: task.save_path,
            title: task.title,
            status: DownloadStatus::Pending,
            bangumi_id: task.bangumi_id,
            resource_id: task.resource_id,
            episode_number: task.episode_number,
            name: task.name,
            name_cn: task.name_cn,
            cover: task.cover,
            total_size: task.total_size,
            created_at: now,
            updated_at: now,
            error_msg: None,
        };
        let repo = self.repo();
        repo.create(&task).await?;
        Ok(handle.id() as i64)
    }

    pub async fn pause_download(&self, id: i64) -> Result<(), AppError> {
        let handle = self
            .session
            .get(TorrentIdOrHash::Id(id as usize))
            .ok_or(AppError::Domain(crate::error::DomainError::NotFound {
                resource_type: "download_task".to_string(),
                resource_id: id,
            }))?;
        self.session
            .pause(&handle)
            .await
            .map_err(|e| AppError::DownloadTask(DownloadTaskError::Failed(e.to_string())))?;
        // 等待 peer/写入线程安全退出
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        // 数据库状态更新
        let repo = self.repo();
        if let Some(mut task) = repo.get_by_id(id).await? {
            task.status = DownloadStatus::Paused;
            task.updated_at = Self::get_current_timestamp();
            match repo.update(&task).await {
                Ok(_) => tracing::debug!(
                    "数据库状态更新成功: task_id={}, status={:?}",
                    id,
                    task.status
                ),
                Err(e) => tracing::error!("数据库状态更新失败: task_id={}, error={}", id, e),
            }
        }
        Ok(())
    }

    pub async fn resume_download(&self, id: i64) -> Result<(), AppError> {
        let handle = self
            .session
            .get(TorrentIdOrHash::Id(id as usize))
            .ok_or(AppError::Domain(crate::error::DomainError::NotFound {
                resource_type: "download_task".to_string(),
                resource_id: id,
            }))?;
        self.session
            .unpause(&handle)
            .await
            .map_err(|e| AppError::DownloadTask(DownloadTaskError::Failed(e.to_string())))?;
        // 数据库状态更新
        let repo = self.repo();
        if let Some(mut task) = repo.get_by_id(id).await? {
            task.status = DownloadStatus::Downloading;
            task.updated_at = Self::get_current_timestamp();
            match repo.update(&task).await {
                Ok(_) => tracing::debug!(
                    "数据库状态更新成功: task_id={}, status={:?}",
                    id,
                    task.status
                ),
                Err(e) => tracing::error!("数据库状态更新失败: task_id={}, error={}", id, e),
            }
        }
        Ok(())
    }

    pub async fn remove_download(&self, id: i64, delete_files: bool) -> Result<(), AppError> {
        self.session
            .delete(TorrentIdOrHash::Id(id as usize), delete_files)
            .await
            .map_err(|e| AppError::DownloadTask(DownloadTaskError::Failed(e.to_string())))?;
        // 等待 peer/写入线程安全退出
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        // 数据库删除任务
        let repo = self.repo();
        repo.delete(id).await?;
        Ok(())
    }

    /// 从session同步任务状态
    fn sync_task_status_from_session(session: &Arc<Session>, id: i64) -> Option<ProgressUpdate> {
        session.get(TorrentIdOrHash::Id(id as usize)).map(|h| {
            let stats = h.stats();
            let total_bytes = stats.total_bytes;
            let speed = stats
                .live
                .as_ref()
                .map(|l| l.download_speed.mbps as f64)
                .unwrap_or(0.0);
            let time_remaining = stats
                .live
                .as_ref()
                .and_then(|l| l.time_remaining.as_ref())
                .map(|d| d.to_string());

            let state_str = stats.state.to_string();
            let status = if stats.finished {
                DownloadStatus::Completed
            } else {
                match state_str.as_str() {
                    "error" => DownloadStatus::Failed,
                    "paused" => DownloadStatus::Paused,
                    "initializing" => DownloadStatus::Pending,
                    "live" => DownloadStatus::Downloading,
                    _ => DownloadStatus::Downloading,
                }
            };
            let error_msg = if state_str == "error" {
                Some(stats.error.unwrap_or_default())
            } else {
                None
            };
            ProgressUpdate {
                id,
                total_bytes,
                progress: if total_bytes > 0 {
                    stats.progress_bytes as f64 / total_bytes as f64
                } else {
                    0.0
                },
                speed,
                time_remaining,
                status,
                error_msg,
            }
        })
    }

    /// 同步session状态到数据库与前端
    pub async fn sync_rtbit(self: Arc<Self>, app_handle: tauri::AppHandle, is_active: Arc<std::sync::atomic::AtomicBool>) {
        let pool = self.pool.clone();
        let session = self.session.clone();
        tauri::async_runtime::spawn(async move {
            let mut ticker = interval(Duration::from_secs(1));
            loop {
                ticker.tick().await;
                if !is_active.load(std::sync::atomic::Ordering::SeqCst) {
                    continue;
                }
                let tasks = Self::all_progress_tasks(&pool)
                    .await
                    .unwrap_or_else(|_| vec![]);
                let mut futures = Vec::new();
                for task in tasks {
                    let session = session.clone();
                    let app_handle = app_handle.clone();
                    let pool = pool.clone();
                    futures.push(async move {
                        let progress_syncer = DefaultProgressSyncer;
                        let status_pusher = DefaultStatusPusher;
                        let task_updater = DefaultTaskUpdater;

                        if let Some(progress) =
                            progress_syncer.sync_progress(&session, &task).await
                        {
                            // 前端同步
                            status_pusher.push_status(&app_handle, &progress).await;
                            // 数据库状态更新
                            task_updater.update_task(&pool, &task, &progress).await;
                        }
                    });
                }
                // 限制最大并发数为8
                futures_util::stream::iter(futures)
                    .buffer_unordered(8)
                    .for_each(|_| async {})
                    .await;
            }
        });
    }

    /// 获取当前时间戳
    fn get_current_timestamp() -> i64 {
        chrono::Utc::now().timestamp()
    }

    /// 获取所有可活跃的下载任务对象
    async fn all_progress_tasks(pool: &Arc<SqlitePool>) -> Result<Vec<DownloadTask>, AppError> {
        let repo = DownloadTaskRepository::new(pool);
        let tasks = repo.list(0, 0).await?;
        // 只返回未完成的任务对象
        Ok(tasks
            .into_iter()
            .filter(|task| {
                // 过滤掉已完成任务
                !matches!(
                    task.status,
                    // DownloadStatus::Completed | DownloadStatus::Paused
                    DownloadStatus::Completed
                )
            })
            .collect())
    }

    /// 获取 DownloadTaskRepository 实例，便于后续 Mock/切换实现
    fn repo(&self) -> DownloadTaskRepository {
        DownloadTaskRepository::new(&self.pool)
    }
}
