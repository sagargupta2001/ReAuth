use crate::adapters::auth::{register_builtins, BuiltinAuthContext};
use crate::application::audit_service::AuditService;
use crate::application::email_delivery_service::EmailDeliveryService;
use crate::application::flow_executor::FlowExecutor;
use crate::application::flow_manager::FlowManager;
use crate::application::flow_service::FlowService;
use crate::application::harbor::client_provider::ClientHarborProvider;
use crate::application::harbor::flow_provider::FlowHarborProvider;
use crate::application::harbor::provider::HarborRegistry;
use crate::application::harbor::realm_provider::RealmHarborProvider;
use crate::application::harbor::role_provider::RoleHarborProvider;
use crate::application::harbor::runner::TokioHarborJobRunner;
use crate::application::harbor::service::HarborService;
use crate::application::harbor::theme_provider::ThemeHarborProvider;
use crate::application::harbor::user_provider::UserHarborProvider;
use crate::application::invitation_service::InvitationService;
use crate::application::node_registry::NodeRegistryService;
use crate::application::oidc_service::OidcService;
use crate::application::passkey_analytics_service::PasskeyAnalyticsService;
use crate::application::passkey_assertion_service::PasskeyAssertionService;
use crate::application::realm_email_settings_service::RealmEmailSettingsService;
use crate::application::realm_passkey_settings_service::RealmPasskeySettingsService;
use crate::application::realm_recovery_settings_service::RealmRecoverySettingsService;
use crate::application::realm_security_headers_service::RealmSecurityHeadersService;
use crate::application::runtime_registry::RuntimeRegistry;
use crate::application::secret_service::SecretService;
use crate::application::theme_service::ThemeResolverService;
use crate::application::user_credentials_service::UserCredentialsService;
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
    pub user_credentials_service: Arc<UserCredentialsService>,
    pub rbac_service: Arc<RbacService>,
    pub realm_service: Arc<RealmService>,
    pub realm_email_settings_service: Arc<RealmEmailSettingsService>,
    pub realm_passkey_settings_service: Arc<RealmPasskeySettingsService>,
    pub realm_recovery_settings_service: Arc<RealmRecoverySettingsService>,
    pub realm_security_headers_service: Arc<RealmSecurityHeadersService>,
    pub passkey_assertion_service: Arc<PasskeyAssertionService>,
    pub passkey_analytics_service: Arc<PasskeyAnalyticsService>,
    pub email_delivery_service: Arc<EmailDeliveryService>,
    pub invitation_service: Arc<InvitationService>,
    pub auth_service: Arc<AuthService>,
    pub audit_service: Arc<AuditService>,
    pub webhook_service: Arc<WebhookService>,
    pub theme_service: Arc<ThemeResolverService>,
    pub harbor_service: Arc<HarborService>,
    pub oidc_service: Arc<OidcService>,
    pub flow_service: Arc<FlowService>,
    pub flow_manager: Arc<FlowManager>,
    pub node_registry: Arc<NodeRegistryService>,
    pub flow_executor: Arc<FlowExecutor>,
}

use crate::ports::telemetry_repository::TelemetryRepository;

use crate::ports::http_client::HttpDeliveryClient;

pub struct ServiceInitContext<'a> {
    pub settings: &'a Settings,
    pub repos: &'a Repositories,
    pub cache: &'a Arc<MokaCacheService>,
    pub event_publisher: Arc<dyn EventPublisher>,
    pub outbox_repo: Arc<dyn OutboxRepository>,
    pub token_service: &'a Arc<JwtService>,
    pub telemetry_repo: Arc<dyn TelemetryRepository>,
    pub tx_manager: &'a Arc<dyn TransactionManager>,
    pub http_client: Arc<dyn HttpDeliveryClient>,
}

pub fn initialize_services(ctx: ServiceInitContext<'_>) -> Services {
    let ServiceInitContext {
        settings,
        repos,
        cache,
        event_publisher,
        outbox_repo,
        token_service,
        telemetry_repo,
        tx_manager,
        http_client,
    } = ctx;
    // 1. Foundation Services
    let user_service = Arc::new(UserService::new(
        repos.user_repo.clone(),
        event_publisher.clone(),
        outbox_repo.clone(),
        tx_manager.clone(),
    ));
    let user_credentials_service = Arc::new(UserCredentialsService::new(
        user_service.clone(),
        repos.passkey_credential_repo.clone(),
        repos.realm_passkey_settings_repo.clone(),
    ));
    let audit_service = Arc::new(AuditService::new(repos.audit_repo.clone()));
    let webhook_service = Arc::new(WebhookService::new(
        repos.webhook_repo.clone(),
        tx_manager.clone(),
        telemetry_repo.clone(),
        http_client.clone(),
    ));
    let theme_service = Arc::new(ThemeResolverService::new(
        repos.theme_repo.clone(),
        tx_manager.clone(),
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
        theme_service.clone(),
        tx_manager.clone(),
    ));

    let realm_email_settings_service = Arc::new(RealmEmailSettingsService::new(
        repos.realm_repo.clone(),
        repos.realm_email_settings_repo.clone(),
    ));

    let realm_passkey_settings_service = Arc::new(RealmPasskeySettingsService::new(
        repos.realm_repo.clone(),
        repos.realm_passkey_settings_repo.clone(),
    ));

    let realm_recovery_settings_service = Arc::new(RealmRecoverySettingsService::new(
        repos.realm_repo.clone(),
        repos.realm_recovery_settings_repo.clone(),
    ));

    let realm_security_headers_service = Arc::new(RealmSecurityHeadersService::new(
        repos.realm_repo.clone(),
        repos.realm_security_headers_repo.clone(),
    ));

    let email_delivery_service = Arc::new(EmailDeliveryService::new(
        repos.realm_repo.clone(),
        repos.realm_email_settings_repo.clone(),
        repos.realm_recovery_settings_repo.clone(),
        settings.clone(),
    ));

    let auth_service = Arc::new(AuthService::new(
        repos.user_repo.clone(),
        repos.realm_repo.clone(),
        repos.session_repo.clone(),
        token_service.clone(),
        rbac_service.clone(),
        settings.auth.clone(),
    ));

    let secret_service = Arc::new(SecretService::from_settings(settings));

    let passkey_assertion_service = Arc::new(PasskeyAssertionService::new(
        repos.auth_session_repo.clone(),
        repos.realm_repo.clone(),
        repos.user_repo.clone(),
        repos.passkey_challenge_repo.clone(),
        repos.passkey_credential_repo.clone(),
        repos.realm_passkey_settings_repo.clone(),
        audit_service.clone(),
        settings.clone(),
    ));

    let passkey_analytics_service = Arc::new(PasskeyAnalyticsService::new(
        repos.realm_repo.clone(),
        repos.audit_repo.clone(),
        repos.passkey_credential_repo.clone(),
        repos.passkey_challenge_repo.clone(),
    ));
    // 2. Runtime Registry (The Brain)
    let mut registry_impl = RuntimeRegistry::new();

    // Register all nodes (Workers + Definitions)
    // This connects PasswordAuthenticator -> "core.auth.password"
    register_builtins(
        &mut registry_impl,
        BuiltinAuthContext {
            user_service: user_service.clone(),
            user_repo: repos.user_repo.clone(),
            realm_repo: repos.realm_repo.clone(),
            rbac_service: rbac_service.clone(),
            login_attempt_repo: repos.login_attempt_repo.clone(),
            lockout_threshold: settings.auth.lockout_threshold,
            lockout_duration_secs: settings.auth.lockout_duration_secs,
            session_repo: repos.session_repo.clone(),
            flow_store: repos.flow_store.clone(),
            action_repo: repos.auth_session_action_repo.clone(),
            recovery_attempt_repo: repos.recovery_attempt_repo.clone(),
            audit_service: audit_service.clone(),
            recovery_settings_repo: repos.realm_recovery_settings_repo.clone(),
            passkey_settings_repo: repos.realm_passkey_settings_repo.clone(),
        },
    );

    // Wrap in Arc for shared use
    let runtime_registry = Arc::new(registry_impl);

    // 3. Node Registry
    let node_registry = Arc::new(NodeRegistryService::new(runtime_registry.clone()));

    // 4. Executor & Manager (The Heart)
    let flow_executor = Arc::new(FlowExecutor::new(
        repos.auth_session_repo.clone(),
        repos.flow_store.clone(),
        runtime_registry.clone(),
        repos.auth_session_action_repo.clone(),
        Some(email_delivery_service.clone()),
        Some(audit_service.clone()),
    ));

    let publish_validator = Arc::new(
        crate::application::flow_publish_validator::UiBindingPublishValidator::new(
            theme_service.clone(),
            node_registry.clone(),
            repos.realm_passkey_settings_repo.clone(),
            settings.clone(),
        ),
    );

    let flow_manager = Arc::new(FlowManager::new(
        repos.flow_store.clone(),
        repos.flow_repo.clone(),
        repos.realm_repo.clone(),
        runtime_registry.clone(),
        publish_validator,
        node_registry.clone(),
    ));

    let invitation_service = Arc::new(InvitationService::new(
        repos.invitation_repo.clone(),
        repos.realm_repo.clone(),
        repos.auth_session_repo.clone(),
        repos.flow_store.clone(),
        flow_executor.clone(),
        user_service.clone(),
    ));

    // 5. OIDC & API Services
    let oidc_service = Arc::new(OidcService::new(
        repos.oidc_repo.clone(),
        repos.user_repo.clone(),
        auth_service.clone(),
        token_service.clone(),
        secret_service,
        repos.auth_session_repo.clone(),
        repos.flow_store.clone(),
        repos.realm_repo.clone(),
    ));

    let mut harbor_registry = HarborRegistry::new();
    harbor_registry.register(Arc::new(ThemeHarborProvider::new(theme_service.clone())));
    harbor_registry.register(Arc::new(ClientHarborProvider::new(oidc_service.clone())));
    harbor_registry.register(Arc::new(FlowHarborProvider::new(flow_manager.clone())));
    harbor_registry.register(Arc::new(RealmHarborProvider::new(
        realm_service.clone(),
        flow_manager.clone(),
    )));
    harbor_registry.register(Arc::new(UserHarborProvider::new(
        repos.user_repo.clone(),
        repos.rbac_repo.clone(),
        oidc_service.clone(),
    )));
    harbor_registry.register(Arc::new(RoleHarborProvider::new(
        repos.rbac_repo.clone(),
        oidc_service.clone(),
    )));
    let harbor_job_runner = Arc::new(TokioHarborJobRunner);
    let harbor_service = Arc::new(HarborService::new(
        harbor_registry,
        theme_service.clone(),
        oidc_service.clone(),
        flow_service.clone(),
        flow_manager.clone(),
        rbac_service.clone(),
        user_service.clone(),
        tx_manager.clone(),
        repos.harbor_job_repo.clone(),
        repos.harbor_job_conflict_repo.clone(),
        harbor_job_runner,
    ));

    Services {
        user_service,
        user_credentials_service,
        rbac_service,
        realm_service,
        realm_email_settings_service,
        realm_passkey_settings_service,
        realm_recovery_settings_service,
        realm_security_headers_service,
        passkey_assertion_service,
        passkey_analytics_service,
        email_delivery_service,
        invitation_service,
        auth_service,
        audit_service,
        webhook_service,
        theme_service,
        harbor_service,
        oidc_service,
        flow_service,
        flow_manager,
        node_registry,
        flow_executor,
    }
}
