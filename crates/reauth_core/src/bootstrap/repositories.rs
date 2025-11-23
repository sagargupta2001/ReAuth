use crate::adapters::persistence::connection::Database;
use crate::adapters::persistence::sqlite_oidc_repository::SqliteOidcRepository;
use crate::ports::oidc_repository::OidcRepository;
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
    pub session_repo: Arc<dyn SessionRepository>,
    pub flow_repo: Arc<dyn FlowRepository>,
    pub oidc_repo: Arc<dyn OidcRepository>,
}

pub fn initialize_repositories(db_pool: &Database) -> Repositories {
    // We instantiate the concrete types but return them as `Arc<dyn Trait>`
    // to enforce the hexagonal architecture.
    let user_repo = Arc::new(SqliteUserRepository::new(db_pool.clone()));
    let rbac_repo = Arc::new(SqliteRbacRepository::new(db_pool.clone()));
    let realm_repo = Arc::new(SqliteRealmRepository::new(db_pool.clone()));
    let session_repo = Arc::new(SqliteSessionRepository::new(db_pool.clone()));
    let flow_repo = Arc::new(SqliteFlowRepository::new(db_pool.clone()));
    let oidc_repo = Arc::new(SqliteOidcRepository::new(db_pool.clone()));

    Repositories {
        user_repo,
        rbac_repo,
        realm_repo,
        session_repo,
        flow_repo,
        oidc_repo,
    }
}
