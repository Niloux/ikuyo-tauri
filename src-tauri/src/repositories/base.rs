use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Repository<T, Id>
where
    T: Send + Sync,
    Id: Send + Sync,
{
    async fn create(&self, entity: &T) -> Result<()>;
    async fn get_by_id(&self, id: Id) -> Result<Option<T>>;
    async fn update(&self, entity: &T) -> Result<()>;
    async fn delete(&self, id: Id) -> Result<()>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<T>>;
}
