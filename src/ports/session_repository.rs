use crate::domain::pagination::{PageRequest, PageResponse};
use crate::domain::session::SessionListFilter;
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
    async fn revoke_all_for_user(&self, realm_id: &Uuid, user_id: &Uuid) -> Result<()>;
    /// Revoke an explicit set of active sessions in a realm in one transaction.
    /// Returns the number of sessions revoked.
    async fn revoke_many(&self, realm_id: &Uuid, ids: &[Uuid]) -> Result<u64>;
    /// Revoke all active sessions for a user in a realm except `except_id`.
    /// Returns the number of sessions revoked.
    async fn revoke_others_for_user(
        &self,
        realm_id: &Uuid,
        user_id: &Uuid,
        except_id: &Uuid,
    ) -> Result<u64>;
    /// Revoke all active sessions for a user in a realm. Returns the count.
    async fn revoke_user_sessions(&self, realm_id: &Uuid, user_id: &Uuid) -> Result<u64>;
    /// Mark a single active session in a realm for forced re-authentication.
    /// Returns true if a matching active session was updated.
    async fn request_step_up(&self, realm_id: &Uuid, id: &Uuid) -> Result<bool>;
    async fn revoke_by_user_and_client(
        &self,
        realm_id: &Uuid,
        user_id: &Uuid,
        client_id: &str,
    ) -> Result<()>;
    /// Revoke root SSO tokens (where client_id IS NULL) for a user in a realm.
    async fn revoke_root_tokens_for_user(&self, realm_id: &Uuid, user_id: &Uuid) -> Result<()>;
    async fn list(
        &self,
        realm_id: &Uuid,
        req: &PageRequest,
        filter: &SessionListFilter,
    ) -> Result<PageResponse<RefreshToken>>;
}
