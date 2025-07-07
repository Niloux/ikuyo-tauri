use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Scheduler Types
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduledJobCreate {
    pub job_id: String,
    pub name: String,
    pub cron_expression: String,
    pub parameters: HashMap<String, serde_json::Value>, // Use serde_json::Value for 'any' type
    pub enabled: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduledJobUpdate {
    pub name: Option<String>,
    pub cron_expression: Option<String>,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub enabled: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduledJobResponse {
    pub id: Option<u32>,
    pub job_id: String,
    pub name: String,
    pub cron_expression: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}