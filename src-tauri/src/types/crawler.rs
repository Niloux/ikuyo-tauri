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
    pub year: Option<u32>,
    pub season: Option<SeasonName>,
    pub limit: Option<u32>,
}

// 爬虫任务相关类型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskResponse {
    pub id: u32,
    pub task_type: String,
    pub status: String,
    pub parameters: Option<String>,
    pub result_summary: Option<String>,
    pub created_at: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
    pub percentage: Option<f32>,
    pub processed_items: Option<u32>,
    pub total_items: Option<u32>,
    pub processing_speed: Option<f32>,
    pub estimated_remaining: Option<f32>,
}