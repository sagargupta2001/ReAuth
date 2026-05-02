use crate::domain::audit::{AuditActionCount, AuditEvent};
use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn insert(&self, event: &AuditEvent) -> Result<()>;
    async fn list_recent(&self, realm_id: &Uuid, limit: usize) -> Result<Vec<AuditEvent>>;
    async fn count_by_actions_since(
        &self,
        realm_id: &Uuid,
        actions: &[&str],
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<AuditActionCount>>;
    async fn list_recent_by_actions(
        &self,
        realm_id: &Uuid,
        actions: &[&str],
        limit: usize,
    ) -> Result<Vec<AuditEvent>>;
}
