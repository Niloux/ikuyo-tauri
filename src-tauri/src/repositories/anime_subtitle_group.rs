use crate::models::AnimeSubtitleGroup;
use crate::error::Result;
use crate::repositories::base::Repository;
use async_trait::async_trait;
use sqlx::{SqlitePool, Transaction};

pub struct AnimeSubtitleGroupRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> AnimeSubtitleGroupRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    // AnimeSubtitleGroupRepository 特有方法
    pub async fn get_by_mikan_and_group(
        &self,
        mikan_id: i64,
        subtitle_group_id: i64,
    ) -> Result<Option<AnimeSubtitleGroup>> {
        Ok(sqlx::query_as::<_, AnimeSubtitleGroup>(
            "SELECT * FROM anime_subtitle_group WHERE mikan_id = ? AND subtitle_group_id = ?",
        )
        .bind(mikan_id)
        .bind(subtitle_group_id)
        .fetch_optional(self.pool)
        .await?)
    }

    pub async fn insert_many_anime_subtitle_groups(&self, tx: &mut Transaction<'_, sqlx::Sqlite>, items: &[AnimeSubtitleGroup]) -> Result<()> {
        for item in items {
            sqlx::query(
                "INSERT OR IGNORE INTO anime_subtitle_group (mikan_id, subtitle_group_id, first_release_date, last_update_date, resource_count, is_active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(item.mikan_id)
            .bind(item.subtitle_group_id)
            .bind(item.first_release_date)
            .bind(item.last_update_date)
            .bind(item.resource_count)
            .bind(item.is_active)
            .bind(item.created_at)
            .bind(item.updated_at)
            .execute(&mut **tx)
            .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<'a> Repository<AnimeSubtitleGroup, i64> for AnimeSubtitleGroupRepository<'a> {
    async fn create(&self, asg: &AnimeSubtitleGroup) -> Result<()> {
        sqlx::query(
            "INSERT INTO anime_subtitle_group (mikan_id, subtitle_group_id, first_release_date, last_update_date, resource_count, is_active, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(asg.mikan_id)
        .bind(asg.subtitle_group_id)
        .bind(asg.first_release_date)
        .bind(asg.last_update_date)
        .bind(asg.resource_count)
        .bind(asg.is_active)
        .bind(asg.created_at)
        .bind(asg.updated_at)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<AnimeSubtitleGroup>> {
        Ok(sqlx::query_as::<_, AnimeSubtitleGroup>("SELECT * FROM anime_subtitle_group WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool)
            .await?)
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<AnimeSubtitleGroup>> {
        let query = if limit > 0 {
            "SELECT * FROM anime_subtitle_group LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM anime_subtitle_group LIMIT -1 OFFSET 0"
        };
        Ok(sqlx::query_as::<_, AnimeSubtitleGroup>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await?)
    }

    async fn update(&self, asg: &AnimeSubtitleGroup) -> Result<()> {
        sqlx::query(
            "UPDATE anime_subtitle_group SET first_release_date = ?, last_update_date = ?, resource_count = ?, is_active = ?, updated_at = ? WHERE id = ?",
        )
        .bind(asg.first_release_date)
        .bind(asg.last_update_date)
        .bind(asg.resource_count)
        .bind(asg.is_active)
        .bind(asg.updated_at)
        .bind(asg.id)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM anime_subtitle_group WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
