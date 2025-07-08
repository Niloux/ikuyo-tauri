use crate::types::scheduler::{ScheduledJobCreate, ScheduledJobResponse, ScheduledJobUpdate};
use crate::repositories::scheduled_job::ScheduledJobRepository;
use crate::models::ScheduledJob;
use sqlx::SqlitePool;
use tauri::State;
use std::collections::HashMap;

fn convert_to_response(job: ScheduledJob) -> ScheduledJobResponse {
    let parameters: HashMap<String, serde_json::Value> = match job.parameters {
        Some(param_str) => serde_json::from_str(&param_str).unwrap_or_default(),
        None => HashMap::new(),
    };
    
    ScheduledJobResponse {
        id: job.id.map(|id| id as u32),
        job_id: job.job_id,
        name: job.name,
        cron_expression: job.cron_expression,
        parameters,
        enabled: job.enabled,
        description: job.description,
        created_at: job.created_at.map(|ts| chrono::DateTime::from_timestamp(ts / 1000, 0)
            .unwrap_or_default()
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string()),
        updated_at: job.updated_at.map(|ts| chrono::DateTime::from_timestamp(ts / 1000, 0)
            .unwrap_or_default()
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string()),
    }
}

#[tauri::command]
pub async fn create_scheduled_job(
    job: ScheduledJobCreate,
    pool: State<'_, SqlitePool>,
) -> Result<ScheduledJobResponse, String> {
    tracing::info!("Creating scheduled job: {:?}", job);
    
    let repo = ScheduledJobRepository::new(&*pool);
    let current_time = chrono::Utc::now().timestamp_millis();
    let parameters_str = serde_json::to_string(&job.parameters).unwrap_or_default();
    
    let new_job = ScheduledJob {
        id: None,
        job_id: job.job_id,
        name: job.name,
        description: job.description,
        cron_expression: job.cron_expression,
        crawler_mode: None, // 可以从parameters中推断
        parameters: Some(parameters_str),
        enabled: job.enabled.unwrap_or(true),
        created_at: Some(current_time),
        updated_at: Some(current_time),
    };
    
    let job_id = repo.create(&new_job).await.map_err(|e| e.to_string())?;
    let created_job = repo.get_by_id(job_id).await.map_err(|e| e.to_string())?;
    
    match created_job {
        Some(job) => Ok(convert_to_response(job)),
        None => Err("计划任务创建失败".to_string()),
    }
}

#[tauri::command]
pub async fn update_scheduled_job(
    job_id: String,
    updates: ScheduledJobUpdate,
    pool: State<'_, SqlitePool>,
) -> Result<ScheduledJobResponse, String> {
    tracing::info!("Updating scheduled job {}: {:?}", job_id, updates);
    
    let repo = ScheduledJobRepository::new(&*pool);
    let existing_job = repo.get_by_job_id(&job_id).await.map_err(|e| e.to_string())?;
    
    match existing_job {
        Some(mut job) => {
            // 更新字段
            if let Some(name) = updates.name {
                job.name = name;
            }
            if let Some(cron) = updates.cron_expression {
                job.cron_expression = cron;
            }
            if let Some(parameters) = updates.parameters {
                job.parameters = Some(serde_json::to_string(&parameters).unwrap_or_default());
            }
            if let Some(enabled) = updates.enabled {
                job.enabled = enabled;
            }
            if let Some(description) = updates.description {
                job.description = Some(description);
            }
            job.updated_at = Some(chrono::Utc::now().timestamp_millis());
            
            repo.update(&job).await.map_err(|e| e.to_string())?;
            Ok(convert_to_response(job))
        }
        None => Err("计划任务不存在".to_string()),
    }
}

#[tauri::command]
pub async fn get_scheduled_jobs(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<ScheduledJobResponse>, String> {
    tracing::info!("Fetching all scheduled jobs");
    
    let repo = ScheduledJobRepository::new(&*pool);
    let jobs = repo.list(0, 0).await.map_err(|e| e.to_string())?; // 获取所有任务
    let responses: Vec<ScheduledJobResponse> = jobs.into_iter().map(convert_to_response).collect();
    
    Ok(responses)
}

#[tauri::command]
pub async fn get_scheduled_job(
    job_id: String,
    pool: State<'_, SqlitePool>,
) -> Result<ScheduledJobResponse, String> {
    tracing::info!("Getting scheduled job: {}", job_id);
    
    let repo = ScheduledJobRepository::new(&*pool);
    let job = repo.get_by_job_id(&job_id).await.map_err(|e| e.to_string())?;
    
    match job {
        Some(job) => Ok(convert_to_response(job)),
        None => Err("计划任务不存在".to_string()),
    }
}

#[tauri::command]
pub async fn delete_scheduled_job(
    job_id: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    tracing::info!("Deleting scheduled job: {}", job_id);
    
    let repo = ScheduledJobRepository::new(&*pool);
    let job = repo.get_by_job_id(&job_id).await.map_err(|e| e.to_string())?;
    
    match job {
        Some(job) => {
            if let Some(id) = job.id {
                repo.delete(id).await.map_err(|e| e.to_string())?;
                Ok(())
            } else {
                Err("任务ID无效".to_string())
            }
        }
        None => Err("计划任务不存在".to_string()),
    }
}

#[tauri::command]
pub async fn toggle_scheduled_job(
    job_id: String,
    pool: State<'_, SqlitePool>,
) -> Result<ScheduledJobResponse, String> {
    tracing::info!("Toggling scheduled job: {}", job_id);
    
    let repo = ScheduledJobRepository::new(&*pool);
    let existing_job = repo.get_by_job_id(&job_id).await.map_err(|e| e.to_string())?;
    
    match existing_job {
        Some(mut job) => {
            job.enabled = !job.enabled;
            job.updated_at = Some(chrono::Utc::now().timestamp_millis());
            
            repo.update(&job).await.map_err(|e| e.to_string())?;
            Ok(convert_to_response(job))
        }
        None => Err("计划任务不存在".to_string()),
    }
}