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

    /// 批量获取字幕组
    pub async fn get_by_ids(&self, ids: &[i64]) -> Result<Vec<SubtitleGroup>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let mut query = String::from("SELECT * FROM subtitle_group WHERE id IN (");
        for (i, _) in ids.iter().enumerate() {
            if i > 0 {
                query.push_str(", ");
            }
            query.push_str("?");
        }
        query.push(')');
        let mut q = sqlx::query_as::<_, SubtitleGroup>(&query);
        for id in ids {
            q = q.bind(id);
        }
        Ok(q.fetch_all(self.pool).await?)
    }

    pub async fn insert_many_subtitle_groups(
        &self,
        tx: &mut Transaction<'_, sqlx::Sqlite>,
        groups: &[SubtitleGroup],
    ) -> Result<()> {
        use sqlx::QueryBuilder;
        if groups.is_empty() {
            return Ok(());
        }
        let mut builder =
            QueryBuilder::new("INSERT INTO subtitle_group (id, name, last_update, created_at) ");
        builder.push("VALUES ");
        for (i, group) in groups.iter().enumerate() {
            if i > 0 {
                builder.push(", ");
            }
            builder
                .push("(")
                .push_bind(group.id)
                .push(", ")
                .push_bind(&group.name)
                .push(", ")
                .push_bind(&group.last_update)
                .push(", ")
                .push_bind(&group.created_at)
                .push(")");
        }
        builder.push(
            " ON CONFLICT(id) DO UPDATE SET \
                name = excluded.name,\
                last_update = excluded.last_update",
        );
        builder.build().execute(&mut **tx).await?;
        Ok(())
    }
}

#[async_trait]
impl<'a> Repository<SubtitleGroup, i64> for SubtitleGroupRepository<'a> {
    async fn create(&self, group: &SubtitleGroup) -> Result<()> {
        sqlx::query(
            "INSERT INTO subtitle_group (id, name, last_update, created_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                last_update = excluded.last_update;",
        )
        .bind(group.id)
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
            "SELECT * FROM subtitle_group ORDER BY id LIMIT -1 OFFSET 0"
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
