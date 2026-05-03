use crate::domain::invitation::{Invitation, InvitationStatus};
use crate::domain::pagination::{PageRequest, PageResponse};
use crate::error::Result;
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait InvitationRepository: Send + Sync {
    async fn create(&self, invitation: &Invitation, tx: Option<&mut dyn Transaction>)
        -> Result<()>;
    async fn update(&self, invitation: &Invitation, tx: Option<&mut dyn Transaction>)
        -> Result<()>;
    async fn find_by_id(&self, realm_id: &Uuid, id: &Uuid) -> Result<Option<Invitation>>;
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<Invitation>>;
    async fn find_pending_by_email(
        &self,
        realm_id: &Uuid,
        email_normalized: &str,
    ) -> Result<Option<Invitation>>;
    async fn expire_pending_before(&self, realm_id: &Uuid, cutoff: DateTime<Utc>) -> Result<u64>;
    async fn list(
        &self,
        realm_id: &Uuid,
        req: &PageRequest,
        status: Option<InvitationStatus>,
    ) -> Result<PageResponse<Invitation>>;
}
