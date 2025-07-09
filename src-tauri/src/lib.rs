mod commands;
mod config;
mod core;
mod db;
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
use sqlx::SqlitePool;
use tauri::path::BaseDirectory;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    // 在编译时嵌入迁移文件
    let migrator = sqlx::migrate!("./migrations");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            // 1. 加载配置
            let config = config::Config::load();
            let app_handle = app.handle().clone(); // CLONE THE HANDLE HERE

            // 2. 设置数据库（首次运行时复制策略）
            let pool = tauri::async_runtime::block_on(async move {
                // app_handle is moved into this block
                let path_resolver = app_handle.path();
                let app_data_dir = path_resolver
                    .app_data_dir()
                    .expect("failed to resolve app data dir");
                if !app_data_dir.exists() {
                    std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
                }

                let writable_db_path = app_data_dir.join("ikuyo.db");

                // 如果可写目录中不存在数据库文件，则从资源中复制
                if !writable_db_path.exists() {
                    let resource_db_path = path_resolver
                        .resolve("ikuyo.db", BaseDirectory::Resource)
                        .expect("failed to resolve resource");

                    std::fs::copy(resource_db_path, &writable_db_path)
                        .expect("failed to copy database file");
                }

                let db_url = format!(
                    "sqlite:{}",
                    writable_db_path
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
            let worker = worker::Worker::new(pool_arc.clone(), notify.clone(), None);
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
