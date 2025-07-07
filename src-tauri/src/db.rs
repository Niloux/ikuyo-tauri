use tauri_plugin_sql::{Migration, MigrationKind};
use sqlx::SqlitePool;
use crate::models::UserSubscription;
use crate::error::{Error, Result};

const DB_VERSION: u64 = 1;

pub fn init_db() -> tauri_plugin_sql::Builder {
    let migrations = vec![
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: include_str!("../migrations/001_create_tables.sql"),
            kind: MigrationKind::Up,
        }
    ];

    tauri_plugin_sql::Builder::new()
        .add_migrations("sqlite:ikuyo.db", migrations)
}

// 获取数据库连接池
pub async fn get_pool(pool: tauri::State<'_, SqlitePool>) -> Result<&SqlitePool> {
    Ok(pool.inner())
}

// 插入用户订阅
pub async fn insert_user_subscription(pool: &SqlitePool, subscription: UserSubscription) -> Result<UserSubscription> {
    let mut conn = pool.acquire().await.map_err(|e| Error::Database(e.to_string()))?;

    let query = "INSERT INTO user_subscriptions (user_id, bangumi_id, subscribed_at, notes, anime_name, anime_name_cn, anime_rating, anime_air_date, anime_air_weekday) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";

    sqlx::query(query)
        .bind(subscription.user_id.clone())
        .bind(subscription.bangumi_id)
        .bind(subscription.subscribed_at)
        .bind(subscription.notes.clone())
        .bind(subscription.anime_name.clone())
        .bind(subscription.anime_name_cn.clone())
        .bind(subscription.anime_rating)
        .bind(subscription.anime_air_date.clone())
        .bind(subscription.anime_air_weekday)
        .execute(&mut *conn)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

    // TODO: 获取插入后的ID并返回完整的UserSubscription
    Ok(subscription)
}

// 获取用户订阅列表
pub async fn get_user_subscriptions_from_db(pool: &SqlitePool) -> Result<Vec<UserSubscription>> {
    let mut conn = pool.acquire().await.map_err(|e| Error::Database(e.to_string()))?;

    let subscriptions = sqlx::query_as::<_, UserSubscription>("SELECT * FROM user_subscriptions")
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

    Ok(subscriptions)
}
