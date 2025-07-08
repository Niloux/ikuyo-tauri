use crate::types::subscription::{UserSubscription as TypesUserSubscription, GetSubscriptionsParams, PaginationInfo, SubscriptionsResponse, SubscriptionResult};
use sqlx::SqlitePool;

#[tauri::command]
pub async fn add_subscription(pool: tauri::State<'_, SqlitePool>, subscription: TypesUserSubscription) -> Result<SubscriptionResult, String> {
    tracing::info!("Adding subscription: {:?}", subscription);
    Ok(SubscriptionResult {
        success: true,
        error: None,
        data: Some(serde_json::to_value(subscription).unwrap_or_default()),
    })
}

#[tauri::command]
pub async fn get_subscriptions(pool: tauri::State<'_, SqlitePool>, params: GetSubscriptionsParams) -> Result<SubscriptionsResponse, String> {
    tracing::info!("Fetching subscriptions with params: {:?}", params);
    Ok(SubscriptionsResponse {
        subscriptions: vec![],
        pagination: PaginationInfo {
            page: 1,
            limit: 10,
            total: 0,
            pages: 0,
        },
    })
}