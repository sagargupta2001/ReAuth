use crate::adapters::cache::moka_cache::MokaCacheService;
use crate::adapters::crypto::jwt_service::JwtService;
use crate::adapters::eventing::in_memory_bus::InMemoryEventBus;
use crate::adapters::persistence::sqlite_rbac_repository::SqliteRbacRepository;
use crate::adapters::persistence::sqlite_realm_repository::SqliteRealmRepository;
use crate::adapters::persistence::sqlite_session_repository::SqliteSessionRepository;
use crate::adapters::SqliteUserRepository;
use crate::application::auth_service::AuthService;
use crate::application::rbac_service::RbacService;
use crate::application::realm_service::RealmService;
use crate::application::user_service::UserService;
use crate::config::Settings;
use std::sync::Arc;

pub fn initialize_services(
    settings: &Settings,
    user_repo: &Arc<SqliteUserRepository>,
    rbac_repo: &Arc<SqliteRbacRepository>,
    realm_repo: &Arc<SqliteRealmRepository>,
    session_repo: &Arc<SqliteSessionRepository>,
    cache: &Arc<MokaCacheService>,
    event_bus: &Arc<InMemoryEventBus>,
) -> (
    Arc<UserService>,
    Arc<RbacService>,
    Arc<RealmService>,
    Arc<AuthService>,
) {
    let user_service = Arc::new(UserService::new(user_repo.clone(), event_bus.clone()));
    let rbac_service = Arc::new(RbacService::new(
        rbac_repo.clone(),
        cache.clone(),
        event_bus.clone(),
    ));
    let realm_service = Arc::new(RealmService::new(realm_repo.clone()));
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        realm_repo.clone(),
        session_repo.clone(),
        Arc::new(JwtService::new(settings.auth.clone())),
        rbac_service.clone(),
        settings.auth.clone(),
    ));

    (user_service, rbac_service, realm_service, auth_service)
}
