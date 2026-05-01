use crate::application::flow_manager::FlowManager;
use crate::application::flow_service::FlowService;
use crate::application::harbor::provider::HarborRegistry;
use crate::application::harbor::runner::HarborJobRunner;
use crate::application::oidc_service::OidcService;
use crate::application::rbac_service::RbacService;
use crate::application::theme_service::ThemeResolverService;
use crate::application::user_service::UserService;
use crate::ports::harbor_job_conflict_repository::HarborJobConflictRepository;
use crate::ports::harbor_job_repository::HarborJobRepository;
use crate::ports::transaction_manager::TransactionManager;
use std::sync::Arc;

pub(crate) const HARBOR_BUNDLE_VERSION: &str = "1.0";
pub(crate) const HARBOR_SCHEMA_VERSION: u32 = 1;
pub(crate) const HARBOR_JOB_TYPE_IMPORT: &str = "import";
pub(crate) const HARBOR_JOB_TYPE_EXPORT: &str = "export";
pub(crate) const HARBOR_JOB_STATUS_IN_PROGRESS: &str = "in_progress";

pub(crate) struct ImportProgress {
    pub(crate) processed: i64,
    pub(crate) created_total: i64,
    pub(crate) updated_total: i64,
}

pub struct HarborService {
    pub(crate) registry: HarborRegistry,
    pub(crate) theme_service: Arc<ThemeResolverService>,
    pub(crate) oidc_service: Arc<OidcService>,
    pub(crate) flow_service: Arc<FlowService>,
    pub(crate) flow_manager: Arc<FlowManager>,
    pub(crate) rbac_service: Arc<RbacService>,
    pub(crate) user_service: Arc<UserService>,
    pub(crate) tx_manager: Arc<dyn TransactionManager>,
    pub(crate) job_repo: Arc<dyn HarborJobRepository>,
    pub(crate) conflict_repo: Arc<dyn HarborJobConflictRepository>,
    pub(crate) job_runner: Arc<dyn HarborJobRunner>,
}

impl HarborService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        registry: HarborRegistry,
        theme_service: Arc<ThemeResolverService>,
        oidc_service: Arc<OidcService>,
        flow_service: Arc<FlowService>,
        flow_manager: Arc<FlowManager>,
        rbac_service: Arc<RbacService>,
        user_service: Arc<UserService>,
        tx_manager: Arc<dyn TransactionManager>,
        job_repo: Arc<dyn HarborJobRepository>,
        conflict_repo: Arc<dyn HarborJobConflictRepository>,
        job_runner: Arc<dyn HarborJobRunner>,
    ) -> Self {
        Self {
            registry,
            theme_service,
            oidc_service,
            flow_service,
            flow_manager,
            rbac_service,
            user_service,
            tx_manager,
            job_repo,
            conflict_repo,
            job_runner,
        }
    }
}
