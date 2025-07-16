mod commands;
mod config;
mod core;
mod error;
mod models;
mod repositories;
mod services;
mod types;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::sync::Notify;
mod worker;

use crate::error::Result;
use commands::{bangumi::*, crawler::*, subscription::*};
use once_cell::sync::OnceCell;
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use tauri::Manager;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*, registry};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, Duration};

const SCHEMA_SQL: &str = include_str!("../schema.sql");
// 日志保留策略：只保留最近30天且最多30个日志文件
const LOG_KEEP_DAYS: u64 = 30;
const LOG_KEEP_MAX: usize = 30;

/// 清理日志目录中过期和超量的日志文件
fn cleanup_old_logs(log_dir: &Path) {
    if !log_dir.exists() {
        return;
    }
    let now = SystemTime::now();
    let mut log_files: Vec<_> = match fs::read_dir(log_dir) {
        Ok(rd) => rd.filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name();
                let name = name.to_string_lossy();
                name.starts_with("ikuyo.log")
            })
            .collect(),
        Err(_) => return,
    };
    // 按修改时间降序排序
    log_files.sort_by_key(|e| std::cmp::Reverse(e.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH)));
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
                    if age > Duration::from_secs(60*60*24*LOG_KEEP_DAYS) {
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
    // 日志清理：只保留最近30天且最多30个日志文件
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            // 环境识别
            let is_dev = cfg!(debug_assertions);
            let app_handle = app.handle().clone();
            let path_resolver = app_handle.path();
            // 日志路径
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

            // 数据库路径
            let db_path = if is_dev {
                std::path::PathBuf::from("ikuyo.db")
            } else {
                let app_data_dir = path_resolver
                    .app_data_dir()
                    .expect("failed to resolve app data dir");
                if !app_data_dir.exists() {
                    std::fs::create_dir_all(&app_data_dir)
                        .expect("failed to create app data dir");
                }
                app_data_dir.join("ikuyo.db")
            };

            // 数据库初始化
            if !db_path.exists() {
                tracing::info!("数据库不存在，自动初始化: {:?}", db_path);
                // 直接用SCHEMA_SQL执行建表
                let db_url = format!("sqlite:{}?mode=rwc", db_path.to_str().unwrap());
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let pool = SqlitePool::connect(&db_url)
                        .await
                        .expect("failed to connect to database");
                    for stmt in SCHEMA_SQL.split(';') {
                        let sql = stmt.trim();
                        if !sql.is_empty() {
                            sqlx::query(sql).execute(&pool).await.expect("schema.sql执行失败");
                        }
                    }
                });
                tracing::info!("数据库初始化完成: {:?}", db_path);
            }

            // 连接数据库
            let db_url = format!("sqlite:{}?mode=rwc", db_path.to_str().unwrap());
            let pool = tauri::async_runtime::block_on(async move {
                SqlitePoolOptions::new()
                    .max_connections(8)
                    .connect(&db_url)
                    .await
                    .expect("failed to connect to database")
            });
            let pool_arc = Arc::new(pool);
            tracing::info!("数据库连接成功: {:?}", db_path);

            // 启动worker前，批量将所有Running状态的任务标记为Failed
            {
                use crate::repositories::crawler_task::CrawlerTaskRepository;
                let repo = CrawlerTaskRepository::new(&pool_arc);
                let msg = "应用重启，任务中断";
                match tauri::async_runtime::block_on(async { repo.mark_all_running_as_failed(msg).await }) {
                    Ok(n) => tracing::info!("已将{n}个Running任务标记为Failed"),
                    Err(e) => tracing::error!("批量标记Running任务为Failed失败: {e}"),
                }
            }
            // 3. 设置并启动后台工作者
            let config = config::Config::load();
            let notify = Arc::new(Notify::new());
            let exit_flag = Arc::new(AtomicBool::new(false));
            let worker = Arc::new(worker::Worker::new(pool_arc.clone(), notify.clone(), config.clone(), None, exit_flag.clone()));
            let worker_handle = Arc::clone(&worker);
            tauri::async_runtime::spawn(async move {
                worker_handle.run().await;
            });

            // 4. 将所有状态注入Tauri
            app.manage(pool_arc.clone());
            app.manage(config.clone());
            app.manage(notify.clone());
            app.manage(exit_flag.clone());
            // 新增：注入 Worker
            app.manage(worker);

            // 注册退出钩子，预留退出流程入口
            let window = app.get_webview_window("main").expect("main window not found");
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    exit_flag.store(true, Ordering::SeqCst);
                    // 数据库退出流程：异步执行，确保所有操作完成
                    let pool = pool_arc.clone();
                    tauri::async_runtime::spawn(async move {
                        tracing::info!("应用退出：执行PRAGMA wal_checkpoint(FULL)");
                        match sqlx::query("PRAGMA wal_checkpoint(FULL);").execute(pool.as_ref()).await {
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
