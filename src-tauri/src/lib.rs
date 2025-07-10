mod commands;
mod config;
mod core;
mod error;
mod models;
mod repositories;
mod services;
mod types;
use std::sync::Arc;
use tokio::sync::Notify;
mod worker;

use crate::error::Result;
use commands::{bangumi::*, crawler::*, scheduler::*, subscription::*};
use once_cell::sync::OnceCell;
use sqlx::SqlitePool;
use tauri::path::BaseDirectory;
use tauri::Manager;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*, registry};

static LOG_GUARD: OnceCell<tracing_appender::non_blocking::WorkerGuard> = OnceCell::new();

fn init_logging(log_path: &std::path::Path) {
    let log_dir = log_path.parent().unwrap();
    if !log_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(log_dir) {
            eprintln!("无法创建日志目录: {:?}", e);
            return;
        }
    }
    let file_appender =
        RollingFileAppender::new(Rotation::DAILY, log_dir, log_path.file_name().unwrap());
    let (non_blocking_file_appender, guard) = tracing_appender::non_blocking(file_appender);
    let console_layer = fmt::layer().with_writer(std::io::stdout).pretty();
    let file_layer = fmt::layer().with_writer(non_blocking_file_appender).json();
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
    // 在编译时嵌入迁移文件
    let migrator = sqlx::migrate!("./migrations");

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            // 1. 加载配置
            let config = config::Config::load();
            let app_handle = app.handle().clone(); // CLONE THE HANDLE HERE

            // 日志初始化
            let path_resolver = app_handle.path();
            let app_data_dir = path_resolver
                .app_data_dir()
                .expect("failed to resolve app data dir");
            let log_dir = app_data_dir.join("logs");
            let log_path = log_dir.join("ikuyo.log");
            init_logging(&log_path);

            // 2. 设置数据库（首次运行时复制策略）
            let pool = tauri::async_runtime::block_on(async move {
                // app_handle is moved into this block
                let path_resolver = app_handle.path();
                let is_dev = std::env::var("IKUYO_ENV").unwrap_or_default() == "dev";
                let (db_path, db_desc) = if is_dev {
                    // 开发环境：数据库放在当前工作目录/ikuyo.db，要求在src-tauri目录下运行
                    let db_path = std::path::PathBuf::from("ikuyo.db");
                    tracing::info!("当前工作目录: {:?}", std::env::current_dir().unwrap());
                    (db_path, "开发环境 ikuyo.db（请在src-tauri目录下运行）")
                } else {
                    // 生产环境：数据库放在 app_data_dir/ikuyo.db
                    let app_data_dir = path_resolver
                        .app_data_dir()
                        .expect("failed to resolve app data dir");
                    if !app_data_dir.exists() {
                        std::fs::create_dir_all(&app_data_dir)
                            .expect("failed to create app data dir");
                    }
                    (
                        app_data_dir.join("ikuyo.db"),
                        "生产环境 app_data_dir/ikuyo.db",
                    )
                };

                if !db_path.exists() {
                    if is_dev {
                        tracing::info!("开发环境自动新建空库并迁移，路径: {:?}", db_path);
                        // 空文件自动由sqlite创建，无需手动touch
                    } else {
                        // 生产环境：必须有模板db
                        let resource_db_path = path_resolver
                            .resolve("ikuyo.db", BaseDirectory::Resource)
                            .expect("ikuyo.db resource not found in production!");
                        std::fs::copy(resource_db_path, &db_path)
                            .expect("failed to copy database file");
                    }
                }

                tracing::info!("数据库路径: {:?} ({})", db_path, db_desc);
                let db_url = format!(
                    "sqlite:{}",
                    db_path
                        .to_str()
                        .expect("failed to convert db path to string")
                );

                let pool = SqlitePool::connect(&db_url)
                    .await
                    .expect("failed to connect to database");
                migrator
                    .run(&pool)
                    .await
                    .expect("failed to run database migrations");
                Ok::<SqlitePool, anyhow::Error>(pool)
            })?;
            let pool_arc = Arc::new(pool);

            // 3. 设置并启动后台工作者
            let notify = Arc::new(Notify::new());
            let worker =
                worker::Worker::new(pool_arc.clone(), notify.clone(), config.clone(), None);
            tauri::async_runtime::spawn(async move {
                worker.run().await;
            });

            // 4. 将所有状态注入Tauri
            app.manage(pool_arc);
            app.manage(config);
            app.manage(notify);

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
            // Scheduler commands
            create_scheduled_job,
            update_scheduled_job,
            get_scheduled_jobs,
            get_scheduled_job,
            delete_scheduled_job,
            toggle_scheduled_job,
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
