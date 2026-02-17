use crate::domain::events::{RoleCompositeChanged, UserRoleChanged};
use crate::{
    domain::{
        events::{DomainEvent, RoleGroupChanged, RolePermissionChanged, UserGroupChanged},
        group::Group,
        permissions,
        rbac::{
            CustomPermission, GroupDeleteSummary, GroupMemberFilter, GroupMemberRow, GroupRoleFilter,
            GroupRoleRow, GroupTreeRow, RoleCompositeFilter, RoleCompositeRow, RoleMemberFilter,
            RoleMemberRow, UserRoleFilter, UserRoleRow,
        },
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

        if let Some(parent_id) = payload.parent_id {
            let _ = self.get_group(realm_id, parent_id).await?;
        }

        let sort_order = self
            .rbac_repo
            .get_next_group_sort_order(&realm_id, payload.parent_id.as_ref())
            .await?;

        let group = Group {
            id: Uuid::new_v4(),
            realm_id,
            parent_id: payload.parent_id,
            name: payload.name,
            description: payload.description,
            sort_order,
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

    pub async fn list_group_roots(
        &self,
        realm_id: Uuid,
        req: PageRequest,
    ) -> Result<PageResponse<GroupTreeRow>> {
        self.rbac_repo.list_group_roots(&realm_id, &req).await
    }

    pub async fn list_group_children(
        &self,
        realm_id: Uuid,
        parent_id: Uuid,
        req: PageRequest,
    ) -> Result<PageResponse<GroupTreeRow>> {
        let _ = self.get_group(realm_id, parent_id).await?;
        self.rbac_repo
            .list_group_children(&realm_id, &parent_id, &req)
            .await
    }

    pub async fn move_group(
        &self,
        realm_id: Uuid,
        group_id: Uuid,
        parent_id: Option<Uuid>,
        before_id: Option<Uuid>,
        after_id: Option<Uuid>,
    ) -> Result<()> {
        if before_id.is_some() && after_id.is_some() {
            return Err(Error::Validation(
                "Provide only one of before_id or after_id.".into(),
            ));
        }

        let group = self.get_group(realm_id, group_id).await?;

        if let Some(parent_id) = parent_id {
            if parent_id == group_id {
                return Err(Error::Validation("Group cannot be its own parent.".into()));
            }

            let _ = self.get_group(realm_id, parent_id).await?;

            if self
                .rbac_repo
                .is_group_descendant(&realm_id, &group_id, &parent_id)
                .await?
            {
                return Err(Error::Validation(
                    "Cannot move a group inside its own subtree.".into(),
                ));
            }
        }

        if let Some(before_id) = before_id {
            let before_group = self.get_group(realm_id, before_id).await?;
            if before_group.parent_id != parent_id {
                return Err(Error::Validation(
                    "before_id must be a sibling under the target parent.".into(),
                ));
            }
        }

        if let Some(after_id) = after_id {
            let after_group = self.get_group(realm_id, after_id).await?;
            if after_group.parent_id != parent_id {
                return Err(Error::Validation(
                    "after_id must be a sibling under the target parent.".into(),
                ));
            }
        }

        let mut siblings = self
            .rbac_repo
            .list_group_ids_by_parent(&realm_id, parent_id.as_ref())
            .await?;

        siblings.retain(|id| id != &group_id);

        let insert_index = if let Some(before_id) = before_id {
            siblings
                .iter()
                .position(|id| id == &before_id)
                .ok_or_else(|| Error::Validation("before_id not found.".into()))?
        } else if let Some(after_id) = after_id {
            let pos = siblings
                .iter()
                .position(|id| id == &after_id)
                .ok_or_else(|| Error::Validation("after_id not found.".into()))?;
            pos + 1
        } else {
            siblings.len()
        };

        siblings.insert(insert_index, group_id);

        self.rbac_repo
            .set_group_orders(&realm_id, parent_id.as_ref(), &siblings)
            .await?;

        if group.parent_id != parent_id {
            let mut old_siblings = self
                .rbac_repo
                .list_group_ids_by_parent(&realm_id, group.parent_id.as_ref())
                .await?;
            old_siblings.retain(|id| id != &group_id);
            self.rbac_repo
                .set_group_orders(&realm_id, group.parent_id.as_ref(), &old_siblings)
                .await?;
        }

        Ok(())
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

    pub async fn list_user_roles(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        filter: UserRoleFilter,
        req: PageRequest,
    ) -> Result<PageResponse<UserRoleRow>> {
        self.rbac_repo
            .list_user_roles(&realm_id, &user_id, filter, &req)
            .await
    }

    pub async fn list_role_composites(
        &self,
        realm_id: Uuid,
        role_id: Uuid,
        filter: RoleCompositeFilter,
        req: PageRequest,
    ) -> Result<PageResponse<RoleCompositeRow>> {
        let role = self.get_role(realm_id, role_id).await?;
        self.rbac_repo
            .list_role_composites(&realm_id, &role_id, &role.client_id, filter, &req)
            .await
    }

    pub async fn list_custom_permissions(
        &self,
        realm_id: Uuid,
        client_id: Option<Uuid>,
    ) -> Result<Vec<CustomPermission>> {
        self.rbac_repo
            .list_custom_permissions(&realm_id, client_id.as_ref())
            .await
    }

    pub async fn create_custom_permission(
        &self,
        realm_id: Uuid,
        payload: CreateCustomPermissionPayload,
    ) -> Result<CustomPermission> {
        let permission = payload.permission.trim().to_string();

        self.validate_custom_permission_key(&permission)?;

        if payload.name.trim().is_empty() {
            return Err(Error::Validation("Permission name cannot be empty".into()));
        }

        if permissions::is_system_permission(&permission) {
            return Err(Error::Validation(
                "Permission conflicts with a system permission".into(),
            ));
        }

        if let Some(existing) = self
            .rbac_repo
            .find_custom_permission_by_key(&realm_id, payload.client_id.as_ref(), &permission)
            .await?
        {
            return Err(Error::Validation(format!(
                "Permission already exists: {}",
                existing.permission
            )));
        }

        let permission = CustomPermission {
            id: Uuid::new_v4(),
            realm_id,
            client_id: payload.client_id,
            permission: permission.clone(),
            name: payload.name.trim().to_string(),
            description: payload
                .description
                .and_then(|d| {
                    let trimmed = d.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                }),
            created_by: None,
        };

        self.rbac_repo.create_custom_permission(&permission).await?;

        Ok(permission)
    }

    pub async fn update_custom_permission(
        &self,
        realm_id: Uuid,
        permission_id: Uuid,
        payload: UpdateCustomPermissionPayload,
    ) -> Result<CustomPermission> {
        if payload.name.trim().is_empty() {
            return Err(Error::Validation("Permission name cannot be empty".into()));
        }

        let existing = self
            .rbac_repo
            .find_custom_permission_by_id(&realm_id, &permission_id)
            .await?
            .ok_or(Error::NotFound("Custom permission not found".into()))?;

        let updated = CustomPermission {
            id: existing.id,
            realm_id: existing.realm_id,
            client_id: existing.client_id,
            permission: existing.permission.clone(),
            name: payload.name.trim().to_string(),
            description: payload
                .description
                .and_then(|d| {
                    let trimmed = d.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                }),
            created_by: existing.created_by,
        };

        self.rbac_repo.update_custom_permission(&updated).await?;
        Ok(updated)
    }

    pub async fn delete_custom_permission(
        &self,
        realm_id: Uuid,
        permission_id: Uuid,
    ) -> Result<()> {
        let permission = self
            .rbac_repo
            .find_custom_permission_by_id(&realm_id, &permission_id)
            .await?
            .ok_or(Error::NotFound("Custom permission not found".into()))?;

        self.rbac_repo
            .remove_role_permissions_by_key(&permission.permission)
            .await?;
        self.rbac_repo
            .delete_custom_permission(&permission_id)
            .await?;

        Ok(())
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

    pub async fn get_group_delete_summary(
        &self,
        realm_id: Uuid,
        group_id: Uuid,
    ) -> Result<GroupDeleteSummary> {
        let group = self.get_group(realm_id, group_id).await?;
        let subtree_ids = self
            .rbac_repo
            .list_group_subtree_ids(&realm_id, &group_id)
            .await?;

        let direct_children_count = self
            .rbac_repo
            .list_group_ids_by_parent(&realm_id, Some(&group_id))
            .await?
            .len() as i64;

        let descendant_count = subtree_ids.len().saturating_sub(1) as i64;
        let member_count = self.rbac_repo.count_user_ids_in_groups(&subtree_ids).await?;
        let role_count = self.rbac_repo.count_role_ids_in_groups(&subtree_ids).await?;

        Ok(GroupDeleteSummary {
            group_id,
            name: group.name,
            direct_children_count,
            descendant_count,
            member_count,
            role_count,
        })
    }

    pub async fn delete_group(
        &self,
        realm_id: Uuid,
        group_id: Uuid,
        cascade: bool,
    ) -> Result<()> {
        let _ = self.get_group(realm_id, group_id).await?;

        let direct_children = self
            .rbac_repo
            .list_group_ids_by_parent(&realm_id, Some(&group_id))
            .await?;
        if !cascade && !direct_children.is_empty() {
            return Err(Error::Validation(
                "Group has child groups. Use cascade delete to remove the subtree.".into(),
            ));
        }

        let group_ids = if cascade {
            self.rbac_repo
                .list_group_subtree_ids(&realm_id, &group_id)
                .await?
        } else {
            vec![group_id]
        };

        let affected_users = self
            .rbac_repo
            .find_user_ids_in_groups(&group_ids)
            .await?;

        self.rbac_repo.delete_groups(&group_ids).await?;

        self.event_bus
            .publish(DomainEvent::GroupDeleted(
                crate::domain::events::GroupDeleted {
                    group_ids,
                    affected_user_ids: affected_users,
                },
            ))
            .await;

        Ok(())
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

    pub async fn assign_composite_role(
        &self,
        realm_id: Uuid,
        parent_role_id: Uuid,
        child_role_id: Uuid,
    ) -> Result<()> {
        if parent_role_id == child_role_id {
            return Err(Error::Validation("Cannot add a role as its own composite".into()));
        }

        let parent = self.get_role(realm_id, parent_role_id).await?;
        let child = self.get_role(realm_id, child_role_id).await?;

        if parent.client_id != child.client_id {
            return Err(Error::Validation(
                "Composite roles must belong to the same client scope".into(),
            ));
        }

        if self
            .rbac_repo
            .is_role_descendant(&child_role_id, &parent_role_id)
            .await?
        {
            return Err(Error::Validation(
                "Composite assignment would create a cycle".into(),
            ));
        }

        self.rbac_repo
            .assign_composite_role(&parent_role_id, &child_role_id)
            .await?;

        self.event_bus
            .publish(DomainEvent::RoleCompositeChanged(RoleCompositeChanged {
                parent_role_id,
                child_role_id,
                action: "assigned".to_string(),
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

    pub async fn remove_composite_role(
        &self,
        realm_id: Uuid,
        parent_role_id: Uuid,
        child_role_id: Uuid,
    ) -> Result<()> {
        let parent = self.get_role(realm_id, parent_role_id).await?;
        let child = self.get_role(realm_id, child_role_id).await?;

        if parent.client_id != child.client_id {
            return Err(Error::Validation(
                "Composite roles must belong to the same client scope".into(),
            ));
        }

        self.rbac_repo
            .remove_composite_role(&parent_role_id, &child_role_id)
            .await?;

        self.event_bus
            .publish(DomainEvent::RoleCompositeChanged(RoleCompositeChanged {
                parent_role_id,
                child_role_id,
                action: "removed".to_string(),
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
        let role = self.get_role(realm_id, role_id).await?;

        self.ensure_permission_assignable(&role, &permission).await?;

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
        let role = self.get_role(realm_id, role_id).await?;

        // 2. Validate Action
        if action != "add" && action != "remove" {
            return Err(Error::Validation("Invalid action. Use 'add' or 'remove'.".into()));
        }

        if action == "add" {
            for permission in &permissions {
                self.ensure_permission_assignable(&role, permission).await?;
            }
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

    pub async fn get_effective_group_role_ids(
        &self,
        realm_id: Uuid,
        group_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        let _ = self.get_group(realm_id, group_id).await?;
        self.rbac_repo
            .find_effective_role_ids_for_group(&group_id)
            .await
    }

    pub async fn get_direct_role_ids_for_user(
        &self,
        _realm_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        self.rbac_repo.find_direct_role_ids_for_user(&user_id).await
    }

    pub async fn get_effective_role_ids_for_user(
        &self,
        _realm_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        self.rbac_repo
            .find_effective_role_ids_for_user(&user_id)
            .await
    }

    pub async fn get_role_composite_ids(
        &self,
        realm_id: Uuid,
        role_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        let _ = self.get_role(realm_id, role_id).await?;
        self.rbac_repo.list_role_composite_ids(&role_id).await
    }

    pub async fn get_effective_role_composite_ids(
        &self,
        realm_id: Uuid,
        role_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        let _ = self.get_role(realm_id, role_id).await?;
        self.rbac_repo
            .list_effective_role_composite_ids(&role_id)
            .await
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

        if !trimmed.contains(':') {
            return Err(Error::Validation(
                "Permission ID must include a namespace (e.g. app:resource:action)".into(),
            ));
        }

        if trimmed == "*" {
            return Err(Error::Validation(
                "Wildcard permissions are reserved for system roles".into(),
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
            .find_custom_permission_by_key(
                &role.realm_id,
                role.client_id.as_ref(),
                permission,
            )
            .await?;

        if custom.is_none() {
            return Err(Error::Validation(
                "Permission not found in custom permissions".into(),
            ));
        }

        Ok(())
    }
}
