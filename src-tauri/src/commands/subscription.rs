use crate::{
    models::UserSubscription,
    repositories::subscription::SubscriptionRepository,
    types::subscription::{SubscriptionIdsResponse, SubscriptionStatus, SubscriptionsResponse},
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{command, State};
use crate::error::AppError;

#[command(rename_all = "snake_case")]
pub async fn get_all_subscription_ids(
    pool: State<'_, Arc<SqlitePool>>,
    user_id: String,
) -> Result<SubscriptionIdsResponse, AppError> {
    let repo = SubscriptionRepository::new(&pool);
    let ids = repo
        .get_all_bangumi_ids_by_user(&user_id)
        .await?;
    Ok(SubscriptionIdsResponse { ids })
}

#[command(rename_all = "snake_case")]
pub async fn subscribe(
    pool: State<'_, Arc<SqlitePool>>,
    config: State<'_, crate::config::Config>,
    user_id: String,
    bangumi_id: i64,
    anime_name: String,
    anime_name_cn: String,
    anime_rating: Option<f64>,
    anime_air_date: Option<String>,
    anime_air_weekday: Option<i64>,
    url: Option<String>,
    item_type: Option<i64>,
    summary: Option<String>,
    rank: Option<i64>,
    images: Option<String>,
) -> Result<UserSubscription, AppError> {
    let service = crate::services::subscription_service::SubscriptionService::new(pool.inner().clone(), config.inner().clone());
    service
        .subscribe(
            user_id,
            bangumi_id,
            anime_name,
            anime_name_cn,
            anime_rating,
            anime_air_date,
            anime_air_weekday,
            url,
            item_type,
            summary,
            rank,
            images,
        )
        .await
}

#[command(rename_all = "snake_case")]
pub async fn unsubscribe(
    pool: State<'_, Arc<SqlitePool>>,
    config: State<'_, crate::config::Config>,
    user_id: String,
    bangumi_id: i64,
) -> Result<(), AppError> {
    let service = crate::services::subscription_service::SubscriptionService::new(pool.inner().clone(), config.inner().clone());
    service.unsubscribe(user_id, bangumi_id).await
}

#[command(rename_all = "snake_case")]
pub async fn get_subscriptions(
    pool: State<'_, Arc<SqlitePool>>,
    user_id: String,
    sort: Option<String>,
    order: Option<String>,
    search: Option<String>,
    page: Option<i64>,
    limit: Option<i64>,
) -> Result<SubscriptionsResponse, AppError> {
    let repo = SubscriptionRepository::new(&pool);
    let current_page = page.unwrap_or(1);
    let current_limit = limit.unwrap_or(10);
    let (subscriptions_from_db, total) = repo
        .list_with_sort_search_page(
            &user_id,
            sort.as_deref().unwrap_or("subscribed_at"),
            order.as_deref().unwrap_or("desc"),
            search.as_deref(),
            current_page,
            current_limit,
        )
        .await?;

    let subscriptions: Vec<crate::types::subscription::UserSubscription> = subscriptions_from_db
        .into_iter()
        .map(|sub| sub.into())
        .collect();

    let total_pages = (total as f64 / current_limit as f64).ceil() as u32;

    Ok(SubscriptionsResponse {
        subscriptions,
        pagination: crate::types::subscription::PaginationInfo {
            page: current_page as u32,
            limit: current_limit as u32,
            total: total as u32,
            pages: total_pages,
        },
    })
}

#[command(rename_all = "snake_case")]
pub async fn check_subscription(
    pool: State<'_, Arc<SqlitePool>>,
    user_id: String,
    bangumi_id: i64,
) -> Result<SubscriptionStatus, AppError> {
    let repo = SubscriptionRepository::new(&pool);
    let subscription = repo
        .get_by_user_and_bangumi(&user_id, bangumi_id)
        .await?;
    let response = if let Some(subscription) = subscription {
        SubscriptionStatus {
            subscribed: true,
            subscribed_at: Some(subscription.subscribed_at as u64),
            notes: subscription.notes,
        }
    } else {
        SubscriptionStatus {
            subscribed: false,
            subscribed_at: None,
            notes: None,
        }
    };
    Ok(response)
}
