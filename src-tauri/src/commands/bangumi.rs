use tauri::{command, State};
use crate::types::bangumi::BangumiWeekday;
use crate::services::bangumi_service::BangumiService;

#[command]
pub async fn get_calendar() -> Result<Vec<BangumiWeekday>, String> {
    let service = BangumiService::new();
    service.get_calendar().await
}
