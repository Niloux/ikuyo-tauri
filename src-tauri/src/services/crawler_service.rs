use crate::models::{Anime, CrawlerTask, CrawlerTaskStatus, Resource, SubtitleGroup};
use crate::repositories::{
    anime::AnimeRepository, base::Repository, crawler_task::CrawlerTaskRepository,
    resource::ResourceRepository, subtitle_group::SubtitleGroupRepository,
};
use sqlx::{SqlitePool, Transaction};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone)]
pub struct CrawlResult {
    pub anime: Option<Anime>,
    pub subtitle_group: Option<SubtitleGroup>,
    pub resource: Option<Resource>,
}

pub fn create_result_channel(buffer: usize) -> (Sender<CrawlResult>, Receiver<CrawlResult>) {
    tokio::sync::mpsc::channel(buffer)
}

pub async fn result_consumer(mut rx: Receiver<CrawlResult>, pool: Arc<SqlitePool>) {
    let anime_repo = AnimeRepository::new(&pool);
    let subtitle_repo = SubtitleGroupRepository::new(&pool);
    let resource_repo = ResourceRepository::new(&pool);
    while let Some(result) = rx.recv().await {
        // 简化：分别写入，实际可批量、去重、加事务
        if let Some(anime) = &result.anime {
            let _ = anime_repo.create(anime).await;
        }
        if let Some(sub) = &result.subtitle_group {
            let _ = subtitle_repo.create(sub).await;
        }
        if let Some(res) = &result.resource {
            let _ = resource_repo.create(res).await;
        }
    }
}

pub struct CrawlerService {
    pub pool: Arc<SqlitePool>,
    pub task_id: i64,
    pub anime_buffer: Vec<Anime>,
    pub subtitle_group_buffer: Vec<SubtitleGroup>,
    pub resource_buffer: Vec<Resource>,
    pub anime_ids: HashSet<i64>,
    pub subtitle_group_ids: HashSet<i64>,
    pub resource_hashes: HashSet<String>,
    pub processed_items: i64,
    pub total_items: i64,
}

impl CrawlerService {
    pub fn new(pool: Arc<SqlitePool>, task_id: i64) -> Self {
        Self {
            pool,
            task_id,
            anime_buffer: Vec::new(),
            subtitle_group_buffer: Vec::new(),
            resource_buffer: Vec::new(),
            anime_ids: HashSet::new(),
            subtitle_group_ids: HashSet::new(),
            resource_hashes: HashSet::new(),
            processed_items: 0,
            total_items: 0,
        }
    }

    pub async fn run(&mut self, result_tx: Option<Sender<CrawlResult>>) {
        self.update_progress(CrawlerTaskStatus::Running, 0.0, None)
            .await;
        let mut error: Option<String> = None;
        let mut success = true;
        let repo = CrawlerTaskRepository::new(&self.pool);
        for i in 1..=10 {
            // 检查是否被取消
            if let Ok(Some(task)) = repo.get_by_id(self.task_id).await {
                if task.status == CrawlerTaskStatus::Cancelled {
                    self.update_progress(
                        CrawlerTaskStatus::Cancelled,
                        (i as f64) / 10.0,
                        Some("任务被取消".to_string()),
                    )
                    .await;
                    return;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            self.processed_items = i;
            self.total_items = 10;
            let percent = (i as f64) / 10.0;
            if let Some(ref tx) = result_tx {
                let _ = tx
                    .send(CrawlResult {
                        anime: None,
                        subtitle_group: None,
                        resource: None,
                    })
                    .await;
            }
            self.update_progress(CrawlerTaskStatus::Running, percent, None)
                .await;
        }
        // 3. 批量入库（如未用channel则本地批量）
        let pool = self.pool.clone();
        let mut tx = match pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error = Some(format!("数据库事务启动失败: {}", e));
                success = false;
                self.update_progress(CrawlerTaskStatus::Failed, 1.0, error.clone())
                    .await;
                return;
            }
        };
        if let Err(e) = self.batch_insert_all(&mut tx).await {
            error = Some(format!("批量入库失败: {}", e));
            success = false;
            self.update_progress(CrawlerTaskStatus::Failed, 1.0, error.clone())
                .await;
            return;
        }
        if let Err(e) = tx.commit().await {
            error = Some(format!("事务提交失败: {}", e));
            success = false;
            self.update_progress(CrawlerTaskStatus::Failed, 1.0, error.clone())
                .await;
            return;
        }
        // 4. 结束状态
        if success {
            self.update_progress(CrawlerTaskStatus::Completed, 1.0, None)
                .await;
        }
    }

    pub async fn batch_insert_all(
        &self,
        tx: &mut Transaction<'_, sqlx::Sqlite>,
    ) -> anyhow::Result<()> {
        // TODO: 调用各repository的批量插入方法
        Ok(())
    }

    pub async fn update_progress(
        &self,
        status: CrawlerTaskStatus,
        percentage: f64,
        error_message: Option<String>,
    ) {
        // TODO: 更新crawler_task表的进度、状态、错误信息
    }

    // 可扩展更多辅助方法，如去重、缓冲flush等
}
