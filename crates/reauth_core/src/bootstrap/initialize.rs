use crate::adapters::logging::banner::print_banner;
use crate::adapters::persistence::transaction::SqliteTransactionManager;
use crate::bootstrap::database::{initialize_database, run_migrations_and_seed};
use crate::bootstrap::events::subscribe_event_listeners;
use crate::bootstrap::infrastructure::initialize_core_infra;
use crate::bootstrap::logging::init_logging;
use crate::bootstrap::plugins::{determine_plugins_path, initialize_plugins};
use crate::bootstrap::repositories::initialize_repositories;
use crate::bootstrap::services::initialize_services;
use crate::config::Settings;
use crate::ports::transaction_manager::TransactionManager;
use crate::AppState;
use std::sync::Arc;

pub async fn initialize() -> anyhow::Result<AppState> {
    let settings = Settings::new()?;
    let log_bus = init_logging();
    dotenvy::dotenv().ok();
    print_banner();

    let plugins_path = determine_plugins_path()?;
    let db_pool = initialize_database(&settings).await?;
    let repos = initialize_repositories(&db_pool);

    let (event_bus, cache_service, jwt_service) = initialize_core_infra(&settings)?;

    let tx_manager: Arc<dyn TransactionManager> =
        Arc::new(SqliteTransactionManager::new(db_pool.clone()));

    let services = initialize_services(
        &settings,
        &repos,
        &cache_service,
        &event_bus,
        &jwt_service,
        &tx_manager,
    );

    let plugin_manager = initialize_plugins(&settings, &plugins_path);

    subscribe_event_listeners(
        &event_bus,
        &cache_service,
        &repos.rbac_repo,
        plugin_manager.clone(),
    )
    .await;

    run_migrations_and_seed(
        &db_pool,
        &services.realm_service,
        &services.user_service,
        repos.flow_repo,
        repos.flow_store.clone(),
        services.flow_manager.clone(),
        &settings,
        &services.oidc_service,
    )
    .await?;

    Ok(AppState {
        settings,
        plugin_manager,
        plugins_path,
        user_service: services.user_service,
        rbac_service: services.rbac_service,
        auth_service: services.auth_service,
        realm_service: services.realm_service,
        log_subscriber: log_bus,
        auth_session_repo: repos.auth_session_repo,
        flow_store: repos.flow_store,
        // flow_engine has been removed
        oidc_service: services.oidc_service,
        flow_service: services.flow_service,
        flow_manager: services.flow_manager,
        node_registry: services.node_registry,
        flow_executor: services.flow_executor,
        session_repo: repos.session_repo,
    })
}
