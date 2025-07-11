use serde::Serialize;
use thiserror::Error;

// Main application error enum
#[derive(Debug, Error, Serialize)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] DatabaseError),
    #[error("API错误: {0}")]
    Api(#[from] ApiError),
    #[error("领域错误: {0}")]
    Domain(#[from] DomainError),
    #[error("缓存错误: {0}")]
    Cache(#[from] CacheError),
    #[error("任务错误: {0}")]
    Task(#[from] TaskError),
    #[error("输入参数无效: {0}")]
    Input(#[from] InputError),
    #[error("未知错误: {0}")]
    Unknown(String),
}

// Specific error types
#[derive(Debug, Error, Serialize)]
pub enum DatabaseError {
    #[error("查询失败: {0}")]
    Query(String),
    #[error("连接失败: {0}")]
    Connection(String),
    #[error("迁移失败: {0}")]
    Migration(String),
    #[error("SQL文件读取失败: {0}")]
    SqlFileRead(String),
}

#[derive(Debug, Error, Serialize)]
pub enum ApiError {
    #[error("请求失败: {0}")]
    Request(String),
    #[error("响应解析失败: {0}")]
    Response(String),
    #[error("第三方API错误: {0}")]
    External(String),
}

#[derive(Debug, Error, Serialize)]
pub enum DomainError {
    #[error("找不到资源: {resource_type} ID {resource_id}")]
    NotFound {
        resource_type: String,
        resource_id: i64,
    },
    #[error("业务规则冲突: {0}")]
    Conflict(String),
    #[error("序列化/反序列化错误: {0}")]
    Serialization(String),
    #[error("其他错误: {0}")]
    Other(String),
}

#[derive(Debug, Error, Serialize)]
pub enum CacheError {
    #[error("缓存操作失败: {0}")]
    Operation(String),
}

#[derive(Debug, Error, Serialize)]
pub enum TaskError {
    #[error("任务执行失败: {0}")]
    Failed(String),
    #[error("任务取消失败: {0}")]
    Cancel(String),
}

#[derive(Debug, Error, Serialize)]
pub enum InputError {
    #[error("无效输入: {0}")]
    Invalid(String),
}

// Direct `From` implementations for `AppError`
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Database(DatabaseError::Query(e.to_string()))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Api(ApiError::Request(e.to_string()))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Domain(DomainError::Serialization(e.to_string()))
    }
}

// Application-wide Result type
pub type Result<T> = std::result::Result<T, AppError>;
