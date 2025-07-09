use crate::config::Config;
use crate::error::{Error, Result};
use sqlx::SqlitePool;
use std::fs;

pub async fn init_pool(config: &Config) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(&config.db_url)
        .await
        .map_err(|e| anyhow::anyhow!(e).context(Error::DatabaseInitialization))?;

    let sql = fs::read_to_string("migrations/001_create_tables.sql")
        .map_err(|e| anyhow::anyhow!(e).context(Error::SqlFileRead))?;

    sqlx::query(&sql)
        .execute(&pool)
        .await
        .map_err(|e| anyhow::anyhow!(e).context(Error::DatabaseMigration))?;

    Ok(pool)
}
