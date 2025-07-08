use tauri::{command};
use crate::types::bangumi::{BangumiSubject, BangumiWeekday};
use crate::services::bangumi_service::BangumiService;

#[command]
pub async fn get_calendar() -> Result<Vec<BangumiWeekday>, String> {
    let service = BangumiService::new();
    service.get_calendar().await
}

#[command(rename_all = "snake_case")]
pub async fn get_subject(id: i64) -> Result<BangumiSubject, String> {
    let service = BangumiService::new();
    service.get_subject(id).await
}

#[command(rename_all = "snake_case")]
pub async fn get_episodes(
    subject_id: i64,
    episode_type: Option<i64>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<serde_json::Value, String> {
    let service = BangumiService::new();
    service.get_episodes(subject_id, episode_type, limit, offset).await
}