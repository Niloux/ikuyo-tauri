use crate::error::Result;
use crate::models::ScheduledJob;
use crate::repositories::base::Repository;
use async_trait::async_trait;
use sqlx::SqlitePool;

pub struct ScheduledJobRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> ScheduledJobRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    // ScheduledJobRepository 特有方法
    pub async fn get_by_job_id(&self, job_id: &str) -> Result<Option<ScheduledJob>> {
        Ok(
            sqlx::query_as::<_, ScheduledJob>("SELECT * FROM scheduled_job WHERE job_id = ?")
                .bind(job_id)
                .fetch_optional(self.pool)
                .await?,
        )
    }

    pub async fn list_enabled(
        &self,
        enabled: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ScheduledJob>> {
        let query = if limit > 0 {
            "SELECT * FROM scheduled_job WHERE enabled = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM scheduled_job WHERE enabled = ? ORDER BY created_at DESC LIMIT -1 OFFSET 0"
        };
        Ok(sqlx::query_as::<_, ScheduledJob>(query)
            .bind(enabled)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await?)
    }

    pub async fn list_by_time_range(
        &self,
        start: i64,
        end: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ScheduledJob>> {
        let query = if limit > 0 {
            "SELECT * FROM scheduled_job WHERE created_at >= ? AND created_at <= ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM scheduled_job WHERE created_at >= ? AND created_at <= ? ORDER BY created_at DESC LIMIT -1 OFFSET 0"
        };
        Ok(sqlx::query_as::<_, ScheduledJob>(query)
            .bind(start)
            .bind(end)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await?)
    }
}

#[async_trait]
impl<'a> Repository<ScheduledJob, i64> for ScheduledJobRepository<'a> {
    async fn create(&self, job: &ScheduledJob) -> Result<()> {
        sqlx::query(
            "INSERT INTO scheduled_job (job_id, name, description, cron_expression, crawler_mode, parameters, enabled, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&job.job_id)
        .bind(&job.name)
        .bind(&job.description)
        .bind(&job.cron_expression)
        .bind(&job.crawler_mode)
        .bind(&job.parameters)
        .bind(job.enabled)
        .bind(job.created_at)
        .bind(job.updated_at)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<ScheduledJob>> {
        Ok(
            sqlx::query_as::<_, ScheduledJob>("SELECT * FROM scheduled_job WHERE id = ?")
                .bind(id)
                .fetch_optional(self.pool)
                .await?,
        )
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<ScheduledJob>> {
        let query = if limit > 0 {
            "SELECT * FROM scheduled_job ORDER BY created_at DESC LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM scheduled_job ORDER BY created_at DESC LIMIT -1 OFFSET 0"
        };
        Ok(sqlx::query_as::<_, ScheduledJob>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await?)
    }

    async fn update(&self, job: &ScheduledJob) -> Result<()> {
        sqlx::query(
            "UPDATE scheduled_job SET name = ?, description = ?, cron_expression = ?, crawler_mode = ?, parameters = ?, enabled = ?, updated_at = ? WHERE job_id = ?",
        )
        .bind(&job.name)
        .bind(&job.description)
        .bind(&job.cron_expression)
        .bind(&job.crawler_mode)
        .bind(&job.parameters)
        .bind(job.enabled)
        .bind(job.updated_at)
        .bind(&job.job_id)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM scheduled_job WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
