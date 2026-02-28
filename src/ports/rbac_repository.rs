use crate::domain::pagination::{PageRequest, PageResponse};
use crate::domain::role::Permission;
use crate::domain::{
    group::Group,
    rbac::{
        CustomPermission, GroupMemberFilter, GroupMemberRow, GroupRoleFilter, GroupRoleRow,
        GroupTreeRow, RoleCompositeFilter, RoleCompositeRow, RoleMemberFilter, RoleMemberRow,
        UserRoleFilter, UserRoleRow,
    },
    role::Role,
};
use crate::error::Result;
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use std::collections::HashSet;
use uuid::Uuid;

#[async_trait]
pub trait RbacRepository: Send + Sync {
    // --- Write ---
    async fn create_role(&self, role: &Role, tx: Option<&mut dyn Transaction>) -> Result<()>;
    async fn create_group(&self, group: &Group, tx: Option<&mut dyn Transaction>) -> Result<()>;
    async fn assign_role_to_group(
        &self,
        role_id: &Uuid,
        group_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn remove_role_from_group(
        &self,
        role_id: &Uuid,
        group_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn assign_user_to_group(
        &self,
        user_id: &Uuid,
        group_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn remove_user_from_group(
        &self,
        user_id: &Uuid,
        group_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn assign_permission_to_role(
        &self,
        permission: &Permission,
        role_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;

    // [NEW] Assign a direct role to a user (Realm Safe)
    async fn assign_role_to_user(
        &self,
        user_id: &Uuid,
        role_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn remove_role_from_user(
        &self,
        user_id: &Uuid,
        role_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;

    // --- Read ---
    async fn find_role_by_name(&self, realm_id: &Uuid, name: &str) -> Result<Option<Role>>;
    async fn find_group_by_name(&self, realm_id: &Uuid, name: &str) -> Result<Option<Group>>;
    async fn find_group_by_id(&self, group_id: &Uuid) -> Result<Option<Group>>;

    async fn list_roles(&self, realm_id: &Uuid, req: &PageRequest) -> Result<PageResponse<Role>>;
    async fn list_client_roles(
        &self,
        realm_id: &Uuid,
        client_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<Role>>;
    // Find a specific role by ID (for validation)
    async fn find_role_by_id(&self, role_id: &Uuid) -> Result<Option<Role>>;
    // List groups in a realm (for listing)
    async fn list_groups(&self, realm_id: &Uuid, req: &PageRequest) -> Result<PageResponse<Group>>;
    async fn list_group_roots(
        &self,
        realm_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<GroupTreeRow>>;
    async fn list_group_children(
        &self,
        realm_id: &Uuid,
        parent_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<GroupTreeRow>>;
    async fn list_role_members(
        &self,
        realm_id: &Uuid,
        role_id: &Uuid,
        filter: RoleMemberFilter,
        req: &PageRequest,
    ) -> Result<PageResponse<RoleMemberRow>>;
    async fn list_group_members(
        &self,
        realm_id: &Uuid,
        group_id: &Uuid,
        filter: GroupMemberFilter,
        req: &PageRequest,
    ) -> Result<PageResponse<GroupMemberRow>>;
    async fn list_group_roles(
        &self,
        realm_id: &Uuid,
        group_id: &Uuid,
        filter: GroupRoleFilter,
        req: &PageRequest,
    ) -> Result<PageResponse<GroupRoleRow>>;
    async fn list_user_roles(
        &self,
        realm_id: &Uuid,
        user_id: &Uuid,
        filter: UserRoleFilter,
        req: &PageRequest,
    ) -> Result<PageResponse<UserRoleRow>>;
    async fn list_role_composites(
        &self,
        realm_id: &Uuid,
        role_id: &Uuid,
        client_id: &Option<Uuid>,
        filter: RoleCompositeFilter,
        req: &PageRequest,
    ) -> Result<PageResponse<RoleCompositeRow>>;
    async fn list_group_ids_by_parent(
        &self,
        realm_id: &Uuid,
        parent_id: Option<&Uuid>,
    ) -> Result<Vec<Uuid>>;
    async fn list_group_subtree_ids(&self, realm_id: &Uuid, root_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn set_group_orders(
        &self,
        realm_id: &Uuid,
        parent_id: Option<&Uuid>,
        ordered_ids: &[Uuid],
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn is_group_descendant(
        &self,
        realm_id: &Uuid,
        ancestor_id: &Uuid,
        candidate_id: &Uuid,
    ) -> Result<bool>;
    async fn get_next_group_sort_order(
        &self,
        realm_id: &Uuid,
        parent_id: Option<&Uuid>,
    ) -> Result<i64>;

    async fn find_user_ids_in_group(&self, group_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn find_user_ids_in_groups(&self, group_ids: &[Uuid]) -> Result<Vec<Uuid>>;
    async fn find_role_ids_for_group(&self, group_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn find_effective_role_ids_for_group(&self, group_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn count_user_ids_in_groups(&self, group_ids: &[Uuid]) -> Result<i64>;
    async fn count_role_ids_in_groups(&self, group_ids: &[Uuid]) -> Result<i64>;
    async fn find_direct_role_ids_for_user(&self, user_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn find_effective_role_ids_for_user(&self, user_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn find_role_ids_for_user(&self, user_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn find_permissions_for_roles(&self, role_ids: &[Uuid]) -> Result<HashSet<Permission>>;
    async fn find_user_ids_for_role(&self, role_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn find_direct_user_ids_for_role(&self, role_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn list_role_composite_ids(&self, role_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn list_effective_role_composite_ids(&self, role_id: &Uuid) -> Result<Vec<Uuid>>;
    async fn get_effective_permissions_for_user(&self, user_id: &Uuid) -> Result<HashSet<String>>;
    async fn find_role_names_for_user(&self, user_id: &Uuid) -> Result<Vec<String>>;
    async fn find_group_names_for_user(&self, user_id: &Uuid) -> Result<Vec<String>>;
    async fn delete_role(&self, role_id: &Uuid, tx: Option<&mut dyn Transaction>) -> Result<()>;
    async fn delete_groups(
        &self,
        group_ids: &[Uuid],
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn update_role(&self, role: &Role, tx: Option<&mut dyn Transaction>) -> Result<()>;
    async fn update_group(&self, group: &Group, tx: Option<&mut dyn Transaction>) -> Result<()>;
    async fn get_permissions_for_role(&self, role_id: &Uuid) -> Result<Vec<String>>;
    async fn remove_permission(
        &self,
        role_id: &Uuid,
        permission: &str,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn bulk_update_permissions(
        &self,
        role_id: &Uuid,
        permissions: Vec<String>,
        action: &str,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;

    async fn assign_composite_role(
        &self,
        parent_role_id: &Uuid,
        child_role_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn remove_composite_role(
        &self,
        parent_role_id: &Uuid,
        child_role_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn is_role_descendant(&self, ancestor_id: &Uuid, candidate_id: &Uuid) -> Result<bool>;

    async fn create_custom_permission(
        &self,
        permission: &CustomPermission,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn update_custom_permission(
        &self,
        permission: &CustomPermission,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn delete_custom_permission(
        &self,
        permission_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn find_custom_permission_by_key(
        &self,
        realm_id: &Uuid,
        client_id: Option<&Uuid>,
        permission: &str,
    ) -> Result<Option<CustomPermission>>;
    async fn find_custom_permission_by_id(
        &self,
        realm_id: &Uuid,
        permission_id: &Uuid,
    ) -> Result<Option<CustomPermission>>;
    async fn list_custom_permissions(
        &self,
        realm_id: &Uuid,
        client_id: Option<&Uuid>,
    ) -> Result<Vec<CustomPermission>>;
    async fn remove_role_permissions_by_key(
        &self,
        permission: &str,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
}
