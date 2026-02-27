use crate::adapters::persistence::connection::Database;
use crate::adapters::persistence::sqlite_audit_repository::SqliteAuditRepository;
use crate::adapters::persistence::sqlite_auth_session_action_repository::SqliteAuthSessionActionRepository;
use crate::adapters::persistence::sqlite_auth_session_repository::SqliteAuthSessionRepository;
use crate::adapters::persistence::sqlite_flow_store::SqliteFlowStore;
use crate::adapters::persistence::sqlite_login_attempt_repository::SqliteLoginAttemptRepository;
use crate::adapters::persistence::sqlite_oidc_repository::SqliteOidcRepository;
use crate::adapters::persistence::sqlite_outbox_repository::SqliteOutboxRepository;
use crate::adapters::persistence::sqlite_webhook_repository::SqliteWebhookRepository;
use crate::ports::audit_repository::AuditRepository;
use crate::ports::auth_session_action_repository::AuthSessionActionRepository;
use crate::ports::auth_session_repository::AuthSessionRepository;
use crate::ports::flow_store::FlowStore;
use crate::ports::oidc_repository::OidcRepository;
use crate::ports::outbox_repository::OutboxRepository;
use crate::ports::webhook_repository::WebhookRepository;
use crate::{
    adapters::persistence::{
        sqlite_flow_repository::SqliteFlowRepository, sqlite_rbac_repository::SqliteRbacRepository,
        sqlite_realm_repository::SqliteRealmRepository,
        sqlite_session_repository::SqliteSessionRepository,
        sqlite_user_repository::SqliteUserRepository,
    },
    ports::{
        // 2. Import traits
        flow_repository::FlowRepository,
        login_attempt_repository::LoginAttemptRepository,
        rbac_repository::RbacRepository,
        realm_repository::RealmRepository,
        session_repository::SessionRepository,
        user_repository::UserRepository,
    },
};
use std::sync::Arc;

/// A struct to hold all initialized repositories.
pub struct Repositories {
    pub user_repo: Arc<dyn UserRepository>,
    pub rbac_repo: Arc<dyn RbacRepository>,
    pub realm_repo: Arc<dyn RealmRepository>,
    pub login_attempt_repo: Arc<dyn LoginAttemptRepository>,
    pub session_repo: Arc<dyn SessionRepository>,
    pub flow_repo: Arc<dyn FlowRepository>,
    pub oidc_repo: Arc<dyn OidcRepository>,
    pub flow_store: Arc<dyn FlowStore>,
    pub auth_session_repo: Arc<dyn AuthSessionRepository>,
    pub auth_session_action_repo: Arc<dyn AuthSessionActionRepository>,
    pub audit_repo: Arc<dyn AuditRepository>,
    pub outbox_repo: Arc<dyn OutboxRepository>,
    pub webhook_repo: Arc<dyn WebhookRepository>,
}

pub fn initialize_repositories(db_pool: &Database) -> Repositories {
    // We instantiate the concrete types but return them as `Arc<dyn Trait>`
    // to enforce the hexagonal architecture.
    let user_repo = Arc::new(SqliteUserRepository::new(db_pool.clone()));
    let rbac_repo = Arc::new(SqliteRbacRepository::new(db_pool.clone()));
    let realm_repo = Arc::new(SqliteRealmRepository::new(db_pool.clone()));
    let login_attempt_repo = Arc::new(SqliteLoginAttemptRepository::new(db_pool.clone()));
    let session_repo = Arc::new(SqliteSessionRepository::new(db_pool.clone()));
    let flow_repo = Arc::new(SqliteFlowRepository::new(db_pool.clone()));
    let oidc_repo = Arc::new(SqliteOidcRepository::new(db_pool.clone()));
    let flow_store = Arc::new(SqliteFlowStore::new(db_pool.clone()));
    let auth_session_repo = Arc::new(SqliteAuthSessionRepository::new(db_pool.clone()));
    let auth_session_action_repo =
        Arc::new(SqliteAuthSessionActionRepository::new(db_pool.clone()));
    let audit_repo = Arc::new(SqliteAuditRepository::new(db_pool.clone()));
    let outbox_repo = Arc::new(SqliteOutboxRepository::new(db_pool.clone()));
    let webhook_repo = Arc::new(SqliteWebhookRepository::new(db_pool.clone()));

    Repositories {
        user_repo,
        rbac_repo,
        realm_repo,
        login_attempt_repo,
        session_repo,
        flow_repo,
        oidc_repo,
        flow_store,
        auth_session_repo,
        auth_session_action_repo,
        audit_repo,
        outbox_repo,
        webhook_repo,
    }
}
