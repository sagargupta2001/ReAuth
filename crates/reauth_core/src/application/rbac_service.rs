use crate::domain::events::UserRoleChanged;
use crate::{
    domain::{
        events::{DomainEvent, RoleGroupChanged, RolePermissionChanged, UserGroupChanged},
        group::Group,
        role::{Permission, Role},
    },
    error::{Error, Result},
    ports::{
        cache_service::CacheService, event_bus::EventPublisher, rbac_repository::RbacRepository,
    },
};
use std::{collections::HashSet, sync::Arc};
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
        Self {
            rbac_repo,
            cache,
            event_bus,
        }
    }

    // --- Write Operations (CRUD & Assignments) ---
    pub async fn create_role(&self, realm_id: Uuid, payload: CreateRolePayload) -> Result<Role> {
        if self
            .rbac_repo
            .find_role_by_name(&realm_id, &payload.name)
            .await?
            .is_some()
        {
            return Err(Error::RoleAlreadyExists);
        }

        let role = Role {
            id: Uuid::new_v4(),
            realm_id,
            name: payload.name,
            description: payload.description,
        };
        self.rbac_repo.create_role(&role).await?;
        Ok(role)
    }

    pub async fn list_roles(&self, realm_id: Uuid, _page: usize) -> Result<Vec<Role>> {
        // You'll need to add `find_roles_by_realm` to your Repository trait first!
        self.rbac_repo.find_roles_by_realm(&realm_id).await
    }

    pub async fn create_group(&self, realm_id: Uuid, payload: CreateGroupPayload) -> Result<Group> {
        if self
            .rbac_repo
            .find_group_by_name(&realm_id, &payload.name)
            .await?
            .is_some()
        {
            return Err(Error::GroupAlreadyExists);
        }

        let group = Group {
            id: Uuid::new_v4(),
            realm_id,
            name: payload.name,
            description: payload.description,
        };
        self.rbac_repo.create_group(&group).await?;
        Ok(group)
    }

    pub async fn list_groups(&self, realm_id: Uuid, _page: usize) -> Result<Vec<Group>> {
        self.rbac_repo.find_groups_by_realm(&realm_id).await
    }

    pub async fn assign_role_to_group(&self, role_id: Uuid, group_id: Uuid) -> Result<()> {
        // Note: Ideally, we should also verify here that role_id and group_id
        // belong to the same realm to prevent cross-realm assignments.
        // The DB FK constraints will block it if IDs are invalid,
        // but explicit checks are better for error messages.

        self.rbac_repo
            .assign_role_to_group(&role_id, &group_id)
            .await?;

        self.event_bus
            .publish(DomainEvent::RoleAssignedToGroup(RoleGroupChanged {
                role_id,
                group_id,
            }))
            .await;

        Ok(())
    }

    pub async fn assign_user_to_group(&self, user_id: Uuid, group_id: Uuid) -> Result<()> {
        self.rbac_repo
            .assign_user_to_group(&user_id, &group_id)
            .await?;

        self.event_bus
            .publish(DomainEvent::UserAssignedToGroup(UserGroupChanged {
                user_id,
                group_id,
            }))
            .await;

        Ok(())
    }

    pub async fn assign_permission_to_role(
        &self,
        realm_id: Uuid,
        permission: Permission,
        role_id: Uuid,
    ) -> Result<()> {
        // 1. Verify Role belongs to Realm
        let role = self
            .rbac_repo
            .find_role_by_id(&role_id)
            .await?
            .ok_or(Error::NotFound("Role not found".into()))?;

        if role.realm_id != realm_id {
            return Err(Error::SecurityViolation(
                "Role belongs to different realm".into(),
            ));
        }

        // 2. Assign
        self.rbac_repo
            .assign_permission_to_role(&permission, &role_id)
            .await?;

        // 3. Event
        self.event_bus
            .publish(DomainEvent::RolePermissionChanged(RolePermissionChanged {
                role_id,
            }))
            .await;

        Ok(())
    }

    pub async fn assign_role_to_user(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<()> {
        // 1. Verify Role belongs to Realm
        let role = self
            .rbac_repo
            .find_role_by_id(&role_id)
            .await?
            .ok_or(Error::NotFound("Role not found".into()))?;
        if role.realm_id != realm_id {
            return Err(Error::SecurityViolation("Cross-realm assignment".into()));
        }

        // 2. Assign (You need to add `assign_role_to_user` to Repository)
        self.rbac_repo
            .assign_role_to_user(&user_id, &role_id)
            .await?;

        // 3. Invalidate Cache
        self.event_bus
            .publish(DomainEvent::UserRoleAssigned(UserRoleChanged {
                user_id,
                role_id,
            }))
            .await;

        Ok(())
    }

    pub async fn get_user_roles_and_groups(
        &self,
        user_id: &Uuid,
    ) -> Result<(Vec<String>, Vec<String>)> {
        // We can run these in parallel for performance
        let (roles, groups) = tokio::try_join!(
            self.rbac_repo.find_role_names_for_user(user_id),
            self.rbac_repo.find_group_names_for_user(user_id)
        )?;
        Ok((roles, groups))
    }

    // --- Read Operations (High-Performance Caching) ---
    pub async fn user_has_permission(&self, user_id: &Uuid, permission: &str) -> Result<bool> {
        let perms = self.get_effective_permissions(user_id).await?;

        // Check exact match OR wildcard match (e.g. "client:create" matches "client:*")
        if perms.contains(permission) {
            return Ok(true);
        }

        // Simple Wildcard Check: "client:*"
        if let Some((resource, _)) = permission.split_once(':') {
            let wildcard = format!("{}:*", resource);
            if perms.contains(&wildcard) {
                return Ok(true);
            }
        }

        // Super Admin Wildcard
        if perms.contains("*") {
            return Ok(true);
        }

        Ok(false)
    }

    pub async fn get_effective_permissions(&self, user_id: &Uuid) -> Result<HashSet<String>> {
        // 1. Try Cache
        if let Some(permissions) = self.cache.get_user_permissions(user_id).await {
            return Ok(permissions);
        }

        // 2. Cache Miss -> Run the Single Optimized Query
        // We no longer loop over roles manually in Rust. The CTE does it.
        let permissions = self
            .rbac_repo
            .get_effective_permissions_for_user(user_id)
            .await?;

        // 3. Update Cache
        self.cache.set_user_permissions(user_id, &permissions).await;

        Ok(permissions)
    }

    pub async fn delete_role(&self, realm_id: Uuid, role_id: Uuid) -> Result<()> {
        // 1. Verification (Realm Security)
        let role = self
            .rbac_repo
            .find_role_by_id(&role_id)
            .await?
            .ok_or(Error::NotFound("Role not found".into()))?;

        if role.realm_id != realm_id {
            return Err(Error::SecurityViolation(
                "Role belongs to different realm".into(),
            ));
        }

        // 2. [CRITICAL] Find affected users BEFORE deletion
        // We use the recursive method we made earlier.
        // This finds everyone: direct assignments + group assignments + inheritance
        let affected_users = self.rbac_repo.find_user_ids_for_role(&role_id).await?;

        // 3. Delete from DB (Cascades will wipe the links)
        self.rbac_repo.delete_role(&role_id).await?;

        // 4. Publish Event to Clear Cache
        // We offload the actual cache clearing to the listener
        self.event_bus
            .publish(DomainEvent::RoleDeleted(
                crate::domain::events::RoleDeleted {
                    role_id,
                    affected_user_ids: affected_users,
                },
            ))
            .await;

        Ok(())
    }
}
