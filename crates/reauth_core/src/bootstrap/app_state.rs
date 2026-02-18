use std::path::PathBuf;
use std::sync::Arc;

use crate::application::flow_executor::FlowExecutor;
use crate::application::flow_manager::FlowManager;
use crate::application::flow_service::FlowService;
use crate::application::node_registry::NodeRegistryService;
use crate::application::oidc_service::OidcService;
use crate::application::{
    auth_service::AuthService, rbac_service::RbacService, realm_service::RealmService,
    user_service::UserService,
};
use crate::config::Settings;
use crate::ports::auth_session_repository::AuthSessionRepository;
use crate::ports::flow_store::FlowStore;
use crate::ports::session_repository::SessionRepository;
use manager::{log_bus::LogSubscriber, PluginManager};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<RwLock<Settings>>,
    pub plugin_manager: PluginManager,
    pub plugins_path: PathBuf,

    // Services
    pub user_service: Arc<UserService>,
    pub rbac_service: Arc<RbacService>,
    pub auth_service: Arc<AuthService>,
    pub realm_service: Arc<RealmService>,
    pub oidc_service: Arc<OidcService>,
    pub flow_service: Arc<FlowService>,
    pub flow_manager: Arc<FlowManager>,
    pub node_registry: Arc<NodeRegistryService>,

    // Infrastructure / Repositories
    pub log_subscriber: Arc<dyn LogSubscriber>,
    pub auth_session_repo: Arc<dyn AuthSessionRepository>,
    pub session_repo: Arc<dyn SessionRepository>,

    pub flow_store: Arc<dyn FlowStore>,
    pub flow_executor: Arc<FlowExecutor>,
}
