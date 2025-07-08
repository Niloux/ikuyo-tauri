use thiserror::Error;

pub type Result<T, E = anyhow::Error> = anyhow::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("数据库初始化失败")]
    DatabaseInitialization,

    #[error("数据库迁移失败")]
    DatabaseMigration,

    #[error("读取 SQL 文件失败")]
    SqlFileRead,
}
