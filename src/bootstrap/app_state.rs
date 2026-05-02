use std::sync::Arc;

use crate::application::delivery_replay_service::DeliveryReplayService;
use crate::application::email_delivery_service::EmailDeliveryService;
use crate::application::flow_executor::FlowExecutor;
use crate::application::flow_manager::FlowManager;
use crate::application::flow_service::FlowService;
use crate::application::harbor::HarborService;
use crate::application::metrics_service::MetricsService;
use crate::application::node_registry::NodeRegistryService;
use crate::application::oidc_service::OidcService;
use crate::application::passkey_analytics_service::PasskeyAnalyticsService;
use crate::application::passkey_assertion_service::PasskeyAssertionService;
use crate::application::realm_email_settings_service::RealmEmailSettingsService;
use crate::application::realm_passkey_settings_service::RealmPasskeySettingsService;
use crate::application::realm_recovery_settings_service::RealmRecoverySettingsService;
use crate::application::realm_security_headers_service::RealmSecurityHeadersService;
use crate::application::theme_service::ThemeResolverService;
use crate::application::webhook_service::WebhookService;
use crate::application::{
    audit_service::AuditService, auth_service::AuthService, rbac_service::RbacService,
    realm_service::RealmService, telemetry_service::TelemetryService,
    user_credentials_service::UserCredentialsService, user_service::UserService,
};
use crate::config::Settings;
use crate::domain::log::LogSubscriber;
use crate::ports::auth_session_repository::AuthSessionRepository;
use crate::ports::cache_service::CacheService;
use crate::ports::flow_store::FlowStore;
use crate::ports::session_repository::SessionRepository;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct SetupState {
    pub required: bool,
    pub token: Option<String>,
}

impl SetupState {
    pub fn pending(token: String) -> Self {
        Self {
            required: true,
            token: Some(token),
        }
    }

    pub fn sealed() -> Self {
        Self {
            required: false,
            token: None,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<RwLock<Settings>>,
    pub setup_state: Arc<RwLock<SetupState>>,

    // Services
    pub user_service: Arc<UserService>,
    pub user_credentials_service: Arc<UserCredentialsService>,
    pub rbac_service: Arc<RbacService>,
    pub auth_service: Arc<AuthService>,
    pub audit_service: Arc<AuditService>,
    pub telemetry_service: Arc<TelemetryService>,
    pub delivery_replay_service: Arc<DeliveryReplayService>,
    pub metrics_service: Arc<MetricsService>,
    pub realm_service: Arc<RealmService>,
    pub realm_email_settings_service: Arc<RealmEmailSettingsService>,
    pub realm_passkey_settings_service: Arc<RealmPasskeySettingsService>,
    pub realm_recovery_settings_service: Arc<RealmRecoverySettingsService>,
    pub realm_security_headers_service: Arc<RealmSecurityHeadersService>,
    pub passkey_assertion_service: Arc<PasskeyAssertionService>,
    pub passkey_analytics_service: Arc<PasskeyAnalyticsService>,
    pub email_delivery_service: Arc<EmailDeliveryService>,
    pub webhook_service: Arc<WebhookService>,
    pub theme_service: Arc<ThemeResolverService>,
    pub harbor_service: Arc<HarborService>,
    pub oidc_service: Arc<OidcService>,
    pub flow_service: Arc<FlowService>,
    pub flow_manager: Arc<FlowManager>,
    pub node_registry: Arc<NodeRegistryService>,

    // Infrastructure / Repositories
    pub log_subscriber: Arc<dyn LogSubscriber>,
    pub cache_service: Arc<dyn CacheService>,
    pub auth_session_repo: Arc<dyn AuthSessionRepository>,
    pub session_repo: Arc<dyn SessionRepository>,

    pub flow_store: Arc<dyn FlowStore>,
    pub flow_executor: Arc<FlowExecutor>,
}

impl AppState {
    pub async fn is_setup_required(&self) -> bool {
        self.setup_state.read().await.required
    }
}
