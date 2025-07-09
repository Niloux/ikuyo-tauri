use crate::error::Result;
use crate::models::{EpisodeResourceCount, Resource};
use crate::repositories::base::Repository;
use async_trait::async_trait;
use sqlx::{QueryBuilder, SqlitePool, Transaction};

pub struct ResourceRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> ResourceRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    // ResourceRepository 特有方法
    pub async fn filter(
        &self,
        mikan_id: i64,
        resolution: Option<String>,
        episode_number: Option<i32>,
        subtitle_type: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Resource>> {
        let mut builder = QueryBuilder::new("SELECT * FROM resource WHERE mikan_id = ");
        builder.push_bind(mikan_id);

        if let Some(res) = resolution {
            builder.push(" AND resolution = ");
            builder.push_bind(res);
        }
        if let Some(ep) = episode_number {
            builder.push(" AND episode_number = ");
            builder.push_bind(ep);
        }
        if let Some(st) = subtitle_type {
            builder.push(" AND subtitle_type = ");
            builder.push_bind(st);
        }

        builder.push(" ORDER BY release_date DESC");

        if limit > 0 {
            builder.push(" LIMIT ");
            builder.push_bind(limit);
        }

        builder.push(" OFFSET ");
        builder.push_bind(offset);

        Ok(builder.build_query_as().fetch_all(self.pool).await?)
    }

    pub async fn count_by_episode(&self, mikan_id: i64) -> Result<Vec<EpisodeResourceCount>> {
        Ok(sqlx::query_as::<_, EpisodeResourceCount>(
            "SELECT episode_number, COUNT(*) as resource_count FROM resource WHERE mikan_id = ? AND episode_number IS NOT NULL GROUP BY episode_number ORDER BY episode_number",
        )
        .bind(mikan_id)
        .fetch_all(self.pool)
        .await?)
    }

    pub async fn insert_many_resources(
        &self,
        tx: &mut Transaction<'_, sqlx::Sqlite>,
        resources: &[Resource],
    ) -> Result<()> {
        for resource in resources {
            sqlx::query(
                "INSERT OR IGNORE INTO resource (mikan_id, subtitle_group_id, episode_number, title, file_size, resolution, subtitle_type, magnet_url, torrent_url, play_url, magnet_hash, release_date, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(resource.mikan_id)
            .bind(resource.subtitle_group_id)
            .bind(&resource.episode_number)
            .bind(&resource.title)
            .bind(&resource.file_size)
            .bind(&resource.resolution)
            .bind(&resource.subtitle_type)
            .bind(&resource.magnet_url)
            .bind(&resource.torrent_url)
            .bind(&resource.play_url)
            .bind(&resource.magnet_hash)
            .bind(&resource.release_date)
            .bind(&resource.created_at)
            .bind(&resource.updated_at)
            .execute(&mut **tx)
            .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<'a> Repository<Resource, i64> for ResourceRepository<'a> {
    async fn create(&self, resource: &Resource) -> Result<()> {
        sqlx::query(
            "INSERT INTO resource (mikan_id, subtitle_group_id, episode_number, title, file_size, resolution, subtitle_type, magnet_url, torrent_url, play_url, magnet_hash, release_date, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(resource.mikan_id)
        .bind(resource.subtitle_group_id)
        .bind(resource.episode_number)
        .bind(&resource.title)
        .bind(&resource.file_size)
        .bind(&resource.resolution)
        .bind(&resource.subtitle_type)
        .bind(&resource.magnet_url)
        .bind(&resource.torrent_url)
        .bind(&resource.play_url)
        .bind(&resource.magnet_hash)
        .bind(resource.release_date)
        .bind(resource.created_at)
        .bind(resource.updated_at)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<Resource>> {
        Ok(
            sqlx::query_as::<_, Resource>("SELECT * FROM resource WHERE id = ?")
                .bind(id)
                .fetch_optional(self.pool)
                .await?,
        )
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Resource>> {
        let query = if limit > 0 {
            "SELECT * FROM resource ORDER BY release_date DESC LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM resource ORDER BY release_date DESC OFFSET ?"
        };
        Ok(sqlx::query_as::<_, Resource>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await?)
    }

    async fn update(&self, resource: &Resource) -> Result<()> {
        sqlx::query(
            "UPDATE resource SET mikan_id = ?, subtitle_group_id = ?, episode_number = ?, title = ?, file_size = ?, resolution = ?, subtitle_type = ?, magnet_url = ?, torrent_url = ?, play_url = ?, magnet_hash = ?, release_date = ?, updated_at = ? WHERE id = ?",
        )
        .bind(resource.mikan_id)
        .bind(resource.subtitle_group_id)
        .bind(resource.episode_number)
        .bind(&resource.title)
        .bind(&resource.file_size)
        .bind(&resource.resolution)
        .bind(&resource.subtitle_type)
        .bind(&resource.magnet_url)
        .bind(&resource.torrent_url)
        .bind(&resource.play_url)
        .bind(&resource.magnet_hash)
        .bind(resource.release_date)
        .bind(resource.updated_at)
        .bind(resource.id)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM resource WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
