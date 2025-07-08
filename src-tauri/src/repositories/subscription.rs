use crate::models::UserSubscription;
use sqlx::SqlitePool;

pub struct SubscriptionRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SubscriptionRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, subscription: &UserSubscription) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO user_subscriptions (user_id, bangumi_id, subscribed_at, notes, anime_name, anime_name_cn, anime_rating, anime_air_date, anime_air_weekday)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&subscription.user_id)
        .bind(subscription.bangumi_id)
        .bind(subscription.subscribed_at)
        .bind(&subscription.notes)
        .bind(&subscription.anime_name)
        .bind(&subscription.anime_name_cn)
        .bind(subscription.anime_rating)
        .bind(&subscription.anime_air_date)
        .bind(subscription.anime_air_weekday)
        .execute(self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get_by_user_and_bangumi(
        &self,
        user_id: &str,
        bangumi_id: i64,
    ) -> Result<Option<UserSubscription>, sqlx::Error> {
        sqlx::query_as::<_, UserSubscription>(
            "SELECT * FROM user_subscriptions WHERE user_id = ? AND bangumi_id = ?",
        )
        .bind(user_id)
        .bind(bangumi_id)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn list_with_sort_search_page(
        &self,
        user_id: &str,
        sort: &str,
        order: &str,
        search: Option<&str>,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<UserSubscription>, i64), sqlx::Error> {
        let search_pattern = search.map(|s| format!("%{}%", s.to_lowercase()));

        // First, get the total count
        let count_query = {
            let mut q = String::from("SELECT COUNT(*) FROM user_subscriptions WHERE user_id = ?");
            if search.is_some() {
                q.push_str(" AND (lower(anime_name) LIKE ? OR lower(anime_name_cn) LIKE ?)");
            }
            q
        };

        let mut count_query_builder = sqlx::query_scalar(&count_query).bind(user_id);
        if let Some(ref pattern) = search_pattern {
            count_query_builder = count_query_builder.bind(pattern).bind(pattern);
        }
        let total: i64 = count_query_builder.fetch_one(self.pool).await?;

        // Then, get the paginated data
        let sort_field = match sort {
            "rating" => "anime_rating",
            "air_date" => "anime_air_date",
            "name" => "anime_name_cn",
            _ => "subscribed_at",
        };

        let order_direction = if order.eq_ignore_ascii_case("desc") {
            "DESC"
        } else {
            "ASC"
        };

        let mut data_query = format!(
            "SELECT * FROM user_subscriptions WHERE user_id = ? ",
        );
        if search.is_some() {
            data_query.push_str("AND (lower(anime_name) LIKE ? OR lower(anime_name_cn) LIKE ?) ");
        }
        data_query.push_str(&format!("ORDER BY {} {} ", sort_field, order_direction));

        if limit > 0 {
            data_query.push_str("LIMIT ? OFFSET ?");
        } else {
            data_query.push_str("OFFSET ?");
        }
        
        let mut data_query_builder = sqlx::query_as::<_, UserSubscription>(&data_query).bind(user_id);
        if let Some(ref pattern) = search_pattern {
            data_query_builder = data_query_builder.bind(pattern).bind(pattern);
        }
        if limit > 0 {
            data_query_builder = data_query_builder.bind(limit);
        }
        data_query_builder = data_query_builder.bind((page - 1) * limit);

        let subscriptions = data_query_builder.fetch_all(self.pool).await?;

        Ok((subscriptions, total))
    }

    pub async fn delete_by_user_and_bangumi(&self, user_id: &str, bangumi_id: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM user_subscriptions WHERE user_id = ? AND bangumi_id = ?")
            .bind(user_id)
            .bind(bangumi_id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn get_all_bangumi_ids_by_user(&self, user_id: &str) -> Result<Vec<i64>, sqlx::Error> {
        sqlx::query_scalar("SELECT bangumi_id FROM user_subscriptions WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(self.pool)
            .await
    }
}
