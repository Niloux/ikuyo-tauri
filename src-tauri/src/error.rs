use thiserror::Error;
use serde::Serialize;

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

#[derive(Debug, Error, Serialize)]
pub enum DatabaseError {
    #[error("连接失败: {0}")]
    Connection(String),
    #[error("迁移失败: {0}")]
    Migration(String),
    #[error("查询失败: {0}")]
    Query(String),
    #[error("SQL文件读取失败: {0}")]
    SqlFileRead(String),
    #[error("其他数据库错误: {0}")]
    Other(String),
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
    NotFound { resource_type: String, resource_id: i64 },
    #[error("业务规则冲突: {0}")]
    Conflict(String),
    #[error("其他领域错误: {0}")]
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

// From trait 实现
impl From<sqlx::Error> for DatabaseError {
    fn from(e: sqlx::Error) -> Self {
        DatabaseError::Other(e.to_string())
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        ApiError::Request(e.to_string())
    }
}

impl From<serde_json::Error> for DomainError {
    fn from(e: serde_json::Error) -> Self {
        DomainError::Other(e.to_string())
    }
}

// 兼容原有 Result 类型
pub type Result<T> = std::result::Result<T, AppError>;
