use super::*;
use crate::domain::rbac::{
    GroupMemberRow, GroupRoleRow, GroupTreeRow, RoleCompositeRow, RoleMemberRow, UserRoleRow,
};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

#[derive(Default)]
struct TestCache {
    inner: Mutex<HashMap<Uuid, HashSet<String>>>,
}

#[async_trait]
impl CacheService for TestCache {
    async fn get_user_permissions(&self, user_id: &Uuid) -> Option<HashSet<String>> {
        self.inner.lock().unwrap().get(user_id).cloned()
    }

    async fn set_user_permissions(&self, user_id: &Uuid, permissions: &HashSet<String>) {
        self.inner
            .lock()
            .unwrap()
            .insert(*user_id, permissions.clone());
    }

    async fn clear_user_permissions(&self, user_id: &Uuid) {
        self.inner.lock().unwrap().remove(user_id);
    }
}

#[derive(Default)]
struct TestEventBus {
    events: Mutex<Vec<DomainEvent>>,
}

#[async_trait]
impl EventPublisher for TestEventBus {
    async fn publish(&self, event: DomainEvent) {
        self.events.lock().unwrap().push(event);
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SetGroupOrdersCall {
    parent_id: Option<Uuid>,
    ordered_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, PartialEq)]
struct BulkUpdateCall {
    role_id: Uuid,
    permissions: Vec<String>,
    action: String,
}

#[derive(Default)]
struct TestRbacRepo {
    roles: Mutex<HashMap<Uuid, Role>>,
    groups: Mutex<HashMap<Uuid, Group>>,
    group_children_by_parent: Mutex<HashMap<Option<Uuid>, Vec<Uuid>>>,
    group_subtree_by_root: Mutex<HashMap<Uuid, Vec<Uuid>>>,
    group_descendant: Mutex<bool>,
    next_group_sort_order: Mutex<i64>,
    count_user_ids_in_groups_result: Mutex<i64>,
    count_role_ids_in_groups_result: Mutex<i64>,
    custom_permissions: Mutex<HashMap<Uuid, CustomPermission>>,
    custom_permissions_by_key: Mutex<HashMap<String, Uuid>>,
    role_descendant: Mutex<bool>,
    effective_permissions: Mutex<HashMap<Uuid, HashSet<String>>>,
    find_user_ids_for_role: Mutex<HashMap<Uuid, Vec<Uuid>>>,
    find_direct_user_ids_for_role: Mutex<HashMap<Uuid, Vec<Uuid>>>,
    find_user_ids_in_groups_result: Mutex<Vec<Uuid>>,
    find_user_ids_in_group: Mutex<HashMap<Uuid, Vec<Uuid>>>,
    find_role_ids_for_group: Mutex<HashMap<Uuid, Vec<Uuid>>>,
    find_effective_role_ids_for_group: Mutex<HashMap<Uuid, Vec<Uuid>>>,
    find_direct_role_ids_for_user: Mutex<HashMap<Uuid, Vec<Uuid>>>,
    find_effective_role_ids_for_user: Mutex<HashMap<Uuid, Vec<Uuid>>>,
    list_role_composite_ids: Mutex<HashMap<Uuid, Vec<Uuid>>>,
    list_effective_role_composite_ids: Mutex<HashMap<Uuid, Vec<Uuid>>>,
    find_role_names_for_user: Mutex<HashMap<Uuid, Vec<String>>>,
    find_group_names_for_user: Mutex<HashMap<Uuid, Vec<String>>>,
    assign_permission_to_role_calls: Mutex<Vec<(Uuid, String)>>,
    remove_permission_calls: Mutex<Vec<(Uuid, String)>>,
    bulk_update_permissions_calls: Mutex<Vec<BulkUpdateCall>>,
    remove_role_permissions_by_key_calls: Mutex<Vec<String>>,
    delete_groups_calls: Mutex<Vec<Vec<Uuid>>>,
    set_group_orders_calls: Mutex<Vec<SetGroupOrdersCall>>,
}

impl TestRbacRepo {
    fn insert_role(&self, role: Role) {
        self.roles.lock().unwrap().insert(role.id, role);
    }

    fn set_role_descendant(&self, value: bool) {
        *self.role_descendant.lock().unwrap() = value;
    }

    fn set_group_descendant(&self, value: bool) {
        *self.group_descendant.lock().unwrap() = value;
    }

    fn set_next_group_sort_order(&self, value: i64) {
        *self.next_group_sort_order.lock().unwrap() = value;
    }

    fn set_count_user_ids_in_groups_result(&self, value: i64) {
        *self.count_user_ids_in_groups_result.lock().unwrap() = value;
    }

    fn set_count_role_ids_in_groups_result(&self, value: i64) {
        *self.count_role_ids_in_groups_result.lock().unwrap() = value;
    }

    fn set_group_children(&self, parent_id: Option<Uuid>, children: Vec<Uuid>) {
        self.group_children_by_parent
            .lock()
            .unwrap()
            .insert(parent_id, children);
    }

    fn set_group_subtree(&self, root_id: Uuid, ids: Vec<Uuid>) {
        self.group_subtree_by_root
            .lock()
            .unwrap()
            .insert(root_id, ids);
    }

    fn set_find_user_ids_for_role(&self, role_id: Uuid, user_ids: Vec<Uuid>) {
        self.find_user_ids_for_role
            .lock()
            .unwrap()
            .insert(role_id, user_ids);
    }

    fn set_find_direct_user_ids_for_role(&self, role_id: Uuid, user_ids: Vec<Uuid>) {
        self.find_direct_user_ids_for_role
            .lock()
            .unwrap()
            .insert(role_id, user_ids);
    }

    fn set_find_user_ids_in_groups_result(&self, user_ids: Vec<Uuid>) {
        *self.find_user_ids_in_groups_result.lock().unwrap() = user_ids;
    }

    fn set_find_user_ids_in_group(&self, group_id: Uuid, user_ids: Vec<Uuid>) {
        self.find_user_ids_in_group
            .lock()
            .unwrap()
            .insert(group_id, user_ids);
    }

    fn set_find_role_ids_for_group(&self, group_id: Uuid, role_ids: Vec<Uuid>) {
        self.find_role_ids_for_group
            .lock()
            .unwrap()
            .insert(group_id, role_ids);
    }

    fn set_find_effective_role_ids_for_group(&self, group_id: Uuid, role_ids: Vec<Uuid>) {
        self.find_effective_role_ids_for_group
            .lock()
            .unwrap()
            .insert(group_id, role_ids);
    }

    fn set_find_direct_role_ids_for_user(&self, user_id: Uuid, role_ids: Vec<Uuid>) {
        self.find_direct_role_ids_for_user
            .lock()
            .unwrap()
            .insert(user_id, role_ids);
    }

    fn set_find_effective_role_ids_for_user(&self, user_id: Uuid, role_ids: Vec<Uuid>) {
        self.find_effective_role_ids_for_user
            .lock()
            .unwrap()
            .insert(user_id, role_ids);
    }

    fn set_list_role_composite_ids(&self, role_id: Uuid, composites: Vec<Uuid>) {
        self.list_role_composite_ids
            .lock()
            .unwrap()
            .insert(role_id, composites);
    }

    fn set_list_effective_role_composite_ids(&self, role_id: Uuid, composites: Vec<Uuid>) {
        self.list_effective_role_composite_ids
            .lock()
            .unwrap()
            .insert(role_id, composites);
    }

    fn set_find_role_names_for_user(&self, user_id: Uuid, roles: Vec<String>) {
        self.find_role_names_for_user
            .lock()
            .unwrap()
            .insert(user_id, roles);
    }

    fn set_find_group_names_for_user(&self, user_id: Uuid, groups: Vec<String>) {
        self.find_group_names_for_user
            .lock()
            .unwrap()
            .insert(user_id, groups);
    }

    fn set_effective_permissions(&self, user_id: Uuid, permissions: HashSet<String>) {
        self.effective_permissions
            .lock()
            .unwrap()
            .insert(user_id, permissions);
    }

    fn permission_key(realm_id: &Uuid, client_id: Option<&Uuid>, permission: &str) -> String {
        format!(
            "{}:{}:{}",
            realm_id,
            client_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "none".to_string()),
            permission
        )
    }

    fn empty_page<T>() -> PageResponse<T> {
        PageResponse::new(Vec::new(), 0, 1, 20)
    }
}

#[async_trait]
#[allow(unused_variables)]
impl RbacRepository for TestRbacRepo {
    async fn create_role(&self, role: &Role) -> Result<()> {
        self.roles.lock().unwrap().insert(role.id, role.clone());
        Ok(())
    }

    async fn create_group(&self, group: &Group) -> Result<()> {
        self.groups.lock().unwrap().insert(group.id, group.clone());
        Ok(())
    }

    async fn assign_role_to_group(&self, role_id: &Uuid, group_id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn remove_role_from_group(&self, role_id: &Uuid, group_id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn assign_user_to_group(&self, user_id: &Uuid, group_id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn remove_user_from_group(&self, user_id: &Uuid, group_id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn assign_permission_to_role(
        &self,
        permission: &Permission,
        role_id: &Uuid,
    ) -> Result<()> {
        self.assign_permission_to_role_calls
            .lock()
            .unwrap()
            .push((*role_id, permission.clone()));
        Ok(())
    }

    async fn assign_role_to_user(&self, user_id: &Uuid, role_id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn remove_role_from_user(&self, user_id: &Uuid, role_id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn find_role_by_name(&self, realm_id: &Uuid, name: &str) -> Result<Option<Role>> {
        Ok(self
            .roles
            .lock()
            .unwrap()
            .values()
            .find(|role| role.realm_id == *realm_id && role.name == name)
            .cloned())
    }

    async fn find_group_by_name(&self, realm_id: &Uuid, name: &str) -> Result<Option<Group>> {
        Ok(self
            .groups
            .lock()
            .unwrap()
            .values()
            .find(|group| group.realm_id == *realm_id && group.name == name)
            .cloned())
    }

    async fn find_group_by_id(&self, group_id: &Uuid) -> Result<Option<Group>> {
        Ok(self.groups.lock().unwrap().get(group_id).cloned())
    }

    async fn list_roles(&self, realm_id: &Uuid, req: &PageRequest) -> Result<PageResponse<Role>> {
        Ok(Self::empty_page())
    }

    async fn list_client_roles(
        &self,
        realm_id: &Uuid,
        client_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<Role>> {
        Ok(Self::empty_page())
    }

    async fn find_role_by_id(&self, role_id: &Uuid) -> Result<Option<Role>> {
        Ok(self.roles.lock().unwrap().get(role_id).cloned())
    }

    async fn list_groups(&self, realm_id: &Uuid, req: &PageRequest) -> Result<PageResponse<Group>> {
        Ok(Self::empty_page())
    }

    async fn list_group_roots(
        &self,
        realm_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<GroupTreeRow>> {
        Ok(Self::empty_page())
    }

    async fn list_group_children(
        &self,
        realm_id: &Uuid,
        parent_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<GroupTreeRow>> {
        Ok(Self::empty_page())
    }

    async fn list_role_members(
        &self,
        realm_id: &Uuid,
        role_id: &Uuid,
        filter: RoleMemberFilter,
        req: &PageRequest,
    ) -> Result<PageResponse<RoleMemberRow>> {
        Ok(Self::empty_page())
    }

    async fn list_group_members(
        &self,
        realm_id: &Uuid,
        group_id: &Uuid,
        filter: GroupMemberFilter,
        req: &PageRequest,
    ) -> Result<PageResponse<GroupMemberRow>> {
        Ok(Self::empty_page())
    }

    async fn list_group_roles(
        &self,
        realm_id: &Uuid,
        group_id: &Uuid,
        filter: GroupRoleFilter,
        req: &PageRequest,
    ) -> Result<PageResponse<GroupRoleRow>> {
        Ok(Self::empty_page())
    }

    async fn list_user_roles(
        &self,
        realm_id: &Uuid,
        user_id: &Uuid,
        filter: UserRoleFilter,
        req: &PageRequest,
    ) -> Result<PageResponse<UserRoleRow>> {
        Ok(Self::empty_page())
    }

    async fn list_role_composites(
        &self,
        realm_id: &Uuid,
        role_id: &Uuid,
        client_id: &Option<Uuid>,
        filter: RoleCompositeFilter,
        req: &PageRequest,
    ) -> Result<PageResponse<RoleCompositeRow>> {
        Ok(Self::empty_page())
    }

    async fn list_group_ids_by_parent(
        &self,
        realm_id: &Uuid,
        parent_id: Option<&Uuid>,
    ) -> Result<Vec<Uuid>> {
        Ok(self
            .group_children_by_parent
            .lock()
            .unwrap()
            .get(&parent_id.copied())
            .cloned()
            .unwrap_or_default())
    }

    async fn list_group_subtree_ids(&self, realm_id: &Uuid, root_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(self
            .group_subtree_by_root
            .lock()
            .unwrap()
            .get(root_id)
            .cloned()
            .unwrap_or_else(|| vec![*root_id]))
    }

    async fn set_group_orders(
        &self,
        realm_id: &Uuid,
        parent_id: Option<&Uuid>,
        ordered_ids: &[Uuid],
    ) -> Result<()> {
        self.set_group_orders_calls
            .lock()
            .unwrap()
            .push(SetGroupOrdersCall {
                parent_id: parent_id.copied(),
                ordered_ids: ordered_ids.to_vec(),
            });
        Ok(())
    }

    async fn is_group_descendant(
        &self,
        realm_id: &Uuid,
        ancestor_id: &Uuid,
        candidate_id: &Uuid,
    ) -> Result<bool> {
        Ok(*self.group_descendant.lock().unwrap())
    }

    async fn get_next_group_sort_order(
        &self,
        realm_id: &Uuid,
        parent_id: Option<&Uuid>,
    ) -> Result<i64> {
        Ok(*self.next_group_sort_order.lock().unwrap())
    }

    async fn find_user_ids_in_group(&self, group_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(self
            .find_user_ids_in_group
            .lock()
            .unwrap()
            .get(group_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn find_user_ids_in_groups(&self, group_ids: &[Uuid]) -> Result<Vec<Uuid>> {
        Ok(self.find_user_ids_in_groups_result.lock().unwrap().clone())
    }

    async fn find_role_ids_for_group(&self, group_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(self
            .find_role_ids_for_group
            .lock()
            .unwrap()
            .get(group_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn find_effective_role_ids_for_group(&self, group_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(self
            .find_effective_role_ids_for_group
            .lock()
            .unwrap()
            .get(group_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn count_user_ids_in_groups(&self, group_ids: &[Uuid]) -> Result<i64> {
        Ok(*self.count_user_ids_in_groups_result.lock().unwrap())
    }

    async fn count_role_ids_in_groups(&self, group_ids: &[Uuid]) -> Result<i64> {
        Ok(*self.count_role_ids_in_groups_result.lock().unwrap())
    }

    async fn find_direct_role_ids_for_user(&self, user_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(self
            .find_direct_role_ids_for_user
            .lock()
            .unwrap()
            .get(user_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn find_effective_role_ids_for_user(&self, user_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(self
            .find_effective_role_ids_for_user
            .lock()
            .unwrap()
            .get(user_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn find_role_ids_for_user(&self, user_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn find_permissions_for_roles(&self, role_ids: &[Uuid]) -> Result<HashSet<Permission>> {
        Ok(HashSet::new())
    }

    async fn find_user_ids_for_role(&self, role_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(self
            .find_user_ids_for_role
            .lock()
            .unwrap()
            .get(role_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn find_direct_user_ids_for_role(&self, role_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(self
            .find_direct_user_ids_for_role
            .lock()
            .unwrap()
            .get(role_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn list_role_composite_ids(&self, role_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(self
            .list_role_composite_ids
            .lock()
            .unwrap()
            .get(role_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn list_effective_role_composite_ids(&self, role_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(self
            .list_effective_role_composite_ids
            .lock()
            .unwrap()
            .get(role_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn get_effective_permissions_for_user(&self, user_id: &Uuid) -> Result<HashSet<String>> {
        Ok(self
            .effective_permissions
            .lock()
            .unwrap()
            .get(user_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn find_role_names_for_user(&self, user_id: &Uuid) -> Result<Vec<String>> {
        Ok(self
            .find_role_names_for_user
            .lock()
            .unwrap()
            .get(user_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn find_group_names_for_user(&self, user_id: &Uuid) -> Result<Vec<String>> {
        Ok(self
            .find_group_names_for_user
            .lock()
            .unwrap()
            .get(user_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn delete_role(&self, role_id: &Uuid) -> Result<()> {
        self.roles.lock().unwrap().remove(role_id);
        Ok(())
    }

    async fn delete_groups(&self, group_ids: &[Uuid]) -> Result<()> {
        let mut groups = self.groups.lock().unwrap();
        for id in group_ids {
            groups.remove(id);
        }
        self.delete_groups_calls
            .lock()
            .unwrap()
            .push(group_ids.to_vec());
        Ok(())
    }

    async fn update_role(&self, role: &Role) -> Result<()> {
        self.roles.lock().unwrap().insert(role.id, role.clone());
        Ok(())
    }

    async fn update_group(&self, group: &Group) -> Result<()> {
        self.groups.lock().unwrap().insert(group.id, group.clone());
        Ok(())
    }

    async fn get_permissions_for_role(&self, role_id: &Uuid) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    async fn remove_permission(&self, role_id: &Uuid, permission: &str) -> Result<()> {
        self.remove_permission_calls
            .lock()
            .unwrap()
            .push((*role_id, permission.to_string()));
        Ok(())
    }

    async fn bulk_update_permissions(
        &self,
        role_id: &Uuid,
        permissions: Vec<String>,
        action: &str,
    ) -> Result<()> {
        self.bulk_update_permissions_calls
            .lock()
            .unwrap()
            .push(BulkUpdateCall {
                role_id: *role_id,
                permissions,
                action: action.to_string(),
            });
        Ok(())
    }

    async fn assign_composite_role(
        &self,
        parent_role_id: &Uuid,
        child_role_id: &Uuid,
    ) -> Result<()> {
        Ok(())
    }

    async fn remove_composite_role(
        &self,
        parent_role_id: &Uuid,
        child_role_id: &Uuid,
    ) -> Result<()> {
        Ok(())
    }

    async fn is_role_descendant(&self, ancestor_id: &Uuid, candidate_id: &Uuid) -> Result<bool> {
        Ok(*self.role_descendant.lock().unwrap())
    }

    async fn create_custom_permission(&self, permission: &CustomPermission) -> Result<()> {
        let key = Self::permission_key(
            &permission.realm_id,
            permission.client_id.as_ref(),
            &permission.permission,
        );
        self.custom_permissions
            .lock()
            .unwrap()
            .insert(permission.id, permission.clone());
        self.custom_permissions_by_key
            .lock()
            .unwrap()
            .insert(key, permission.id);
        Ok(())
    }

    async fn update_custom_permission(&self, permission: &CustomPermission) -> Result<()> {
        self.custom_permissions
            .lock()
            .unwrap()
            .insert(permission.id, permission.clone());
        Ok(())
    }

    async fn delete_custom_permission(&self, permission_id: &Uuid) -> Result<()> {
        self.custom_permissions
            .lock()
            .unwrap()
            .remove(permission_id);
        Ok(())
    }

    async fn find_custom_permission_by_key(
        &self,
        realm_id: &Uuid,
        client_id: Option<&Uuid>,
        permission: &str,
    ) -> Result<Option<CustomPermission>> {
        let key = Self::permission_key(realm_id, client_id, permission);
        let permissions = self.custom_permissions.lock().unwrap();
        Ok(self
            .custom_permissions_by_key
            .lock()
            .unwrap()
            .get(&key)
            .and_then(|id| permissions.get(id))
            .cloned())
    }

    async fn find_custom_permission_by_id(
        &self,
        realm_id: &Uuid,
        permission_id: &Uuid,
    ) -> Result<Option<CustomPermission>> {
        Ok(self
            .custom_permissions
            .lock()
            .unwrap()
            .get(permission_id)
            .cloned())
    }

    async fn list_custom_permissions(
        &self,
        realm_id: &Uuid,
        client_id: Option<&Uuid>,
    ) -> Result<Vec<CustomPermission>> {
        Ok(self
            .custom_permissions
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect())
    }

    async fn remove_role_permissions_by_key(&self, permission: &str) -> Result<()> {
        self.remove_role_permissions_by_key_calls
            .lock()
            .unwrap()
            .push(permission.to_string());
        Ok(())
    }
}

struct RbacTestHarness {
    service: RbacService,
    cache: Arc<TestCache>,
    repo: Arc<TestRbacRepo>,
    events: Arc<TestEventBus>,
}

fn harness() -> RbacTestHarness {
    let repo = Arc::new(TestRbacRepo::default());
    let cache = Arc::new(TestCache::default());
    let events = Arc::new(TestEventBus::default());
    let service = RbacService::new(repo.clone(), cache.clone(), events.clone());

    RbacTestHarness {
        service,
        cache,
        repo,
        events,
    }
}

#[tokio::test]
async fn create_custom_permission_rejects_missing_namespace() {
    let harness = harness();
    let realm_id = Uuid::new_v4();

    let result = harness
        .service
        .create_custom_permission(
            realm_id,
            CreateCustomPermissionPayload {
                permission: "invalid".to_string(),
                name: "Test".to_string(),
                description: None,
                client_id: None,
            },
        )
        .await;

    match result {
        Err(Error::Validation(message)) => {
            assert!(message.contains("namespace"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn create_custom_permission_rejects_system_permission() {
    let harness = harness();
    let realm_id = Uuid::new_v4();

    let result = harness
        .service
        .create_custom_permission(
            realm_id,
            CreateCustomPermissionPayload {
                permission: permissions::REALM_READ.to_string(),
                name: "Realm Read".to_string(),
                description: None,
                client_id: None,
            },
        )
        .await;

    match result {
        Err(Error::Validation(message)) => {
            assert!(message.contains("system permission"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn create_custom_permission_trims_fields() {
    let harness = harness();
    let realm_id = Uuid::new_v4();

    let created = harness
        .service
        .create_custom_permission(
            realm_id,
            CreateCustomPermissionPayload {
                permission: "  app:read  ".to_string(),
                name: "  App Read  ".to_string(),
                description: Some("   ".to_string()),
                client_id: None,
            },
        )
        .await
        .expect("create permission");

    assert_eq!(created.permission, "app:read");
    assert_eq!(created.name, "App Read");
    assert!(created.description.is_none());
}

#[tokio::test]
async fn assign_permission_to_client_role_rejects_system_permission() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: Some(Uuid::new_v4()),
        name: "client-role".to_string(),
        description: None,
    });

    let result = harness
        .service
        .assign_permission_to_role(realm_id, role_id, permissions::USER_READ.to_string())
        .await;

    match result {
        Err(Error::Validation(message)) => {
            assert!(message.contains("System permissions cannot be assigned"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn assign_composite_role_rejects_cycles() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let parent_id = Uuid::new_v4();
    let child_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: parent_id,
        realm_id,
        client_id: None,
        name: "parent".to_string(),
        description: None,
    });
    harness.repo.insert_role(Role {
        id: child_id,
        realm_id,
        client_id: None,
        name: "child".to_string(),
        description: None,
    });
    harness.repo.set_role_descendant(true);

    let result = harness
        .service
        .assign_composite_role(realm_id, parent_id, child_id)
        .await;

    match result {
        Err(Error::Validation(message)) => {
            assert!(message.contains("create a cycle"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn user_has_permission_matches_exact() {
    let harness = harness();
    let user_id = Uuid::new_v4();

    let mut permissions_set = HashSet::new();
    permissions_set.insert(permissions::USER_READ.to_string());
    harness
        .cache
        .set_user_permissions(&user_id, &permissions_set)
        .await;

    let has_permission = harness
        .service
        .user_has_permission(&user_id, permissions::USER_READ)
        .await
        .expect("permission check");

    assert!(has_permission);
}

#[tokio::test]
async fn user_has_permission_matches_resource_wildcard() {
    let harness = harness();
    let user_id = Uuid::new_v4();

    let mut permissions_set = HashSet::new();
    permissions_set.insert("user:*".to_string());
    harness
        .cache
        .set_user_permissions(&user_id, &permissions_set)
        .await;

    let has_permission = harness
        .service
        .user_has_permission(&user_id, permissions::USER_WRITE)
        .await
        .expect("permission check");

    assert!(has_permission);
}

#[tokio::test]
async fn user_has_permission_matches_global_wildcard() {
    let harness = harness();
    let user_id = Uuid::new_v4();

    let mut permissions_set = HashSet::new();
    permissions_set.insert("*".to_string());
    harness
        .cache
        .set_user_permissions(&user_id, &permissions_set)
        .await;

    let has_permission = harness
        .service
        .user_has_permission(&user_id, "any:permission")
        .await
        .expect("permission check");

    assert!(has_permission);
}

#[tokio::test]
async fn user_has_permission_returns_false_when_missing() {
    let harness = harness();
    let user_id = Uuid::new_v4();

    let mut permissions_set = HashSet::new();
    permissions_set.insert(permissions::USER_READ.to_string());
    harness
        .cache
        .set_user_permissions(&user_id, &permissions_set)
        .await;

    let has_permission = harness
        .service
        .user_has_permission(&user_id, permissions::REALM_READ)
        .await
        .expect("permission check");

    assert!(!has_permission);
}

#[tokio::test]
async fn get_effective_permissions_caches_repo_result() {
    let harness = harness();
    let user_id = Uuid::new_v4();

    let mut permissions_set = HashSet::new();
    permissions_set.insert(permissions::USER_READ.to_string());
    harness
        .repo
        .set_effective_permissions(user_id, permissions_set.clone());

    let resolved = harness
        .service
        .get_effective_permissions(&user_id)
        .await
        .expect("resolve permissions");

    assert_eq!(resolved, permissions_set);

    let cached = harness.cache.get_user_permissions(&user_id).await;
    assert_eq!(cached, Some(permissions_set));
}

#[tokio::test]
async fn create_role_rejects_duplicate_name() {
    let harness = harness();
    let realm_id = Uuid::new_v4();

    let existing = Role {
        id: Uuid::new_v4(),
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    };
    harness.repo.insert_role(existing);

    let result = harness
        .service
        .create_role(
            realm_id,
            CreateRolePayload {
                client_id: None,
                name: "admin".to_string(),
                description: None,
            },
        )
        .await;

    assert!(matches!(result, Err(Error::RoleAlreadyExists)));
}

#[tokio::test]
async fn create_group_rejects_duplicate_name() {
    let harness = harness();
    let realm_id = Uuid::new_v4();

    let existing = Group {
        id: Uuid::new_v4(),
        realm_id,
        parent_id: None,
        name: "engineering".to_string(),
        description: None,
        sort_order: 0,
    };
    harness
        .repo
        .groups
        .lock()
        .unwrap()
        .insert(existing.id, existing);

    let result = harness
        .service
        .create_group(
            realm_id,
            CreateGroupPayload {
                parent_id: None,
                name: "engineering".to_string(),
                description: None,
            },
        )
        .await;

    assert!(matches!(result, Err(Error::GroupAlreadyExists)));
}

#[tokio::test]
async fn create_group_uses_next_sort_order() {
    let harness = harness();
    let realm_id = Uuid::new_v4();

    harness.repo.set_next_group_sort_order(42);

    let group = harness
        .service
        .create_group(
            realm_id,
            CreateGroupPayload {
                parent_id: None,
                name: "engineering".to_string(),
                description: None,
            },
        )
        .await
        .expect("create group");

    assert_eq!(group.sort_order, 42);
}

#[tokio::test]
async fn delete_role_publishes_event_with_affected_users() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let affected_users = vec![Uuid::new_v4(), Uuid::new_v4()];

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });
    harness
        .repo
        .set_find_user_ids_for_role(role_id, affected_users.clone());

    harness
        .service
        .delete_role(realm_id, role_id)
        .await
        .expect("delete role");

    let events = harness.events.events.lock().unwrap().clone();
    let has_event = events.iter().any(|event| match event {
        DomainEvent::RoleDeleted(payload) => {
            payload.role_id == role_id && payload.affected_user_ids == affected_users
        }
        _ => false,
    });

    assert!(has_event, "expected RoleDeleted event");
}

#[tokio::test]
async fn delete_group_requires_cascade_when_children_exist() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();

    harness.repo.groups.lock().unwrap().insert(
        group_id,
        Group {
            id: group_id,
            realm_id,
            parent_id: None,
            name: "root".to_string(),
            description: None,
            sort_order: 0,
        },
    );
    harness
        .repo
        .set_group_children(Some(group_id), vec![Uuid::new_v4()]);

    let result = harness
        .service
        .delete_group(realm_id, group_id, false)
        .await;

    match result {
        Err(Error::Validation(message)) => {
            assert!(message.contains("cascade"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn delete_group_cascade_deletes_subtree_and_publishes_event() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let root_id = Uuid::new_v4();
    let child_id = Uuid::new_v4();
    let subtree = vec![root_id, child_id];
    let affected_users = vec![Uuid::new_v4()];

    harness.repo.groups.lock().unwrap().insert(
        root_id,
        Group {
            id: root_id,
            realm_id,
            parent_id: None,
            name: "root".to_string(),
            description: None,
            sort_order: 0,
        },
    );
    harness.repo.groups.lock().unwrap().insert(
        child_id,
        Group {
            id: child_id,
            realm_id,
            parent_id: Some(root_id),
            name: "child".to_string(),
            description: None,
            sort_order: 1,
        },
    );
    harness.repo.set_group_subtree(root_id, subtree.clone());
    harness
        .repo
        .set_find_user_ids_in_groups_result(affected_users.clone());

    harness
        .service
        .delete_group(realm_id, root_id, true)
        .await
        .expect("delete group");

    let delete_calls = harness.repo.delete_groups_calls.lock().unwrap().clone();
    assert!(delete_calls.iter().any(|call| call == &subtree));

    let events = harness.events.events.lock().unwrap().clone();
    let has_event = events.iter().any(|event| match event {
        DomainEvent::GroupDeleted(payload) => {
            payload.group_ids == subtree && payload.affected_user_ids == affected_users
        }
        _ => false,
    });

    assert!(has_event, "expected GroupDeleted event");
}

#[tokio::test]
async fn assign_permission_to_role_requires_custom_permission() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });

    let result = harness
        .service
        .assign_permission_to_role(realm_id, role_id, "app:read".to_string())
        .await;

    match result {
        Err(Error::Validation(message)) => {
            assert!(message.contains("custom permissions"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn update_custom_permission_requires_existing_record() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let permission_id = Uuid::new_v4();

    let result = harness
        .service
        .update_custom_permission(
            realm_id,
            permission_id,
            UpdateCustomPermissionPayload {
                name: "Updated".to_string(),
                description: None,
            },
        )
        .await;

    assert!(matches!(result, Err(Error::NotFound(_))));
}

#[tokio::test]
async fn delete_custom_permission_removes_role_permissions_by_key() {
    let harness = harness();
    let realm_id = Uuid::new_v4();

    let created = harness
        .service
        .create_custom_permission(
            realm_id,
            CreateCustomPermissionPayload {
                permission: "app:read".to_string(),
                name: "App Read".to_string(),
                description: None,
                client_id: None,
            },
        )
        .await
        .expect("create custom permission");

    harness
        .service
        .delete_custom_permission(realm_id, created.id)
        .await
        .expect("delete custom permission");

    let calls = harness
        .repo
        .remove_role_permissions_by_key_calls
        .lock()
        .unwrap()
        .clone();
    assert_eq!(calls, vec![created.permission]);
}

#[tokio::test]
async fn bulk_update_permissions_rejects_invalid_action() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });

    let result = harness
        .service
        .bulk_update_permissions(
            realm_id,
            role_id,
            vec!["app:read".to_string()],
            "invalid".to_string(),
        )
        .await;

    assert!(matches!(result, Err(Error::Validation(_))));
}

#[tokio::test]
async fn move_group_rejects_before_not_sibling() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let before_id = Uuid::new_v4();
    let other_parent = Uuid::new_v4();

    harness.repo.groups.lock().unwrap().insert(
        group_id,
        Group {
            id: group_id,
            realm_id,
            parent_id: None,
            name: "target".to_string(),
            description: None,
            sort_order: 0,
        },
    );
    harness.repo.groups.lock().unwrap().insert(
        before_id,
        Group {
            id: before_id,
            realm_id,
            parent_id: Some(other_parent),
            name: "before".to_string(),
            description: None,
            sort_order: 1,
        },
    );

    let result = harness
        .service
        .move_group(realm_id, group_id, None, Some(before_id), None)
        .await;

    match result {
        Err(Error::Validation(message)) => {
            assert!(message.contains("before_id"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn move_group_rejects_descendant_parent() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let parent_id = Uuid::new_v4();

    harness.repo.groups.lock().unwrap().insert(
        group_id,
        Group {
            id: group_id,
            realm_id,
            parent_id: None,
            name: "target".to_string(),
            description: None,
            sort_order: 0,
        },
    );
    harness.repo.groups.lock().unwrap().insert(
        parent_id,
        Group {
            id: parent_id,
            realm_id,
            parent_id: None,
            name: "parent".to_string(),
            description: None,
            sort_order: 1,
        },
    );
    harness.repo.set_group_descendant(true);

    let result = harness
        .service
        .move_group(realm_id, group_id, Some(parent_id), None, None)
        .await;

    match result {
        Err(Error::Validation(message)) => {
            assert!(message.contains("subtree"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn create_role_persists_in_repo() {
    let harness = harness();
    let realm_id = Uuid::new_v4();

    let role = harness
        .service
        .create_role(
            realm_id,
            CreateRolePayload {
                client_id: None,
                name: "admin".to_string(),
                description: Some("Admin role".to_string()),
            },
        )
        .await
        .expect("create role");

    let stored = harness.repo.roles.lock().unwrap().get(&role.id).cloned();
    assert!(stored.is_some());
    let stored = stored.expect("stored role");
    assert_eq!(stored.name, "admin");
    assert_eq!(stored.description.as_deref(), Some("Admin role"));
}

#[tokio::test]
async fn get_role_rejects_cross_realm_access() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let other_realm = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });

    let result = harness.service.get_role(other_realm, role_id).await;

    assert!(matches!(result, Err(Error::SecurityViolation(_))));
}

#[tokio::test]
async fn update_role_updates_repo_state() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });

    let updated = harness
        .service
        .update_role(
            realm_id,
            role_id,
            CreateRolePayload {
                client_id: None,
                name: "super-admin".to_string(),
                description: Some("Updated".to_string()),
            },
        )
        .await
        .expect("update role");

    assert_eq!(updated.name, "super-admin");
    let stored = harness.repo.roles.lock().unwrap().get(&role_id).cloned();
    assert!(stored.is_some());
    let stored = stored.expect("stored role");
    assert_eq!(stored.name, "super-admin");
    assert_eq!(stored.description.as_deref(), Some("Updated"));
}

#[tokio::test]
async fn delete_role_returns_not_found_for_missing_role() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    let result = harness.service.delete_role(realm_id, role_id).await;

    assert!(matches!(result, Err(Error::NotFound(_))));
}

#[tokio::test]
async fn create_group_requires_existing_parent() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let parent_id = Uuid::new_v4();

    let result = harness
        .service
        .create_group(
            realm_id,
            CreateGroupPayload {
                parent_id: Some(parent_id),
                name: "child".to_string(),
                description: None,
            },
        )
        .await;

    assert!(matches!(result, Err(Error::NotFound(_))));
}

#[tokio::test]
async fn move_group_rejects_before_and_after_together() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();

    harness.repo.groups.lock().unwrap().insert(
        group_id,
        Group {
            id: group_id,
            realm_id,
            parent_id: None,
            name: "target".to_string(),
            description: None,
            sort_order: 0,
        },
    );

    let result = harness
        .service
        .move_group(
            realm_id,
            group_id,
            None,
            Some(Uuid::new_v4()),
            Some(Uuid::new_v4()),
        )
        .await;

    match result {
        Err(Error::Validation(message)) => {
            assert!(message.contains("before_id or after_id"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn move_group_updates_order_for_new_and_old_parent() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let old_parent = Uuid::new_v4();
    let new_parent = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let before_id = Uuid::new_v4();
    let old_sibling = Uuid::new_v4();
    let new_sibling = Uuid::new_v4();

    harness.repo.groups.lock().unwrap().insert(
        group_id,
        Group {
            id: group_id,
            realm_id,
            parent_id: Some(old_parent),
            name: "target".to_string(),
            description: None,
            sort_order: 0,
        },
    );
    harness.repo.groups.lock().unwrap().insert(
        before_id,
        Group {
            id: before_id,
            realm_id,
            parent_id: Some(new_parent),
            name: "before".to_string(),
            description: None,
            sort_order: 1,
        },
    );
    harness.repo.groups.lock().unwrap().insert(
        new_parent,
        Group {
            id: new_parent,
            realm_id,
            parent_id: None,
            name: "new-parent".to_string(),
            description: None,
            sort_order: 0,
        },
    );
    harness
        .repo
        .set_group_children(Some(new_parent), vec![before_id, new_sibling]);
    harness
        .repo
        .set_group_children(Some(old_parent), vec![group_id, old_sibling]);

    harness
        .service
        .move_group(realm_id, group_id, Some(new_parent), Some(before_id), None)
        .await
        .expect("move group");

    let calls = harness.repo.set_group_orders_calls.lock().unwrap().clone();
    assert!(
        calls.contains(&SetGroupOrdersCall {
            parent_id: Some(new_parent),
            ordered_ids: vec![group_id, before_id, new_sibling],
        }),
        "expected new parent order update"
    );
    assert!(
        calls.contains(&SetGroupOrdersCall {
            parent_id: Some(old_parent),
            ordered_ids: vec![old_sibling],
        }),
        "expected old parent order update"
    );
}

#[tokio::test]
async fn get_group_delete_summary_returns_counts() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let root_id = Uuid::new_v4();

    harness.repo.groups.lock().unwrap().insert(
        root_id,
        Group {
            id: root_id,
            realm_id,
            parent_id: None,
            name: "root".to_string(),
            description: None,
            sort_order: 0,
        },
    );
    harness
        .repo
        .set_group_subtree(root_id, vec![root_id, Uuid::new_v4(), Uuid::new_v4()]);
    harness
        .repo
        .set_group_children(Some(root_id), vec![Uuid::new_v4(), Uuid::new_v4()]);
    harness.repo.set_count_user_ids_in_groups_result(5);
    harness.repo.set_count_role_ids_in_groups_result(3);

    let summary = harness
        .service
        .get_group_delete_summary(realm_id, root_id)
        .await
        .expect("summary");

    assert_eq!(summary.direct_children_count, 2);
    assert_eq!(summary.descendant_count, 2);
    assert_eq!(summary.member_count, 5);
    assert_eq!(summary.role_count, 3);
}

#[tokio::test]
async fn delete_group_without_children_deletes_single_group() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let affected_users = vec![Uuid::new_v4()];

    harness.repo.groups.lock().unwrap().insert(
        group_id,
        Group {
            id: group_id,
            realm_id,
            parent_id: None,
            name: "root".to_string(),
            description: None,
            sort_order: 0,
        },
    );
    harness
        .repo
        .set_find_user_ids_in_groups_result(affected_users.clone());

    harness
        .service
        .delete_group(realm_id, group_id, false)
        .await
        .expect("delete group");

    let delete_calls = harness.repo.delete_groups_calls.lock().unwrap().clone();
    assert!(delete_calls.iter().any(|call| call == &vec![group_id]));

    let events = harness.events.events.lock().unwrap().clone();
    let has_event = events.iter().any(|event| match event {
        DomainEvent::GroupDeleted(payload) => {
            payload.group_ids == vec![group_id] && payload.affected_user_ids == affected_users
        }
        _ => false,
    });
    assert!(has_event, "expected GroupDeleted event");
}

#[tokio::test]
async fn assign_role_to_group_publishes_event() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });
    harness.repo.groups.lock().unwrap().insert(
        group_id,
        Group {
            id: group_id,
            realm_id,
            parent_id: None,
            name: "group".to_string(),
            description: None,
            sort_order: 0,
        },
    );

    harness
        .service
        .assign_role_to_group(realm_id, role_id, group_id)
        .await
        .expect("assign role to group");

    let events = harness.events.events.lock().unwrap().clone();
    let has_event = events.iter().any(|event| matches!(event, DomainEvent::RoleAssignedToGroup(RoleGroupChanged { role_id: rid, group_id: gid }) if *rid == role_id && *gid == group_id));
    assert!(has_event, "expected RoleAssignedToGroup event");
}

#[tokio::test]
async fn remove_role_from_group_publishes_event() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });
    harness.repo.groups.lock().unwrap().insert(
        group_id,
        Group {
            id: group_id,
            realm_id,
            parent_id: None,
            name: "group".to_string(),
            description: None,
            sort_order: 0,
        },
    );

    harness
        .service
        .remove_role_from_group(realm_id, role_id, group_id)
        .await
        .expect("remove role from group");

    let events = harness.events.events.lock().unwrap().clone();
    let has_event = events.iter().any(|event| matches!(event, DomainEvent::RoleRemovedFromGroup(RoleGroupChanged { role_id: rid, group_id: gid }) if *rid == role_id && *gid == group_id));
    assert!(has_event, "expected RoleRemovedFromGroup event");
}

#[tokio::test]
async fn assign_user_to_group_publishes_event() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    harness.repo.groups.lock().unwrap().insert(
        group_id,
        Group {
            id: group_id,
            realm_id,
            parent_id: None,
            name: "group".to_string(),
            description: None,
            sort_order: 0,
        },
    );

    harness
        .service
        .assign_user_to_group(realm_id, user_id, group_id)
        .await
        .expect("assign user to group");

    let events = harness.events.events.lock().unwrap().clone();
    let has_event = events.iter().any(|event| matches!(event, DomainEvent::UserAssignedToGroup(UserGroupChanged { user_id: uid, group_id: gid }) if *uid == user_id && *gid == group_id));
    assert!(has_event, "expected UserAssignedToGroup event");
}

#[tokio::test]
async fn remove_user_from_group_publishes_event() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    harness.repo.groups.lock().unwrap().insert(
        group_id,
        Group {
            id: group_id,
            realm_id,
            parent_id: None,
            name: "group".to_string(),
            description: None,
            sort_order: 0,
        },
    );

    harness
        .service
        .remove_user_from_group(realm_id, user_id, group_id)
        .await
        .expect("remove user from group");

    let events = harness.events.events.lock().unwrap().clone();
    let has_event = events.iter().any(|event| matches!(event, DomainEvent::UserRemovedFromGroup(UserGroupChanged { user_id: uid, group_id: gid }) if *uid == user_id && *gid == group_id));
    assert!(has_event, "expected UserRemovedFromGroup event");
}

#[tokio::test]
async fn assign_role_to_user_publishes_event() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });

    harness
        .service
        .assign_role_to_user(realm_id, user_id, role_id)
        .await
        .expect("assign role to user");

    let events = harness.events.events.lock().unwrap().clone();
    let has_event = events.iter().any(|event| matches!(event, DomainEvent::UserRoleAssigned(UserRoleChanged { user_id: uid, role_id: rid }) if *uid == user_id && *rid == role_id));
    assert!(has_event, "expected UserRoleAssigned event");
}

#[tokio::test]
async fn remove_role_from_user_publishes_event() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });

    harness
        .service
        .remove_role_from_user(realm_id, user_id, role_id)
        .await
        .expect("remove role from user");

    let events = harness.events.events.lock().unwrap().clone();
    let has_event = events.iter().any(|event| matches!(event, DomainEvent::UserRoleRemoved(UserRoleChanged { user_id: uid, role_id: rid }) if *uid == user_id && *rid == role_id));
    assert!(has_event, "expected UserRoleRemoved event");
}

#[tokio::test]
async fn assign_role_to_user_rejects_cross_realm() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let other_realm = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });

    let result = harness
        .service
        .assign_role_to_user(other_realm, user_id, role_id)
        .await;

    assert!(matches!(result, Err(Error::SecurityViolation(_))));
}

#[tokio::test]
async fn assign_composite_role_publishes_event() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let parent_role_id = Uuid::new_v4();
    let child_role_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: parent_role_id,
        realm_id,
        client_id: None,
        name: "parent".to_string(),
        description: None,
    });
    harness.repo.insert_role(Role {
        id: child_role_id,
        realm_id,
        client_id: None,
        name: "child".to_string(),
        description: None,
    });

    harness
        .service
        .assign_composite_role(realm_id, parent_role_id, child_role_id)
        .await
        .expect("assign composite role");

    let events = harness.events.events.lock().unwrap().clone();
    let has_event = events.iter().any(|event| match event {
        DomainEvent::RoleCompositeChanged(payload) => {
            payload.parent_role_id == parent_role_id
                && payload.child_role_id == child_role_id
                && payload.action == "assigned"
        }
        _ => false,
    });
    assert!(has_event, "expected RoleCompositeChanged assigned event");
}

#[tokio::test]
async fn remove_composite_role_publishes_event() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let parent_role_id = Uuid::new_v4();
    let child_role_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: parent_role_id,
        realm_id,
        client_id: None,
        name: "parent".to_string(),
        description: None,
    });
    harness.repo.insert_role(Role {
        id: child_role_id,
        realm_id,
        client_id: None,
        name: "child".to_string(),
        description: None,
    });

    harness
        .service
        .remove_composite_role(realm_id, parent_role_id, child_role_id)
        .await
        .expect("remove composite role");

    let events = harness.events.events.lock().unwrap().clone();
    let has_event = events.iter().any(|event| match event {
        DomainEvent::RoleCompositeChanged(payload) => {
            payload.parent_role_id == parent_role_id
                && payload.child_role_id == child_role_id
                && payload.action == "removed"
        }
        _ => false,
    });
    assert!(has_event, "expected RoleCompositeChanged removed event");
}

#[tokio::test]
async fn assign_permission_to_role_allows_system_permission_for_realm_role() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });

    harness
        .service
        .assign_permission_to_role(realm_id, role_id, permissions::REALM_READ.to_string())
        .await
        .expect("assign permission");

    let calls = harness
        .repo
        .assign_permission_to_role_calls
        .lock()
        .unwrap()
        .clone();
    assert!(calls.contains(&(role_id, permissions::REALM_READ.to_string())));

    let events = harness.events.events.lock().unwrap().clone();
    let has_event = events.iter().any(|event| match event {
        DomainEvent::RolePermissionChanged(payload) => {
            payload.role_id == role_id
                && payload.permission == permissions::REALM_READ
                && payload.action == "assigned"
        }
        _ => false,
    });
    assert!(has_event, "expected RolePermissionChanged assigned event");
}

#[tokio::test]
async fn revoke_permission_publishes_event_and_calls_repo() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let permission = "app:read".to_string();

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });

    harness
        .service
        .revoke_permission(realm_id, role_id, permission.clone())
        .await
        .expect("revoke permission");

    let calls = harness.repo.remove_permission_calls.lock().unwrap().clone();
    assert!(calls.contains(&(role_id, permission.clone())));

    let events = harness.events.events.lock().unwrap().clone();
    let has_event = events.iter().any(|event| match event {
        DomainEvent::RolePermissionChanged(payload) => {
            payload.role_id == role_id
                && payload.permission == permission
                && payload.action == "revoked"
        }
        _ => false,
    });
    assert!(has_event, "expected RolePermissionChanged revoked event");
}

#[tokio::test]
async fn bulk_update_permissions_add_calls_repo_and_emits_events() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });

    let perms = vec!["app:read".to_string(), "app:write".to_string()];
    for perm in &perms {
        harness
            .service
            .create_custom_permission(
                realm_id,
                CreateCustomPermissionPayload {
                    permission: perm.clone(),
                    name: perm.clone(),
                    description: None,
                    client_id: None,
                },
            )
            .await
            .expect("create custom permission");
    }

    harness
        .service
        .bulk_update_permissions(realm_id, role_id, perms.clone(), "add".to_string())
        .await
        .expect("bulk update add");

    let calls = harness
        .repo
        .bulk_update_permissions_calls
        .lock()
        .unwrap()
        .clone();
    assert!(calls.contains(&BulkUpdateCall {
        role_id,
        permissions: perms.clone(),
        action: "add".to_string(),
    }));

    let events = harness.events.events.lock().unwrap().clone();
    for perm in &perms {
        let has_event = events.iter().any(|event| match event {
            DomainEvent::RolePermissionChanged(payload) => {
                payload.role_id == role_id
                    && payload.permission == *perm
                    && payload.action == "assigned"
            }
            _ => false,
        });
        assert!(has_event, "expected assigned event for {perm}");
    }
}

#[tokio::test]
async fn bulk_update_permissions_remove_calls_repo_and_emits_events() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });

    let perms = vec!["app:read".to_string(), "app:write".to_string()];

    harness
        .service
        .bulk_update_permissions(realm_id, role_id, perms.clone(), "remove".to_string())
        .await
        .expect("bulk update remove");

    let calls = harness
        .repo
        .bulk_update_permissions_calls
        .lock()
        .unwrap()
        .clone();
    assert!(calls.contains(&BulkUpdateCall {
        role_id,
        permissions: perms.clone(),
        action: "remove".to_string(),
    }));

    let events = harness.events.events.lock().unwrap().clone();
    for perm in &perms {
        let has_event = events.iter().any(|event| match event {
            DomainEvent::RolePermissionChanged(payload) => {
                payload.role_id == role_id
                    && payload.permission == *perm
                    && payload.action == "revoked"
            }
            _ => false,
        });
        assert!(has_event, "expected revoked event for {perm}");
    }
}

#[tokio::test]
async fn get_direct_user_ids_for_role_returns_repo_data() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let users = vec![Uuid::new_v4(), Uuid::new_v4()];

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });
    harness
        .repo
        .set_find_direct_user_ids_for_role(role_id, users.clone());

    let result = harness
        .service
        .get_direct_user_ids_for_role(realm_id, role_id)
        .await
        .expect("direct user ids");

    assert_eq!(result, users);
}

#[tokio::test]
async fn get_effective_user_ids_for_role_returns_repo_data() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let users = vec![Uuid::new_v4()];

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });
    harness
        .repo
        .set_find_user_ids_for_role(role_id, users.clone());

    let result = harness
        .service
        .get_effective_user_ids_for_role(realm_id, role_id)
        .await
        .expect("effective user ids");

    assert_eq!(result, users);
}

#[tokio::test]
async fn get_group_member_ids_returns_repo_data() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let users = vec![Uuid::new_v4(), Uuid::new_v4()];

    harness.repo.groups.lock().unwrap().insert(
        group_id,
        Group {
            id: group_id,
            realm_id,
            parent_id: None,
            name: "group".to_string(),
            description: None,
            sort_order: 0,
        },
    );
    harness
        .repo
        .set_find_user_ids_in_group(group_id, users.clone());

    let result = harness
        .service
        .get_group_member_ids(realm_id, group_id)
        .await
        .expect("group member ids");

    assert_eq!(result, users);
}

#[tokio::test]
async fn get_group_role_ids_returns_repo_data() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let roles = vec![Uuid::new_v4()];

    harness.repo.groups.lock().unwrap().insert(
        group_id,
        Group {
            id: group_id,
            realm_id,
            parent_id: None,
            name: "group".to_string(),
            description: None,
            sort_order: 0,
        },
    );
    harness
        .repo
        .set_find_role_ids_for_group(group_id, roles.clone());

    let result = harness
        .service
        .get_group_role_ids(realm_id, group_id)
        .await
        .expect("group role ids");

    assert_eq!(result, roles);
}

#[tokio::test]
async fn get_effective_group_role_ids_returns_repo_data() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let roles = vec![Uuid::new_v4()];

    harness.repo.groups.lock().unwrap().insert(
        group_id,
        Group {
            id: group_id,
            realm_id,
            parent_id: None,
            name: "group".to_string(),
            description: None,
            sort_order: 0,
        },
    );
    harness
        .repo
        .set_find_effective_role_ids_for_group(group_id, roles.clone());

    let result = harness
        .service
        .get_effective_group_role_ids(realm_id, group_id)
        .await
        .expect("effective group role ids");

    assert_eq!(result, roles);
}

#[tokio::test]
async fn get_direct_role_ids_for_user_returns_repo_data() {
    let harness = harness();
    let user_id = Uuid::new_v4();
    let roles = vec![Uuid::new_v4(), Uuid::new_v4()];

    harness
        .repo
        .set_find_direct_role_ids_for_user(user_id, roles.clone());

    let result = harness
        .service
        .get_direct_role_ids_for_user(Uuid::new_v4(), user_id)
        .await
        .expect("direct role ids");

    assert_eq!(result, roles);
}

#[tokio::test]
async fn get_effective_role_ids_for_user_returns_repo_data() {
    let harness = harness();
    let user_id = Uuid::new_v4();
    let roles = vec![Uuid::new_v4()];

    harness
        .repo
        .set_find_effective_role_ids_for_user(user_id, roles.clone());

    let result = harness
        .service
        .get_effective_role_ids_for_user(Uuid::new_v4(), user_id)
        .await
        .expect("effective role ids");

    assert_eq!(result, roles);
}

#[tokio::test]
async fn get_role_composite_ids_returns_repo_data() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let composites = vec![Uuid::new_v4()];

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });
    harness
        .repo
        .set_list_role_composite_ids(role_id, composites.clone());

    let result = harness
        .service
        .get_role_composite_ids(realm_id, role_id)
        .await
        .expect("role composite ids");

    assert_eq!(result, composites);
}

#[tokio::test]
async fn get_effective_role_composite_ids_returns_repo_data() {
    let harness = harness();
    let realm_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();
    let composites = vec![Uuid::new_v4(), Uuid::new_v4()];

    harness.repo.insert_role(Role {
        id: role_id,
        realm_id,
        client_id: None,
        name: "admin".to_string(),
        description: None,
    });
    harness
        .repo
        .set_list_effective_role_composite_ids(role_id, composites.clone());

    let result = harness
        .service
        .get_effective_role_composite_ids(realm_id, role_id)
        .await
        .expect("effective role composite ids");

    assert_eq!(result, composites);
}

#[tokio::test]
async fn get_user_roles_and_groups_returns_repo_data() {
    let harness = harness();
    let user_id = Uuid::new_v4();
    let roles = vec!["admin".to_string(), "viewer".to_string()];
    let groups = vec!["engineering".to_string()];

    harness
        .repo
        .set_find_role_names_for_user(user_id, roles.clone());
    harness
        .repo
        .set_find_group_names_for_user(user_id, groups.clone());

    let result = harness
        .service
        .get_user_roles_and_groups(&user_id)
        .await
        .expect("user roles and groups");

    assert_eq!(result.0, roles);
    assert_eq!(result.1, groups);
}
