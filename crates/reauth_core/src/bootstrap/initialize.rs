use crate::adapters::eventing::outbox_worker::OutboxWorker;
use crate::adapters::logging::banner::print_banner;
use crate::adapters::observability::sqlite_telemetry_repository::SqliteTelemetryRepository;
use crate::adapters::observability::telemetry_store::init_telemetry_db;
use crate::adapters::observability::telemetry_writer::TelemetryWriter;
use crate::adapters::persistence::transaction::SqliteTransactionManager;
use crate::application::delivery_replay_service::DeliveryReplayService;
use crate::application::metrics_service::MetricsService;
use crate::application::telemetry_service::TelemetryService;
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
use chrono::{Duration, Utc};
use notify::{RecursiveMode, Watcher};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Clone, Copy)]
struct InitializeOptions {
    print_banner: bool,
    log_summary: bool,
    watch_config: bool,
    enable_telemetry_cleanup: bool,
    enable_outbox_worker: bool,
}

pub async fn initialize() -> anyhow::Result<AppState> {
    let settings = Settings::new()?;
    initialize_with_settings(
        settings,
        InitializeOptions {
            print_banner: true,
            log_summary: true,
            watch_config: true,
            enable_telemetry_cleanup: true,
            enable_outbox_worker: true,
        },
    )
    .await
}

pub async fn initialize_for_tests() -> anyhow::Result<AppState> {
    let settings = Settings::new()?;
    initialize_with_settings(
        settings,
        InitializeOptions {
            print_banner: false,
            log_summary: false,
            watch_config: false,
            enable_telemetry_cleanup: false,
            enable_outbox_worker: false,
        },
    )
    .await
}

async fn initialize_with_settings(
    settings: Settings,
    options: InitializeOptions,
) -> anyhow::Result<AppState> {
    let log_bus = init_logging(&settings);
    if options.print_banner {
        print_banner(&settings);
    }
    if options.log_summary {
        log_config_summary(&settings);
        warn_public_url_mismatch(&settings);
    }

    let plugins_path = determine_plugins_path()?;
    let telemetry_db = init_telemetry_db(&settings.observability.telemetry_db_path).await?;
    let telemetry_repo = Arc::new(SqliteTelemetryRepository::new(telemetry_db.clone()));
    let telemetry_service = Arc::new(TelemetryService::new(telemetry_repo.clone()));
    TelemetryWriter::new(telemetry_repo).spawn(log_bus.clone());
    let metrics_service = Arc::new(MetricsService::new());
    let db_pool = initialize_database(&settings).await?;
    let repos = initialize_repositories(&db_pool);

    let (event_bus, cache_service, jwt_service) = initialize_core_infra(&settings)?;

    let tx_manager: Arc<dyn TransactionManager> =
        Arc::new(SqliteTransactionManager::new(db_pool.clone()));

    let services = initialize_services(crate::bootstrap::services::ServiceInitContext {
        settings: &settings,
        repos: &repos,
        cache: &cache_service,
        event_publisher: event_bus.clone(),
        outbox_repo: repos.outbox_repo.clone(),
        token_service: &jwt_service,
        telemetry_db: &telemetry_db,
        tx_manager: &tx_manager,
    });

    let plugin_manager = initialize_plugins(&settings, &plugins_path);
    let delivery_replay_service = Arc::new(DeliveryReplayService::new(
        telemetry_service.clone(),
        services.webhook_service.clone(),
        telemetry_db.clone(),
        db_pool.clone(),
        plugin_manager.clone(),
    ));

    subscribe_event_listeners(&event_bus, &cache_service, &repos.rbac_repo).await;

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
    if options.watch_config {
        spawn_config_watcher(settings_shared.clone());
    }
    if options.enable_telemetry_cleanup {
        spawn_telemetry_cleanup(settings_shared.clone(), telemetry_service.clone());
    }
    if options.enable_outbox_worker {
        OutboxWorker::new(db_pool.clone(), telemetry_db, plugin_manager.clone()).spawn();
    }

    Ok(AppState {
        settings: settings_shared,
        plugin_manager,
        plugins_path,
        user_service: services.user_service,
        rbac_service: services.rbac_service,
        auth_service: services.auth_service,
        audit_service: services.audit_service,
        telemetry_service,
        delivery_replay_service,
        metrics_service,
        realm_service: services.realm_service,
        webhook_service: services.webhook_service,
        log_subscriber: log_bus,
        cache_service: cache_service.clone(),
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
            warn!(
                "Failed to watch config file {}: {}",
                config_path.display(),
                err
            );
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

fn spawn_telemetry_cleanup(
    settings: Arc<RwLock<Settings>>,
    telemetry_service: Arc<TelemetryService>,
) {
    tokio::spawn(async move {
        let interval_secs = { settings.read().await.observability.cleanup_interval_secs };
        if interval_secs == 0 {
            info!("Telemetry cleanup disabled (cleanup_interval_secs=0).");
            return;
        }

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
        loop {
            interval.tick().await;

            let (log_days, trace_days) = {
                let current = settings.read().await;
                (
                    current.observability.log_retention_days,
                    current.observability.trace_retention_days,
                )
            };

            if log_days <= 0 && trace_days <= 0 {
                continue;
            }

            let log_before = retention_cutoff(log_days);
            let trace_before = retention_cutoff(trace_days);

            if let Some(before) = log_before.as_deref() {
                if let Ok(deleted) = telemetry_service.clear_logs(Some(before)).await {
                    info!(
                        "Telemetry cleanup removed {} logs (before {}).",
                        deleted, before
                    );
                }
            }

            if let Some(before) = trace_before.as_deref() {
                if let Ok(deleted) = telemetry_service.clear_traces(Some(before)).await {
                    info!(
                        "Telemetry cleanup removed {} traces (before {}).",
                        deleted, before
                    );
                }
            }
        }
    });
}

fn retention_cutoff(days: i64) -> Option<String> {
    if days <= 0 {
        return None;
    }
    Some((Utc::now() - Duration::days(days)).to_rfc3339())
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
