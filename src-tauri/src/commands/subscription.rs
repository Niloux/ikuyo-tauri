use crate::types::subscription::{UserSubscription as TypesUserSubscription, GetSubscriptionsParams, PaginationInfo, SubscriptionsResponse, SubscriptionResult};
use crate::models::UserSubscription as ModelsUserSubscription;
use sqlx::SqlitePool;
use crate::db;
use crate::error::Error; // Import our custom Error type

#[tauri::command]
pub async fn add_subscription(pool: tauri::State<'_, SqlitePool>, subscription: TypesUserSubscription) -> Result<SubscriptionResult, String> {
    tracing::info!("Adding subscription: {:?}", subscription);
    let db_pool = db::get_pool(pool).await.map_err(|e| e.to_string())?;

    // Convert TypesUserSubscription to ModelsUserSubscription for database insertion
    let models_subscription: ModelsUserSubscription = subscription.into();

    match db::insert_user_subscription(db_pool, models_subscription.clone()).await {
        Ok(inserted_sub) => {
            // Convert ModelsUserSubscription back to TypesUserSubscription for frontend response
            let types_inserted_sub: TypesUserSubscription = inserted_sub.into();
            Ok(SubscriptionResult {
                success: true,
                error: None,
                data: Some(serde_json::to_value(types_inserted_sub).unwrap_or_default()),
            })
        },
        Err(e) => {
            tracing::error!("Failed to add subscription: {:?}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_subscriptions(pool: tauri::State<'_, SqlitePool>, params: GetSubscriptionsParams) -> Result<SubscriptionsResponse, String> {
    tracing::info!("Fetching subscriptions with params: {:?}", params);
    let db_pool = db::get_pool(pool).await.map_err(|e| e.to_string())?;

    match db::get_user_subscriptions_from_db(db_pool).await {
        Ok(subscriptions) => {
            // Convert Vec<ModelsUserSubscription> to Vec<TypesUserSubscription>
            let types_subscriptions: Vec<TypesUserSubscription> = subscriptions.into_iter().map(|s| s.into()).collect();

            let total = types_subscriptions.len() as u32;
            let limit = params.limit.unwrap_or(10);
            let page = params.page.unwrap_or(1);
            let pages = (total as f32 / limit as f32).ceil() as u32;

            Ok(SubscriptionsResponse {
                subscriptions: types_subscriptions,
                pagination: PaginationInfo {
                    page,
                    limit,
                    total,
                    pages,
                },
            })
        },
        Err(e) => {
            tracing::error!("Failed to get subscriptions: {:?}", e);
            Err(e.to_string())
        }
    }
}