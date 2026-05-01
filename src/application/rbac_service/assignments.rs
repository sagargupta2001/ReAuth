use super::RbacService;
use crate::domain::events::{
    DomainEvent, RoleCompositeChanged, RoleGroupChanged, RolePermissionChanged, UserGroupChanged,
    UserRoleChanged,
};
use crate::domain::pagination::{PageRequest, PageResponse};
use crate::domain::rbac::*;
use crate::domain::role::Permission;
use crate::error::{Error, Result};
use std::collections::HashSet;
use tracing::instrument;
use uuid::Uuid;

impl RbacService {
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

    // --- Assignment Operations ---

    pub async fn assign_role_to_group(
        &self,
        realm_id: Uuid,
        role_id: Uuid,
        group_id: Uuid,
    ) -> Result<()> {
        let _ = self.get_role(realm_id, role_id).await?;
        let _ = self.get_group(realm_id, group_id).await?;

        let event = DomainEvent::RoleAssignedToGroup(RoleGroupChanged { role_id, group_id });

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.rbac_repo
                .assign_role_to_group(&role_id, &group_id, Some(&mut *tx))
                .await?;
            self.write_outbox(&event, Some(realm_id), &mut *tx).await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.tx_manager.commit(tx).await?;
                self.event_bus.publish(event).await;
            }
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

        Ok(())
    }

    pub async fn assign_composite_role(
        &self,
        realm_id: Uuid,
        parent_role_id: Uuid,
        child_role_id: Uuid,
    ) -> Result<()> {
        if parent_role_id == child_role_id {
            return Err(Error::Validation(
                "Cannot add a role as its own composite".into(),
            ));
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

        let event = DomainEvent::RoleCompositeChanged(RoleCompositeChanged {
            parent_role_id,
            child_role_id,
            action: "assigned".to_string(),
        });

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.rbac_repo
                .assign_composite_role(&parent_role_id, &child_role_id, Some(&mut *tx))
                .await?;
            self.write_outbox(&event, Some(realm_id), &mut *tx).await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.tx_manager.commit(tx).await?;
                self.event_bus.publish(event).await;
            }
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

        Ok(())
    }

    pub async fn assign_user_to_group(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        group_id: Uuid,
    ) -> Result<()> {
        let _ = self.get_group(realm_id, group_id).await?;

        let event = DomainEvent::UserAssignedToGroup(UserGroupChanged { user_id, group_id });

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.rbac_repo
                .assign_user_to_group(&user_id, &group_id, Some(&mut *tx))
                .await?;
            self.write_outbox(&event, Some(realm_id), &mut *tx).await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.tx_manager.commit(tx).await?;
                self.event_bus.publish(event).await;
            }
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

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

        let event = DomainEvent::RoleRemovedFromGroup(RoleGroupChanged { role_id, group_id });

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.rbac_repo
                .remove_role_from_group(&role_id, &group_id, Some(&mut *tx))
                .await?;
            self.write_outbox(&event, Some(realm_id), &mut *tx).await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.tx_manager.commit(tx).await?;
                self.event_bus.publish(event).await;
            }
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

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

        let event = DomainEvent::RoleCompositeChanged(RoleCompositeChanged {
            parent_role_id,
            child_role_id,
            action: "removed".to_string(),
        });

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.rbac_repo
                .remove_composite_role(&parent_role_id, &child_role_id, Some(&mut *tx))
                .await?;
            self.write_outbox(&event, Some(realm_id), &mut *tx).await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.tx_manager.commit(tx).await?;
                self.event_bus.publish(event).await;
            }
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

        Ok(())
    }

    pub async fn remove_user_from_group(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        group_id: Uuid,
    ) -> Result<()> {
        let _ = self.get_group(realm_id, group_id).await?;

        let event = DomainEvent::UserRemovedFromGroup(UserGroupChanged { user_id, group_id });

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.rbac_repo
                .remove_user_from_group(&user_id, &group_id, Some(&mut *tx))
                .await?;
            self.write_outbox(&event, Some(realm_id), &mut *tx).await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.tx_manager.commit(tx).await?;
                self.event_bus.publish(event).await;
            }
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

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

        let event = DomainEvent::UserRoleAssigned(UserRoleChanged { user_id, role_id });

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.rbac_repo
                .assign_role_to_user(&user_id, &role_id, Some(&mut *tx))
                .await?;
            self.write_outbox(&event, Some(realm_id), &mut *tx).await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.tx_manager.commit(tx).await?;
                self.event_bus.publish(event).await;
            }
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

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

        let event = DomainEvent::UserRoleRemoved(UserRoleChanged { user_id, role_id });

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.rbac_repo
                .remove_role_from_user(&user_id, &role_id, Some(&mut *tx))
                .await?;
            self.write_outbox(&event, Some(realm_id), &mut *tx).await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.tx_manager.commit(tx).await?;
                self.event_bus.publish(event).await;
            }
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

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

        self.ensure_permission_assignable(&role, &permission)
            .await?;

        // 2. Assign
        let event = DomainEvent::RolePermissionChanged(RolePermissionChanged {
            role_id,
            permission: permission.clone(),
            action: "assigned".to_string(),
        });

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.rbac_repo
                .assign_permission_to_role(&permission, &role_id, Some(&mut *tx))
                .await?;
            self.write_outbox(&event, Some(realm_id), &mut *tx).await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.tx_manager.commit(tx).await?;
                self.event_bus.publish(event).await;
            }
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

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
        let event = DomainEvent::RolePermissionChanged(RolePermissionChanged {
            role_id,
            permission: permission.clone(),
            action: "revoked".to_string(),
        });

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.rbac_repo
                .remove_permission(&role_id, &permission, Some(&mut *tx))
                .await?;
            self.write_outbox(&event, Some(realm_id), &mut *tx).await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.tx_manager.commit(tx).await?;
                self.event_bus.publish(event).await;
            }
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

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
            return Err(Error::Validation(
                "Invalid action. Use 'add' or 'remove'.".into(),
            ));
        }

        if action == "add" {
            for permission in &permissions {
                self.ensure_permission_assignable(&role, permission).await?;
            }
        }

        // 3. Perform Bulk Update
        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.rbac_repo
                .bulk_update_permissions(&role_id, permissions.clone(), &action, Some(&mut *tx))
                .await?;

            for perm in &permissions {
                let event = DomainEvent::RolePermissionChanged(RolePermissionChanged {
                    role_id,
                    permission: perm.clone(),
                    action: if action == "add" {
                        "assigned".to_string()
                    } else {
                        "revoked".to_string()
                    },
                });
                self.write_outbox(&event, Some(realm_id), &mut *tx).await?;
            }

            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.tx_manager.commit(tx).await?;
                for perm in permissions {
                    self.event_bus
                        .publish(DomainEvent::RolePermissionChanged(RolePermissionChanged {
                            role_id,
                            permission: perm,
                            action: if action == "add" {
                                "assigned".to_string()
                            } else {
                                "revoked".to_string()
                            },
                        }))
                        .await;
                }
            }
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
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

    pub async fn get_group_member_ids(&self, realm_id: Uuid, group_id: Uuid) -> Result<Vec<Uuid>> {
        let _ = self.get_group(realm_id, group_id).await?;
        self.rbac_repo.find_user_ids_in_group(&group_id).await
    }

    pub async fn get_group_role_ids(&self, realm_id: Uuid, group_id: Uuid) -> Result<Vec<Uuid>> {
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

    pub async fn get_role_composite_ids(&self, realm_id: Uuid, role_id: Uuid) -> Result<Vec<Uuid>> {
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

    #[instrument(skip_all, fields(telemetry = "span"))]
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

    #[instrument(skip_all, fields(telemetry = "span"))]
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
