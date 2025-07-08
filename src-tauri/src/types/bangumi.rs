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
