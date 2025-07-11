use crate::error::Result;
use crate::models::Anime;
use crate::repositories::base::Repository;
use async_trait::async_trait;
use sqlx::{SqlitePool, Transaction};

pub struct AnimeRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> AnimeRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    // 这些是 AnimeRepository 特有的方法
    pub async fn get_by_bangumi_id(&self, bangumi_id: i64) -> Result<Option<Anime>> {
        Ok(
            sqlx::query_as::<_, Anime>("SELECT * FROM anime WHERE bangumi_id = ?")
                .bind(bangumi_id)
                .fetch_optional(self.pool)
                .await
                .map_err(|e| {
                    crate::error::AppError::Database(crate::error::DatabaseError::Other(
                        e.to_string(),
                    ))
                })?,
        )
    }

    pub async fn search_by_title(
        &self,
        title: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Anime>> {
        let query = if limit > 0 {
            "SELECT * FROM anime WHERE lower(title) LIKE ? LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM anime WHERE lower(title) LIKE ? LIMIT -1 OFFSET 0"
        };
        Ok(sqlx::query_as::<_, Anime>(query)
            .bind(format!("%{}%", title.to_lowercase()))
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await
            .map_err(|e| {
                crate::error::AppError::Database(crate::error::DatabaseError::Other(e.to_string()))
            })?)
    }

    pub async fn count_by_title(&self, title: &str) -> Result<i64> {
        Ok(
            sqlx::query_scalar("SELECT COUNT(*) FROM anime WHERE lower(title) LIKE ?")
                .bind(format!("%{}%", title.to_lowercase()))
                .fetch_one(self.pool)
                .await
                .map_err(|e| {
                    crate::error::AppError::Database(crate::error::DatabaseError::Other(
                        e.to_string(),
                    ))
                })?,
        )
    }

    pub async fn insert_many_animes(
        &self,
        tx: &mut Transaction<'_, sqlx::Sqlite>,
        animes: &[Anime],
    ) -> Result<()> {
        use sqlx::QueryBuilder;
        if animes.is_empty() {
            return Ok(());
        }
        let mut builder = QueryBuilder::new(
            "INSERT INTO anime (mikan_id, bangumi_id, title, original_title, broadcast_day, broadcast_start, official_website, bangumi_url, description, status, created_at, updated_at) "
        );
        builder.push("VALUES ");
        for (i, anime) in animes.iter().enumerate() {
            if i > 0 {
                builder.push(", ");
            }
            builder
                .push("(")
                .push_bind(anime.mikan_id)
                .push(", ")
                .push_bind(anime.bangumi_id)
                .push(", ")
                .push_bind(&anime.title)
                .push(", ")
                .push_bind(&anime.original_title)
                .push(", ")
                .push_bind(&anime.broadcast_day)
                .push(", ")
                .push_bind(&anime.broadcast_start)
                .push(", ")
                .push_bind(&anime.official_website)
                .push(", ")
                .push_bind(&anime.bangumi_url)
                .push(", ")
                .push_bind(&anime.description)
                .push(", ")
                .push_bind(&anime.status)
                .push(", ")
                .push_bind(&anime.created_at)
                .push(", ")
                .push_bind(&anime.updated_at)
                .push(")");
        }
        builder.push(
            " ON CONFLICT(mikan_id) DO UPDATE SET \
                bangumi_id = excluded.bangumi_id,\
                title = excluded.title,\
                original_title = excluded.original_title,\
                broadcast_day = excluded.broadcast_day,\
                broadcast_start = excluded.broadcast_start,\
                official_website = excluded.official_website,\
                bangumi_url = excluded.bangumi_url,\
                description = excluded.description,\
                status = excluded.status,\
                updated_at = excluded.updated_at",
        );
        builder.build().execute(&mut **tx).await.map_err(|e| {
            crate::error::AppError::Database(crate::error::DatabaseError::Other(e.to_string()))
        })?;
        Ok(())
    }
}

#[async_trait]
impl<'a> Repository<Anime, i64> for AnimeRepository<'a> {
    async fn create(&self, anime: &Anime) -> Result<()> {
        sqlx::query(
            "INSERT INTO anime (mikan_id, bangumi_id, title, original_title, broadcast_day, broadcast_start, official_website, bangumi_url, description, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(mikan_id) DO UPDATE SET
                bangumi_id = excluded.bangumi_id,
                title = excluded.title,
                original_title = excluded.original_title,
                broadcast_day = excluded.broadcast_day,
                broadcast_start = excluded.broadcast_start,
                official_website = excluded.official_website,
                bangumi_url = excluded.bangumi_url,
                description = excluded.description,
                status = excluded.status,
                updated_at = excluded.updated_at;",
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
        .await.map_err(|e| crate::error::AppError::Database(crate::error::DatabaseError::Other(e.to_string())))?;
        Ok(())
    }

    async fn get_by_id(&self, mikan_id: i64) -> Result<Option<Anime>> {
        Ok(
            sqlx::query_as::<_, Anime>("SELECT * FROM anime WHERE mikan_id = ?")
                .bind(mikan_id)
                .fetch_optional(self.pool)
                .await
                .map_err(|e| {
                    crate::error::AppError::Database(crate::error::DatabaseError::Other(
                        e.to_string(),
                    ))
                })?,
        )
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Anime>> {
        let query = if limit > 0 {
            "SELECT * FROM anime LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM anime LIMIT -1 OFFSET 0"
        };
        Ok(sqlx::query_as::<_, Anime>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await
            .map_err(|e| {
                crate::error::AppError::Database(crate::error::DatabaseError::Other(e.to_string()))
            })?)
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
        .await.map_err(|e| crate::error::AppError::Database(crate::error::DatabaseError::Other(e.to_string())))?;
        Ok(())
    }

    async fn delete(&self, mikan_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM anime WHERE mikan_id = ?")
            .bind(mikan_id)
            .execute(self.pool)
            .await
            .map_err(|e| {
                crate::error::AppError::Database(crate::error::DatabaseError::Other(e.to_string()))
            })?;
        Ok(())
    }
}
