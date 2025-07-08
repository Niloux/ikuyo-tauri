use crate::error::Result;
use sqlx::SqlitePool;
use tauri_plugin_sql::{Migration, MigrationKind};

pub fn init_db() -> tauri_plugin_sql::Builder {
    let migrations = vec![Migration {
        version: 1,
        description: "create_initial_tables",
        sql: include_str!("../migrations/001_create_tables.sql"),
        kind: MigrationKind::Up,
    }];

    tauri_plugin_sql::Builder::new().add_migrations(
        "sqlite:ikuyo.db?foreign_keys=true&max_connections=8",
        migrations,
    )
}

// 获取数据库连接池
pub async fn get_pool(pool: tauri::State<'_, SqlitePool>) -> Result<&SqlitePool> {
    Ok(pool.inner())
}
