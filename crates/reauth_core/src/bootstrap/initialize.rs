use crate::adapters::logging::banner::print_banner;
use crate::bootstrap::database::{initialize_database, run_migrations_and_seed};
use crate::bootstrap::events::subscribe_event_listeners;
use crate::bootstrap::infrastructure::initialize_core_infra;
use crate::bootstrap::logging::init_logging;
use crate::bootstrap::plugins::{determine_plugins_path, initialize_plugins};
use crate::bootstrap::repositories::initialize_repositories;
use crate::bootstrap::services::initialize_services;
use crate::config::Settings;
use crate::AppState;

/// Performs all initialization logic: env, plugins, DB, migrations, and DI.
pub async fn initialize() -> anyhow::Result<AppState> {
    let settings = Settings::new()?;

    let log_bus = init_logging();
    dotenvy::dotenv().ok();
    print_banner();

    let plugins_path = determine_plugins_path()?;
    let db_pool = initialize_database(&settings).await?;
    let (user_repo, rbac_repo, realm_repo, session_repo) = initialize_repositories(&db_pool);
    let (event_bus, cache_service, _jwt_service) = initialize_core_infra(&settings);
    let (user_service, rbac_service, realm_service, auth_service) = initialize_services(
        &settings,
        &user_repo,
        &rbac_repo,
        &realm_repo,
        &session_repo,
        &cache_service,
        &event_bus,
    );

    let plugin_manager = initialize_plugins(&settings, &plugins_path);

    subscribe_event_listeners(
        &event_bus,
        &cache_service,
        &rbac_repo,
        plugin_manager.clone(),
    )
    .await;

    run_migrations_and_seed(&db_pool, &realm_service, &user_service, &settings).await?;

    Ok(AppState {
        settings,
        plugin_manager,
        plugins_path,
        user_service,
        rbac_service,
        auth_service,
        realm_service,
        log_subscriber: log_bus,
    })
}
