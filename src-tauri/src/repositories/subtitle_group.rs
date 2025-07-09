use crate::error::Result;
use crate::models::SubtitleGroup;
use crate::repositories::base::Repository;
use async_trait::async_trait;
use sqlx::{SqlitePool, Transaction};

pub struct SubtitleGroupRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SubtitleGroupRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    // SubtitleGroupRepository 特有方法
    pub async fn get_by_name(&self, name: &str) -> Result<Option<SubtitleGroup>> {
        Ok(
            sqlx::query_as::<_, SubtitleGroup>("SELECT * FROM subtitle_group WHERE name = ?")
                .bind(name)
                .fetch_optional(self.pool)
                .await?,
        )
    }

    pub async fn insert_many_subtitle_groups(
        &self,
        tx: &mut Transaction<'_, sqlx::Sqlite>,
        groups: &[SubtitleGroup],
    ) -> Result<()> {
        for group in groups {
            sqlx::query(
                "INSERT OR IGNORE INTO subtitle_group (id, name, last_update, created_at) VALUES (?, ?, ?, ?)"
            )
            .bind(group.id)
            .bind(&group.name)
            .bind(&group.last_update)
            .bind(&group.created_at)
            .execute(&mut **tx)
            .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<'a> Repository<SubtitleGroup, i64> for SubtitleGroupRepository<'a> {
    async fn create(&self, group: &SubtitleGroup) -> Result<()> {
        sqlx::query("INSERT INTO subtitle_group (name, last_update, created_at) VALUES (?, ?, ?)")
            .bind(&group.name)
            .bind(group.last_update)
            .bind(group.created_at)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<SubtitleGroup>> {
        Ok(
            sqlx::query_as::<_, SubtitleGroup>("SELECT * FROM subtitle_group WHERE id = ?")
                .bind(id)
                .fetch_optional(self.pool)
                .await?,
        )
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<SubtitleGroup>> {
        let query = if limit > 0 {
            "SELECT * FROM subtitle_group ORDER BY id LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM subtitle_group ORDER BY id OFFSET ?"
        };
        Ok(sqlx::query_as::<_, SubtitleGroup>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await?)
    }

    async fn update(&self, group: &SubtitleGroup) -> Result<()> {
        sqlx::query("UPDATE subtitle_group SET name = ?, last_update = ? WHERE id = ?")
            .bind(&group.name)
            .bind(group.last_update)
            .bind(group.id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM subtitle_group WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
