use super::AuthService;
use crate::application::rbac_service::RbacService;
use crate::config::AuthConfig;
use crate::constants::DEFAULT_REALM_NAME;
use crate::domain::auth_flow::AuthFlow;
use crate::domain::group::Group;
use crate::domain::pagination::{PageRequest, PageResponse};
use crate::domain::rbac::{
    CustomPermission, GroupMemberFilter, GroupMemberRow, GroupRoleFilter, GroupRoleRow,
    GroupTreeRow, RoleCompositeFilter, RoleCompositeRow, RoleMemberFilter, RoleMemberRow,
    UserRoleFilter, UserRoleRow,
};
use crate::domain::role::{Permission, Role};
use crate::domain::session::RefreshToken;
use crate::domain::user::User;
use crate::error::{Error, Result};
use crate::ports::rbac_repository::RbacRepository;
use crate::ports::realm_repository::RealmRepository;
use crate::ports::session_repository::SessionRepository;
use crate::ports::token_service::{AccessTokenClaims, TokenService};
use crate::ports::transaction_manager::Transaction;
use crate::ports::user_repository::UserRepository;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

fn empty_page<T>() -> PageResponse<T> {
    PageResponse::new(Vec::new(), 0, 1, 10)
}

struct TestRbacRepo;

#[allow(clippy::unused_async)]
#[async_trait]
impl RbacRepository for TestRbacRepo {
    async fn create_role(&self, _role: &Role) -> Result<()> {
        Ok(())
    }

    async fn create_group(&self, _group: &Group) -> Result<()> {
        Ok(())
    }

    async fn assign_role_to_group(&self, _role_id: &Uuid, _group_id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn remove_role_from_group(&self, _role_id: &Uuid, _group_id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn assign_user_to_group(&self, _user_id: &Uuid, _group_id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn remove_user_from_group(&self, _user_id: &Uuid, _group_id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn assign_permission_to_role(
        &self,
        _permission: &Permission,
        _role_id: &Uuid,
    ) -> Result<()> {
        Ok(())
    }

    async fn assign_role_to_user(&self, _user_id: &Uuid, _role_id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn remove_role_from_user(&self, _user_id: &Uuid, _role_id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn find_role_by_name(&self, _realm_id: &Uuid, _name: &str) -> Result<Option<Role>> {
        Ok(None)
    }

    async fn find_group_by_name(&self, _realm_id: &Uuid, _name: &str) -> Result<Option<Group>> {
        Ok(None)
    }

    async fn find_group_by_id(&self, _group_id: &Uuid) -> Result<Option<Group>> {
        Ok(None)
    }

    async fn list_roles(&self, _realm_id: &Uuid, _req: &PageRequest) -> Result<PageResponse<Role>> {
        Ok(empty_page())
    }

    async fn list_client_roles(
        &self,
        _realm_id: &Uuid,
        _client_id: &Uuid,
        _req: &PageRequest,
    ) -> Result<PageResponse<Role>> {
        Ok(empty_page())
    }

    async fn find_role_by_id(&self, _role_id: &Uuid) -> Result<Option<Role>> {
        Ok(None)
    }

    async fn list_groups(
        &self,
        _realm_id: &Uuid,
        _req: &PageRequest,
    ) -> Result<PageResponse<Group>> {
        Ok(empty_page())
    }

    async fn list_group_roots(
        &self,
        _realm_id: &Uuid,
        _req: &PageRequest,
    ) -> Result<PageResponse<GroupTreeRow>> {
        Ok(empty_page())
    }

    async fn list_group_children(
        &self,
        _realm_id: &Uuid,
        _parent_id: &Uuid,
        _req: &PageRequest,
    ) -> Result<PageResponse<GroupTreeRow>> {
        Ok(empty_page())
    }

    async fn list_role_members(
        &self,
        _realm_id: &Uuid,
        _role_id: &Uuid,
        _filter: RoleMemberFilter,
        _req: &PageRequest,
    ) -> Result<PageResponse<RoleMemberRow>> {
        Ok(empty_page())
    }

    async fn list_group_members(
        &self,
        _realm_id: &Uuid,
        _group_id: &Uuid,
        _filter: GroupMemberFilter,
        _req: &PageRequest,
    ) -> Result<PageResponse<GroupMemberRow>> {
        Ok(empty_page())
    }

    async fn list_group_roles(
        &self,
        _realm_id: &Uuid,
        _group_id: &Uuid,
        _filter: GroupRoleFilter,
        _req: &PageRequest,
    ) -> Result<PageResponse<GroupRoleRow>> {
        Ok(empty_page())
    }

    async fn list_user_roles(
        &self,
        _realm_id: &Uuid,
        _user_id: &Uuid,
        _filter: UserRoleFilter,
        _req: &PageRequest,
    ) -> Result<PageResponse<UserRoleRow>> {
        Ok(empty_page())
    }

    async fn list_role_composites(
        &self,
        _realm_id: &Uuid,
        _role_id: &Uuid,
        _client_id: &Option<Uuid>,
        _filter: RoleCompositeFilter,
        _req: &PageRequest,
    ) -> Result<PageResponse<RoleCompositeRow>> {
        Ok(empty_page())
    }

    async fn list_group_ids_by_parent(
        &self,
        _realm_id: &Uuid,
        _parent_id: Option<&Uuid>,
    ) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn list_group_subtree_ids(&self, _realm_id: &Uuid, _root_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn set_group_orders(
        &self,
        _realm_id: &Uuid,
        _parent_id: Option<&Uuid>,
        _ordered_ids: &[Uuid],
    ) -> Result<()> {
        Ok(())
    }

    async fn is_group_descendant(
        &self,
        _realm_id: &Uuid,
        _ancestor_id: &Uuid,
        _candidate_id: &Uuid,
    ) -> Result<bool> {
        Ok(false)
    }

    async fn get_next_group_sort_order(
        &self,
        _realm_id: &Uuid,
        _parent_id: Option<&Uuid>,
    ) -> Result<i64> {
        Ok(1)
    }

    async fn find_user_ids_in_group(&self, _group_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn find_user_ids_in_groups(&self, _group_ids: &[Uuid]) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn find_role_ids_for_group(&self, _group_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn find_effective_role_ids_for_group(&self, _group_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn count_user_ids_in_groups(&self, _group_ids: &[Uuid]) -> Result<i64> {
        Ok(0)
    }

    async fn count_role_ids_in_groups(&self, _group_ids: &[Uuid]) -> Result<i64> {
        Ok(0)
    }

    async fn find_direct_role_ids_for_user(&self, _user_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn find_effective_role_ids_for_user(&self, _user_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn find_role_ids_for_user(&self, _user_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn find_permissions_for_roles(&self, _role_ids: &[Uuid]) -> Result<HashSet<Permission>> {
        Ok(HashSet::new())
    }

    async fn find_user_ids_for_role(&self, _role_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn find_direct_user_ids_for_role(&self, _role_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn list_role_composite_ids(&self, _role_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn list_effective_role_composite_ids(&self, _role_id: &Uuid) -> Result<Vec<Uuid>> {
        Ok(Vec::new())
    }

    async fn get_effective_permissions_for_user(&self, _user_id: &Uuid) -> Result<HashSet<String>> {
        Ok(HashSet::new())
    }

    async fn find_role_names_for_user(&self, _user_id: &Uuid) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    async fn find_group_names_for_user(&self, _user_id: &Uuid) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    async fn delete_role(&self, _role_id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn delete_groups(&self, _group_ids: &[Uuid]) -> Result<()> {
        Ok(())
    }

    async fn update_role(&self, _role: &Role) -> Result<()> {
        Ok(())
    }

    async fn update_group(&self, _group: &Group) -> Result<()> {
        Ok(())
    }

    async fn get_permissions_for_role(&self, _role_id: &Uuid) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    async fn remove_permission(&self, _role_id: &Uuid, _permission: &str) -> Result<()> {
        Ok(())
    }

    async fn bulk_update_permissions(
        &self,
        _role_id: &Uuid,
        _permissions: Vec<String>,
        _action: &str,
    ) -> Result<()> {
        Ok(())
    }

    async fn assign_composite_role(
        &self,
        _parent_role_id: &Uuid,
        _child_role_id: &Uuid,
    ) -> Result<()> {
        Ok(())
    }

    async fn remove_composite_role(
        &self,
        _parent_role_id: &Uuid,
        _child_role_id: &Uuid,
    ) -> Result<()> {
        Ok(())
    }

    async fn is_role_descendant(&self, _ancestor_id: &Uuid, _candidate_id: &Uuid) -> Result<bool> {
        Ok(false)
    }

    async fn create_custom_permission(&self, _permission: &CustomPermission) -> Result<()> {
        Ok(())
    }

    async fn update_custom_permission(&self, _permission: &CustomPermission) -> Result<()> {
        Ok(())
    }

    async fn delete_custom_permission(&self, _permission_id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn find_custom_permission_by_key(
        &self,
        _realm_id: &Uuid,
        _client_id: Option<&Uuid>,
        _permission: &str,
    ) -> Result<Option<CustomPermission>> {
        Ok(None)
    }

    async fn find_custom_permission_by_id(
        &self,
        _realm_id: &Uuid,
        _permission_id: &Uuid,
    ) -> Result<Option<CustomPermission>> {
        Ok(None)
    }

    async fn list_custom_permissions(
        &self,
        _realm_id: &Uuid,
        _client_id: Option<&Uuid>,
    ) -> Result<Vec<CustomPermission>> {
        Ok(Vec::new())
    }

    async fn remove_role_permissions_by_key(&self, _permission: &str) -> Result<()> {
        Ok(())
    }
}

#[derive(Default)]
struct TestUserRepo {
    users: Mutex<HashMap<Uuid, User>>,
}

impl TestUserRepo {
    fn insert(&self, user: User) {
        self.users.lock().unwrap().insert(user.id, user);
    }
}

#[allow(clippy::unused_async)]
#[async_trait]
impl UserRepository for TestUserRepo {
    async fn find_by_username(&self, _realm_id: &Uuid, _username: &str) -> Result<Option<User>> {
        Ok(None)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>> {
        Ok(self.users.lock().unwrap().get(id).cloned())
    }

    async fn save(&self, user: &User) -> Result<()> {
        self.users.lock().unwrap().insert(user.id, user.clone());
        Ok(())
    }

    async fn update(&self, user: &User) -> Result<()> {
        self.users.lock().unwrap().insert(user.id, user.clone());
        Ok(())
    }

    async fn list(&self, _realm_id: &Uuid, _req: &PageRequest) -> Result<PageResponse<User>> {
        Ok(empty_page())
    }
}

#[derive(Default)]
struct TestRealmRepo {
    by_name: Mutex<HashMap<String, crate::domain::realm::Realm>>,
    by_id: Mutex<HashMap<Uuid, crate::domain::realm::Realm>>,
}

impl TestRealmRepo {
    fn insert(&self, realm: crate::domain::realm::Realm) {
        self.by_name
            .lock()
            .unwrap()
            .insert(realm.name.clone(), realm.clone());
        self.by_id.lock().unwrap().insert(realm.id, realm);
    }
}

#[allow(clippy::unused_async)]
#[async_trait]
impl RealmRepository for TestRealmRepo {
    async fn create<'a>(
        &self,
        _realm: &crate::domain::realm::Realm,
        _tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()> {
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<crate::domain::realm::Realm>> {
        Ok(self.by_id.lock().unwrap().get(id).cloned())
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<crate::domain::realm::Realm>> {
        Ok(self.by_name.lock().unwrap().get(name).cloned())
    }

    async fn list_all(&self) -> Result<Vec<crate::domain::realm::Realm>> {
        Ok(self.by_id.lock().unwrap().values().cloned().collect())
    }

    async fn update<'a>(
        &self,
        _realm: &crate::domain::realm::Realm,
        _tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()> {
        Ok(())
    }

    async fn list_flows_by_realm(&self, _realm_id: &Uuid) -> Result<Vec<AuthFlow>> {
        Ok(Vec::new())
    }

    async fn update_flow_binding<'a>(
        &self,
        _realm_id: &Uuid,
        _slot: &str,
        _flow_id: &Uuid,
        _tx: Option<&'a mut dyn Transaction>,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Default)]
struct TestSessionRepo {
    saved: Mutex<Vec<RefreshToken>>,
    stored: Mutex<HashMap<Uuid, RefreshToken>>,
    deleted: Mutex<Vec<Uuid>>,
}

impl TestSessionRepo {
    fn insert(&self, token: RefreshToken) {
        self.stored.lock().unwrap().insert(token.id, token);
    }

    fn saved_tokens(&self) -> Vec<RefreshToken> {
        self.saved.lock().unwrap().clone()
    }

    fn deleted_tokens(&self) -> Vec<Uuid> {
        self.deleted.lock().unwrap().clone()
    }
}

#[allow(clippy::unused_async)]
#[async_trait]
impl SessionRepository for TestSessionRepo {
    async fn save(&self, token: &RefreshToken) -> Result<()> {
        self.saved.lock().unwrap().push(token.clone());
        self.stored.lock().unwrap().insert(token.id, token.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<RefreshToken>> {
        Ok(self.stored.lock().unwrap().get(id).cloned())
    }

    async fn delete_by_id(&self, id: &Uuid) -> Result<()> {
        self.stored.lock().unwrap().remove(id);
        self.deleted.lock().unwrap().push(*id);
        Ok(())
    }

    async fn list(
        &self,
        _realm_id: &Uuid,
        _req: &PageRequest,
    ) -> Result<PageResponse<RefreshToken>> {
        Ok(empty_page())
    }
}

#[derive(Default)]
struct TestTokenService {
    claims: Mutex<Option<AccessTokenClaims>>,
    access_tokens: Mutex<Vec<Uuid>>,
    id_tokens: Mutex<Vec<String>>,
}

impl TestTokenService {
    fn set_claims(&self, claims: AccessTokenClaims) {
        *self.claims.lock().unwrap() = Some(claims);
    }

    fn id_token_calls(&self) -> usize {
        self.id_tokens.lock().unwrap().len()
    }
}

#[allow(clippy::unused_async)]
#[async_trait]
impl TokenService for TestTokenService {
    async fn create_access_token(
        &self,
        _user: &User,
        session_id: Uuid,
        _permissions: &HashSet<String>,
        _roles: &[String],
        _groups: &[String],
    ) -> Result<String> {
        self.access_tokens.lock().unwrap().push(session_id);
        Ok("access-token".to_string())
    }

    async fn create_id_token(
        &self,
        _user: &User,
        client_id: &str,
        _groups: &[String],
    ) -> Result<String> {
        self.id_tokens.lock().unwrap().push(client_id.to_string());
        Ok("id-token".to_string())
    }

    async fn validate_access_token(&self, _token: &str) -> Result<AccessTokenClaims> {
        if let Some(claims) = self.claims.lock().unwrap().as_ref() {
            Ok(AccessTokenClaims {
                sub: claims.sub,
                sid: claims.sid,
                perms: claims.perms.clone(),
                roles: claims.roles.clone(),
                groups: claims.groups.clone(),
                exp: claims.exp,
            })
        } else {
            Err(Error::InvalidCredentials)
        }
    }

    fn get_key_id(&self) -> &str {
        "kid"
    }

    fn get_jwks(&self) -> Result<serde_json::Value> {
        Ok(json!({}))
    }
}

fn build_service(
    user_repo: Arc<TestUserRepo>,
    realm_repo: Arc<TestRealmRepo>,
    session_repo: Arc<TestSessionRepo>,
    token_service: Arc<TestTokenService>,
) -> AuthService {
    let rbac_repo = Arc::new(TestRbacRepo);
    let cache = Arc::new(crate::adapters::cache::moka_cache::MokaCacheService::default());
    let event_bus = Arc::new(crate::adapters::eventing::in_memory_bus::InMemoryEventBus::default());
    let rbac_service = Arc::new(RbacService::new(rbac_repo, cache, event_bus));
    let settings = AuthConfig {
        jwt_secret: "secret".to_string(),
        jwt_key_id: "kid".to_string(),
        issuer: "http://issuer".to_string(),
        access_token_ttl_secs: 60,
        refresh_token_ttl_secs: 120,
    };

    AuthService::new(
        user_repo,
        realm_repo,
        session_repo,
        token_service,
        rbac_service,
        settings,
    )
}

fn base_realm() -> crate::domain::realm::Realm {
    crate::domain::realm::Realm {
        id: Uuid::new_v4(),
        name: DEFAULT_REALM_NAME.to_string(),
        access_token_ttl_secs: 300,
        refresh_token_ttl_secs: 900,
        browser_flow_id: None,
        registration_flow_id: None,
        direct_grant_flow_id: None,
        reset_credentials_flow_id: None,
    }
}

#[tokio::test]
async fn create_session_returns_tokens_and_refresh_token() {
    let user_repo = Arc::new(TestUserRepo::default());
    let user = User {
        id: Uuid::new_v4(),
        realm_id: Uuid::new_v4(),
        username: "user".to_string(),
        hashed_password: "hash".to_string(),
    };
    user_repo.insert(user.clone());

    let realm_repo = Arc::new(TestRealmRepo::default());
    let mut realm = base_realm();
    realm.id = user.realm_id;
    realm_repo.insert(realm);

    let session_repo = Arc::new(TestSessionRepo::default());
    let token_service = Arc::new(TestTokenService::default());

    let service = build_service(
        user_repo,
        realm_repo,
        session_repo.clone(),
        token_service.clone(),
    );

    let (login, refresh) = service
        .create_session(
            &user,
            Some("client".to_string()),
            Some("127.0.0.1".to_string()),
            Some("agent".to_string()),
        )
        .await
        .expect("create_session failed");

    assert_eq!(login.access_token, "access-token");
    assert_eq!(login.id_token, Some("id-token".to_string()));
    assert_eq!(refresh.user_id, user.id);
    assert_eq!(refresh.client_id, Some("client".to_string()));
    assert_eq!(session_repo.saved_tokens().len(), 1);
    assert_eq!(token_service.id_token_calls(), 1);
}

#[tokio::test]
async fn create_session_without_client_id_skips_id_token() {
    let user_repo = Arc::new(TestUserRepo::default());
    let user = User {
        id: Uuid::new_v4(),
        realm_id: Uuid::new_v4(),
        username: "user".to_string(),
        hashed_password: "hash".to_string(),
    };
    user_repo.insert(user.clone());

    let realm_repo = Arc::new(TestRealmRepo::default());
    let mut realm = base_realm();
    realm.id = user.realm_id;
    realm_repo.insert(realm);

    let session_repo = Arc::new(TestSessionRepo::default());
    let token_service = Arc::new(TestTokenService::default());

    let service = build_service(user_repo, realm_repo, session_repo, token_service.clone());

    let (login, _) = service
        .create_session(&user, None, None, None)
        .await
        .expect("create_session failed");

    assert_eq!(login.id_token, None);
    assert_eq!(token_service.id_token_calls(), 0);
}

#[tokio::test]
async fn create_session_errors_when_realm_missing() {
    let user_repo = Arc::new(TestUserRepo::default());
    let user = User {
        id: Uuid::new_v4(),
        realm_id: Uuid::new_v4(),
        username: "user".to_string(),
        hashed_password: "hash".to_string(),
    };
    user_repo.insert(user.clone());

    let realm_repo = Arc::new(TestRealmRepo::default());
    let session_repo = Arc::new(TestSessionRepo::default());
    let token_service = Arc::new(TestTokenService::default());

    let service = build_service(user_repo, realm_repo, session_repo, token_service);

    match service.create_session(&user, None, None, None).await {
        Err(Error::RealmNotFound(name)) => assert_eq!(name, DEFAULT_REALM_NAME),
        Err(other) => panic!("unexpected error: {:?}", other),
        Ok(_) => panic!("expected error"),
    }
}

#[tokio::test]
async fn validate_token_and_get_user_rejects_revoked_sessions() {
    let user_repo = Arc::new(TestUserRepo::default());
    let realm_repo = Arc::new(TestRealmRepo::default());
    let session_repo = Arc::new(TestSessionRepo::default());
    let token_service = Arc::new(TestTokenService::default());

    token_service.set_claims(AccessTokenClaims {
        sub: Uuid::new_v4(),
        sid: Uuid::new_v4(),
        perms: HashSet::new(),
        roles: Vec::new(),
        groups: Vec::new(),
        exp: 0,
    });

    let service = build_service(user_repo, realm_repo, session_repo, token_service);

    let err = service
        .validate_token_and_get_user("token")
        .await
        .unwrap_err();

    match err {
        Error::SessionRevoked => {}
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn validate_token_and_get_user_returns_user() {
    let user_repo = Arc::new(TestUserRepo::default());
    let user = User {
        id: Uuid::new_v4(),
        realm_id: Uuid::new_v4(),
        username: "user".to_string(),
        hashed_password: "hash".to_string(),
    };
    user_repo.insert(user.clone());

    let realm_repo = Arc::new(TestRealmRepo::default());
    let session_repo = Arc::new(TestSessionRepo::default());
    let token_service = Arc::new(TestTokenService::default());

    let sid = Uuid::new_v4();
    session_repo.insert(RefreshToken {
        id: sid,
        user_id: user.id,
        realm_id: user.realm_id,
        client_id: None,
        expires_at: Utc::now() + Duration::seconds(60),
        ip_address: None,
        user_agent: None,
        created_at: Utc::now(),
        last_used_at: Utc::now(),
    });

    token_service.set_claims(AccessTokenClaims {
        sub: user.id,
        sid,
        perms: HashSet::new(),
        roles: Vec::new(),
        groups: Vec::new(),
        exp: 0,
    });

    let service = build_service(user_repo, realm_repo, session_repo, token_service);
    let fetched = service
        .validate_token_and_get_user("token")
        .await
        .expect("expected user");

    assert_eq!(fetched.id, user.id);
}

#[tokio::test]
async fn refresh_session_errors_when_token_missing() {
    let service = build_service(
        Arc::new(TestUserRepo::default()),
        Arc::new(TestRealmRepo::default()),
        Arc::new(TestSessionRepo::default()),
        Arc::new(TestTokenService::default()),
    );

    match service.refresh_session(Uuid::new_v4()).await {
        Err(Error::InvalidRefreshToken) => {}
        Err(other) => panic!("unexpected error: {:?}", other),
        Ok(_) => panic!("expected error"),
    }
}

#[tokio::test]
async fn refresh_session_errors_when_user_missing() {
    let session_repo = Arc::new(TestSessionRepo::default());
    let refresh_id = Uuid::new_v4();
    session_repo.insert(RefreshToken {
        id: refresh_id,
        user_id: Uuid::new_v4(),
        realm_id: Uuid::new_v4(),
        client_id: None,
        expires_at: Utc::now() + Duration::seconds(60),
        ip_address: None,
        user_agent: None,
        created_at: Utc::now(),
        last_used_at: Utc::now(),
    });

    let service = build_service(
        Arc::new(TestUserRepo::default()),
        Arc::new(TestRealmRepo::default()),
        session_repo,
        Arc::new(TestTokenService::default()),
    );

    match service.refresh_session(refresh_id).await {
        Err(Error::UserNotFound) => {}
        Err(other) => panic!("unexpected error: {:?}", other),
        Ok(_) => panic!("expected error"),
    }
}

#[tokio::test]
async fn refresh_session_errors_when_realm_missing() {
    let user_repo = Arc::new(TestUserRepo::default());
    let user = User {
        id: Uuid::new_v4(),
        realm_id: Uuid::new_v4(),
        username: "user".to_string(),
        hashed_password: "hash".to_string(),
    };
    user_repo.insert(user.clone());

    let session_repo = Arc::new(TestSessionRepo::default());
    let refresh_id = Uuid::new_v4();
    session_repo.insert(RefreshToken {
        id: refresh_id,
        user_id: user.id,
        realm_id: user.realm_id,
        client_id: None,
        expires_at: Utc::now() + Duration::seconds(60),
        ip_address: None,
        user_agent: None,
        created_at: Utc::now(),
        last_used_at: Utc::now(),
    });

    let service = build_service(
        user_repo,
        Arc::new(TestRealmRepo::default()),
        session_repo,
        Arc::new(TestTokenService::default()),
    );

    match service.refresh_session(refresh_id).await {
        Err(Error::RealmNotFound(_)) => {}
        Err(other) => panic!("unexpected error: {:?}", other),
        Ok(_) => panic!("expected error"),
    }
}

#[tokio::test]
async fn refresh_session_rotates_tokens_and_issues_id_token() {
    let user_repo = Arc::new(TestUserRepo::default());
    let user = User {
        id: Uuid::new_v4(),
        realm_id: Uuid::new_v4(),
        username: "user".to_string(),
        hashed_password: "hash".to_string(),
    };
    user_repo.insert(user.clone());

    let realm_repo = Arc::new(TestRealmRepo::default());
    let mut realm = base_realm();
    realm.id = user.realm_id;
    realm_repo.insert(realm);

    let session_repo = Arc::new(TestSessionRepo::default());
    let refresh_id = Uuid::new_v4();
    session_repo.insert(RefreshToken {
        id: refresh_id,
        user_id: user.id,
        realm_id: user.realm_id,
        client_id: Some("client".to_string()),
        expires_at: Utc::now() + Duration::seconds(60),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("agent".to_string()),
        created_at: Utc::now(),
        last_used_at: Utc::now(),
    });

    let token_service = Arc::new(TestTokenService::default());

    let service = build_service(
        user_repo,
        realm_repo,
        session_repo.clone(),
        token_service.clone(),
    );

    let (login, refresh) = service
        .refresh_session(refresh_id)
        .await
        .expect("refresh_session failed");

    assert_eq!(login.access_token, "access-token");
    assert_eq!(login.id_token, Some("id-token".to_string()));
    assert_eq!(refresh.client_id, Some("client".to_string()));
    assert_eq!(session_repo.deleted_tokens(), vec![refresh_id]);
    assert_eq!(session_repo.saved_tokens().len(), 1);
    assert_eq!(token_service.id_token_calls(), 1);
}
