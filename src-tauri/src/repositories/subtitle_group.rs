use crate::models::SubtitleGroup;
use sqlx::SqlitePool;

pub struct SubtitleGroupRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SubtitleGroupRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, group: &SubtitleGroup) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO subtitle_group (name, last_update, created_at) VALUES (?, ?, ?)",
        )
        .bind(&group.name)
        .bind(group.last_update)
        .bind(group.created_at)
        .execute(self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get_by_id(&self, id: i64) -> Result<Option<SubtitleGroup>, sqlx::Error> {
        sqlx::query_as::<_, SubtitleGroup>("SELECT * FROM subtitle_group WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<SubtitleGroup>, sqlx::Error> {
        let query = if limit > 0 {
            "SELECT * FROM subtitle_group ORDER BY id LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM subtitle_group ORDER BY id OFFSET ?"
        };
        sqlx::query_as::<_, SubtitleGroup>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await
    }

    pub async fn update(&self, group: &SubtitleGroup) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE subtitle_group SET name = ?, last_update = ? WHERE id = ?")
            .bind(&group.name)
            .bind(group.last_update)
            .bind(group.id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, id: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM subtitle_group WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Option<SubtitleGroup>, sqlx::Error> {
        sqlx::query_as::<_, SubtitleGroup>("SELECT * FROM subtitle_group WHERE name = ?")
            .bind(name)
            .fetch_optional(self.pool)
            .await
    }
}
