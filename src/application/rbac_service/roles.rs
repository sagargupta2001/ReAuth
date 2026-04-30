use super::RbacService;
use super::{CreateCustomPermissionPayload, CreateRolePayload, UpdateCustomPermissionPayload};
use crate::domain::events::DomainEvent;
use crate::domain::pagination::{PageRequest, PageResponse};
use crate::domain::permissions;
use crate::domain::rbac::CustomPermission;
use crate::domain::role::Role;
use crate::error::{Error, Result};
use uuid::Uuid;

impl RbacService {
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
        self.rbac_repo.create_role(&role, None).await?;
        Ok(role)
    }

    pub async fn find_role_by_name(&self, realm_id: Uuid, name: &str) -> Result<Option<Role>> {
        self.rbac_repo.find_role_by_name(&realm_id, name).await
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
        self.rbac_repo
            .list_client_roles(&realm_id, &client_id, &req)
            .await
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
        payload: CreateRolePayload,
    ) -> Result<Role> {
        let mut role = self.get_role(realm_id, role_id).await?;

        // Update fields
        role.name = payload.name;
        role.description = payload.description;

        // Persist
        self.rbac_repo.update_role(&role, None).await?;

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

        let event = DomainEvent::RoleDeleted(crate::domain::events::RoleDeleted {
            role_id,
            affected_user_ids: affected_users,
        });

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            // 3. Delete from DB (Cascades will wipe the links)
            self.rbac_repo.delete_role(&role_id, Some(&mut *tx)).await?;
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
            description: payload.description.and_then(|d| {
                let trimmed = d.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }),
            created_by: None,
        };

        self.rbac_repo
            .create_custom_permission(&permission, None)
            .await?;

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
            description: payload.description.and_then(|d| {
                let trimmed = d.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }),
            created_by: existing.created_by,
        };

        self.rbac_repo
            .update_custom_permission(&updated, None)
            .await?;
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
            .remove_role_permissions_by_key(&permission.permission, None)
            .await?;
        self.rbac_repo
            .delete_custom_permission(&permission_id, None)
            .await?;

        Ok(())
    }
}
