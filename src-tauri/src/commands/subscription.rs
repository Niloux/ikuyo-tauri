use crate::{
    models::UserSubscription,
    repositories::subscription::SubscriptionRepository,
    types::subscription::{
        SubscriptionIdsResponse, SubscriptionStatus, SubscriptionsResponse,
    },
};
use sqlx::SqlitePool;
use tauri::{command, State};
use std::sync::Arc;

#[command(rename_all = "snake_case")]
pub async fn get_all_subscription_ids(
    pool: State<'_, Arc<SqlitePool>>,
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
    pool: State<'_, Arc<SqlitePool>>,
    config: State<'_, crate::config::Config>,
    user_id: String,
    bangumi_id: i64,
    anime_name: String,
    anime_name_cn: String,
    anime_rating: Option<f64>,
    anime_air_date: Option<String>,
    anime_air_weekday: Option<i64>,
    // 新增参数
    url: Option<String>,
    item_type: Option<i64>,
    summary: Option<String>,
    rank: Option<i64>,
    images: Option<String>, // 存储 BangumiImages 的 JSON 字符串
) -> Result<UserSubscription, String> {
    let repo = SubscriptionRepository::new(&pool);
    let user_id_clone = user_id.clone();

    // 检查是否已订阅
    let existing_subscription = repo
        .get_by_user_and_bangumi(&user_id, bangumi_id)
        .await
        .map_err(|e| e.to_string())?;
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
        anime_rating,
        anime_air_date,
        anime_air_weekday,
        // 新增字段赋值
        url,
        item_type,
        summary,
        rank,
        images,
    };

    repo.create(&new_subscription)
        .await
        .map_err(|e| e.to_string())?;

    // 创建成功后，强制刷新缓存并将TTL设为1小时
    {
        use crate::services::bangumi_service::BangumiService;
        let service = BangumiService::new(pool.inner().clone(), config.inner().clone());
        let _ = service.get_subject(bangumi_id).await;
        let _ = service.get_episodes(bangumi_id, None, None, None).await;
        // 更新TTL为配置值
        let sub_ttl = config.bangumi_sub_ttl.unwrap_or(3600);
        let _ = sqlx::query("UPDATE bangumi_subject_cache SET ttl = ? WHERE id = ?")
            .bind(sub_ttl)
            .bind(bangumi_id)
            .execute(&**pool)
            .await;
        let _ = sqlx::query("UPDATE bangumi_episodes_cache SET ttl = ? WHERE id = ?")
            .bind(sub_ttl)
            .bind(bangumi_id)
            .execute(&**pool)
            .await;
    }

    // 创建成功后，重新获取完整的订阅信息
    let created_subscription = repo
        .get_by_user_and_bangumi(&user_id_clone, bangumi_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "创建订阅后无法立即找到该订阅".to_string())?;

    Ok(created_subscription)
}

#[command(rename_all = "snake_case")]
pub async fn unsubscribe(
    pool: State<'_, Arc<SqlitePool>>,
    config: State<'_, crate::config::Config>,
    user_id: String,
    bangumi_id: i64,
) -> Result<(), String> {
    let repo = SubscriptionRepository::new(&pool);
    repo.delete_by_user_and_bangumi(&user_id, bangumi_id)
        .await
        .map_err(|e| e.to_string())?;
    // 取消订阅后，将TTL设为配置值
    {
        let nonsub_ttl = config.bangumi_nonsub_ttl.unwrap_or(43200);
        let _ = sqlx::query("UPDATE bangumi_subject_cache SET ttl = ? WHERE id = ?")
            .bind(nonsub_ttl)
            .bind(bangumi_id)
            .execute(&**pool)
            .await;
        let _ = sqlx::query("UPDATE bangumi_episodes_cache SET ttl = ? WHERE id = ?")
            .bind(nonsub_ttl)
            .bind(bangumi_id)
            .execute(&**pool)
            .await;
    }
    Ok(())
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
) -> Result<SubscriptionsResponse, String> {
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
        .await
        .map_err(|e| e.to_string())?;

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
) -> Result<SubscriptionStatus, String> {
    let repo = SubscriptionRepository::new(&pool);
    let subscription = repo
        .get_by_user_and_bangumi(&user_id, bangumi_id)
        .await
        .map_err(|e| e.to_string())?;
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
