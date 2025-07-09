use crate::{
    repositories::{
        anime::AnimeRepository,
        base::Repository, // <--- 导入 trait
        resource::ResourceRepository,
        subtitle_group::SubtitleGroupRepository,
    },
    services::bangumi_service::BangumiService,
    types::bangumi::{
        BangumiEpisodesData, BangumiSubject, BangumiWeekday, EpisodeAvailabilityData,
        EpisodeResource, EpisodeResourcesData, Pagination, SearchLibraryResponse,
        SubtitleGroupResource,
    },
};
use sqlx::SqlitePool;
use tauri::{command, State};
use std::sync::Arc;

#[command(rename_all = "snake_case")]
pub async fn get_calendar() -> Result<Vec<BangumiWeekday>, String> {
    let service = BangumiService::new();
    service.get_calendar().await
}

#[command(rename_all = "snake_case")]
pub async fn get_subject(id: i64) -> Result<BangumiSubject, String> {
    let service = BangumiService::new();
    service.get_subject(id).await
}

#[command(rename_all = "snake_case")]
pub async fn get_episodes(
    subject_id: i64,
    episode_type: Option<i64>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<BangumiEpisodesData, String> {
    let service = BangumiService::new();
    service
        .get_episodes(subject_id, episode_type, limit, offset)
        .await
}

#[command(rename_all = "snake_case")]
pub async fn get_episode_availability(
    bangumi_id: i64,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Option<EpisodeAvailabilityData>, String> {
    let anime_repo = AnimeRepository::new(&pool);
    let resource_repo = ResourceRepository::new(&pool);

    let anime = anime_repo.get_by_bangumi_id(bangumi_id).await.map_err(|e| e.to_string())?;

    if let Some(anime) = anime {
        let episode_counts = resource_repo.count_by_episode(anime.mikan_id).await.map_err(|e| e.to_string())?;

        let mut episodes_map = std::collections::HashMap::new();
        for count in episode_counts {
            episodes_map.insert(
                count.episode_number.to_string(),
                crate::types::bangumi::EpisodeAvailability {
                    available: count.resource_count > 0,
                    resource_count: count.resource_count,
                },
            );
        }

        Ok(Some(EpisodeAvailabilityData {
            bangumi_id,
            episodes: episodes_map,
        }))
    } else {
        Ok(None)
    }
}

#[command(rename_all = "snake_case")]
pub async fn get_episode_resources(
    bangumi_id: i64,
    episode: i64,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Option<EpisodeResourcesData>, String> {
    let anime_repo = AnimeRepository::new(&pool);
    let resource_repo = ResourceRepository::new(&pool);
    let subtitle_group_repo = SubtitleGroupRepository::new(&pool);

    let anime = anime_repo.get_by_bangumi_id(bangumi_id).await.map_err(|e| e.to_string())?;

    if let Some(anime) = anime {
        let resources = resource_repo
            .filter(anime.mikan_id, None, Some(episode as i32), None, 0, 0)
            .await
            .map_err(|e| e.to_string())?;

        let mut subtitle_groups_map: std::collections::HashMap<i64, SubtitleGroupResource> =
            std::collections::HashMap::new();
        let mut total_resources = 0;

        for res in resources {
            total_resources += 1;
            let group_id = res.subtitle_group_id;
            let group_name = subtitle_group_repo
                .get_by_id(group_id)
                .await
                .map_err(|e| e.to_string())?
                .map_or("Unknown".to_string(), |g| g.name);

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

        Ok(Some(EpisodeResourcesData {
            total_resources,
            subtitle_groups: subtitle_groups_map.into_values().collect(),
        }))
    } else {
        Ok(None)
    }
}

#[command(rename_all = "snake_case")]
pub async fn search_library(
    query: String,
    page: i64,
    limit: i64,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<SearchLibraryResponse, String> {
    let anime_repo = AnimeRepository::new(&pool);

    let offset = (page - 1) * limit;
    let animes = anime_repo.search_by_title(&query, limit, offset).await.map_err(|e| e.to_string())?;

    let total_animes = anime_repo.count_by_title(&query).await.map_err(|e| e.to_string())?;

    let bangumi_ids: Vec<i64> = animes.into_iter().map(|anime| anime.bangumi_id).collect();

    let total_pages = (total_animes as f64 / limit as f64).ceil() as i64;

    let pagination = Pagination {
        current_page: page,
        per_page: limit,
        total: total_animes,
        total_pages,
        has_next: (page * limit) < total_animes,
        has_prev: page > 1,
    };

    Ok(SearchLibraryResponse {
        bangumi_ids,
        pagination,
    })
}

#[command(rename_all = "snake_case")]
pub async fn get_anime_resources(
    bangumi_id: i64,
    resolution: Option<String>,
    subtitle_type: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Option<EpisodeResourcesData>, String> {
    let anime_repo = AnimeRepository::new(&pool);
    let resource_repo = ResourceRepository::new(&pool);
    let subtitle_group_repo = SubtitleGroupRepository::new(&pool);

    let anime = anime_repo.get_by_bangumi_id(bangumi_id).await.map_err(|e| e.to_string())?;

    if let Some(anime) = anime {
        let resources = resource_repo
            .filter(
                anime.mikan_id,
                resolution,
                None,
                subtitle_type,
                limit.unwrap_or(0),
                offset.unwrap_or(0),
            )
            .await
            .map_err(|e| e.to_string())?;

        let mut subtitle_groups_map: std::collections::HashMap<i64, SubtitleGroupResource> =
            std::collections::HashMap::new();
        let mut total_resources = 0;

        for res in resources {
            total_resources += 1;
            let group_id = res.subtitle_group_id;
            let group_name = subtitle_group_repo
                .get_by_id(group_id)
                .await
                .map_err(|e| e.to_string())?
                .map_or("Unknown".to_string(), |g| g.name);

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

        Ok(Some(EpisodeResourcesData {
            total_resources,
            subtitle_groups: subtitle_groups_map.into_values().collect(),
        }))
    } else {
        Ok(None)
    }
}
