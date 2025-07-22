use crate::error::Result;
use crate::models::DownloadTask;
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
            "INSERT INTO download_task (id, magnet_url, save_path, title, status, bangumi_id, resource_id, episode_number, name, name_cn, cover, total_size, created_at, updated_at, error_msg)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(task.id)
        .bind(&task.magnet_url)
        .bind(&task.save_path)
        .bind(&task.title)
        .bind(&task.status)
        .bind(task.bangumi_id)
        .bind(task.resource_id)
        .bind(task.episode_number)
        .bind(&task.name)
        .bind(&task.name_cn)
        .bind(&task.cover)
        .bind(task.total_size)
        .bind(task.created_at)
        .bind(task.updated_at)
        .bind(&task.error_msg)
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

    // 更新任务状态和错误信息
    async fn update(&self, task: &DownloadTask) -> Result<()> {
        sqlx::query(
            "UPDATE download_task SET status = ?, error_msg = ?, updated_at = ?, total_size = ? WHERE id = ?",
        )
        .bind(&task.status)
        .bind(&task.error_msg)
        .bind(task.updated_at)
        .bind(task.total_size)
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
