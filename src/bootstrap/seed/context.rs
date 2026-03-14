use crate::application::flow_manager::FlowManager;
use crate::application::harbor::HarborService;
use crate::application::oidc_service::OidcService;
use crate::application::realm_service::RealmService;
use crate::application::theme_service::ThemeResolverService;
use crate::config::Settings;
use crate::ports::flow_repository::FlowRepository;
use crate::ports::flow_store::FlowStore;
use std::sync::Arc;

pub struct SeedContext<'a> {
    pub realm_service: &'a Arc<RealmService>,
    pub flow_repo: &'a Arc<dyn FlowRepository>,
    pub flow_store: &'a Arc<dyn FlowStore>,
    pub flow_manager: &'a Arc<FlowManager>,
    pub settings: &'a Settings,
    pub oidc_service: &'a Arc<OidcService>,
    pub theme_service: &'a Arc<ThemeResolverService>,
    pub harbor_service: &'a Arc<HarborService>,
}
