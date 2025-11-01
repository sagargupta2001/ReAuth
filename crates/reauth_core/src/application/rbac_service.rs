use crate::{
    domain::{
        events::{DomainEvent, RoleGroupChanged, RolePermissionChanged, UserGroupChanged},
        group::Group,
        role::{Permission, Role},
    },
    error::{Error, Result},
    ports::{
        cache_service::CacheService,
        event_bus::EventPublisher,
        rbac_repository::RbacRepository,
    },
};
use std::{collections::HashSet, sync::Arc};
use tracing::info;
use uuid::Uuid;

// --- Payloads for API requests ---
#[derive(serde::Deserialize, Clone, Default)]
pub struct CreateRolePayload {
    pub name: String,
    pub description: Option<String>,
}

#[derive(serde::Deserialize, Clone, Default)]
pub struct CreateGroupPayload {
    pub name: String,
    pub description: Option<String>,
}

/// The application service for handling all RBAC logic.
pub struct RbacService {
    rbac_repo: Arc<dyn RbacRepository>,
    cache: Arc<dyn CacheService>,
    event_bus: Arc<dyn EventPublisher>,
}

impl RbacService {
    pub fn new(
        rbac_repo: Arc<dyn RbacRepository>,
        cache: Arc<dyn CacheService>,
        event_bus: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { rbac_repo, cache, event_bus }
    }

    // --- Write Operations (CRUD & Assignments) ---
    pub async fn create_role(&self, payload: CreateRolePayload) -> Result<Role> {
        if self.rbac_repo.find_role_by_name(&payload.name).await?.is_some() {
            return Err(Error::RoleAlreadyExists);
        }
        let role = Role {
            id: Uuid::new_v4(),
            name: payload.name,
            description: payload.description,
        };
        self.rbac_repo.create_role(&role).await?;
        Ok(role)
    }

    pub async fn create_group(&self, payload: CreateGroupPayload) -> Result<Group> {
        if self.rbac_repo.find_group_by_name(&payload.name).await?.is_some() {
            return Err(Error::GroupAlreadyExists);
        }
        let group = Group {
            id: Uuid::new_v4(),
            name: payload.name,
            description: payload.description,
        };
        self.rbac_repo.create_group(&group).await?;
        Ok(group)
    }

    pub async fn assign_role_to_group(&self, role_id: Uuid, group_id: Uuid) -> Result<()> {
        self.rbac_repo.assign_role_to_group(&role_id, &group_id).await?;

        self.event_bus.publish(DomainEvent::RoleAssignedToGroup(
            RoleGroupChanged { role_id, group_id }
        )).await;

        Ok(())
    }

    pub async fn assign_user_to_group(&self, user_id: Uuid, group_id: Uuid) -> Result<()> {
        self.rbac_repo.assign_user_to_group(&user_id, &group_id).await?;

        self.event_bus.publish(DomainEvent::UserAssignedToGroup(
            UserGroupChanged { user_id, group_id }
        )).await;

        Ok(())
    }

    pub async fn assign_permission_to_role(&self, permission: Permission, role_id: Uuid) -> Result<()> {
        self.rbac_repo.assign_permission_to_role(&permission, &role_id).await?;

        self.event_bus.publish(DomainEvent::RolePermissionChanged(
            RolePermissionChanged { role_id }
        )).await;

        Ok(())
    }

    // --- Read Operations (High-Performance Caching) ---
    pub async fn user_has_permission(&self, user_id: &Uuid, permission: &str) -> Result<bool> {
        let effective_permissions = self.get_effective_permissions(user_id).await?;
        Ok(effective_permissions.contains(permission))
    }

    pub async fn get_effective_permissions(&self, user_id: &Uuid) -> Result<HashSet<Permission>> {
        if let Some(permissions) = self.cache.get_user_permissions(user_id).await {
            info!("Cache HIT for user: {}", user_id);
            return Ok(permissions);
        }

        info!("Cache MISS for user: {}", user_id);
        let role_ids = self.rbac_repo.find_role_ids_for_user(user_id).await?;

        let permissions = if role_ids.is_empty() {
            HashSet::new()
        } else {
            self.rbac_repo.find_permissions_for_roles(&role_ids).await?
        };

        self.cache.set_user_permissions(user_id, &permissions).await;

        Ok(permissions)
    }
}