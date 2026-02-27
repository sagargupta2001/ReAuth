use crate::adapters::auth::register_builtins;
use crate::adapters::observability::telemetry_store::TelemetryDatabase;
use crate::application::audit_service::AuditService;
use crate::application::flow_executor::FlowExecutor;
use crate::application::flow_manager::FlowManager;
use crate::application::flow_service::FlowService;
use crate::application::node_registry::NodeRegistryService;
use crate::application::oidc_service::OidcService;
use crate::application::runtime_registry::RuntimeRegistry;
use crate::application::webhook_service::WebhookService;
use crate::ports::transaction_manager::TransactionManager;
use crate::{
    adapters::{cache::moka_cache::MokaCacheService, crypto::jwt_service::JwtService},
    application::{
        auth_service::AuthService, rbac_service::RbacService, realm_service::RealmService,
        user_service::UserService,
    },
    bootstrap::repositories::Repositories,
    config::Settings,
    ports::event_bus::EventPublisher,
    ports::outbox_repository::OutboxRepository,
};
use std::sync::Arc;

/// A struct to hold all initialized application services.
pub struct Services {
    pub user_service: Arc<UserService>,
    pub rbac_service: Arc<RbacService>,
    pub realm_service: Arc<RealmService>,
    pub auth_service: Arc<AuthService>,
    pub audit_service: Arc<AuditService>,
    pub webhook_service: Arc<WebhookService>,
    // Removed Legacy FlowEngine
    pub oidc_service: Arc<OidcService>,
    pub flow_service: Arc<FlowService>,
    pub flow_manager: Arc<FlowManager>,
    pub node_registry: Arc<NodeRegistryService>,
    pub flow_executor: Arc<FlowExecutor>,
}

pub struct ServiceInitContext<'a> {
    pub settings: &'a Settings,
    pub repos: &'a Repositories,
    pub cache: &'a Arc<MokaCacheService>,
    pub event_publisher: Arc<dyn EventPublisher>,
    pub outbox_repo: Arc<dyn OutboxRepository>,
    pub token_service: &'a Arc<JwtService>,
    pub telemetry_db: &'a TelemetryDatabase,
    pub tx_manager: &'a Arc<dyn TransactionManager>,
}

pub fn initialize_services(ctx: ServiceInitContext<'_>) -> Services {
    let ServiceInitContext {
        settings,
        repos,
        cache,
        event_publisher,
        outbox_repo,
        token_service,
        telemetry_db,
        tx_manager,
    } = ctx;
    // 1. Foundation Services
    let user_service = Arc::new(UserService::new(
        repos.user_repo.clone(),
        event_publisher.clone(),
        outbox_repo.clone(),
        tx_manager.clone(),
    ));
    let audit_service = Arc::new(AuditService::new(repos.audit_repo.clone()));
    let webhook_service = Arc::new(WebhookService::new(
        repos.webhook_repo.clone(),
        tx_manager.clone(),
        telemetry_db.clone(),
    ));
    let rbac_service = Arc::new(RbacService::new(
        repos.rbac_repo.clone(),
        cache.clone(),
        event_publisher.clone(),
        outbox_repo.clone(),
        tx_manager.clone(),
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

    // 2. Runtime Registry (The Brain)
    let mut registry_impl = RuntimeRegistry::new();

    // Register all nodes (Workers + Definitions)
    // This connects PasswordAuthenticator -> "core.auth.password"
    register_builtins(
        &mut registry_impl,
        repos.user_repo.clone(),
        repos.realm_repo.clone(),
        repos.login_attempt_repo.clone(),
        settings.auth.lockout_threshold,
        settings.auth.lockout_duration_secs,
        repos.session_repo.clone(),
    );

    // Wrap in Arc for shared use
    let runtime_registry = Arc::new(registry_impl);

    // 3. Executor & Manager (The Heart)
    let flow_executor = Arc::new(FlowExecutor::new(
        repos.auth_session_repo.clone(),
        repos.flow_store.clone(),
        runtime_registry.clone(),
    ));

    let flow_manager = Arc::new(FlowManager::new(
        repos.flow_store.clone(),
        repos.flow_repo.clone(),
        repos.realm_repo.clone(),
        runtime_registry.clone(),
    ));

    // 4. OIDC & API Services
    let oidc_service = Arc::new(OidcService::new(
        repos.oidc_repo.clone(),
        repos.user_repo.clone(),
        auth_service.clone(),
        token_service.clone(),
        repos.auth_session_repo.clone(),
        repos.flow_store.clone(),
        repos.realm_repo.clone(),
    ));

    let node_registry = Arc::new(NodeRegistryService::new());

    Services {
        user_service,
        rbac_service,
        realm_service,
        auth_service,
        audit_service,
        webhook_service,
        oidc_service,
        flow_service,
        flow_manager,
        node_registry,
        flow_executor,
    }
}
