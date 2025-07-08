use crate::types::bangumi::BangumiCalendarItem;
use serde::{Deserialize, Serialize}; // Import from bangumi types

/**
 * 订阅功能相关类型定义
 */

// 基础订阅记录
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSubscription {
    pub id: Option<i64>, // u32 -> i64
    pub user_id: String,
    pub bangumi_id: i64,    // u32 -> i64
    pub subscribed_at: i64, // u64 -> i64
    pub notes: Option<String>,
    // 缓存的番剧数据
    pub anime_name: Option<String>,
    pub anime_name_cn: Option<String>,
    pub anime_rating: Option<f64>, // f32 -> f64
    pub anime_air_date: Option<String>,
    pub anime_air_weekday: Option<i64>, // u32 -> i64
    // 新增字段
    pub url: Option<String>,
    pub item_type: Option<i64>,
    pub summary: Option<String>,
    pub rank: Option<i64>,
    pub images: Option<String>,
}

// 包含完整番剧信息的订阅记录
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscriptionWithAnime {
    #[serde(flatten)] // Flatten the fields of UserSubscription into this struct
    pub user_subscription: UserSubscription,
    pub anime: BangumiCalendarItem,
}

// 订阅状态检查响应
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscriptionStatus {
    pub subscribed: bool,
    pub subscribed_at: Option<u64>,
    pub notes: Option<String>,
}

// 获取订阅列表的请求参数
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetSubscriptionsParams {
    pub sort: Option<String>,  // Can be an enum if values are fixed
    pub order: Option<String>, // Can be an enum if values are fixed
    pub search: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

// 分页信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginationInfo {
    pub page: u32,
    pub limit: u32,
    pub total: u32,
    pub pages: u32,
}

// 获取订阅列表的响应
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscriptionsResponse {
    pub subscriptions: Vec<UserSubscription>,
    pub pagination: PaginationInfo,
}

// 订阅列表的完整响应（包含番剧详情）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscriptionsWithAnimeResponse {
    pub subscriptions: Vec<SubscriptionWithAnime>,
    pub pagination: PaginationInfo,
}

// API错误响应
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscriptionError {
    pub message: String,
    pub code: Option<String>,
}

// 订阅操作的结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscriptionResult {
    pub success: bool,
    pub error: Option<SubscriptionError>,
    pub data: Option<serde_json::Value>, // Use serde_json::Value for 'any' type
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscriptionIdsResponse {
    pub ids: Vec<i64>,
}
