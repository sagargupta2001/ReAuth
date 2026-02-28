use crate::application::flow_manager::FlowManager;
use crate::application::oidc_service::OidcService;
use crate::application::rbac_service::RbacService;
use crate::application::realm_service::RealmService;
use crate::application::user_service::UserService;
use crate::config::Settings;
use crate::ports::flow_repository::FlowRepository;
use crate::ports::flow_store::FlowStore;
use std::sync::Arc;

pub struct SeedContext<'a> {
    pub realm_service: &'a Arc<RealmService>,
    pub user_service: &'a Arc<UserService>,
    pub flow_repo: &'a Arc<dyn FlowRepository>,
    pub flow_store: &'a Arc<dyn FlowStore>,
    pub flow_manager: &'a Arc<FlowManager>,
    pub settings: &'a Settings,
    pub oidc_service: &'a Arc<OidcService>,
    pub rbac_service: &'a Arc<RbacService>,
}
