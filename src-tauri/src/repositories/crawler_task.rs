use crate::error::Result;
use crate::models::{CrawlerTask, CrawlerTaskStatus, CrawlerTaskType};
use crate::repositories::base::Repository;
use async_trait::async_trait;
use sqlx::SqlitePool;

pub struct CrawlerTaskRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> CrawlerTaskRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    // CrawlerTaskRepository 特有方法
    pub async fn list_by_status(
        &self,
        status: CrawlerTaskStatus,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CrawlerTask>> {
        let query = if limit > 0 {
            "SELECT * FROM crawler_task WHERE status = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM crawler_task WHERE status = ? ORDER BY created_at DESC LIMIT -1 OFFSET 0"
        };
        Ok(sqlx::query_as::<_, CrawlerTask>(query)
            .bind(status)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await?)
    }

    pub async fn list_by_type(
        &self,
        task_type: CrawlerTaskType,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CrawlerTask>> {
        let query = if limit > 0 {
            "SELECT * FROM crawler_task WHERE task_type = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM crawler_task WHERE task_type = ? ORDER BY created_at DESC LIMIT -1 OFFSET 0"
        };
        Ok(sqlx::query_as::<_, CrawlerTask>(query)
            .bind(task_type)
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
    ) -> Result<Vec<CrawlerTask>> {
        let query = if limit > 0 {
            "SELECT * FROM crawler_task WHERE created_at >= ? AND created_at <= ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM crawler_task WHERE created_at >= ? AND created_at <= ? ORDER BY created_at DESC LIMIT -1 OFFSET 0"
        };
        Ok(sqlx::query_as::<_, CrawlerTask>(query)
            .bind(start)
            .bind(end)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await?)
    }

    // 批量将Running状态的任务标记为Failed，并写入错误信息
    pub async fn mark_all_running_as_failed(&self, error_message: &str) -> Result<u64> {
        let now = chrono::Utc::now().timestamp_millis();
        let result = sqlx::query(
            "UPDATE crawler_task SET status = ?, completed_at = ?, error_message = ? WHERE status = ?"
        )
        .bind(CrawlerTaskStatus::Failed)
        .bind(now)
        .bind(error_message)
        .bind(CrawlerTaskStatus::Running)
        .execute(self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

#[async_trait]
impl<'a> Repository<CrawlerTask, i64> for CrawlerTaskRepository<'a> {
    async fn create(&self, task: &CrawlerTask) -> Result<()> {
        sqlx::query(
            "INSERT INTO crawler_task (task_type, status, parameters, result_summary, created_at, started_at, completed_at, error_message, percentage, processed_items, total_items, processing_speed, estimated_remaining)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&task.task_type)
        .bind(&task.status)
        .bind(&task.parameters)
        .bind(&task.result_summary)
        .bind(task.created_at)
        .bind(task.started_at)
        .bind(task.completed_at)
        .bind(&task.error_message)
        .bind(task.percentage)
        .bind(task.processed_items)
        .bind(task.total_items)
        .bind(task.processing_speed)
        .bind(task.estimated_remaining)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<CrawlerTask>> {
        Ok(
            sqlx::query_as::<_, CrawlerTask>("SELECT * FROM crawler_task WHERE id = ?")
                .bind(id)
                .fetch_optional(self.pool)
                .await?,
        )
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<CrawlerTask>> {
        let query = if limit > 0 {
            "SELECT * FROM crawler_task ORDER BY created_at DESC LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM crawler_task ORDER BY created_at DESC LIMIT -1 OFFSET 0"
        };
        Ok(sqlx::query_as::<_, CrawlerTask>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await?)
    }

    async fn update(&self, task: &CrawlerTask) -> Result<()> {
        sqlx::query(
            "UPDATE crawler_task SET task_type = ?, status = ?, parameters = ?, result_summary = ?, started_at = ?, completed_at = ?, error_message = ?, percentage = ?, processed_items = ?, total_items = ?, processing_speed = ?, estimated_remaining = ? WHERE id = ?",
        )
        .bind(&task.task_type)
        .bind(&task.status)
        .bind(&task.parameters)
        .bind(&task.result_summary)
        .bind(task.started_at)
        .bind(task.completed_at)
        .bind(&task.error_message)
        .bind(task.percentage)
        .bind(task.processed_items)
        .bind(task.total_items)
        .bind(task.processing_speed)
        .bind(task.estimated_remaining)
        .bind(task.id)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM crawler_task WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
