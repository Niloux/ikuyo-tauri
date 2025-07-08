use crate::repositories::subscription::SubscriptionRepository;
use crate::models::UserSubscription;
use crate::types::subscription::{SubscriptionIdsResponse, SubscriptionsResponse, SubscriptionStatus};
use sqlx::SqlitePool;
use tauri::{command, State};

#[command(rename_all = "snake_case")]
pub async fn get_all_subscription_ids(
    pool: State<'_, SqlitePool>,
    user_id: String,
) -> Result<SubscriptionIdsResponse, String> {
    let repo = SubscriptionRepository::new(&pool);
    let ids = repo
        .get_all_bangumi_ids_by_user(&user_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(SubscriptionIdsResponse { ids })
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

    // 检查是否已订阅
    let existing_subscription = repo.get_by_user_and_bangumi(&user_id, bangumi_id).await.map_err(|e| e.to_string())?;
    if existing_subscription.is_some() {
        return Err("番剧已订阅".to_string());
    }

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
) -> Result<SubscriptionsResponse, String> {
    let repo = SubscriptionRepository::new(&pool);
    let current_page = page.unwrap_or(1);
    let current_limit = limit.unwrap_or(10);
    let (subscriptions_from_db, total) = repo.list_with_sort_search_page(
        &user_id,
        sort.as_deref().unwrap_or("subscribed_at"),
        order.as_deref().unwrap_or("desc"),
        search.as_deref(),
        current_page,
        current_limit,
    ).await.map_err(|e| e.to_string())?;

    let subscriptions: Vec<crate::types::subscription::UserSubscription> = subscriptions_from_db.into_iter().map(|sub| {
        crate::types::subscription::UserSubscription {
            id: sub.id.map(|id| id as u32),
            user_id: sub.user_id,
            bangumi_id: sub.bangumi_id as u32,
            subscribed_at: sub.subscribed_at as u64,
            notes: sub.notes,
            anime_name: sub.anime_name,
            anime_name_cn: sub.anime_name_cn,
            anime_rating: sub.anime_rating.map(|r| r as f32),
            anime_air_date: sub.anime_air_date,
            anime_air_weekday: sub.anime_air_weekday.map(|w| w as u32),
        }
    }).collect();

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
    pool: State<'_, SqlitePool>,
    user_id: String,
    bangumi_id: i64,
) -> Result<SubscriptionStatus, String> {
    let repo = SubscriptionRepository::new(&pool);
    let subscription = repo.get_by_user_and_bangumi(&user_id, bangumi_id).await.map_err(|e| e.to_string())?;
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
