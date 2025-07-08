use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// 对应前端的 BangumiCalendarItem
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiCalendarItem {
    pub id: i64,
    pub url: String,
    #[serde(rename = "type")]
    pub item_type: i64, // 避免与 Rust 关键字冲突
    pub name: String,
    pub name_cn: String,
    pub summary: String,
    pub air_date: String,
    pub air_weekday: i64,
    pub rating: Option<BangumiRating>,
    pub rank: Option<i64>,
    pub images: Option<BangumiImages>,
}

// 对应前端的 BangumiRating
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiRating {
    pub total: i64,
    pub count: HashMap<String, i64>,
    pub score: f64,
}

// 对应前端的 BangumiImages
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiImages {
    pub large: Option<String>,
    pub common: Option<String>,
    pub medium: Option<String>,
    pub small: Option<String>,
    pub grid: Option<String>,
}

// 对应前端的 BangumiWeekday
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiWeekday {
    pub weekday: WeekdayInfo,
    pub items: Vec<BangumiCalendarItem>,
}

// 对应前端的 weekday 内部结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeekdayInfo {
    pub en: String,
    pub cn: String,
    pub ja: String,
    pub id: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiCollection {
    pub wish: i64,
    pub collect: i64,
    pub doing: i64,
    pub on_hold: i64,
    pub dropped: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiTag {
    pub name: String,
    pub count: i64,
    #[serde(rename = "total_cont")]
    pub total_cont: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiSubject {
    pub id: i64,
    pub name: String,
    pub name_cn: String,
    pub summary: String,
    #[serde(rename = "date")]
    pub air_date: Option<String>,
    pub air_weekday: Option<i64>,
    pub eps: Option<i64>,
    pub total_episodes: Option<i64>,
    pub rating: Option<BangumiRating>,
    pub rank: Option<i64>,
    pub images: Option<BangumiImages>,
    pub collection: Option<BangumiCollection>,
    pub tags: Option<Vec<BangumiTag>>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodeAvailability {
    pub available: bool,
    pub resource_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodeAvailabilityData {
    pub bangumi_id: i64,
    pub episodes: HashMap<String, EpisodeAvailability>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodeResource {
    pub id: i64,
    pub episode_number: i64,
    pub title: String,
    pub resolution: String,
    pub subtitle_type: String,
    pub magnet_url: String,
    pub torrent_url: String,
    pub release_date: String,
    pub size: String,
    pub group_id: i64,
    pub group_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubtitleGroupResource {
    pub id: i64,
    pub name: String,
    pub resource_count: i64,
    pub resources: Vec<EpisodeResource>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodeResourcesData {
    pub total_resources: i64,
    pub subtitle_groups: Vec<SubtitleGroupResource>,
}

// Bangumi章节相关类型定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiEpisode {
    pub id: i64,
    #[serde(rename = "type")]
    pub episode_type: i64, // 0:正片, 1:SP, 2:OP, 3:ED, 4:PV, 6:其他
    pub name: String,
    pub name_cn: String,
    pub sort: i64,
    pub ep: Option<i64>,
    pub airdate: Option<String>,
    pub comment: i64,
    pub duration: String,
    pub desc: String,
    pub disc: i64,
    pub duration_seconds: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiEpisodesData {
    pub data: Vec<BangumiEpisode>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pagination {
    pub current_page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchLibraryResponse {
    pub bangumi_ids: Vec<i64>,
    pub pagination: Pagination,
}
