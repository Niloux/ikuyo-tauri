use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub db_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_url: "sqlite:ikuyo.db?mode=rwc".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        // 目前，我们只使用默认值
        // 未来可以扩展为从文件或环境变量加载
        Self::default()
    }
} 