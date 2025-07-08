// 数据库模型,时间信息统一用unix时间戳
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::types::subscription as types_subscription;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize,)]
pub enum AnimeStatus {
    Unknown,
    Airing,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize,)]
pub enum CrawlerTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize,)]
pub enum CrawlerTaskType {
    Manual,
    Scheduled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Anime {
    pub mikan_id: i64,
    pub bangumi_id: i64,
    pub title: String,
    pub original_title: Option<String>,
    pub broadcast_day: Option<String>,
    pub broadcast_start: Option<i64>,
    pub official_website: Option<String>,
    pub bangumi_url: Option<String>,
    pub description: Option<String>,
    pub status: Option<AnimeStatus>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct AnimeSubtitleGroup {
    pub id: Option<i64>,
    pub mikan_id: i64,
    pub subtitle_group_id: i64,
    pub first_release_date: Option<i64>,
    pub last_update_date: Option<i64>,
    pub resource_count: Option<i64>,
    pub is_active: Option<bool>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct CrawlerTask {
    pub id: Option<i64>,
    pub task_type: CrawlerTaskType,
    pub status: CrawlerTaskStatus,
    pub parameters: Option<String>,
    pub result_summary: Option<String>,
    pub created_at: Option<i64>,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub error_message: Option<String>,
    pub percentage: Option<f64>,
    pub processed_items: Option<i64>,
    pub total_items: Option<i64>,
    pub processing_speed: Option<f64>,
    pub estimated_remaining: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct ScheduledJob {
    pub id: Option<i64>,
    pub job_id: String,
    pub name: String,
    pub description: Option<String>,
    pub cron_expression: String,
    pub crawler_mode: Option<String>,
    pub parameters: Option<String>,
    pub enabled: bool,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct SubtitleGroup {
    pub id: Option<i64>,
    pub name: String,
    pub last_update: Option<i64>,
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct UserSubscription {
    pub id: Option<i64>,
    pub user_id: String,
    pub bangumi_id: i64,
    pub subscribed_at: i64,
    pub notes: Option<String>,
    pub anime_name: Option<String>,
    pub anime_name_cn: Option<String>,
    pub anime_rating: Option<f64>,
    pub anime_air_date: Option<String>,
    pub anime_air_weekday: Option<i64>,
}

// 实现前端struct到后端struct的转换
impl From<types_subscription::UserSubscription> for UserSubscription {
    fn from(item: types_subscription::UserSubscription) -> Self {
        UserSubscription {
            id: item.id.map(|id| id as i64),
            user_id: item.user_id,
            bangumi_id: item.bangumi_id as i64,
            subscribed_at: item.subscribed_at as i64,
            notes: item.notes,
            anime_name: item.anime_name,
            anime_name_cn: item.anime_name_cn,
            anime_rating: item.anime_rating.map(|r| r as f64),
            anime_air_date: item.anime_air_date,
            anime_air_weekday: item.anime_air_weekday.map(|w| w as i64),
        }
    }
}

// 实现后端struct到前端struct的转变
impl From<UserSubscription> for types_subscription::UserSubscription {
    fn from(item: UserSubscription) -> Self {
        types_subscription::UserSubscription {
            id: item.id.map(|id| id as u32),
            user_id: item.user_id,
            bangumi_id: item.bangumi_id as u32,
            subscribed_at: item.subscribed_at as u64,
            notes: item.notes,
            anime_name: item.anime_name,
            anime_name_cn: item.anime_name_cn,
            anime_rating: item.anime_rating.map(|r| r as f32),
            anime_air_date: item.anime_air_date,
            anime_air_weekday: item.anime_air_weekday.map(|w| w as u32),
        }
    }
}
