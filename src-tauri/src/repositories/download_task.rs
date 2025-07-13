use crate::error::Result;
use crate::models::{DownloadTask, DownloadStatus};
use crate::repositories::base::Repository;
use async_trait::async_trait;
use sqlx::SqlitePool;

pub struct DownloadTaskRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> DownloadTaskRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> Repository<DownloadTask, i64> for DownloadTaskRepository<'a> {
    async fn create(&self, task: &DownloadTask) -> Result<()> {
        sqlx::query(
            "INSERT INTO download_task (anime_id, episode_id, resource_id, resource_url, file_path, status, progress, speed, error_msg, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(task.anime_id)
        .bind(task.episode_id)
        .bind(task.resource_id)
        .bind(&task.resource_url)
        .bind(&task.file_path)
        .bind(&task.status)
        .bind(task.progress)
        .bind(task.speed)
        .bind(&task.error_msg)
        .bind(task.created_at)
        .bind(task.updated_at)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<DownloadTask>> {
        Ok(
            sqlx::query_as::<_, DownloadTask>("SELECT * FROM download_task WHERE id = ?")
                .bind(id)
                .fetch_optional(self.pool)
                .await?,
        )
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<DownloadTask>> {
        let query = if limit > 0 {
            "SELECT * FROM download_task ORDER BY created_at DESC LIMIT ? OFFSET ?"
        } else {
            "SELECT * FROM download_task ORDER BY created_at DESC LIMIT -1 OFFSET 0"
        };
        Ok(sqlx::query_as::<_, DownloadTask>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await?)
    }

    async fn update(&self, task: &DownloadTask) -> Result<()> {
        sqlx::query(
            "UPDATE download_task SET anime_id = ?, episode_id = ?, resource_id = ?, resource_url = ?, file_path = ?, status = ?, progress = ?, speed = ?, error_msg = ?, created_at = ?, updated_at = ? WHERE id = ?"
        )
        .bind(task.anime_id)
        .bind(task.episode_id)
        .bind(task.resource_id)
        .bind(&task.resource_url)
        .bind(&task.file_path)
        .bind(&task.status)
        .bind(task.progress)
        .bind(task.speed)
        .bind(&task.error_msg)
        .bind(task.created_at)
        .bind(task.updated_at)
        .bind(task.id)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM download_task WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}