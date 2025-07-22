use crate::config::Config;
use crate::error::{ApiError, AppError};
use crate::types::bangumi::{BangumiEpisodesData, BangumiSubject, BangumiWeekday};
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct BangumiService {
    base_url: String,
    client: reqwest::Client,
    pool: Arc<SqlitePool>,
    config: Config,
}

impl BangumiService {
    pub fn new(pool: Arc<SqlitePool>, config: Config) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Ikuyo-App/1.0 (https://github.com/your-repo-link)") // 添加User-Agent
            .build()
            .expect("reqwest client 构建失败");
        Self {
            base_url: "https://api.bgm.tv".to_string(),
            client,
            pool,
            config,
        }
    }

    pub async fn get_calendar(&self) -> Result<Vec<BangumiWeekday>, AppError> {
        use chrono::Utc;
        use sqlx::Row;
        let now = Utc::now().timestamp();
        let default_ttl = 24 * 3600; // 24小时
        let ttl = self.config.bangumi_calendar_ttl.unwrap_or(default_ttl);
        let cache_id = 1;

        // 1. 查缓存表
        let row =
            sqlx::query("SELECT content, updated_at, ttl FROM bangumi_calendar_cache WHERE id = ?")
                .bind(cache_id)
                .fetch_optional(&*self.pool)
                .await?;

        if let Some(row) = &row {
            let content: String = row.get(0);
            let updated_at: i64 = row.get(1);
            let ttl: i64 = row.get(2);
            if now - updated_at < ttl {
                let data: Vec<BangumiWeekday> = serde_json::from_str(&content)?;
                return Ok(data);
            }
        }

        // 2. 请求API
        let url = format!("{}/calendar", self.base_url);
        let response = self.client.get(&url).send().await;
        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let data: Vec<BangumiWeekday> = resp.json().await?;
                    let content = serde_json::to_string(&data)?;
                    let updated_at = Utc::now().timestamp();
                    let _ = sqlx::query(
                        "INSERT INTO bangumi_calendar_cache (id, content, updated_at, ttl) VALUES (?, ?, ?, ?) \
                        ON CONFLICT(id) DO UPDATE SET content=excluded.content, updated_at=excluded.updated_at, ttl=excluded.ttl"
                    )
                    .bind(cache_id)
                    .bind(&content)
                    .bind(updated_at)
                    .bind(ttl)
                    .execute(&*self.pool)
                    .await;
                    Ok(data)
                } else {
                    // API失败，降级返回旧缓存
                    if let Some(row) = row {
                        let content: String = row.get(0);
                        let data: Vec<BangumiWeekday> = serde_json::from_str(&content)?;
                        return Ok(data);
                    }
                    Err(AppError::Api(ApiError::Response(format!(
                        "请求失败: {}",
                        resp.status()
                    ))))
                }
            }
            Err(e) => {
                // API失败，降级返回旧缓存
                if let Some(row) = row {
                    let content: String = row.get(0);
                    let data: Vec<BangumiWeekday> = serde_json::from_str(&content)?;
                    return Ok(data);
                }
                Err(e.into())
            }
        }
    }

    pub async fn get_subject(&self, id: i64) -> Result<BangumiSubject, AppError> {
        use chrono::Utc;
        use sqlx::Row;
        let now = Utc::now().timestamp();
        let default_ttl = 12 * 3600; // 12小时，后续可参数化
        let ttl = self.config.bangumi_nonsub_ttl.unwrap_or(default_ttl); // 非订阅默认

        // 1. 查缓存表
        let row =
            sqlx::query("SELECT content, updated_at, ttl FROM bangumi_subject_cache WHERE id = ?")
                .bind(id)
                .fetch_optional(&*self.pool)
                .await?;

        if let Some(row) = row {
            let content: String = row.get(0);
            let updated_at: i64 = row.get(1);
            let ttl: i64 = row.get(2);
            if now - updated_at < ttl {
                // 未过期，直接返回
                let subject: BangumiSubject = serde_json::from_str(&content)?;
                return Ok(subject);
            }
        }

        // 2. 请求API
        let url = format!("{}/v0/subjects/{}", self.base_url, id);
        let response = self.client.get(&url).send().await?;
        let data: BangumiSubject = response.json().await?;
        let content = serde_json::to_string(&data)?;
        let updated_at = Utc::now().timestamp();
        // 3. 写入/更新缓存
        let _ = sqlx::query(
            "INSERT INTO bangumi_subject_cache (id, content, updated_at, ttl) VALUES (?, ?, ?, ?) \
            ON CONFLICT(id) DO UPDATE SET content=excluded.content, updated_at=excluded.updated_at, ttl=excluded.ttl"
        )
        .bind(id)
        .bind(&content)
        .bind(updated_at)
        .bind(ttl)
        .execute(&*self.pool)
        .await;
        Ok(data)
    }

    pub async fn get_episodes(
        &self,
        subject_id: i64,
        episode_type: Option<i64>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<BangumiEpisodesData, AppError> {
        use chrono::Utc;
        use sqlx::Row;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let now = Utc::now().timestamp();
        let default_ttl = 12 * 3600; // 12小时
        let ttl = self.config.bangumi_nonsub_ttl.unwrap_or(default_ttl); // 非订阅默认

        // 生成params_hash
        let mut hasher = DefaultHasher::new();
        subject_id.hash(&mut hasher);
        episode_type.hash(&mut hasher);
        limit.hash(&mut hasher);
        offset.hash(&mut hasher);
        let params_hash = format!("{:x}", hasher.finish());

        // 1. 查缓存表
        let row = sqlx::query("SELECT content, updated_at, ttl FROM bangumi_episodes_cache WHERE id = ? AND params_hash = ?")
            .bind(subject_id)
            .bind(&params_hash)
            .fetch_optional(&*self.pool)
            .await?;

        if let Some(row) = &row {
            let content: String = row.get(0);
            let updated_at: i64 = row.get(1);
            let ttl: i64 = row.get(2);
            if now - updated_at < ttl {
                let data: BangumiEpisodesData = serde_json::from_str(&content)?;
                return Ok(data);
            }
        }

        // 2. 请求API
        let mut url = format!("{}/v0/episodes", self.base_url);
        let mut params = Vec::new();
        params.push(format!("subject_id={}", subject_id));
        if let Some(ep_type) = episode_type {
            params.push(format!("type={}", ep_type));
        }
        if let Some(lim) = limit {
            params.push(format!("limit={}", lim));
        }
        if let Some(off) = offset {
            params.push(format!("offset={}", off));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }
        let response = self.client.get(&url).send().await;
        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let data: BangumiEpisodesData = resp.json().await?;
                    let content = serde_json::to_string(&data)?;
                    let updated_at = Utc::now().timestamp();
                    let _ = sqlx::query(
                        "INSERT INTO bangumi_episodes_cache (id, params_hash, content, updated_at, ttl) VALUES (?, ?, ?, ?, ?) \
                        ON CONFLICT(id, params_hash) DO UPDATE SET content=excluded.content, updated_at=excluded.updated_at, ttl=excluded.ttl"
                    )
                    .bind(subject_id)
                    .bind(&params_hash)
                    .bind(&content)
                    .bind(updated_at)
                    .bind(ttl)
                    .execute(&*self.pool)
                    .await;
                    Ok(data)
                } else {
                    // API失败，降级返回旧缓存
                    if let Some(row) = row {
                        let content: String = row.get(0);
                        let data: BangumiEpisodesData = serde_json::from_str(&content)?;
                        return Ok(data);
                    }
                    Err(AppError::Api(ApiError::Response(format!(
                        "请求失败: {}",
                        resp.status()
                    ))))
                }
            }
            Err(e) => {
                // API失败，降级返回旧缓存
                if let Some(row) = row {
                    let content: String = row.get(0);
                    let data: BangumiEpisodesData = serde_json::from_str(&content)?;
                    return Ok(data);
                }
                Err(e.into())
            }
        }
    }

    /// 通用资源聚合函数
    pub async fn aggregate_resources(
        &self,
        bangumi_id: i64,
        episode: Option<i64>,
        resolution: Option<String>,
        subtitle_type: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Option<crate::types::bangumi::EpisodeResourcesData>, AppError> {
        use crate::repositories::anime::AnimeRepository;
        use crate::repositories::resource::ResourceRepository;
        use crate::repositories::subtitle_group::SubtitleGroupRepository;
        use crate::types::bangumi::{EpisodeResource, EpisodeResourcesData, SubtitleGroupResource};
        use std::collections::HashMap;
        let anime_repo = AnimeRepository::new(&self.pool);
        let resource_repo = ResourceRepository::new(&self.pool);
        let subtitle_group_repo = SubtitleGroupRepository::new(&self.pool);

        let anime = anime_repo.get_by_bangumi_id(bangumi_id).await?;
        if let Some(anime) = anime {
            let resources = resource_repo
                .filter(
                    anime.mikan_id,
                    resolution,
                    episode.map(|e| e as i32),
                    subtitle_type,
                    limit.unwrap_or(0),
                    offset.unwrap_or(0),
                )
                .await?;

            // 收集所有 group_id
            let group_ids: Vec<i64> = resources.iter().map(|r| r.subtitle_group_id).collect();
            let groups = subtitle_group_repo.get_by_ids(&group_ids).await?;
            let group_map: HashMap<i64, String> = groups
                .into_iter()
                .filter_map(|g| g.id.map(|id| (id, g.name)))
                .collect();

            let mut subtitle_groups_map: HashMap<i64, SubtitleGroupResource> = HashMap::new();
            let mut total_resources = 0;

            for res in resources {
                total_resources += 1;
                let group_id = res.subtitle_group_id;
                let group_name = group_map
                    .get(&group_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string());

                let entry =
                    subtitle_groups_map
                        .entry(group_id)
                        .or_insert_with(|| SubtitleGroupResource {
                            id: group_id,
                            name: group_name.clone(),
                            resource_count: 0,
                            resources: Vec::new(),
                        });

                entry.resource_count += 1;
                entry.resources.push(EpisodeResource {
                    id: res.id.unwrap_or_default(),
                    episode_number: res.episode_number.unwrap_or_default() as i64,
                    title: res.title,
                    resolution: res.resolution.unwrap_or_default(),
                    subtitle_type: res.subtitle_type.unwrap_or_default(),
                    magnet_url: res.magnet_url.unwrap_or_default(),
                    torrent_url: res.torrent_url.unwrap_or_default(),
                    release_date: res.release_date.unwrap_or_default().to_string(),
                    size: res.file_size.unwrap_or_default(),
                    group_id: res.subtitle_group_id,
                    group_name,
                });
            }

            let mut subtitle_groups: Vec<SubtitleGroupResource> =
                subtitle_groups_map.into_values().collect();
            subtitle_groups.sort_by_key(|g| g.id);

            Ok(Some(EpisodeResourcesData {
                total_resources,
                subtitle_groups,
            }))
        } else {
            Ok(None)
        }
    }
}
