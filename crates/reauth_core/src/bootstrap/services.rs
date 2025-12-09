use crate::application::flow_manager::FlowManager;
use crate::application::flow_service::FlowService;
use crate::application::node_registry::NodeRegistryService;
use crate::application::oidc_service::OidcService;
use crate::ports::transaction_manager::TransactionManager;
use crate::{
    adapters::{
        auth::password_authenticator::PasswordAuthenticator, cache::moka_cache::MokaCacheService,
        crypto::jwt_service::JwtService, eventing::in_memory_bus::InMemoryEventBus,
    },
    application::{
        auth_service::AuthService,
        flow_engine::{AuthenticatorRegistry, FlowEngine},
        rbac_service::RbacService,
        realm_service::RealmService,
        user_service::UserService,
    },
    bootstrap::repositories::Repositories,
    config::Settings,
    ports::authenticator::Authenticator,
};
use std::{collections::HashMap, sync::Arc};

/// A struct to hold all initialized application services.
pub struct Services {
    pub user_service: Arc<UserService>,
    pub rbac_service: Arc<RbacService>,
    pub realm_service: Arc<RealmService>,
    pub auth_service: Arc<AuthService>,
    pub flow_engine: Arc<FlowEngine>,
    pub oidc_service: Arc<OidcService>,
    pub flow_service: Arc<FlowService>,
    pub flow_manager: Arc<FlowManager>,
    pub node_registry: Arc<NodeRegistryService>,
}

pub fn initialize_services(
    settings: &Settings,
    repos: &Repositories,
    cache: &Arc<MokaCacheService>,
    event_bus: &Arc<InMemoryEventBus>,
    token_service: &Arc<JwtService>,
    tx_manager: &Arc<dyn TransactionManager>,
) -> Services {
    let user_service = Arc::new(UserService::new(repos.user_repo.clone(), event_bus.clone()));
    let rbac_service = Arc::new(RbacService::new(
        repos.rbac_repo.clone(),
        cache.clone(),
        event_bus.clone(),
    ));
    let flow_service = Arc::new(FlowService::new(repos.flow_repo.clone()));

    let realm_service = Arc::new(RealmService::new(
        repos.realm_repo.clone(),
        flow_service.clone(),
        tx_manager.clone(),
    ));
    let auth_service = Arc::new(AuthService::new(
        repos.user_repo.clone(),
        repos.realm_repo.clone(),
        repos.session_repo.clone(),
        token_service.clone(),
        rbac_service.clone(),
        settings.auth.clone(),
    ));
    let oidc_service = Arc::new(OidcService::new(
        repos.oidc_repo.clone(),
        repos.user_repo.clone(),
        auth_service.clone(),
        token_service.clone(),
    ));

    // Build the registry for the Flow Engine
    let mut authenticator_map = HashMap::<String, Arc<dyn Authenticator>>::new();

    let password_auth = Arc::new(PasswordAuthenticator::new(repos.user_repo.clone()));
    authenticator_map.insert(password_auth.name().to_string(), password_auth);
    // TODO: Load plugin authenticators and add them to the map
    let authenticator_registry = Arc::new(AuthenticatorRegistry::new(authenticator_map));

    let flow_engine = Arc::new(FlowEngine::new(
        authenticator_registry,
        repos.flow_repo.clone(),
        repos.realm_repo.clone(),
        auth_service.clone(),
        repos.user_repo.clone(),
    ));

    let node_registry = Arc::new(NodeRegistryService::new());

    let flow_manager = Arc::new(FlowManager::new(
        repos.flow_store.clone(),
        repos.realm_repo.clone(),
    ));

    Services {
        user_service,
        rbac_service,
        realm_service,
        auth_service,
        flow_engine,
        oidc_service,
        flow_service,
        flow_manager,
        node_registry,
    }
}
