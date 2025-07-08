mod commands;
mod db;
mod error;
mod models;
mod types;
mod services;

use commands::{ bangumi::*, crawler::*, scheduler::*, subscription::*, };

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

    let db_builder = db::init_db();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(db_builder.build())
        .invoke_handler(tauri::generate_handler![
            // Bangumi commands
            get_calendar,
            // Crawler commands
            create_crawler_task,
            get_crawler_task_status,
            // Scheduler commands
            create_scheduled_job,
            update_scheduled_job,
            get_scheduled_jobs,
            // Subscription commands
            get_all_subscription_ids,
            add_subscription,
            get_subscriptions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}