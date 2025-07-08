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