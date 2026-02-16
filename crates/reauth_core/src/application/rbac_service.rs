use crate::domain::events::UserRoleChanged;
use crate::{
    domain::{
        events::{DomainEvent, RoleGroupChanged, RolePermissionChanged, UserGroupChanged},
        group::Group,
        rbac::{GroupMemberFilter, GroupMemberRow, GroupRoleFilter, GroupRoleRow, RoleMemberFilter, RoleMemberRow},
        role::{Permission, Role},
    },
    error::{Error, Result},
    ports::{
        cache_service::CacheService, event_bus::EventPublisher, rbac_repository::RbacRepository,
    },
};
use std::{collections::HashSet, sync::Arc};
use uuid::Uuid;
use crate::domain::pagination::{PageRequest, PageResponse};

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

    // --- Write Operations (CRUD) ---
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
            client_id: payload.client_id,
            name: payload.name,
            description: payload.description,
        };
        self.rbac_repo.create_role(&role).await?;
        Ok(role)
    }

    pub async fn list_roles(&self, realm_id: Uuid, req: PageRequest) -> Result<PageResponse<Role>> {
        self.rbac_repo.list_roles(&realm_id, &req).await
    }

    pub async fn list_client_roles(
        &self,
        realm_id: Uuid,
        client_id: Uuid,
        req: PageRequest,
    ) -> Result<PageResponse<Role>> {
        // Optional: Verify client exists and belongs to realm
        self.rbac_repo.list_client_roles(&realm_id, &client_id, &req).await
    }

    pub async fn get_role(&self, realm_id: Uuid, role_id: Uuid) -> Result<Role> {
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

        Ok(role)
    }

    pub async fn update_role(
        &self,
        realm_id: Uuid,
        role_id: Uuid,
        payload: CreateRolePayload
    ) -> Result<Role> {
        let mut role = self.get_role(realm_id, role_id).await?;

        // Update fields
        role.name = payload.name;
        role.description = payload.description;

        // Persist
        self.rbac_repo.update_role(&role).await?;

        // Invalidate caches (Logic depends on your cache strategy, e.g. simply clearing user permissions cache)
        // self.event_bus.publish(...)

        Ok(role)
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

        // 2. Find affected users BEFORE deletion
        let affected_users = self.rbac_repo.find_user_ids_for_role(&role_id).await?;

        // 3. Delete from DB (Cascades will wipe the links)
        self.rbac_repo.delete_role(&role_id).await?;

        // 4. Publish Event to Clear Cache
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

    // --- Group Operations ---
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

    pub async fn list_groups(
        &self,
        realm_id: Uuid,
        req: PageRequest,
    ) -> Result<PageResponse<Group>> {
        self.rbac_repo.list_groups(&realm_id, &req).await
    }

    pub async fn list_role_members(
        &self,
        realm_id: Uuid,
        role_id: Uuid,
        filter: RoleMemberFilter,
        req: PageRequest,
    ) -> Result<PageResponse<RoleMemberRow>> {
        let _ = self.get_role(realm_id, role_id).await?;
        self.rbac_repo
            .list_role_members(&realm_id, &role_id, filter, &req)
            .await
    }

    pub async fn list_group_members(
        &self,
        realm_id: Uuid,
        group_id: Uuid,
        filter: GroupMemberFilter,
        req: PageRequest,
    ) -> Result<PageResponse<GroupMemberRow>> {
        let _ = self.get_group(realm_id, group_id).await?;
        self.rbac_repo
            .list_group_members(&realm_id, &group_id, filter, &req)
            .await
    }

    pub async fn list_group_roles(
        &self,
        realm_id: Uuid,
        group_id: Uuid,
        filter: GroupRoleFilter,
        req: PageRequest,
    ) -> Result<PageResponse<GroupRoleRow>> {
        let _ = self.get_group(realm_id, group_id).await?;
        self.rbac_repo
            .list_group_roles(&realm_id, &group_id, filter, &req)
            .await
    }

    pub async fn get_group(&self, realm_id: Uuid, group_id: Uuid) -> Result<Group> {
        let group = self
            .rbac_repo
            .find_group_by_id(&group_id)
            .await?
            .ok_or(Error::NotFound("Group not found".into()))?;

        if group.realm_id != realm_id {
            return Err(Error::SecurityViolation(
                "Group belongs to different realm".into(),
            ));
        }

        Ok(group)
    }

    pub async fn update_group(
        &self,
        realm_id: Uuid,
        group_id: Uuid,
        payload: CreateGroupPayload,
    ) -> Result<Group> {
        let mut group = self.get_group(realm_id, group_id).await?;

        group.name = payload.name;
        group.description = payload.description;

        self.rbac_repo.update_group(&group).await?;

        Ok(group)
    }

    // --- Assignment Operations ---

    pub async fn assign_role_to_group(
        &self,
        realm_id: Uuid,
        role_id: Uuid,
        group_id: Uuid,
    ) -> Result<()> {
        let _ = self.get_role(realm_id, role_id).await?;
        let _ = self.get_group(realm_id, group_id).await?;

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

    pub async fn assign_user_to_group(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        group_id: Uuid,
    ) -> Result<()> {
        let _ = self.get_group(realm_id, group_id).await?;

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

    pub async fn remove_role_from_group(
        &self,
        realm_id: Uuid,
        role_id: Uuid,
        group_id: Uuid,
    ) -> Result<()> {
        let _ = self.get_role(realm_id, role_id).await?;
        let _ = self.get_group(realm_id, group_id).await?;

        self.rbac_repo
            .remove_role_from_group(&role_id, &group_id)
            .await?;

        self.event_bus
            .publish(DomainEvent::RoleRemovedFromGroup(RoleGroupChanged {
                role_id,
                group_id,
            }))
            .await;

        Ok(())
    }

    pub async fn remove_user_from_group(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        group_id: Uuid,
    ) -> Result<()> {
        let _ = self.get_group(realm_id, group_id).await?;

        self.rbac_repo
            .remove_user_from_group(&user_id, &group_id)
            .await?;

        self.event_bus
            .publish(DomainEvent::UserRemovedFromGroup(UserGroupChanged {
                user_id,
                group_id,
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
        let role = self
            .rbac_repo
            .find_role_by_id(&role_id)
            .await?
            .ok_or(Error::NotFound("Role not found".into()))?;

        if role.realm_id != realm_id {
            return Err(Error::SecurityViolation("Cross-realm assignment".into()));
        }

        self.rbac_repo
            .assign_role_to_user(&user_id, &role_id)
            .await?;

        self.event_bus
            .publish(DomainEvent::UserRoleAssigned(UserRoleChanged {
                user_id,
                role_id,
            }))
            .await;

        Ok(())
    }

    pub async fn remove_role_from_user(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<()> {
        let role = self
            .rbac_repo
            .find_role_by_id(&role_id)
            .await?
            .ok_or(Error::NotFound("Role not found".into()))?;

        if role.realm_id != realm_id {
            return Err(Error::SecurityViolation("Cross-realm assignment".into()));
        }

        self.rbac_repo
            .remove_role_from_user(&user_id, &role_id)
            .await?;

        self.event_bus
            .publish(DomainEvent::UserRoleRemoved(UserRoleChanged {
                user_id,
                role_id,
            }))
            .await;

        Ok(())
    }

    // --- Permission Management Operations ---

    pub async fn get_permissions_for_role(
        &self,
        realm_id: Uuid,
        role_id: Uuid,
    ) -> Result<Vec<String>> {
        // Ensure role exists and belongs to realm
        let _ = self.get_role(realm_id, role_id).await?;

        self.rbac_repo.get_permissions_for_role(&role_id).await
    }

    pub async fn assign_permission_to_role(
        &self,
        realm_id: Uuid,
        role_id: Uuid,
        permission: Permission,
    ) -> Result<()> {
        // 1. Verify Role belongs to Realm
        let _ = self.get_role(realm_id, role_id).await?;

        // 2. Assign
        self.rbac_repo
            .assign_permission_to_role(&permission, &role_id)
            .await?;

        // 3. Event
        self.event_bus
            .publish(DomainEvent::RolePermissionChanged(RolePermissionChanged {
                role_id,
                permission: permission.clone(),
                action: "assigned".to_string(),
            }))
            .await;

        Ok(())
    }

    pub async fn revoke_permission(
        &self,
        realm_id: Uuid,
        role_id: Uuid,
        permission: String,
    ) -> Result<()> {
        // 1. Verify Role belongs to Realm
        let _ = self.get_role(realm_id, role_id).await?;

        // 2. Remove
        self.rbac_repo
            .remove_permission(&role_id, &permission)
            .await?;

        // 3. Event
        self.event_bus
            .publish(DomainEvent::RolePermissionChanged(RolePermissionChanged {
                role_id,
                permission,
                action: "revoked".to_string(),
            }))
            .await;

        Ok(())
    }

    pub async fn bulk_update_permissions(
        &self,
        realm_id: Uuid,
        role_id: Uuid,
        permissions: Vec<String>,
        action: String,
    ) -> Result<()> {
        // 1. Verify Role belongs to Realm
        let _ = self.get_role(realm_id, role_id).await?;

        // 2. Validate Action
        if action != "add" && action != "remove" {
            return Err(Error::Validation("Invalid action. Use 'add' or 'remove'.".into()));
        }

        // 3. Perform Bulk Update
        self.rbac_repo
            .bulk_update_permissions(&role_id, permissions.clone(), &action)
            .await?;

        // 4. Emit Events (Ideally batch this or emit one "Bulk" event)
        // For now we assume a simple generic event or skip high-volume auditing for bulk ops
        // or emit one event per permission if strict audit is required.
        // Keeping it simple here:
        for perm in permissions {
             self.event_bus
            .publish(DomainEvent::RolePermissionChanged(RolePermissionChanged {
                role_id,
                permission: perm,
                action: if action == "add" { "assigned".to_string() } else { "revoked".to_string() },
            }))
            .await; // Note: awaiting inside loop might be slow for massive updates, consider backgrounding or batch event
        }

        Ok(())
    }

    // --- User Query Operations ---

    pub async fn get_user_roles_and_groups(
        &self,
        user_id: &Uuid,
    ) -> Result<(Vec<String>, Vec<String>)> {
        let (roles, groups) = tokio::try_join!(
            self.rbac_repo.find_role_names_for_user(user_id),
            self.rbac_repo.find_group_names_for_user(user_id)
        )?;
        Ok((roles, groups))
    }

    pub async fn get_direct_user_ids_for_role(
        &self,
        realm_id: Uuid,
        role_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        let _ = self.get_role(realm_id, role_id).await?;
        self.rbac_repo.find_direct_user_ids_for_role(&role_id).await
    }

    pub async fn get_effective_user_ids_for_role(
        &self,
        realm_id: Uuid,
        role_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        let _ = self.get_role(realm_id, role_id).await?;
        self.rbac_repo.find_user_ids_for_role(&role_id).await
    }

    pub async fn get_group_member_ids(
        &self,
        realm_id: Uuid,
        group_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        let _ = self.get_group(realm_id, group_id).await?;
        self.rbac_repo.find_user_ids_in_group(&group_id).await
    }

    pub async fn get_group_role_ids(
        &self,
        realm_id: Uuid,
        group_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        let _ = self.get_group(realm_id, group_id).await?;
        self.rbac_repo.find_role_ids_for_group(&group_id).await
    }

    pub async fn user_has_permission(&self, user_id: &Uuid, permission: &str) -> Result<bool> {
        let perms = self.get_effective_permissions(user_id).await?;

        if perms.contains(permission) {
            return Ok(true);
        }

        if let Some((resource, _)) = permission.split_once(':') {
            let wildcard = format!("{}:*", resource);
            if perms.contains(&wildcard) {
                return Ok(true);
            }
        }

        if perms.contains("*") {
            return Ok(true);
        }

        Ok(false)
    }

    pub async fn get_effective_permissions(&self, user_id: &Uuid) -> Result<HashSet<String>> {
        if let Some(permissions) = self.cache.get_user_permissions(user_id).await {
            return Ok(permissions);
        }

        let permissions = self
            .rbac_repo
            .get_effective_permissions_for_user(user_id)
            .await?;

        self.cache.set_user_permissions(user_id, &permissions).await;

        Ok(permissions)
    }
}
