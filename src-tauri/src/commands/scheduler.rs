use crate::types::scheduler::{ScheduledJobCreate, ScheduledJobResponse, ScheduledJobUpdate};
use std::collections::HashMap;

#[tauri::command]
pub fn create_scheduled_job(job: ScheduledJobCreate) -> Result<ScheduledJobResponse, String> {
    tracing::info!("Creating scheduled job: {:?}", job);
    // 返回示例数据
    Ok(ScheduledJobResponse {
        id: Some(1),
        job_id: job.job_id,
        name: job.name,
        cron_expression: job.cron_expression,
        parameters: job.parameters,
        enabled: job.enabled.unwrap_or(true),
        description: job.description,
        created_at: Some("2023-07-07T11:00:00Z".to_string()),
        updated_at: Some("2023-07-07T11:00:00Z".to_string()),
    })
}

#[tauri::command]
pub fn update_scheduled_job(job_id: String, updates: ScheduledJobUpdate) -> Result<ScheduledJobResponse, String> {
    tracing::info!("Updating scheduled job {}: {:?}", job_id, updates);
    // 返回示例数据
    Ok(ScheduledJobResponse {
        id: Some(1),
        job_id,
        name: updates.name.unwrap_or("Updated Job Name".to_string()),
        cron_expression: updates.cron_expression.unwrap_or("0 0 * * * *".to_string()),
        parameters: updates.parameters.unwrap_or_else(HashMap::new),
        enabled: updates.enabled.unwrap_or(true),
        description: updates.description,
        created_at: Some("2023-07-07T11:00:00Z".to_string()),
        updated_at: Some("2023-07-07T11:30:00Z".to_string()),
    })
}

#[tauri::command]
pub fn get_scheduled_jobs() -> Result<Vec<ScheduledJobResponse>, String> {
    tracing::info!("Fetching all scheduled jobs");
    // 返回示例数据
    Ok(vec![
        ScheduledJobResponse {
            id: Some(1),
            job_id: "job-1".to_string(),
            name: "Daily Crawl".to_string(),
            cron_expression: "0 0 * * * *".to_string(),
            parameters: HashMap::new(),
            enabled: true,
            description: Some("Daily crawl for new bangumi".to_string()),
            created_at: Some("2023-07-01T00:00:00Z".to_string()),
            updated_at: Some("2023-07-01T00:00:00Z".to_string()),
        },
        ScheduledJobResponse {
            id: Some(2),
            job_id: "job-2".to_string(),
            name: "Weekly Update".to_string(),
            cron_expression: "0 0 * * 0 *".to_string(),
            parameters: HashMap::new(),
            enabled: false,
            description: Some("Weekly update for subscriptions".to_string()),
            created_at: Some("2023-06-25T00:00:00Z".to_string()),
            updated_at: Some("2023-06-25T00:00:00Z".to_string()),
        },
    ])
}