use serde::{Deserialize, Serialize};
use toml;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub db_url: String,
    // Bangumi缓存相关参数（单位：秒）
    pub bangumi_sub_ttl: Option<i64>,
    pub bangumi_nonsub_ttl: Option<i64>,
    pub bangumi_calendar_ttl: Option<i64>,
    pub bangumi_sub_refresh_interval: Option<i64>,
    pub bangumi_nonsub_refresh_interval: Option<i64>,
    pub bangumi_calendar_refresh_interval: Option<i64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_url: "sqlite:ikuyo.db?mode=rwc".to_string(),
            bangumi_sub_ttl: Some(3600),                    // 1小时
            bangumi_nonsub_ttl: Some(43200),                // 12小时
            bangumi_calendar_ttl: Some(86400),              // 24小时
            bangumi_sub_refresh_interval: Some(3600),       // 1小时
            bangumi_nonsub_refresh_interval: Some(43200),   // 12小时
            bangumi_calendar_refresh_interval: Some(86400), // 24小时
        }
    }
}

impl Config {
    pub fn load() -> Self {
        // 支持从config.toml加载
        let path = std::path::Path::new("config.toml");
        if path.exists() {
            let content = std::fs::read_to_string(path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_else(|_| Self::default())
        } else {
            Self::default()
        }
    }
}
