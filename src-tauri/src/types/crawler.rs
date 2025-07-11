use serde::{Deserialize, Serialize};

// =============================================================================
// Crawler Types
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CrawlerMode {
    #[serde(rename = "homepage")]
    Homepage,
    #[serde(rename = "season")]
    Season,
    #[serde(rename = "year")]
    Year,
}

impl CrawlerMode {
    pub fn as_str(&self) -> &str {
        match self {
            CrawlerMode::Homepage => "homepage",
            CrawlerMode::Season => "season",
            CrawlerMode::Year => "year",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SeasonName {
    #[serde(rename = "春")]
    Spring,
    #[serde(rename = "夏")]
    Summer,
    #[serde(rename = "秋")]
    Autumn,
    #[serde(rename = "冬")]
    Winter,
}

impl SeasonName {
    pub fn as_str(&self) -> &str {
        match self {
            SeasonName::Spring => "春",
            SeasonName::Summer => "夏",
            SeasonName::Autumn => "秋",
            SeasonName::Winter => "冬",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrawlerTaskCreate {
    pub mode: CrawlerMode,
    pub year: Option<i64>,
    pub season: Option<SeasonName>,
    pub limit: Option<i64>,
}

impl Default for CrawlerTaskCreate {
    fn default() -> Self {
        Self {
            mode: CrawlerMode::Homepage,
            year: None,
            season: None,
            limit: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CrawlerTaskType {
    Manual,
    Schedule,
}

impl From<crate::models::CrawlerTaskType> for CrawlerTaskType {
    fn from(value: crate::models::CrawlerTaskType) -> Self {
        match value {
            crate::models::CrawlerTaskType::Manual => Self::Manual,
            crate::models::CrawlerTaskType::Scheduled => Self::Schedule,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CrawlerTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl From<crate::models::CrawlerTaskStatus> for CrawlerTaskStatus {
    fn from(value: crate::models::CrawlerTaskStatus) -> Self {
        match value {
            crate::models::CrawlerTaskStatus::Pending => Self::Pending,
            crate::models::CrawlerTaskStatus::Running => Self::Running,
            crate::models::CrawlerTaskStatus::Completed => Self::Completed,
            crate::models::CrawlerTaskStatus::Failed => Self::Failed,
            crate::models::CrawlerTaskStatus::Cancelled => Self::Cancelled,
        }
    }
}

// 爬虫任务相关类型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskResponse {
    pub id: i64,
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
