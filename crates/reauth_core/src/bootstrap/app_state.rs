use std::path::PathBuf;
use std::sync::Arc;

use crate::application::flow_engine::FlowEngine;
use crate::application::flow_service::FlowService;
use crate::application::oidc_service::OidcService;
use crate::application::{
    auth_service::AuthService, rbac_service::RbacService, realm_service::RealmService,
    user_service::UserService,
};
use crate::config::Settings;
use manager::{log_bus::LogSubscriber, PluginManager};

pub struct AppState {
    pub settings: Settings,
    pub plugin_manager: PluginManager,
    pub plugins_path: PathBuf,
    pub user_service: Arc<UserService>,
    pub rbac_service: Arc<RbacService>,
    pub auth_service: Arc<AuthService>,
    pub realm_service: Arc<RealmService>,
    pub log_subscriber: Arc<dyn LogSubscriber>,
    pub flow_engine: Arc<FlowEngine>,
    pub oidc_service: Arc<OidcService>,
    pub flow_service: Arc<FlowService>,
}
