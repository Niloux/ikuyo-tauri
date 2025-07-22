// 数据库模型,时间信息统一用unix时间戳
use crate::types::subscription as types_subscription;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum AnimeStatus {
    Unknown,
    Airing,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum CrawlerTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
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
    pub broadcast_start: Option<String>,
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
    // 新增字段
    pub url: Option<String>,
    pub item_type: Option<i64>, // 对应 BangumiCalendarItem 的 type
    pub summary: Option<String>,
    pub rank: Option<i64>,
    pub images: Option<String>, // 存储 BangumiImages 的 JSON 字符串
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Resource {
    pub id: Option<i64>,
    pub mikan_id: i64,
    pub subtitle_group_id: i64,
    pub episode_number: Option<i32>,
    pub title: String,
    pub file_size: Option<String>,
    pub resolution: Option<String>,
    pub subtitle_type: Option<String>,
    pub magnet_url: Option<String>,
    pub torrent_url: Option<String>,
    pub play_url: Option<String>,
    pub magnet_hash: Option<String>,
    pub release_date: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct EpisodeResourceCount {
    pub episode_number: i32,
    pub resource_count: i64,
}

// 实现前端struct到后端struct的转换
impl From<types_subscription::UserSubscription> for UserSubscription {
    fn from(item: types_subscription::UserSubscription) -> Self {
        UserSubscription {
            id: item.id,
            user_id: item.user_id,
            bangumi_id: item.bangumi_id,
            subscribed_at: item.subscribed_at,
            notes: item.notes,
            anime_name: item.anime_name,
            anime_name_cn: item.anime_name_cn,
            anime_rating: item.anime_rating,
            anime_air_date: item.anime_air_date,
            anime_air_weekday: item.anime_air_weekday,
            url: item.url,
            item_type: item.item_type,
            summary: item.summary,
            rank: item.rank,
            images: item.images,
        }
    }
}

// 实现后端struct到前端struct的转变
impl From<UserSubscription> for types_subscription::UserSubscription {
    fn from(item: UserSubscription) -> Self {
        types_subscription::UserSubscription {
            id: item.id,
            user_id: item.user_id,
            bangumi_id: item.bangumi_id,
            subscribed_at: item.subscribed_at,
            notes: item.notes,
            anime_name: item.anime_name,
            anime_name_cn: item.anime_name_cn,
            anime_rating: item.anime_rating,
            anime_air_date: item.anime_air_date,
            anime_air_weekday: item.anime_air_weekday,
            url: item.url,
            item_type: item.item_type,
            summary: item.summary,
            rank: item.rank,
            images: item.images,
        }
    }
}

// download表模型
// 下载任务状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Pending,     // 待下载
    Downloading, // 下载中
    Paused,      // 已暂停
    Completed,   // 已完成
    Failed,      // 失败
    Deleted,     // 已删除
}

// 下载任务表
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct DownloadTask {
    // 核心字段
    pub id: Option<i64>,
    pub magnet_url: String,
    pub save_path: Option<String>,
    pub status: DownloadStatus,
    pub title: String,
    // 元数据
    pub bangumi_id: i64,
    pub resource_id: i64,
    pub episode_number: i64,
    pub name: String,
    pub name_cn: String,
    pub cover: String,
    pub total_size: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub error_msg: Option<String>,
}
