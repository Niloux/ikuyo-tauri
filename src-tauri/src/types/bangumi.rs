use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// 对应前端的 BangumiCalendarItem
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiCalendarItem {
    pub id: u32,
    pub url: String,
    #[serde(rename = "type")]
    pub item_type: u32, // 避免与 Rust 关键字冲突
    pub name: String,
    pub name_cn: String,
    pub summary: String,
    pub air_date: String,
    pub air_weekday: u32,
    pub rating: Option<BangumiRating>,
    pub rank: Option<u32>,
    pub images: Option<BangumiImages>,
}

// 对应前端的 BangumiRating
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiRating {
    pub total: u32,
    pub count: HashMap<String, u32>,
    pub score: f32,
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
    pub id: u32,
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
pub struct BangumiTag {
    pub name: String,
    pub count: u32,
    #[serde(rename = "total_cont")]
    pub total_cont: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiSubject {
    pub id: i64,
    pub name: String,
    pub name_cn: String,
    pub summary: String,
    #[serde(rename = "date")]
    pub air_date: Option<String>,
    pub air_weekday: Option<u32>,
    pub eps: Option<u32>,
    pub total_episodes: Option<u32>,
    pub rating: Option<BangumiRating>,
    pub rank: Option<u32>,
    pub images: Option<BangumiImages>,
    pub collection: Option<BangumiCollection>,
    pub tags: Option<Vec<BangumiTag>>,
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