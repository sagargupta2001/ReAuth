use crate::domain::audit::AuditEvent;
use crate::error::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn insert(&self, event: &AuditEvent) -> Result<()>;
    async fn list_recent(&self, realm_id: &Uuid, limit: usize) -> Result<Vec<AuditEvent>>;
}
