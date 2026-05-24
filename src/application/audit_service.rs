use crate::domain::audit::{AuditActionCount, AuditEvent, NewAuditEvent};
use crate::error::Result;
use crate::ports::audit_repository::AuditRepository;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct AuditService {
    repo: Arc<dyn AuditRepository>,
}

impl AuditService {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }

    pub async fn record(&self, event: NewAuditEvent) -> Result<()> {
        let audit_event = AuditEvent {
            id: Uuid::new_v4(),
            realm_id: event.realm_id,
            actor_user_id: event.actor_user_id,
            action: event.action,
            target_type: event.target_type,
            target_id: event.target_id,
            metadata: event.metadata,
            created_at: Utc::now().to_rfc3339(),
        };

        self.repo.insert(&audit_event).await
    }

    pub async fn list_recent(&self, realm_id: Uuid, limit: usize) -> Result<Vec<AuditEvent>> {
        self.repo.list_recent(&realm_id, limit).await
    }

    pub async fn count_by_actions_since(
        &self,
        realm_id: Uuid,
        actions: &[&str],
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<AuditActionCount>> {
        self.repo
            .count_by_actions_since(&realm_id, actions, since)
            .await
    }

    pub async fn count_by_target_and_actions_since(
        &self,
        realm_id: Uuid,
        target_type: &str,
        target_id: &str,
        actions: &[&str],
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<AuditActionCount>> {
        self.repo
            .count_by_target_and_actions_since(&realm_id, target_type, target_id, actions, since)
            .await
    }

    pub async fn list_recent_by_target_and_actions(
        &self,
        realm_id: Uuid,
        target_type: &str,
        target_id: &str,
        actions: &[&str],
        limit: usize,
    ) -> Result<Vec<AuditEvent>> {
        self.repo
            .list_recent_by_target_and_actions(&realm_id, target_type, target_id, actions, limit)
            .await
    }
}
