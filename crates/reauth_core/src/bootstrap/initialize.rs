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
use notify::{RecursiveMode, Watcher};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

pub async fn initialize() -> anyhow::Result<AppState> {
    let settings = Settings::new()?;
    let log_bus = init_logging(&settings);
    print_banner(&settings);
    log_config_summary(&settings);
    warn_public_url_mismatch(&settings);

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
        services.rbac_service.clone(),
    )
    .await?;

    let settings_shared = Arc::new(RwLock::new(settings.clone()));
    spawn_config_watcher(settings_shared.clone());

    Ok(AppState {
        settings: settings_shared,
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

fn log_config_summary(settings: &Settings) {
    tracing::info!(
        "Config summary: public_url={}, data_dir={}, db_url={}, ui_dev_url={}, cors_allowed_origins={}",
        settings.server.public_url,
        settings.database.data_dir,
        settings.database.url,
        settings.ui.dev_url,
        settings.cors.allowed_origins.len()
    );
}

fn warn_public_url_mismatch(settings: &Settings) {
    if let Some((public_origin, bind_origins)) = settings.public_url_mismatch() {
        warn!(
            "server.public_url origin ({}) does not match bind origin(s) {:?}. This may break cookies or redirect URIs.",
            public_origin, bind_origins
        );
    }
}

fn spawn_config_watcher(settings: Arc<RwLock<Settings>>) {
    let Some(config_path) = Settings::resolve_config_watch_path() else {
        info!("Config hot reload disabled (no config file found).");
        return;
    };

    info!("Config hot reload enabled for {}", config_path.display());

    tokio::spawn(async move {
        let (tx, mut rx) = tokio::sync::mpsc::channel(16);
        let mut watcher = match notify::recommended_watcher(move |res| {
            let _ = tx.blocking_send(res);
        }) {
            Ok(watcher) => watcher,
            Err(err) => {
                warn!("Failed to start config watcher: {}", err);
                return;
            }
        };

        if let Err(err) = watcher.watch(&config_path, RecursiveMode::NonRecursive) {
            warn!("Failed to watch config file {}: {}", config_path.display(), err);
            return;
        }

        let mut missing_warned = false;
        loop {
            let event = match rx.recv().await {
                Some(event) => event,
                None => break,
            };

            if let Err(err) = event {
                warn!("Config watcher error: {}", err);
                continue;
            }

            // Debounce rapid successive events.
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            while rx.try_recv().is_ok() {}

            if !config_path.exists() {
                if !missing_warned {
                    warn!(
                        "Config hot reload disabled: watched config file {} no longer exists.",
                        config_path.display()
                    );
                    missing_warned = true;
                }
                continue;
            }

            if missing_warned {
                info!(
                    "Config file {} restored; hot reload resumed.",
                    config_path.display()
                );
                missing_warned = false;
            }

            match Settings::new() {
                Ok(new_settings) => {
                    apply_settings_update(&settings, new_settings).await;
                }
                Err(err) => {
                    warn!("Config reload failed: {}", err);
                }
            }
        }
    });
}

pub(crate) async fn apply_settings_update(
    settings: &Arc<RwLock<Settings>>,
    new_settings: Settings,
) {
    let mut guard = settings.write().await;
    let old_settings = guard.clone();
    *guard = new_settings.clone();

    info!("Config reloaded from disk.");
    log_config_summary(&new_settings);
    warn_public_url_mismatch(&new_settings);
    warn_non_reloadable_changes(&old_settings, &new_settings);
}

fn warn_non_reloadable_changes(old: &Settings, new: &Settings) {
    let mut changes = Vec::new();

    if old.server.scheme != new.server.scheme {
        changes.push("server.scheme");
    }
    if old.server.host != new.server.host {
        changes.push("server.host");
    }
    if old.server.port != new.server.port {
        changes.push("server.port");
    }
    if old.database.url != new.database.url {
        changes.push("database.url");
    }
    if old.database.data_dir != new.database.data_dir {
        changes.push("database.data_dir");
    }
    if old.auth.jwt_secret != new.auth.jwt_secret {
        changes.push("auth.jwt_secret");
    }
    if old.auth.jwt_key_id != new.auth.jwt_key_id {
        changes.push("auth.jwt_key_id");
    }
    if old.auth.issuer != new.auth.issuer {
        changes.push("auth.issuer");
    }

    if !changes.is_empty() {
        warn!(
            "Config hot reload applied, but changes to {} require a restart to fully take effect.",
            changes.join(", ")
        );
    }
}
