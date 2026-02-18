use super::OidcService;
use crate::application::auth_service::AuthService;
use crate::application::rbac_service::RbacService;
use crate::config::AuthConfig;
use crate::constants::DEFAULT_REALM_NAME;
use crate::domain::auth_flow::AuthFlow;
use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::ExecutionPlan;
use crate::domain::group::Group;
use crate::domain::oidc::{AuthCode, OidcClient, OidcContext, OidcRequest};
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
use crate::ports::auth_session_repository::AuthSessionRepository;
use crate::ports::flow_store::FlowStore;
use crate::ports::oidc_repository::OidcRepository;
use crate::ports::rbac_repository::RbacRepository;
use crate::ports::realm_repository::RealmRepository;
use crate::ports::session_repository::SessionRepository;
use crate::ports::token_service::{AccessTokenClaims, TokenService};
use crate::ports::transaction_manager::Transaction;
use crate::ports::user_repository::UserRepository;
use async_trait::async_trait;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{Duration, Utc};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

fn empty_page<T>() -> PageResponse<T> {
    PageResponse::new(Vec::new(), 0, 1, 10)
}

fn pkce_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

#[derive(Default)]
struct TestAuthSessionRepo {
    created: Mutex<Vec<AuthenticationSession>>,
}

impl TestAuthSessionRepo {
    fn created_sessions(&self) -> Vec<AuthenticationSession> {
        self.created.lock().unwrap().clone()
    }
}

#[async_trait]
impl AuthSessionRepository for TestAuthSessionRepo {
    async fn create(&self, session: &AuthenticationSession) -> Result<()> {
        self.created.lock().unwrap().push(session.clone());
        Ok(())
    }

    async fn find_by_id(&self, _id: &Uuid) -> Result<Option<AuthenticationSession>> {
        Ok(None)
    }

    async fn update(&self, _session: &AuthenticationSession) -> Result<()> {
        Ok(())
    }

    async fn delete(&self, _id: &Uuid) -> Result<()> {
        Ok(())
    }
}

#[derive(Default)]
struct TestFlowStore {
    active_versions: Mutex<HashMap<Uuid, Option<String>>>,
    latest_versions: Mutex<HashMap<Uuid, Option<String>>>,
    versions: Mutex<HashMap<String, String>>,
}

impl TestFlowStore {
    fn set_active_version(&self, flow_id: Uuid, version_id: Option<String>) {
        self.active_versions
            .lock()
            .unwrap()
            .insert(flow_id, version_id);
    }

    fn set_latest_version(&self, flow_id: Uuid, version_id: Option<String>) {
        self.latest_versions
            .lock()
            .unwrap()
            .insert(flow_id, version_id);
    }

    fn set_version(&self, version_id: &str, execution_artifact: &str) {
        self.versions
            .lock()
            .unwrap()
            .insert(version_id.to_string(), execution_artifact.to_string());
    }

    fn build_version(&self, id: &str) -> Option<crate::domain::flow::models::FlowVersion> {
        self.versions.lock().unwrap().get(id).map(|artifact| {
            crate::domain::flow::models::FlowVersion {
                id: id.to_string(),
                flow_id: Uuid::new_v4().to_string(),
                version_number: 1,
                execution_artifact: artifact.clone(),
                graph_json: json!({}).to_string(),
                checksum: "checksum".to_string(),
                created_at: Utc::now(),
            }
        })
    }
}

#[allow(clippy::unused_async)]
#[async_trait]
impl FlowStore for TestFlowStore {
    async fn create_draft(&self, _draft: &crate::domain::flow::models::FlowDraft) -> Result<()> {
        Ok(())
    }

    async fn update_draft(&self, _draft: &crate::domain::flow::models::FlowDraft) -> Result<()> {
        Ok(())
    }

    async fn get_draft_by_id(
        &self,
        _id: &Uuid,
    ) -> Result<Option<crate::domain::flow::models::FlowDraft>> {
        Ok(None)
    }

    async fn list_drafts(
        &self,
        _realm_id: &Uuid,
        _req: &PageRequest,
    ) -> Result<PageResponse<crate::domain::flow::models::FlowDraft>> {
        Ok(empty_page())
    }

    async fn list_all_drafts(
        &self,
        _realm_id: &Uuid,
    ) -> Result<Vec<crate::domain::flow::models::FlowDraft>> {
        Ok(Vec::new())
    }

    async fn delete_draft(&self, _id: &Uuid) -> Result<()> {
        Ok(())
    }

    async fn create_version(
        &self,
        _version: &crate::domain::flow::models::FlowVersion,
    ) -> Result<()> {
        Ok(())
    }

    async fn get_version(
        &self,
        id: &Uuid,
    ) -> Result<Option<crate::domain::flow::models::FlowVersion>> {
        Ok(self.build_version(&id.to_string()))
    }

    async fn list_versions(
        &self,
        _flow_id: &Uuid,
        _req: &PageRequest,
    ) -> Result<PageResponse<crate::domain::flow::models::FlowVersion>> {
        Ok(empty_page())
    }

    async fn set_deployment(
        &self,
        _deployment: &crate::domain::flow::models::FlowDeployment,
    ) -> Result<()> {
        Ok(())
    }

    async fn get_deployment(
        &self,
        _realm_id: &Uuid,
        _flow_type: &str,
    ) -> Result<Option<crate::domain::flow::models::FlowDeployment>> {
        Ok(None)
    }

    async fn get_latest_version_number(&self, _flow_id: &Uuid) -> Result<Option<i32>> {
        Ok(None)
    }

    async fn get_latest_version(
        &self,
        flow_id: &Uuid,
    ) -> Result<Option<crate::domain::flow::models::FlowVersion>> {
        let version_id = self
            .latest_versions
            .lock()
            .unwrap()
            .get(flow_id)
            .cloned()
            .unwrap_or(None);
        Ok(version_id.and_then(|id| self.build_version(&id)))
    }

    async fn get_deployed_version_number(
        &self,
        _realm_id: &Uuid,
        _flow_type: &str,
        _flow_id: &Uuid,
    ) -> Result<Option<i32>> {
        Ok(None)
    }

    async fn get_version_by_number(
        &self,
        _flow_id: &Uuid,
        _version_number: i32,
    ) -> Result<Option<crate::domain::flow::models::FlowVersion>> {
        Ok(None)
    }

    async fn get_active_version(
        &self,
        flow_id: &Uuid,
    ) -> Result<Option<crate::domain::flow::models::FlowVersion>> {
        let version_id = self
            .active_versions
            .lock()
            .unwrap()
            .get(flow_id)
            .cloned()
            .unwrap_or(None);
        Ok(version_id.and_then(|id| self.build_version(&id)))
    }
}

#[derive(Default)]
struct TestOidcRepo {
    client: Mutex<Option<OidcClient>>,
    auth_codes: Mutex<HashMap<String, AuthCode>>,
    deleted_codes: Mutex<Vec<String>>,
}

impl TestOidcRepo {
    fn set_client(&self, client: Option<OidcClient>) {
        *self.client.lock().unwrap() = client;
    }

    fn insert_auth_code(&self, auth_code: AuthCode) {
        self.auth_codes
            .lock()
            .unwrap()
            .insert(auth_code.code.clone(), auth_code);
    }

    fn deleted_codes(&self) -> Vec<String> {
        self.deleted_codes.lock().unwrap().clone()
    }
}

#[allow(clippy::unused_async)]
#[async_trait]
impl OidcRepository for TestOidcRepo {
    async fn find_client_by_id(
        &self,
        realm_id: &Uuid,
        client_id: &str,
    ) -> Result<Option<OidcClient>> {
        let client = self.client.lock().unwrap().clone();
        Ok(client.filter(|c| c.realm_id == *realm_id && c.client_id == client_id))
    }

    async fn create_client(&self, _client: &OidcClient) -> Result<()> {
        Ok(())
    }

    async fn find_clients_by_realm(
        &self,
        _realm_id: &Uuid,
        _req: &PageRequest,
    ) -> Result<PageResponse<OidcClient>> {
        Ok(empty_page())
    }

    async fn find_client_by_uuid(&self, _id: &Uuid) -> Result<Option<OidcClient>> {
        Ok(None)
    }

    async fn update_client(&self, _client: &OidcClient) -> Result<()> {
        Ok(())
    }

    async fn save_auth_code(&self, code: &AuthCode) -> Result<()> {
        self.auth_codes
            .lock()
            .unwrap()
            .insert(code.code.clone(), code.clone());
        Ok(())
    }

    async fn find_auth_code_by_code(&self, code: &str) -> Result<Option<AuthCode>> {
        Ok(self.auth_codes.lock().unwrap().get(code).cloned())
    }

    async fn delete_auth_code(&self, code: &str) -> Result<()> {
        self.auth_codes.lock().unwrap().remove(code);
        self.deleted_codes.lock().unwrap().push(code.to_string());
        Ok(())
    }

    async fn is_origin_allowed(&self, _origin: &str) -> Result<bool> {
        Ok(true)
    }
}

#[derive(Default)]
struct TestRealmRepo {
    realm: Mutex<Option<crate::domain::realm::Realm>>,
}

impl TestRealmRepo {
    fn set_realm(&self, realm: Option<crate::domain::realm::Realm>) {
        *self.realm.lock().unwrap() = realm;
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

    async fn find_by_id(&self, _id: &Uuid) -> Result<Option<crate::domain::realm::Realm>> {
        Ok(self.realm.lock().unwrap().clone())
    }

    async fn find_by_name(&self, _name: &str) -> Result<Option<crate::domain::realm::Realm>> {
        Ok(self.realm.lock().unwrap().clone())
    }

    async fn list_all(&self) -> Result<Vec<crate::domain::realm::Realm>> {
        Ok(Vec::new())
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
struct TestSessionRepo {
    saved: Mutex<Vec<RefreshToken>>,
    stored: Mutex<HashMap<Uuid, RefreshToken>>,
}

impl TestSessionRepo {
    fn saved_tokens(&self) -> Vec<RefreshToken> {
        self.saved.lock().unwrap().clone()
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
    access_tokens: Mutex<Vec<Uuid>>,
    id_tokens: Mutex<Vec<String>>,
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
        Err(Error::InvalidCredentials)
    }

    fn get_key_id(&self) -> &str {
        "kid"
    }

    fn get_jwks(&self) -> Result<serde_json::Value> {
        Ok(json!({}))
    }
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

fn build_auth_service(
    user_repo: Arc<TestUserRepo>,
    realm_repo: Arc<TestRealmRepo>,
    session_repo: Arc<TestSessionRepo>,
    token_service: Arc<TestTokenService>,
) -> Arc<AuthService> {
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

    Arc::new(AuthService::new(
        user_repo,
        realm_repo,
        session_repo,
        token_service,
        rbac_service,
        settings,
    ))
}

fn build_service(
    oidc_repo: Arc<TestOidcRepo>,
    auth_session_repo: Arc<TestAuthSessionRepo>,
    flow_store: Arc<TestFlowStore>,
    realm_repo: Arc<TestRealmRepo>,
    user_repo: Arc<TestUserRepo>,
    session_repo: Arc<TestSessionRepo>,
    token_service: Arc<TestTokenService>,
) -> OidcService {
    let auth_service = build_auth_service(
        user_repo.clone(),
        realm_repo.clone(),
        session_repo.clone(),
        token_service.clone(),
    );

    OidcService::new(
        oidc_repo,
        user_repo,
        auth_service,
        token_service,
        auth_session_repo,
        flow_store,
        realm_repo,
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

fn build_oidc_request(client_id: &str, redirect_uri: &str) -> OidcRequest {
    OidcRequest {
        client_id: client_id.to_string(),
        redirect_uri: redirect_uri.to_string(),
        response_type: "code".to_string(),
        scope: Some("openid".to_string()),
        state: Some("state".to_string()),
        nonce: Some("nonce".to_string()),
        code_challenge: Some("challenge".to_string()),
        code_challenge_method: Some("S256".to_string()),
    }
}

fn build_client(realm_id: Uuid, client_id: &str, redirect_uris: Vec<&str>) -> OidcClient {
    OidcClient {
        id: Uuid::new_v4(),
        realm_id,
        client_id: client_id.to_string(),
        client_secret: None,
        redirect_uris: serde_json::to_string(&redirect_uris).unwrap(),
        scopes: serde_json::to_string(&vec!["openid"]).unwrap(),
        web_origins: serde_json::to_string(&vec!["http://localhost"]).unwrap(),
        managed_by_config: false,
    }
}

#[tokio::test]
async fn validate_client_rejects_missing_client() {
    let oidc_repo = Arc::new(TestOidcRepo::default());
    let auth_session_repo = Arc::new(TestAuthSessionRepo::default());
    let flow_store = Arc::new(TestFlowStore::default());
    let realm_repo = Arc::new(TestRealmRepo::default());
    let user_repo = Arc::new(TestUserRepo::default());
    let session_repo = Arc::new(TestSessionRepo::default());
    let token_service = Arc::new(TestTokenService::default());

    let service = build_service(
        oidc_repo,
        auth_session_repo,
        flow_store,
        realm_repo,
        user_repo,
        session_repo,
        token_service,
    );

    let err = service
        .validate_client(&Uuid::new_v4(), "client", "http://localhost")
        .await
        .unwrap_err();

    match err {
        Error::OidcClientNotFound(_) => {}
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn validate_client_rejects_invalid_redirect_uri() {
    let realm_id = Uuid::new_v4();
    let oidc_repo = Arc::new(TestOidcRepo::default());
    oidc_repo.set_client(Some(build_client(
        realm_id,
        "client",
        vec!["https://allowed"],
    )));

    let service = build_service(
        oidc_repo,
        Arc::new(TestAuthSessionRepo::default()),
        Arc::new(TestFlowStore::default()),
        Arc::new(TestRealmRepo::default()),
        Arc::new(TestUserRepo::default()),
        Arc::new(TestSessionRepo::default()),
        Arc::new(TestTokenService::default()),
    );

    let err = service
        .validate_client(&realm_id, "client", "https://bad")
        .await
        .unwrap_err();

    match err {
        Error::OidcInvalidRedirect(_) => {}
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn validate_client_rejects_invalid_redirect_json() {
    let realm_id = Uuid::new_v4();
    let oidc_repo = Arc::new(TestOidcRepo::default());
    let mut client = build_client(realm_id, "client", vec!["http://localhost"]);
    client.redirect_uris = "not-json".to_string();
    oidc_repo.set_client(Some(client));

    let service = build_service(
        oidc_repo,
        Arc::new(TestAuthSessionRepo::default()),
        Arc::new(TestFlowStore::default()),
        Arc::new(TestRealmRepo::default()),
        Arc::new(TestUserRepo::default()),
        Arc::new(TestSessionRepo::default()),
        Arc::new(TestTokenService::default()),
    );

    let err = service
        .validate_client(&realm_id, "client", "http://localhost")
        .await
        .unwrap_err();

    match err {
        Error::Unexpected(message) => {
            assert!(format!("{:?}", message).contains("redirect_uris"));
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn initiate_browser_login_requires_realm() {
    let realm_id = Uuid::new_v4();
    let oidc_repo = Arc::new(TestOidcRepo::default());
    oidc_repo.set_client(Some(build_client(
        realm_id,
        "client",
        vec!["http://localhost"],
    )));

    let service = build_service(
        oidc_repo,
        Arc::new(TestAuthSessionRepo::default()),
        Arc::new(TestFlowStore::default()),
        Arc::new(TestRealmRepo::default()),
        Arc::new(TestUserRepo::default()),
        Arc::new(TestSessionRepo::default()),
        Arc::new(TestTokenService::default()),
    );

    let err = service
        .initiate_browser_login(realm_id, build_oidc_request("client", "http://localhost"))
        .await
        .unwrap_err();

    match err {
        Error::NotFound(message) => assert!(message.contains("Realm not found")),
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn initiate_browser_login_requires_flow_binding() {
    let realm_id = Uuid::new_v4();
    let oidc_repo = Arc::new(TestOidcRepo::default());
    oidc_repo.set_client(Some(build_client(
        realm_id,
        "client",
        vec!["http://localhost"],
    )));

    let realm_repo = Arc::new(TestRealmRepo::default());
    let mut realm = base_realm();
    realm.id = realm_id;
    realm_repo.set_realm(Some(realm));

    let service = build_service(
        oidc_repo,
        Arc::new(TestAuthSessionRepo::default()),
        Arc::new(TestFlowStore::default()),
        realm_repo,
        Arc::new(TestUserRepo::default()),
        Arc::new(TestSessionRepo::default()),
        Arc::new(TestTokenService::default()),
    );

    let err = service
        .initiate_browser_login(realm_id, build_oidc_request("client", "http://localhost"))
        .await
        .unwrap_err();

    match err {
        Error::Validation(message) => assert!(message.contains("browser flow configured")),
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn initiate_browser_login_requires_flow_version() {
    let realm_id = Uuid::new_v4();
    let flow_id = Uuid::new_v4();

    let oidc_repo = Arc::new(TestOidcRepo::default());
    oidc_repo.set_client(Some(build_client(
        realm_id,
        "client",
        vec!["http://localhost"],
    )));

    let realm_repo = Arc::new(TestRealmRepo::default());
    let mut realm = base_realm();
    realm.id = realm_id;
    realm.browser_flow_id = Some(flow_id.to_string());
    realm_repo.set_realm(Some(realm));

    let service = build_service(
        oidc_repo,
        Arc::new(TestAuthSessionRepo::default()),
        Arc::new(TestFlowStore::default()),
        realm_repo,
        Arc::new(TestUserRepo::default()),
        Arc::new(TestSessionRepo::default()),
        Arc::new(TestTokenService::default()),
    );

    let err = service
        .initiate_browser_login(realm_id, build_oidc_request("client", "http://localhost"))
        .await
        .unwrap_err();

    match err {
        Error::NotFound(message) => assert!(message.contains("Flow version not found")),
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn initiate_browser_login_rejects_corrupt_execution_artifact() {
    let realm_id = Uuid::new_v4();
    let flow_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();

    let oidc_repo = Arc::new(TestOidcRepo::default());
    oidc_repo.set_client(Some(build_client(
        realm_id,
        "client",
        vec!["http://localhost"],
    )));

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.set_active_version(flow_id, Some(version_id.to_string()));
    flow_store.set_version(&version_id.to_string(), "not-json");

    let realm_repo = Arc::new(TestRealmRepo::default());
    let mut realm = base_realm();
    realm.id = realm_id;
    realm.browser_flow_id = Some(flow_id.to_string());
    realm_repo.set_realm(Some(realm));

    let service = build_service(
        oidc_repo,
        Arc::new(TestAuthSessionRepo::default()),
        flow_store,
        realm_repo,
        Arc::new(TestUserRepo::default()),
        Arc::new(TestSessionRepo::default()),
        Arc::new(TestTokenService::default()),
    );

    let err = service
        .initiate_browser_login(realm_id, build_oidc_request("client", "http://localhost"))
        .await
        .unwrap_err();

    match err {
        Error::Unexpected(message) => assert!(format!("{:?}", message).contains("Corrupt")),
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn initiate_browser_login_creates_session() {
    let realm_id = Uuid::new_v4();
    let flow_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();

    let oidc_repo = Arc::new(TestOidcRepo::default());
    oidc_repo.set_client(Some(build_client(
        realm_id,
        "client",
        vec!["http://localhost"],
    )));

    let flow_store = Arc::new(TestFlowStore::default());
    flow_store.set_latest_version(flow_id, Some(version_id.to_string()));
    flow_store.set_version(
        &version_id.to_string(),
        &serde_json::to_string(&ExecutionPlan {
            start_node_id: "start".to_string(),
            nodes: HashMap::new(),
        })
        .unwrap(),
    );

    let realm_repo = Arc::new(TestRealmRepo::default());
    let mut realm = base_realm();
    realm.id = realm_id;
    realm.browser_flow_id = Some(flow_id.to_string());
    realm_repo.set_realm(Some(realm));

    let auth_session_repo = Arc::new(TestAuthSessionRepo::default());

    let service = build_service(
        oidc_repo,
        auth_session_repo.clone(),
        flow_store,
        realm_repo,
        Arc::new(TestUserRepo::default()),
        Arc::new(TestSessionRepo::default()),
        Arc::new(TestTokenService::default()),
    );

    let session = service
        .initiate_browser_login(realm_id, build_oidc_request("client", "http://localhost"))
        .await
        .expect("expected session");

    assert_eq!(session.flow_version_id, version_id);
    let created = auth_session_repo.created_sessions();
    assert_eq!(created.len(), 1);
    let context = created[0].context.get("oidc").cloned();
    assert!(context
        .and_then(|value| serde_json::from_value::<OidcContext>(value).ok())
        .is_some());
}

#[tokio::test]
async fn exchange_code_for_token_rejects_missing_code() {
    let service = build_service(
        Arc::new(TestOidcRepo::default()),
        Arc::new(TestAuthSessionRepo::default()),
        Arc::new(TestFlowStore::default()),
        Arc::new(TestRealmRepo::default()),
        Arc::new(TestUserRepo::default()),
        Arc::new(TestSessionRepo::default()),
        Arc::new(TestTokenService::default()),
    );

    match service
        .exchange_code_for_token("missing", "verifier", None, None)
        .await
    {
        Err(Error::OidcInvalidCode) => {}
        Err(other) => panic!("unexpected error: {:?}", other),
        Ok(_) => panic!("expected error"),
    }
}

#[tokio::test]
async fn exchange_code_for_token_rejects_invalid_pkce() {
    let oidc_repo = Arc::new(TestOidcRepo::default());
    oidc_repo.insert_auth_code(AuthCode {
        code: "code".to_string(),
        user_id: Uuid::new_v4(),
        client_id: "client".to_string(),
        redirect_uri: "http://localhost".to_string(),
        nonce: None,
        code_challenge: Some("expected".to_string()),
        code_challenge_method: "S256".to_string(),
        expires_at: Utc::now() + Duration::seconds(60),
    });

    let service = build_service(
        oidc_repo,
        Arc::new(TestAuthSessionRepo::default()),
        Arc::new(TestFlowStore::default()),
        Arc::new(TestRealmRepo::default()),
        Arc::new(TestUserRepo::default()),
        Arc::new(TestSessionRepo::default()),
        Arc::new(TestTokenService::default()),
    );

    match service
        .exchange_code_for_token("code", "bad", None, None)
        .await
    {
        Err(Error::OidcInvalidCode) => {}
        Err(other) => panic!("unexpected error: {:?}", other),
        Ok(_) => panic!("expected error"),
    }
}

#[tokio::test]
async fn exchange_code_for_token_requires_user() {
    let verifier = "verifier";
    let oidc_repo = Arc::new(TestOidcRepo::default());
    oidc_repo.insert_auth_code(AuthCode {
        code: "code".to_string(),
        user_id: Uuid::new_v4(),
        client_id: "client".to_string(),
        redirect_uri: "http://localhost".to_string(),
        nonce: None,
        code_challenge: Some(pkce_challenge(verifier)),
        code_challenge_method: "S256".to_string(),
        expires_at: Utc::now() + Duration::seconds(60),
    });

    let service = build_service(
        oidc_repo,
        Arc::new(TestAuthSessionRepo::default()),
        Arc::new(TestFlowStore::default()),
        Arc::new(TestRealmRepo::default()),
        Arc::new(TestUserRepo::default()),
        Arc::new(TestSessionRepo::default()),
        Arc::new(TestTokenService::default()),
    );

    match service
        .exchange_code_for_token("code", verifier, None, None)
        .await
    {
        Err(Error::UserNotFound) => {}
        Err(other) => panic!("unexpected error: {:?}", other),
        Ok(_) => panic!("expected error"),
    }
}

#[tokio::test]
async fn exchange_code_for_token_returns_tokens_and_deletes_code() {
    let verifier = "verifier";
    let user_id = Uuid::new_v4();

    let oidc_repo = Arc::new(TestOidcRepo::default());
    oidc_repo.insert_auth_code(AuthCode {
        code: "code".to_string(),
        user_id,
        client_id: "client".to_string(),
        redirect_uri: "http://localhost".to_string(),
        nonce: None,
        code_challenge: Some(pkce_challenge(verifier)),
        code_challenge_method: "S256".to_string(),
        expires_at: Utc::now() + Duration::seconds(60),
    });

    let user_repo = Arc::new(TestUserRepo::default());
    user_repo.insert(User {
        id: user_id,
        realm_id: Uuid::new_v4(),
        username: "user".to_string(),
        hashed_password: "hash".to_string(),
    });

    let realm_repo = Arc::new(TestRealmRepo::default());
    let realm = base_realm();
    realm_repo.set_realm(Some(realm));

    let session_repo = Arc::new(TestSessionRepo::default());
    let token_service = Arc::new(TestTokenService::default());

    let service = build_service(
        oidc_repo.clone(),
        Arc::new(TestAuthSessionRepo::default()),
        Arc::new(TestFlowStore::default()),
        realm_repo,
        user_repo,
        session_repo.clone(),
        token_service,
    );

    let (token_response, refresh_token) = service
        .exchange_code_for_token("code", verifier, None, None)
        .await
        .expect("expected success");

    assert_eq!(token_response.access_token, "access-token");
    assert_eq!(token_response.id_token, "id-token");
    assert_eq!(token_response.token_type, "Bearer");
    assert!(token_response.expires_in > 0);

    assert_eq!(refresh_token.user_id, user_id);
    assert_eq!(session_repo.saved_tokens().len(), 1);
    assert_eq!(oidc_repo.deleted_codes(), vec!["code".to_string()]);
}
