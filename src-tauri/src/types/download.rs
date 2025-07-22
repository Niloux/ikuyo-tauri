use serde::{Deserialize, Serialize};
use crate::models::DownloadStatus;

// 下载事件结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgressUpdate {
    pub id: i64,
    pub total_bytes: u64,
    pub progress: f64,
    pub speed: f64,
    pub time_remaining: Option<String>,
    pub status: DownloadStatus,
    pub error_msg: Option<String>,
    // 可扩展更多字段，如剩余时间、状态等
}

// 发起下载任务参数
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StartDownloadTask {
    pub magnet_url: String,
    pub save_path: Option<String>,
    pub title: String,
    pub bangumi_id: i64,
    pub resource_id: i64,
    pub episode_number: i64,
    pub name: String,
    pub name_cn: String,
    pub cover: String,
    pub total_size: i64,
}