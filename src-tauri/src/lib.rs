mod commands;
mod db;
mod error;
mod models;
mod types;
mod services;
mod repositories;

use commands::{ bangumi::*, crawler::*, scheduler::*, subscription::*, };
use tauri::async_runtime::block_on;
use sqlx::SqlitePool;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let pool: SqlitePool = block_on(crate::db::init_pool());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(pool)
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
            // Scheduler commands
            create_scheduled_job,
            update_scheduled_job,
            get_scheduled_jobs,
            // Subscription commands
            get_all_subscription_ids,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}