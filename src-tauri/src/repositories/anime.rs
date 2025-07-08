use crate::models::Anime;
use crate::error::Result;
use crate::repositories::base::Repository;
use async_trait::async_trait;
use sqlx::SqlitePool;

pub struct AnimeRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> AnimeRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    // 这些是 AnimeRepository 特有的方法
    pub async fn get_by_bangumi_id(&self, bangumi_id: i64) -> Result<Option<Anime>> {
        Ok(sqlx::query_as::<_, Anime>("SELECT * FROM anime WHERE bangumi_id = ?")
            .bind(bangumi_id)
            .fetch_optional(self.pool)
            .await?)
    }

    pub async fn search_by_title(&self, title: &str, limit: i64, offset: i64) -> Result<Vec<Anime>> {
        let query = if limit > 0 {
            "SELECT * FROM anime WHERE lower(title) LIKE ? LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM anime WHERE lower(title) LIKE ? OFFSET ?"
        };
        Ok(sqlx::query_as::<_, Anime>(query)
        .bind(format!("%{}%", title.to_lowercase()))
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?)
    }

    pub async fn count_by_title(&self, title: &str) -> Result<i64> {
        Ok(sqlx::query_scalar("SELECT COUNT(*) FROM anime WHERE lower(title) LIKE ?")
            .bind(format!("%{}%", title.to_lowercase()))
            .fetch_one(self.pool)
            .await?)
    }
}

#[async_trait]
impl<'a> Repository<Anime, i64> for AnimeRepository<'a> {
    async fn create(&self, anime: &Anime) -> Result<()> {
        sqlx::query(
            "INSERT INTO anime (mikan_id, bangumi_id, title, original_title, broadcast_day, broadcast_start, official_website, bangumi_url, description, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(anime.mikan_id)
        .bind(anime.bangumi_id)
        .bind(&anime.title)
        .bind(&anime.original_title)
        .bind(&anime.broadcast_day)
        .bind(&anime.broadcast_start)
        .bind(&anime.official_website)
        .bind(&anime.bangumi_url)
        .bind(&anime.description)
        .bind(&anime.status)
        .bind(&anime.created_at)
        .bind(&anime.updated_at)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    async fn get_by_id(&self, mikan_id: i64) -> Result<Option<Anime>> {
        Ok(sqlx::query_as::<_, Anime>("SELECT * FROM anime WHERE mikan_id = ?")
            .bind(mikan_id)
            .fetch_optional(self.pool)
            .await?)
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Anime>> {
        let query = if limit > 0 {
            "SELECT * FROM anime LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM anime OFFSET ?"
        };
        Ok(sqlx::query_as::<_, Anime>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await?)
    }

    async fn update(&self, anime: &Anime) -> Result<()> {
        sqlx::query(
            "UPDATE anime SET bangumi_id = ?, title = ?, original_title = ?, broadcast_day = ?, broadcast_start = ?, official_website = ?, bangumi_url = ?, description = ?, status = ?, updated_at = ? WHERE mikan_id = ?",
        )
        .bind(anime.bangumi_id)
        .bind(&anime.title)
        .bind(&anime.original_title)
        .bind(&anime.broadcast_day)
        .bind(&anime.broadcast_start)
        .bind(&anime.official_website)
        .bind(&anime.bangumi_url)
        .bind(&anime.description)
        .bind(&anime.status)
        .bind(&anime.updated_at)
        .bind(anime.mikan_id)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, mikan_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM anime WHERE mikan_id = ?")
            .bind(mikan_id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}