use crate::models::UserSubscription;
use crate::repositories::subscription::SubscriptionRepository;
use crate::error::{AppError, DomainError};
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct SubscriptionService {
    pub pool: Arc<SqlitePool>,
    pub config: crate::config::Config,
}

impl SubscriptionService {
    pub fn new(pool: Arc<SqlitePool>, config: crate::config::Config) -> Self {
        Self { pool, config }
    }

    pub async fn subscribe(
        &self,
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
        let repo = SubscriptionRepository::new(&self.pool);
        let user_id_clone = user_id.clone();
        // 检查是否已订阅
        let existing_subscription = repo
            .get_by_user_and_bangumi(&user_id, bangumi_id)
            .await?;
        if existing_subscription.is_some() {
            return Err(AppError::Domain(DomainError::Conflict("番剧已订阅".to_string())));
        }
        let new_subscription = UserSubscription {
            id: None,
            user_id,
            bangumi_id,
            subscribed_at: chrono::Utc::now().timestamp_millis(),
            notes: None,
            anime_name: Some(anime_name),
            anime_name_cn: Some(anime_name_cn),
            anime_rating,
            anime_air_date,
            anime_air_weekday,
            url,
            item_type,
            summary,
            rank,
            images,
        };
        repo.create(&new_subscription).await?;
        // 创建成功后，强制刷新缓存并将TTL设为1小时
        {
            use crate::services::bangumi_service::BangumiService;
            let service = BangumiService::new(self.pool.clone(), self.config.clone());
            let _ = service.get_subject(bangumi_id).await;
            let _ = service.get_episodes(bangumi_id, Some(0), Some(1000), Some(0)).await;
            let sub_ttl = self.config.bangumi_sub_ttl.unwrap_or(3600);
            let _ = sqlx::query("UPDATE bangumi_subject_cache SET ttl = ? WHERE id = ?")
                .bind(sub_ttl)
                .bind(bangumi_id)
                .execute(&*self.pool)
                .await?;
            let _ = sqlx::query("UPDATE bangumi_episodes_cache SET ttl = ? WHERE id = ?")
                .bind(sub_ttl)
                .bind(bangumi_id)
                .execute(&*self.pool)
                .await?;
        }
        // 创建成功后，重新获取完整的订阅信息
        let created_subscription = repo
            .get_by_user_and_bangumi(&user_id_clone, bangumi_id)
            .await?
            .ok_or_else(|| AppError::Domain(DomainError::NotFound { resource_type: "subscription".to_string(), resource_id: bangumi_id }))?;
        Ok(created_subscription)
    }

    pub async fn unsubscribe(
        &self,
        user_id: String,
        bangumi_id: i64,
    ) -> Result<(), AppError> {
        let repo = SubscriptionRepository::new(&self.pool);
        repo.delete_by_user_and_bangumi(&user_id, bangumi_id).await?;
        // 取消订阅后，将TTL设为配置值
        {
            let nonsub_ttl = self.config.bangumi_nonsub_ttl.unwrap_or(43200);
            let _ = sqlx::query("UPDATE bangumi_subject_cache SET ttl = ? WHERE id = ?")
                .bind(nonsub_ttl)
                .bind(bangumi_id)
                .execute(&*self.pool)
                .await?;
            let _ = sqlx::query("UPDATE bangumi_episodes_cache SET ttl = ? WHERE id = ?")
                .bind(nonsub_ttl)
                .bind(bangumi_id)
                .execute(&*self.pool)
                .await?;
        }
        Ok(())
    }
} 