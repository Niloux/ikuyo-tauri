use crate::models::AnimeSubtitleGroup;
use sqlx::SqlitePool;

pub struct AnimeSubtitleGroupRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> AnimeSubtitleGroupRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, asg: &AnimeSubtitleGroup) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
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
        Ok(result.last_insert_rowid())
    }

    pub async fn get_by_id(&self, id: i64) -> Result<Option<AnimeSubtitleGroup>, sqlx::Error> {
        sqlx::query_as::<_, AnimeSubtitleGroup>("SELECT * FROM anime_subtitle_group WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<AnimeSubtitleGroup>, sqlx::Error> {
        let query = if limit > 0 {
            "SELECT * FROM anime_subtitle_group LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM anime_subtitle_group OFFSET ?"
        };
        sqlx::query_as::<_, AnimeSubtitleGroup>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await
    }

    pub async fn update(&self, asg: &AnimeSubtitleGroup) -> Result<(), sqlx::Error> {
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

    pub async fn delete(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM anime_subtitle_group WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_by_mikan_and_group(
        &self,
        mikan_id: i64,
        subtitle_group_id: i64,
    ) -> Result<Option<AnimeSubtitleGroup>, sqlx::Error> {
        sqlx::query_as::<_, AnimeSubtitleGroup>(
            "SELECT * FROM anime_subtitle_group WHERE mikan_id = ? AND subtitle_group_id = ?",
        )
        .bind(mikan_id)
        .bind(subtitle_group_id)
        .fetch_optional(self.pool)
        .await
    }
}
