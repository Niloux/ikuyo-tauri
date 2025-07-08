use crate::repositories::subscription::SubscriptionRepository;
use sqlx::SqlitePool;
use tauri::{command, State};

#[command(rename_all = "snake_case")]
pub async fn get_all_subscription_ids(
    pool: State<'_, SqlitePool>,
    user_id: String,
) -> Result<Vec<i64>, String> {
    let repo = SubscriptionRepository::new(&pool);
    let ids = repo
        .get_all_bangumi_ids_by_user(&user_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(ids)
}
