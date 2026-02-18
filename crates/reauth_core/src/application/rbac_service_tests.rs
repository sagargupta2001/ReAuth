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

#[derive(Default)]
struct TestRbacRepo {
    roles: Mutex<HashMap<Uuid, Role>>,
    groups: Mutex<HashMap<Uuid, Group>>,
    group_children_by_parent: Mutex<HashMap<Option<Uuid>, Vec<Uuid>>>,
    group_subtree_by_root: Mutex<HashMap<Uuid, Vec<Uuid>>>,
    group_descendant: Mutex<bool>,
    next_group_sort_order: Mutex<i64>,
    custom_permissions: Mutex<HashMap<Uuid, CustomPermission>>,
    custom_permissions_by_key: Mutex<HashMap<String, Uuid>>,
    role_descendant: Mutex<bool>,
    effective_permissions: Mutex<HashMap<Uuid, HashSet<String>>>,
    find_user_ids_for_role: Mutex<HashMap<Uuid, Vec<Uuid>>>,
    find_user_ids_in_groups_result: Mutex<Vec<Uuid>>,
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

    fn set_find_user_ids_in_groups_result(&self, user_ids: Vec<Uuid>) {
        *self.find_user_ids_in_groups_result.lock().unwrap() = user_ids;
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
        self.set_group_orders_calls.lock().unwrap().push(
            SetGroupOrdersCall {
                parent_id: parent_id.copied(),
                ordered_ids: ordered_ids.to_vec(),
            },
        );
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
        Ok(Vec::new())
    }

    async fn find_user_ids_in_groups(&self, group_ids: &[Uuid]) -> Result<Vec<Uuid>> {
        Ok(self.find_user_ids_in_groups_result.lock().unwrap().clone())
    }

    async fn find_role_ids_for_group(&self, group_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn find_effective_role_ids_for_group(&self, group_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn count_user_ids_in_groups(&self, group_ids: &[Uuid]) -> Result<i64> {
        Ok(0)
    }

    async fn count_role_ids_in_groups(&self, group_ids: &[Uuid]) -> Result<i64> {
        Ok(0)
    }

    async fn find_direct_role_ids_for_user(&self, user_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn find_effective_role_ids_for_user(&self, user_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
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
        Ok(Vec::new())
    }

    async fn list_role_composite_ids(&self, role_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn list_effective_role_composite_ids(&self, role_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
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
        Ok(Vec::new())
    }

    async fn find_group_names_for_user(&self, user_id: &Uuid) -> Result<Vec<String>> {
        Ok(Vec::new())
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
        Ok(())
    }

    async fn bulk_update_permissions(
        &self,
        role_id: &Uuid,
        permissions: Vec<String>,
        action: &str,
    ) -> Result<()> {
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
