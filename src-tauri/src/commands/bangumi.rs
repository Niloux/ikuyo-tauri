use crate::error::AppError;
use crate::{
    repositories::{anime::AnimeRepository, resource::ResourceRepository},
    services::bangumi_service::BangumiService,
    types::bangumi::{
        BangumiEpisodesData, BangumiSubject, BangumiWeekday, EpisodeAvailabilityData,
        EpisodeResourcesData, Pagination, SearchLibraryResponse,
    },
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{command, State};

#[command(rename_all = "snake_case")]
pub async fn get_calendar(
    pool: State<'_, Arc<SqlitePool>>,
    config: State<'_, crate::config::Config>,
) -> Result<Vec<BangumiWeekday>, AppError> {
    let service = BangumiService::new(pool.inner().clone(), config.inner().clone());
    service.get_calendar().await
}

#[command(rename_all = "snake_case")]
pub async fn get_subject(
    id: i64,
    pool: State<'_, Arc<SqlitePool>>,
    config: State<'_, crate::config::Config>,
) -> Result<BangumiSubject, AppError> {
    let service = BangumiService::new(pool.inner().clone(), config.inner().clone());
    service.get_subject(id).await
}

#[command(rename_all = "snake_case")]
pub async fn get_episodes(
    subject_id: i64,
    episode_type: Option<i64>,
    limit: Option<i64>,
    offset: Option<i64>,
    pool: State<'_, Arc<SqlitePool>>,
    config: State<'_, crate::config::Config>,
) -> Result<BangumiEpisodesData, AppError> {
    let service = BangumiService::new(pool.inner().clone(), config.inner().clone());
    service
        .get_episodes(subject_id, episode_type, limit, offset)
        .await
}

#[command(rename_all = "snake_case")]
pub async fn get_episode_availability(
    bangumi_id: i64,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<Option<EpisodeAvailabilityData>, AppError> {
    let anime_repo = AnimeRepository::new(&pool);
    let resource_repo = ResourceRepository::new(&pool);

    let anime = anime_repo.get_by_bangumi_id(bangumi_id).await?;

    if let Some(anime) = anime {
        let episode_counts = resource_repo.count_by_episode(anime.mikan_id).await?;

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
    config: State<'_, crate::config::Config>,
) -> Result<Option<EpisodeResourcesData>, AppError> {
    let service = BangumiService::new(pool.inner().clone(), config.inner().clone());
    service
        .aggregate_resources(bangumi_id, Some(episode), None, None, None, None)
        .await
}

#[command(rename_all = "snake_case")]
pub async fn search_library(
    query: String,
    page: i64,
    limit: i64,
    pool: State<'_, Arc<SqlitePool>>,
) -> Result<SearchLibraryResponse, AppError> {
    let anime_repo = AnimeRepository::new(&pool);

    let offset = (page - 1) * limit;
    let animes = anime_repo.search_by_title(&query, limit, offset).await?;

    let total_animes = anime_repo.count_by_title(&query).await?;

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
    config: State<'_, crate::config::Config>,
) -> Result<Option<EpisodeResourcesData>, AppError> {
    let service = BangumiService::new(pool.inner().clone(), config.inner().clone());
    service
        .aggregate_resources(bangumi_id, None, resolution, subtitle_type, limit, offset)
        .await
}
