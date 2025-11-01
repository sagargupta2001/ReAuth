pub mod config;
mod constants;
pub mod domain;
mod ports;
pub mod application;
pub mod adapters;

pub mod error;

use std::env;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use once_cell::sync::Lazy;
use tracing::info;
use manager::{ManagerConfig, PluginManager};
use tokio::fs;

use crate::adapters::logging::{banner::print_banner, logging::LOGGER};
use crate::config::Settings;
use crate::adapters::{init_db, run_migrations, start_server};
use crate::adapters::persistence::sqlite_rbac_repository::SqliteRbacRepository;
use crate::adapters::SqliteUserRepository;
use crate::application::rbac_service::RbacService;
use crate::application::user_service::UserService;

/// Starts the full ReAuth Core application.
///
/// This function is exposed so that integration tests (or other binaries)
/// can start the application without duplicating logic from `main.rs`.
pub async fn run() -> anyhow::Result<()> {
    Lazy::force(&LOGGER);
    dotenvy::dotenv().ok();
    print_banner();

    let settings = Settings::new()?;

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

    let plugins_path_clone = plugins_path.clone();
    info!(
        "Loading plugins from: {:?}",
        plugins_path.canonicalize().unwrap_or_else(|_| plugins_path.clone())
    );

    let plugin_manager = PluginManager::new(manager_config);
    let manager_clone = plugin_manager.clone();

    tokio::spawn(async move {
        if let Some(path_str) = plugins_path.to_str() {
            manager_clone.discover_and_run(path_str).await;
        }
    });

    info!("Initializing database...");
    if let Some(db_path_str) = settings.database.url.strip_prefix("sqlite:") {
        let db_path = Path::new(db_path_str);

        // Ensure the parent directory exists.
        if let Some(parent_dir) = db_path.parent() {
            if !parent_dir.exists() {
                info!("Creating database directory at: {:?}", parent_dir);
                fs::create_dir_all(parent_dir).await?;
            }
        }

        // Explicitly create the database file if it's missing.
        if !db_path.exists() {
            info!("Creating database file at: {:?}", db_path);
            OpenOptions::new().write(true).create_new(true).open(db_path)?;
        }
    }

    let db_pool = init_db(&settings.database).await?;

    let server_url = format!(
        "{}://{}:{}",
        settings.server.scheme, settings.server.host, settings.server.port
    );

    if cfg!(feature = "embed-ui") {
        None
    } else {
        Some(settings.ui.dev_url.as_str())
    };

    info!("🖥 Server started at: {}", server_url);
    info!("Database status: {}", "Up & Running");

    // --- Dependency Injection ---

    let user_repo = Arc::new(SqliteUserRepository::new(db_pool.clone()));
    let rbac_repo = Arc::new(SqliteRbacRepository::new(db_pool.clone()));

    let user_service = Arc::new(UserService::new(user_repo));
    let rbac_service = Arc::new(RbacService::new(rbac_repo));

    if let Err(e) = run_migrations(&db_pool).await {
        tracing::warn!("Migration warning: {}", e);
    }

    info!("Starting server...");
    start_server(
        settings,
        plugin_manager,
        plugins_path_clone,
        user_service,
        rbac_service,
    )
        .await?;

    Ok(())
}
