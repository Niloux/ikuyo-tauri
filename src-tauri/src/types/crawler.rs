use serde::{Deserialize, Serialize};

// =============================================================================
// Crawler Types
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CrawlerMode {
    #[serde(rename = "homepage")]
    Homepage,
    #[serde(rename = "season")]
    Season,
    #[serde(rename = "year")]
    Year,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrawlerTaskCreate {
    pub mode: CrawlerMode,
    pub year: Option<i64>,
    pub season: Option<SeasonName>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CrawlerTaskType {
    Manual,
    Schedule,
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