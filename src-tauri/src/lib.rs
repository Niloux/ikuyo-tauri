// =====================================
// ikuyo-app Tauri 后端主入口
// 职责：负责应用启动流程、全局依赖初始化、Tauri插件与命令注册、主流程调度
// 主流程结构：
// 1. 环境识别
// 2. 日志系统初始化
// 3. 数据库连接与迁移
// 4. 配置加载
// 5. Worker 启动
// 6. 全局依赖注入
// 7. 主窗口事件注册
// 8. 命令注册与 Tauri 启动
// =====================================

mod commands;
mod config;
mod core;
mod error;
mod models;
mod repositories;
mod services;
mod types;
mod worker;

use dirs;
use librqbit::Session;
use once_cell::sync::OnceCell;
use sqlx::sqlite::SqlitePoolOptions;
use std::fs;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, SystemTime};
use tauri::Manager;
use tokio::sync::Notify;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*, registry};
// 补充命令注册相关 use 导入
use commands::{bangumi::*, crawler::*, download::*, subscription::*};

// 日志保留策略：只保留最近30天且最多30个日志文件
const LOG_KEEP_DAYS: u64 = 30;
const LOG_KEEP_MAX: usize = 30;

// ========== 日志系统初始化 ==========
fn cleanup_old_logs(log_dir: &Path) {
    if !log_dir.exists() {
        return;
    }
    let now = SystemTime::now();
    let mut log_files: Vec<_> = match fs::read_dir(log_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name();
                let name = name.to_string_lossy();
                name.starts_with("ikuyo.log")
            })
            .collect(),
        Err(_) => return,
    };
    // 按修改时间降序排序
    log_files.sort_by_key(|e| {
        std::cmp::Reverse(
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH),
        )
    });
    // 1. 超过最大数量的全部删除
    for entry in log_files.iter().skip(LOG_KEEP_MAX) {
        tracing::info!("删除过期日志文件: {:?}", entry.path());
        let _ = fs::remove_file(entry.path());
    }
    // 2. 超过保留天数的全部删除
    for entry in &log_files {
        if let Ok(meta) = entry.metadata() {
            if let Ok(modified) = meta.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age > Duration::from_secs(60 * 60 * 24 * LOG_KEEP_DAYS) {
                        tracing::info!("删除过期日志文件: {:?}", entry.path());
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }
}

static LOG_GUARD: OnceCell<tracing_appender::non_blocking::WorkerGuard> = OnceCell::new();

fn init_logging(log_path: &std::path::Path) {
    let log_dir = log_path.parent().unwrap();
    cleanup_old_logs(log_dir);
    if !log_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(log_dir) {
            tracing::error!("无法创建日志目录: {:?}", e);
            return;
        }
    }
    let file_appender =
        RollingFileAppender::new(Rotation::DAILY, log_dir, log_path.file_name().unwrap());
    let (non_blocking_file_appender, guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer().with_writer(non_blocking_file_appender).json();
    let console_layer = fmt::layer().with_writer(std::io::stdout).pretty();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();
    let _ = LOG_GUARD.set(guard);
    tracing::info!("日志系统已初始化，日志文件: {:?}", log_path);
}

// ========== 数据库连接与迁移 ==========
fn init_db(db_path: &std::path::Path) -> Arc<sqlx::SqlitePool> {
    let db_url = format!("sqlite:{}?mode=rwc", db_path.to_str().unwrap());
    let pool = tauri::async_runtime::block_on(async move {
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect(&db_url)
            .await
            .expect("failed to connect to database");
        // 执行数据库迁移
        if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
            tracing::error!("数据库迁移失败: {e}");
            panic!("数据库迁移失败: {e}");
        }
        pool
    });
    let pool_arc = Arc::new(pool);
    tracing::info!("数据库连接成功: {:?}", db_path);
    pool_arc
}

// ========== 配置加载 ==========
fn load_config() -> config::Config {
    config::Config::load()
}

// ========== Worker 启动 ==========
fn start_worker(
    pool_arc: Arc<sqlx::SqlitePool>,
    notify: Arc<Notify>,
    config: config::Config,
    exit_flag: Arc<AtomicBool>,
) -> Arc<worker::Worker> {
    let worker = Arc::new(worker::Worker::new(
        pool_arc.clone(),
        notify.clone(),
        config.clone(),
        Some(2),
        exit_flag.clone(),
    ));
    let worker_handle = Arc::clone(&worker);
    tauri::async_runtime::spawn(async move {
        worker_handle.run().await;
    });
    worker
}

// ========== 下载目录初始化 ==========
fn init_download_dir() -> std::path::PathBuf {
    let home_dir = dirs::home_dir().expect("无法获取用户主目录");
    let download_dir = home_dir.join("download");
    if !download_dir.exists() {
        std::fs::create_dir_all(&download_dir).expect("无法创建 download 目录");
    }
    download_dir
}

// ========== 主入口 ==========
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> crate::error::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            // 1. 环境识别
            let is_dev = cfg!(debug_assertions);
            let app_handle = app.handle().clone();
            let path_resolver = app_handle.path();

            // 2. 日志系统初始化
            let log_path = if is_dev {
                std::path::PathBuf::from("logs/ikuyo.log")
            } else {
                let app_data_dir = path_resolver
                    .app_data_dir()
                    .expect("failed to resolve app data dir");
                app_data_dir.join("logs/ikuyo.log")
            };
            init_logging(&log_path);
            tracing::info!("is_dev: {}", is_dev);

            // 3. 数据库连接与迁移
            let db_path = if is_dev {
                std::path::PathBuf::from("ikuyo.db")
            } else {
                let app_data_dir = path_resolver
                    .app_data_dir()
                    .expect("failed to resolve app data dir");
                if !app_data_dir.exists() {
                    std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
                }
                app_data_dir.join("ikuyo.db")
            };
            let pool_arc = init_db(&db_path);

            // ===== 下载服务初始化与自动恢复 =====
            let download_dir = init_download_dir();
            let session = tauri::async_runtime::block_on(Session::new(download_dir))
                .expect("session初始化失败");
            let download_service = Arc::new(services::download_service::DownloadService::new(
                pool_arc.clone(),
                session.clone(),
            ));
            // 自动恢复未完成任务和推送下载进度信息
            let ds_clone = download_service.clone();
            tauri::async_runtime::spawn(async move {
                let _ = ds_clone.restore_all_tasks_on_start().await;
                ds_clone.spawn_progress_notifier(app_handle.clone()).await;
            });

            // 4. 配置加载
            let config = load_config();

            // 5. Worker 启动前，批量将所有Running状态的任务标记为Failed
            {
                use crate::repositories::crawler_task::CrawlerTaskRepository;
                let repo = CrawlerTaskRepository::new(&pool_arc);
                let msg = "应用重启，任务中断";
                match tauri::async_runtime::block_on(async {
                    repo.mark_all_running_as_failed(msg).await
                }) {
                    Ok(n) => tracing::info!("已将{n}个Running任务标记为Failed"),
                    Err(e) => tracing::error!("批量标记Running任务为Failed失败: {e}"),
                }
            }

            // 6. Worker 启动
            let notify = Arc::new(Notify::new());
            let exit_flag = Arc::new(AtomicBool::new(false));
            let worker = start_worker(
                pool_arc.clone(),
                notify.clone(),
                config.clone(),
                exit_flag.clone(),
            );

            // 7. 全局依赖注入
            app.manage(pool_arc.clone());
            app.manage(config.clone());
            app.manage(notify.clone());
            app.manage(exit_flag.clone());
            app.manage(worker);
            app.manage(download_service);

            // 8. 主窗口事件注册
            let window = app
                .get_webview_window("main")
                .expect("main window not found");
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    exit_flag.store(true, Ordering::SeqCst);
                    // 数据库退出流程：异步执行，确保所有操作完成
                    let pool = pool_arc.clone();
                    tauri::async_runtime::spawn(async move {
                        tracing::info!("应用退出：执行PRAGMA wal_checkpoint(FULL)");
                        match sqlx::query("PRAGMA wal_checkpoint(FULL);")
                            .execute(pool.as_ref())
                            .await
                        {
                            Ok(res) => tracing::info!("wal_checkpoint执行成功: {:?}", res),
                            Err(e) => tracing::error!("wal_checkpoint执行失败: {}", e),
                        }
                        tracing::info!("应用退出：关闭数据库连接池");
                        pool.close().await;
                    });
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Bangumi commands
            get_calendar,
            get_subject,
            get_episodes,
            get_episode_availability,
            get_episode_resources,
            search_library,
            get_anime_resources,
            // Crawler commands
            create_crawler_task,
            get_crawler_task_status,
            list_crawler_tasks,
            get_crawler_task,
            cancel_crawler_task,
            delete_crawler_task,
            // Subscription commands
            subscribe,
            unsubscribe,
            get_subscriptions,
            check_subscription,
            get_all_subscription_ids,
            // Download commands
            start_download,
            pause_download,
            resume_download,
            remove_download,
            fetch_all_downloads,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
