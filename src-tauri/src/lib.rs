mod commands;
mod db;
mod error;
mod models;
mod types;
mod services;
mod repositories;
mod config;
mod core;
use std::sync::Arc;
use tokio::sync::Notify;
mod worker;

use commands::{ bangumi::*, crawler::*, scheduler::*, subscription::*, };
use tauri::async_runtime::block_on;
use sqlx::SqlitePool;
use crate::error::Result;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    let config = config::Config::load();
    let pool: SqlitePool = block_on(crate::db::init_pool(&config))?;
    let pool_arc = Arc::new(pool);
    let notify = Arc::new(Notify::new());

    // 启动worker池（单worker）
    let worker = worker::Worker::new(pool_arc.clone(), notify.clone(), None);
    tauri::async_runtime::spawn(async move {
        worker.run().await;
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(pool_arc)
        .manage(config) // 将 config 也注入
        .manage(notify)
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
