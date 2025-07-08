use sqlx::SqlitePool;
use std::fs;

pub async fn init_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite:ikuyo.db?mode=rwc").await.expect("数据库连接失败");
    // 自动执行建表 SQL
    let sql = fs::read_to_string("migrations/001_create_tables.sql").expect("无法读取建表SQL");
    sqlx::query(&sql).execute(&pool).await.expect("建表SQL执行失败");
    pool
}
