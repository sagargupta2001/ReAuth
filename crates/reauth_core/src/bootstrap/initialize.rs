use crate::adapters::cache::cache_invalidator::CacheInvalidator;
use crate::adapters::cache::moka_cache::MokaCacheService;
use crate::adapters::crypto::jwt_service::JwtService;
use crate::adapters::eventing::in_memory_bus::InMemoryEventBus;
use crate::adapters::eventing::log_broadcast_bus::LogBroadcastBus;
use crate::adapters::logging::banner::print_banner;
use crate::adapters::logging::tracing_adapter::TracingLogAdapter;
use crate::adapters::persistence::sqlite_rbac_repository::SqliteRbacRepository;
use crate::adapters::persistence::sqlite_realm_repository::SqliteRealmRepository;
use crate::adapters::persistence::sqlite_session_repository::SqliteSessionRepository;
use crate::adapters::{init_db, run_migrations, PluginEventGateway, SqliteUserRepository};
use crate::application::auth_service::AuthService;
use crate::application::rbac_service::RbacService;
use crate::application::realm_service::RealmService;
use crate::application::user_service::UserService;
use crate::bootstrap::seed::seed_database;
use crate::config::Settings;
use crate::ports::event_bus::EventSubscriber;
use crate::{constants, AppState};
use manager::{ManagerConfig, PluginManager};
use std::env;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tracing::{info, warn};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Performs all initialization logic: env, plugins, DB, migrations, and DI.
pub async fn initialize() -> anyhow::Result<AppState> {
    let settings = Settings::new()?;
    let log_bus = Arc::new(LogBroadcastBus::new());
    let log_adapter = TracingLogAdapter::new(log_bus.clone());
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(log_adapter)
        .init();

    dotenvy::dotenv().ok();
    print_banner();

    let manager_config = ManagerConfig {
        handshake_timeout_secs: settings.plugins.handshake_timeout_secs,
    };

    let exe_path = env::current_exe()?;
    let is_dev_run = exe_path
        .ancestors()
        .any(|p| p.ends_with(constants::DEV_ENVIRONMENT_DIR));

    let plugins_path = if is_dev_run {
        PathBuf::from(constants::PLUGINS_DIR)
    } else {
        let mut prod_path = exe_path;
        prod_path.pop();
        prod_path.join(constants::PLUGINS_DIR)
    };

    info!(
        "Loading plugins from: {:?}",
        plugins_path
            .canonicalize()
            .unwrap_or_else(|_| plugins_path.clone())
    );

    info!("Initializing database...");
    if let Some(db_path_str) = settings.database.url.strip_prefix("sqlite:") {
        let db_path = Path::new(db_path_str);

        if let Some(parent_dir) = db_path.parent() {
            if !parent_dir.exists() {
                info!("Creating database directory at: {:?}", parent_dir);
                fs::create_dir_all(parent_dir).await?;
            }
        }

        if !db_path.exists() {
            info!("Creating database file at: {:?}", db_path);
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(db_path)?;
        }
    }

    let db_pool = init_db(&settings.database).await?;

    // Initialize Adapters
    let user_repo = Arc::new(SqliteUserRepository::new(db_pool.clone()));
    let rbac_repo = Arc::new(SqliteRbacRepository::new(db_pool.clone()));
    let realm_repo = Arc::new(SqliteRealmRepository::new(db_pool.clone()));
    let session_repo = Arc::new(SqliteSessionRepository::new(db_pool.clone()));
    let realm_service = Arc::new(RealmService::new(realm_repo.clone()));
    let cache_service = Arc::new(MokaCacheService::new());
    let token_service = Arc::new(JwtService::new(settings.auth.clone()));
    let event_bus = Arc::new(InMemoryEventBus::new());

    // Initialize Application Services
    let user_service = Arc::new(UserService::new(user_repo.clone(), event_bus.clone()));
    let rbac_service = Arc::new(RbacService::new(
        rbac_repo.clone(),
        cache_service.clone(),
        event_bus.clone(),
    ));
    let auth_service = Arc::new(AuthService::new(
        user_repo,
        realm_repo,
        session_repo,
        token_service,
        rbac_service.clone(),
        settings.auth.clone(),
    ));

    // Spawn plugin discovery in the background
    let plugin_manager = PluginManager::new(manager_config, plugins_path.clone());

    // Initialize and Subscribe Listeners
    let cache_invalidator = Arc::new(CacheInvalidator::new(cache_service, rbac_repo));
    event_bus.subscribe(cache_invalidator).await;
    let plugin_gateway = Arc::new(PluginEventGateway::new(plugin_manager.clone()));
    event_bus.subscribe(plugin_gateway).await;

    // Run Migrations
    if let Err(e) = run_migrations(&db_pool).await {
        warn!("Migration warning: {}", e);
    }

    info!("Running database seeding...");
    seed_database(&realm_service, &user_service, &settings.default_admin).await?;

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
