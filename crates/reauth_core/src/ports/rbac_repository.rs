use std::collections::HashSet;
use crate::domain::{group::Group, role::Role};
use crate::error::Result;
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::role::Permission;

#[async_trait]
pub trait RbacRepository: Send + Sync {
    // --- Write ---
    async fn create_role(&self, role: &Role) -> Result<()>;
    async fn create_group(&self, group: &Group) -> Result<()>;
    async fn assign_role_to_group(&self, role_id: &Uuid, group_id: &Uuid) -> Result<()>;
    async fn assign_user_to_group(&self, user_id: &Uuid, group_id: &Uuid) -> Result<()>;
    async fn assign_permission_to_role(&self, permission: &Permission, role_id: &Uuid) -> Result<()>;

    // --- Read ---
    async fn find_role_by_name(&self, name: &str) -> Result<Option<Role>>;
    async fn find_group_by_name(&self, name: &str) -> Result<Option<Group>>;
    async fn find_user_ids_in_group(&self, group_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn find_role_ids_for_user(&self, user_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn find_permissions_for_roles(&self, role_ids: &[Uuid]) -> Result<HashSet<Permission>>;
    async fn find_user_ids_for_role(&self, role_id: &Uuid) -> Result<Vec<Uuid>>;
}