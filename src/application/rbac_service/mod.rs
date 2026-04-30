use crate::{
    domain::{events::DomainEvent, permissions, role::Role},
    error::{Error, Result},
    ports::{
        cache_service::CacheService,
        event_bus::EventPublisher,
        outbox_repository::OutboxRepository,
        rbac_repository::RbacRepository,
        transaction_manager::{Transaction, TransactionManager},
    },
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[derive(serde::Deserialize, Clone, Default)]
pub struct CreateRolePayload {
    pub name: String,
    pub description: Option<String>,
    pub client_id: Option<Uuid>,
}

#[derive(serde::Deserialize, Clone, Default)]
pub struct CreateGroupPayload {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
}

#[derive(serde::Deserialize, Clone, Default)]
pub struct CreateCustomPermissionPayload {
    pub permission: String,
    pub name: String,
    pub description: Option<String>,
    pub client_id: Option<Uuid>,
}

#[derive(serde::Deserialize, Clone, Default)]
pub struct UpdateCustomPermissionPayload {
    pub name: String,
    pub description: Option<String>,
}

/// The application service for handling all RBAC logic.
pub struct RbacService {
    rbac_repo: Arc<dyn RbacRepository>,
    cache: Arc<dyn CacheService>,
    event_bus: Arc<dyn EventPublisher>,
    outbox_repo: Arc<dyn OutboxRepository>,
    tx_manager: Arc<dyn TransactionManager>,
}

pub mod assignments;
pub mod groups;
pub mod roles;

impl RbacService {
    pub fn new(
        rbac_repo: Arc<dyn RbacRepository>,
        cache: Arc<dyn CacheService>,
        event_bus: Arc<dyn EventPublisher>,
        outbox_repo: Arc<dyn OutboxRepository>,
        tx_manager: Arc<dyn TransactionManager>,
    ) -> Self {
        Self {
            rbac_repo,
            cache,
            event_bus,
            outbox_repo,
            tx_manager,
        }
    }

    pub(crate) async fn write_outbox(
        &self,
        event: &DomainEvent,
        realm_id: Option<Uuid>,
        tx: &mut dyn Transaction,
    ) -> Result<()> {
        let envelope = event.to_envelope(Uuid::new_v4(), Utc::now(), realm_id, None);
        self.outbox_repo.insert(&envelope, Some(tx)).await
    }

    fn validate_custom_permission_key(&self, permission: &str) -> Result<()> {
        let trimmed = permission.trim();
        if trimmed.is_empty() {
            return Err(Error::Validation("Permission ID cannot be empty".into()));
        }

        if trimmed.contains(char::is_whitespace) {
            return Err(Error::Validation(
                "Permission ID cannot contain whitespace".into(),
            ));
        }

        if trimmed == "*" {
            return Err(Error::Validation(
                "Wildcard permissions are reserved for system roles".into(),
            ));
        }

        if !trimmed.contains(':') {
            return Err(Error::Validation(
                "Permission ID must include a namespace (e.g. app:resource:action)".into(),
            ));
        }

        Ok(())
    }

    async fn ensure_permission_assignable(&self, role: &Role, permission: &str) -> Result<()> {
        if permissions::is_system_permission(permission) {
            if role.client_id.is_some() {
                return Err(Error::Validation(
                    "System permissions cannot be assigned to client roles".into(),
                ));
            }
            return Ok(());
        }

        let custom = self
            .rbac_repo
            .find_custom_permission_by_key(&role.realm_id, role.client_id.as_ref(), permission)
            .await?;

        if custom.is_none() {
            return Err(Error::Validation(
                "Permission not found in custom permissions".into(),
            ));
        }

        Ok(())
    }
}
