use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Bangumi Types
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiCalendarItem {
    pub id: u32,
    pub url: String,
    #[serde(rename = "type")]
    pub item_type: u32,
    pub name: String,
    pub name_cn: String,
    pub summary: String,
    pub air_date: String,
    pub air_weekday: u32,
    pub rating: BangumiRating,
    pub rank: u32,
    pub images: BangumiImages,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiSubject {
    pub id: u32,
    pub name: String,
    pub name_cn: String,
    pub summary: String,
    pub date: String,
    pub air_weekday: u32,
    pub eps: u32,
    pub total_episodes: u32,
    pub rating: BangumiRating,
    pub rank: u32,
    pub images: BangumiImages,
    pub collection: BangumiCollection,
    pub tags: Vec<BangumiTag>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiRating {
    pub total: u32,
    pub count: HashMap<String, u32>,
    pub score: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiImages {
    pub large: String,
    pub common: String,
    pub medium: String,
    pub small: String,
    pub grid: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiCollection {
    pub wish: u32,
    pub collect: u32,
    pub doing: u32,
    pub on_hold: u32,
    pub dropped: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiWeekday {
    pub weekday: WeekdayInfo,
    pub items: Vec<BangumiCalendarItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeekdayInfo {
    pub en: String,
    pub cn: String,
    pub ja: String,
    pub id: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiTag {
    pub name: String,
    pub count: u32,
    pub total_cont: u32,
}

// 集数可用性相关类型定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodeAvailabilityData {
    pub bangumi_id: u32,
    pub episodes: HashMap<String, EpisodeAvailabilityStatus>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodeAvailabilityStatus {
    pub available: bool,
    pub resource_count: u32,
}

// Bangumi章节相关类型定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiEpisode {
    pub id: u32,
    #[serde(rename = "type")]
    pub episode_type: u32, // 0:正片, 1:SP, 2:OP, 3:ED, 4:PV, 6:其他
    pub name: String,
    pub name_cn: String,
    pub sort: u32,
    pub ep: Option<u32>,
    pub airdate: Option<String>,
    pub comment: u32,
    pub duration: String,
    pub desc: String,
    pub disc: u32,
    pub duration_seconds: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiEpisodesData {
    pub data: Vec<BangumiEpisode>,
    pub total: u32,
}

// 后端章节API响应格式
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiEpisodesResponse {
    pub data: Vec<BangumiEpisode>,
    pub total: u32,
    // ApiResponse 字段，如果需要可以添加
    // pub code: u32,
    // pub message: String,
}

// 资源相关类型定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubtitleGroupResource {
    pub id: u32,
    pub name: String,
    pub resource_count: u32,
    pub resources: Vec<EpisodeResource>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodeResource {
    pub id: u32,
    pub episode_number: u32,
    pub title: String,
    pub resolution: String,
    pub subtitle_type: String,
    pub magnet_url: String,
    pub torrent_url: String,
    pub release_date: String,
    pub size: String,
    pub group_id: u32,
    pub group_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodeResourcesData {
    pub total_resources: u32,
    pub subtitle_groups: Vec<SubtitleGroupResource>,
}