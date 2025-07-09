use crate::models::{Anime, CrawlerTask, CrawlerTaskStatus, Resource, SubtitleGroup};
use crate::repositories::{
    anime::AnimeRepository, base::Repository, crawler_task::CrawlerTaskRepository,
    resource::ResourceRepository, subtitle_group::SubtitleGroupRepository,
};
use sqlx::{SqlitePool, Transaction};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crate::core::mikan_fetcher::{MikanFetcher, AnimeData};

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

    pub async fn run(&mut self) {
        self.update_progress(CrawlerTaskStatus::Running, 0.0, None).await;
        let mut error: Option<String> = None;
        let mut success = true;
        let repo = CrawlerTaskRepository::new(&self.pool);
        let base_url = "https://mikanani.me"; // TODO: 可从配置获取
        let list_url = format!("{}/Home", base_url); // TODO: 支持多种模式
        let fetcher = MikanFetcher::new(base_url, Some("http://127.0.0.1:7890"));
        let limit = None; // TODO: 支持limit
        let mut detail_urls = Vec::new();
        match fetcher.fetch_and_parse_list(&list_url, limit).await {
            Ok(urls) => {
                self.total_items = urls.len() as i64;
                detail_urls = urls;
            }
            Err(e) => {
                error = Some(format!("列表页抓取失败: {}", e));
                self.update_progress(CrawlerTaskStatus::Failed, 0.0, error.clone()).await;
                return;
            }
        }
        for (i, url) in detail_urls.iter().enumerate() {
            // 检查任务是否被取消
            if let Ok(Some(task)) = repo.get_by_id(self.task_id).await {
                if task.status == CrawlerTaskStatus::Cancelled {
                    self.update_progress(
                        CrawlerTaskStatus::Cancelled,
                        (i as f64) / (self.total_items as f64),
                        Some("任务被取消".to_string()),
                    ).await;
                    return;
                }
            }
            // 详情页抓取与解析
            match fetcher.fetch_and_parse_detail(url).await {
                Ok(anime_data) => {
                    // Anime 去重与缓冲
                    if let Some(anime) = anime_data.anime {
                        let mikan_id = anime.mikan_id;
                        if !self.anime_ids.contains(&mikan_id) {
                            self.anime_ids.insert(mikan_id);
                            self.anime_buffer.push(anime);
                        }
                    }
                    // SubtitleGroup 去重与缓冲
                    for group in anime_data.subtitle_groups {
                        if let Some(group_id) = group.id {
                            if !self.subtitle_group_ids.contains(&group_id) {
                                self.subtitle_group_ids.insert(group_id);
                                self.subtitle_group_buffer.push(group);
                            }
                        }
                    }
                    // Resource 去重与缓冲
                    for res in anime_data.resources {
                        if let Some(hash) = &res.magnet_hash {
                            if !self.resource_hashes.contains(hash) {
                                self.resource_hashes.insert(hash.clone());
                                self.resource_buffer.push(res);
                            }
                        }
                    }
                    self.processed_items += 1;
                }
                Err(e) => {
                    error = Some(format!("详情页抓取失败: {}", e));
                    success = false;
                    break;
                }
            }
            // 每处理10条更新一次进度
            if i % 10 == 0 || i + 1 == detail_urls.len() {
                let percent = (i + 1) as f64 / (self.total_items as f64);
                self.update_progress(CrawlerTaskStatus::Running, percent, None).await;
            }
        }
        // 批量入库
        if success {
            let pool = self.pool.clone();
            let mut tx = match pool.begin().await {
                Ok(tx) => tx,
                Err(e) => {
                    error = Some(format!("数据库事务启动失败: {}", e));
                    self.update_progress(CrawlerTaskStatus::Failed, 1.0, error.clone()).await;
                    return;
                }
            };
            if let Err(e) = self.batch_insert_all(&mut tx).await {
                error = Some(format!("批量入库失败: {}", e));
                self.update_progress(CrawlerTaskStatus::Failed, 1.0, error.clone()).await;
                return;
            }
            if let Err(e) = tx.commit().await {
                error = Some(format!("事务提交失败: {}", e));
                self.update_progress(CrawlerTaskStatus::Failed, 1.0, error.clone()).await;
                return;
            }
            self.update_progress(CrawlerTaskStatus::Completed, 1.0, None).await;
        } else {
            self.update_progress(CrawlerTaskStatus::Failed, 1.0, error.clone()).await;
        }
    }

    pub async fn batch_insert_all(
        &self,
        tx: &mut Transaction<'_, sqlx::Sqlite>,
    ) -> anyhow::Result<()> {
        let anime_repo = AnimeRepository::new(&self.pool);
        let subtitle_repo = SubtitleGroupRepository::new(&self.pool);
        let resource_repo = ResourceRepository::new(&self.pool);
        for anime in &self.anime_buffer {
            let _ = anime_repo.create(anime).await;
        }
        for group in &self.subtitle_group_buffer {
            let _ = subtitle_repo.create(group).await;
        }
        for res in &self.resource_buffer {
            let _ = resource_repo.create(res).await;
        }
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
