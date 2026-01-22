use crate::domain::role::Permission;
use crate::domain::{group::Group, role::Role};
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::collections::HashSet;
use std::format;
use sqlx::QueryBuilder;
use uuid::Uuid;
use crate::domain::pagination::{PageRequest, PageResponse, SortDirection};

#[async_trait]
pub trait RbacRepository: Send + Sync {
    // --- Write ---
    async fn create_role(&self, role: &Role) -> Result<()>;
    async fn create_group(&self, group: &Group) -> Result<()>;
    async fn assign_role_to_group(&self, role_id: &Uuid, group_id: &Uuid) -> Result<()>;
    async fn assign_user_to_group(&self, user_id: &Uuid, group_id: &Uuid) -> Result<()>;
    async fn assign_permission_to_role(
        &self,
        permission: &Permission,
        role_id: &Uuid,
    ) -> Result<()>;

    // [NEW] Assign a direct role to a user (Realm Safe)
    async fn assign_role_to_user(&self, user_id: &Uuid, role_id: &Uuid) -> Result<()>;

    // --- Read ---
    async fn find_role_by_name(&self, realm_id: &Uuid, name: &str) -> Result<Option<Role>>;
    async fn find_group_by_name(&self, realm_id: &Uuid, name: &str) -> Result<Option<Group>>;

    async fn list_roles(
        &self,
        realm_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<Role>>;
    async fn list_client_roles(
        &self,
        realm_id: &Uuid,
        client_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<Role>>;
    // Find a specific role by ID (for validation)
    async fn find_role_by_id(&self, role_id: &Uuid) -> Result<Option<Role>>;
    // Find all groups in a realm (for listing)
    async fn find_groups_by_realm(&self, realm_id: &Uuid) -> Result<Vec<Group>>;

    async fn find_user_ids_in_group(&self, group_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn find_role_ids_for_user(&self, user_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn find_permissions_for_roles(&self, role_ids: &[Uuid]) -> Result<HashSet<Permission>>;
    async fn find_user_ids_for_role(&self, role_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn get_effective_permissions_for_user(&self, user_id: &Uuid) -> Result<HashSet<String>>;
    async fn find_role_names_for_user(&self, user_id: &Uuid) -> Result<Vec<String>>;
    async fn find_group_names_for_user(&self, user_id: &Uuid) -> Result<Vec<String>>;
    async fn delete_role(&self, role_id: &Uuid) -> Result<()>;
    async fn update_role(&self, role: &Role) -> Result<()>;
    async fn get_permissions_for_role(&self, role_id: &Uuid) -> Result<Vec<String>>;
    async fn remove_permission(&self, role_id: &Uuid, permission: &str) -> Result<()>;
    async fn bulk_update_permissions(&self, role_id: &Uuid, permissions: Vec<String>, action: &str) -> Result<()>;
}
