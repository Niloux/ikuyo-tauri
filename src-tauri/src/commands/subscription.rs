use crate::repositories::subscription::SubscriptionRepository;
use crate::models::UserSubscription;
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

#[command(rename_all = "snake_case")]
pub async fn subscribe(
    pool: State<'_, SqlitePool>,
    user_id: String,
    bangumi_id: i64,
    anime_name: String,
    anime_name_cn: String,
    anime_rating: Option<f64>,
    anime_air_date: Option<String>,
    anime_air_weekday: Option<i64>,
) -> Result<UserSubscription, String> {
    let repo = SubscriptionRepository::new(&pool);
    let new_subscription = UserSubscription {
        id: None, // Will be set by the database
        user_id,
        bangumi_id,
        subscribed_at: chrono::Utc::now().timestamp_millis(),
        notes: None,
        anime_name: Some(anime_name),
        anime_name_cn: Some(anime_name_cn),
        anime_rating: anime_rating,
        anime_air_date: anime_air_date,
        anime_air_weekday: anime_air_weekday,
    };

    repo.create(&new_subscription).await.map_err(|e| e.to_string())?;
    Ok(new_subscription)
}

#[command(rename_all = "snake_case")]
pub async fn unsubscribe(
    pool: State<'_, SqlitePool>,
    user_id: String,
    bangumi_id: i64,
) -> Result<(), String> {
    let repo = SubscriptionRepository::new(&pool);
    repo.delete_by_user_and_bangumi(&user_id, bangumi_id).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[command(rename_all = "snake_case")]
pub async fn get_subscriptions(
    pool: State<'_, SqlitePool>,
    user_id: String,
    sort: Option<String>,
    order: Option<String>,
    search: Option<String>,
    page: Option<i64>,
    limit: Option<i64>,
) -> Result<serde_json::Value, String> {
    let repo = SubscriptionRepository::new(&pool);
    let (subscriptions, total) = repo.list_with_sort_search_page(
        &user_id,
        sort.as_deref().unwrap_or("subscribed_at"),
        order.as_deref().unwrap_or("desc"),
        search.as_deref(),
        page.unwrap_or(1),
        limit.unwrap_or(10),
    ).await.map_err(|e| e.to_string())?;

    let response = serde_json::json!({
        "data": subscriptions,
        "total": total,
        "page": page.unwrap_or(1),
        "limit": limit.unwrap_or(10),
    });

    Ok(response)
}

#[command(rename_all = "snake_case")]
pub async fn check_subscription(
    pool: State<'_, SqlitePool>,
    user_id: String,
    bangumi_id: i64,
) -> Result<serde_json::Value, String> {
    let repo = SubscriptionRepository::new(&pool);
    let subscription = repo.get_by_user_and_bangumi(&user_id, bangumi_id).await.map_err(|e| e.to_string())?;
    let response = if let Some(subscription) = subscription {
        serde_json::json!({
            "subscribed": true,
            "subscribed_at": subscription.subscribed_at,
            "notes": subscription.notes
        })
    } else {
        serde_json::json!({
            "subscribed": false
        })
    };
    Ok(response)
}
