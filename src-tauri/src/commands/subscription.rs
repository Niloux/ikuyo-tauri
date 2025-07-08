use crate::types::subscription::{
    GetSubscriptionsParams, PaginationInfo, SubscriptionResult, SubscriptionsResponse,
    UserSubscription as TypesUserSubscription,
};
use sqlx::SqlitePool;
use tauri::{command, State};

#[command]
pub async fn get_all_subscription_ids() -> Result<Vec<i64>, String> {
    Ok(vec![])
}

#[command]
pub async fn add_subscription(
    pool: State<'_, SqlitePool>,
    subscription: TypesUserSubscription,
) -> Result<SubscriptionResult, String> {
    tracing::info!("Adding subscription: {:?}", subscription);
    Ok(SubscriptionResult {
        success: true,
        error: None,
        data: Some(serde_json::to_value(subscription).unwrap_or_default()),
    })
}

#[command]
pub async fn get_subscriptions(
    pool: State<'_, SqlitePool>,
    params: GetSubscriptionsParams,
) -> Result<SubscriptionsResponse, String> {
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
