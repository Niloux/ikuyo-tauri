use crate::error::{AppError, DownloadTaskError};
use crate::models::{DownloadStatus, DownloadTask};
use crate::repositories::base::Repository;
use crate::repositories::download_task::DownloadTaskRepository;
use crate::types::download::{ProgressUpdate, StartDownloadTask};
use librqbit::api::TorrentIdOrHash;
use librqbit::{AddTorrent, AddTorrentOptions, Session};
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::Emitter;
use tokio::time::{interval, Duration};

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
        let now = chrono::Utc::now().timestamp();
        let task = DownloadTask {
            id: Some(handle.id() as i64),
            magnet_url: task.magnet_url,
            save_path: task.save_path,
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
        let repo = DownloadTaskRepository::new(&self.pool);
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
        // 数据库状态更新
        let repo = DownloadTaskRepository::new(&self.pool);
        if let Some(mut task) = repo.get_by_id(id).await? {
            task.status = DownloadStatus::Paused;
            task.updated_at = chrono::Utc::now().timestamp();
            match repo.update(&task).await {
                Ok(_) => tracing::info!("更新数据库成功: id={}, status={:?}", id, task.status),
                Err(e) => tracing::error!("更新数据库失败: id={}, err={}", id, e),
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
        let repo = DownloadTaskRepository::new(&self.pool);
        if let Some(mut task) = repo.get_by_id(id).await? {
            task.status = DownloadStatus::Downloading;
            task.updated_at = chrono::Utc::now().timestamp();
            match repo.update(&task).await {
                Ok(_) => tracing::info!("更新数据库成功: id={}, status={:?}", id, task.status),
                Err(e) => tracing::error!("更新数据库失败: id={}, err={}", id, e),
            }
        }
        Ok(())
    }

    pub async fn remove_download(&self, id: i64, delete_files: bool) -> Result<(), AppError> {
        self.session
            .delete(TorrentIdOrHash::Id(id as usize), delete_files)
            .await
            .map_err(|e| AppError::DownloadTask(DownloadTaskError::Failed(e.to_string())))?;
        // 数据库状态更新
        let repo = DownloadTaskRepository::new(&self.pool);
        if let Some(mut task) = repo.get_by_id(id).await? {
            task.status = DownloadStatus::Deleted;
            task.updated_at = chrono::Utc::now().timestamp();
            match repo.update(&task).await {
                Ok(_) => tracing::info!("更新数据库成功: id={}, status={:?}", id, task.status),
                Err(e) => tracing::error!("更新数据库失败: id={}, err={}", id, e),
            }
        }
        Ok(())
    }

    pub fn get_progress(&self, id: i64) -> Option<ProgressUpdate> {
        self.session.get(TorrentIdOrHash::Id(id as usize)).map(|h| {
            let stats = h.stats();
            // tracing::info!("stats: {:?}", stats);
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
            let status = if stats.finished {
                DownloadStatus::Completed
            } else {
                match stats.state.to_string().as_str() {
                    "error" => DownloadStatus::Failed,
                    "paused" => DownloadStatus::Paused,
                    "initializing" => DownloadStatus::Pending,
                    "live" => DownloadStatus::Downloading,
                    _ => DownloadStatus::Downloading,
                }
            };
            let error_msg = if stats.state.to_string() == "error" {
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

    /// 启动进度推送定时器，每秒推送所有任务进度到前端
    pub async fn spawn_progress_notifier(self: Arc<Self>, app_handle: tauri::AppHandle) {
        let pool = self.pool.clone();
        tauri::async_runtime::spawn(async move {
            let mut ticker = interval(Duration::from_secs(1));
            loop {
                ticker.tick().await;
                let ids = Self::all_task_ids(&pool).await;
                for id in ids {
                    if let Some(progress) = self.get_progress(id) {
                        let _ = app_handle.emit("download_progress", &progress);

                        // 只用 progress.status 作为权威状态
                        let repo = DownloadTaskRepository::new(&pool);
                        if let Ok(Some(mut task)) = repo.get_by_id(id).await {
                            if task.status != progress.status {
                                task.status = progress.status;
                                task.total_size = progress.total_bytes as i64;
                                task.error_msg = progress.error_msg;
                                task.updated_at = chrono::Utc::now().timestamp();
                                match repo.update(&task).await {
                                    Ok(_) => tracing::info!("更新数据库成功: id={}, status={:?}", id, task.status),
                                    Err(e) => tracing::error!("更新数据库失败: id={}, err={}", id, e),
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    /// 服务启动时自动恢复所有未完成任务
    pub async fn restore_all_tasks_on_start(&self) -> Result<(), AppError> {
        tracing::info!("正在恢复所有未完成任务");
        let repo = DownloadTaskRepository::new(&self.pool);
        tracing::info!("查询所有未完成任务");
        let tasks = repo
            .get_all_resumable_tasks()
            .await
            .map_err(|e| {
                tracing::error!("get_all_resumable_tasks 查询失败: {}", e);
                AppError::DownloadTask(DownloadTaskError::Failed(e.to_string()))
            })?;
        for task in tasks {
            tracing::info!("恢复任务: {:?}", task.id);
            // 只恢复有 magnet_url 的任务
            if !task.magnet_url.is_empty() {
                let add = AddTorrent::Url(task.magnet_url.clone().into());
                let opts = AddTorrentOptions {
                    paused: false,
                    overwrite: true,
                    output_folder: task.save_path,
                    preferred_id: Some(task.id.unwrap() as usize),
                    ..Default::default()
                };
                // 忽略已存在/重复的错误，保证幂等
                let _ = self.session.add_torrent(add, Some(opts)).await;
            }
        }
        Ok(())
    }

    /// 获取所有任务id（通过数据库查询所有未完成任务）
    async fn all_task_ids(pool: &Arc<SqlitePool>) -> Vec<i64> {
        let repo = DownloadTaskRepository::new(pool);
        match repo.get_all_resumable_tasks().await {
            Ok(tasks) => tasks.into_iter().filter_map(|t| t.id).collect(),
            Err(_) => vec![],
        }
    }
}
