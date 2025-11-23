use crate::{domain::session::RefreshToken, error::Result};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn save(&self, token: &RefreshToken) -> Result<()>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<RefreshToken>>;
    async fn delete_by_id(&self, id: &Uuid) -> Result<()>;
}
