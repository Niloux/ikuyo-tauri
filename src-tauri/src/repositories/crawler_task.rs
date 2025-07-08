use crate::models::{CrawlerTask, CrawlerTaskStatus, CrawlerTaskType};
use sqlx::SqlitePool;

pub struct CrawlerTaskRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> CrawlerTaskRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, task: &CrawlerTask) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
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
        Ok(result.last_insert_rowid())
    }

    pub async fn get_by_id(&self, id: i64) -> Result<Option<CrawlerTask>, sqlx::Error> {
        sqlx::query_as::<_, CrawlerTask>("SELECT * FROM crawler_task WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<CrawlerTask>, sqlx::Error> {
        if limit > 0 {
            sqlx::query_as::<_, CrawlerTask>(
                "SELECT * FROM crawler_task ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await
        } else {
            // 当limit为0时，获取所有记录，忽略offset
            sqlx::query_as::<_, CrawlerTask>(
                "SELECT * FROM crawler_task ORDER BY created_at DESC"
            )
            .fetch_all(self.pool)
            .await
        }
    }

    pub async fn update(&self, task: &CrawlerTask) -> Result<(), sqlx::Error> {
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

    pub async fn delete(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM crawler_task WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_by_status(
        &self,
        status: CrawlerTaskStatus,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CrawlerTask>, sqlx::Error> {
        if limit > 0 {
            sqlx::query_as::<_, CrawlerTask>(
                "SELECT * FROM crawler_task WHERE status = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .bind(status)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await
        } else {
            sqlx::query_as::<_, CrawlerTask>(
                "SELECT * FROM crawler_task WHERE status = ? ORDER BY created_at DESC"
            )
            .bind(status)
            .fetch_all(self.pool)
            .await
        }
    }

    pub async fn list_by_type(
        &self,
        task_type: CrawlerTaskType,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CrawlerTask>, sqlx::Error> {
        if limit > 0 {
            sqlx::query_as::<_, CrawlerTask>(
                "SELECT * FROM crawler_task WHERE task_type = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .bind(task_type)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await
        } else {
            sqlx::query_as::<_, CrawlerTask>(
                "SELECT * FROM crawler_task WHERE task_type = ? ORDER BY created_at DESC"
            )
            .bind(task_type)
            .fetch_all(self.pool)
            .await
        }
    }

    pub async fn list_by_time_range(
        &self,
        start: i64,
        end: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CrawlerTask>, sqlx::Error> {
        if limit > 0 {
            sqlx::query_as::<_, CrawlerTask>(
                "SELECT * FROM crawler_task WHERE created_at >= ? AND created_at <= ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
            )
            .bind(start)
            .bind(end)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await
        } else {
            sqlx::query_as::<_, CrawlerTask>(
                "SELECT * FROM crawler_task WHERE created_at >= ? AND created_at <= ? ORDER BY created_at DESC"
            )
            .bind(start)
            .bind(end)
            .fetch_all(self.pool)
            .await
        }
    }
}
