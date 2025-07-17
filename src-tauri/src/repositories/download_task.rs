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

    // 查询所有未完成的任务,用于初始化时恢复下载
    pub async fn get_all_resumable_tasks(&self) -> Result<Vec<DownloadTask>> {
        let query = "SELECT * FROM download_task WHERE status != 'completed' AND status != 'failed' AND status != 'deleted'";
        Ok(sqlx::query_as::<_, DownloadTask>(query)
            .fetch_all(self.pool)
            .await?)
    }

    // 查询所有未删除的任务,用于前端fetch_all_downloads
    pub async fn fetch_all_downloads(&self) -> Result<Vec<DownloadTask>> {
        let query = "SELECT * FROM download_task WHERE status != 'deleted'";
        Ok(sqlx::query_as::<_, DownloadTask>(query)
            .fetch_all(self.pool)
            .await?)
    }
}

#[async_trait]
impl<'a> Repository<DownloadTask, i64> for DownloadTaskRepository<'a> {
    // 插入一个新任务，初始状态为metadata，返回新的任务id
    async fn create(&self, task: &DownloadTask) -> Result<()> {
        sqlx::query(
            "INSERT INTO download_task (magnet_url, save_path, status, bangumi_id, resource_id, episode_number, name, name_cn, cover, total_size, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&task.magnet_url)
        .bind(&task.save_path)
        .bind("metadata")
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
            "UPDATE download_task SET status = ?, error_msg = ?, updated_at = ? total_size = ? WHERE id = ?",
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
