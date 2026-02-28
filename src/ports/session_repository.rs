use crate::domain::pagination::{PageRequest, PageResponse};
use crate::{domain::session::RefreshToken, error::Result};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn save(&self, token: &RefreshToken) -> Result<()>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<RefreshToken>>;
    async fn find_by_id_any(&self, id: &Uuid) -> Result<Option<RefreshToken>>;
    async fn delete_by_id(&self, id: &Uuid) -> Result<()>;
    async fn mark_replaced(&self, old_id: &Uuid, new_id: &Uuid) -> Result<()>;
    async fn revoke_family(&self, family_id: &Uuid) -> Result<()>;
    async fn list(&self, realm_id: &Uuid, req: &PageRequest) -> Result<PageResponse<RefreshToken>>;
}
