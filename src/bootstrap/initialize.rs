use crate::adapters::eventing::outbox_worker::OutboxWorker;
use crate::adapters::logging::banner::print_banner;
use crate::adapters::observability::sqlite_telemetry_repository::SqliteTelemetryRepository;
use crate::adapters::observability::telemetry_store::init_telemetry_db;
use crate::adapters::observability::telemetry_writer::TelemetryWriter;
use crate::adapters::persistence::connection::Database;
use crate::adapters::persistence::transaction::SqliteTransactionManager;
use crate::adapters::web::outbound_http_client::ReqwestDeliveryClient;
use crate::application::delivery_replay_service::DeliveryReplayService;
use crate::application::metrics_service::MetricsService;
use crate::application::telemetry_service::TelemetryService;
use crate::bootstrap::app_state::SetupState;
use crate::bootstrap::database::{initialize_database, run_migrations_and_seed};
use crate::bootstrap::events::subscribe_event_listeners;
use crate::bootstrap::infrastructure::initialize_core_infra;
use crate::bootstrap::logging::init_logging;
use crate::bootstrap::repositories::initialize_repositories;
use crate::bootstrap::services::initialize_services;
use crate::config::Settings;
use crate::constants::DEFAULT_REALM_NAME;
use crate::ports::passkey_challenge_repository::PasskeyChallengeRepository;
use crate::ports::transaction_manager::TransactionManager;
use crate::AppState;
use chrono::{Duration, Utc};
use notify::{RecursiveMode, Watcher};
use rand::distr::{Alphanumeric, SampleString};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Clone, Copy)]
struct InitializeOptions {
    print_banner: bool,
    log_summary: bool,
    watch_config: bool,
    enable_telemetry_cleanup: bool,
    enable_outbox_worker: bool,
    enable_refresh_cleanup: bool,
    enable_harbor_cleanup: bool,
    enable_passkey_challenge_cleanup: bool,
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
            enable_refresh_cleanup: true,
            enable_harbor_cleanup: true,
            enable_passkey_challenge_cleanup: true,
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
            enable_refresh_cleanup: false,
            enable_harbor_cleanup: false,
            enable_passkey_challenge_cleanup: false,
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

    let telemetry_db = init_telemetry_db(&settings.observability.telemetry_db_path).await?;
    let telemetry_repo = Arc::new(SqliteTelemetryRepository::new(telemetry_db.clone()));
    let telemetry_service = Arc::new(TelemetryService::new(telemetry_repo.clone()));
    TelemetryWriter::new(telemetry_repo.clone()).spawn(log_bus.clone());
    let metrics_service = Arc::new(MetricsService::new());
    let db_pool = initialize_database(&settings).await?;
    let repos = initialize_repositories(&db_pool);

    let (event_bus, cache_service, jwt_service) = initialize_core_infra(&settings)?;

    let tx_manager: Arc<dyn TransactionManager> =
        Arc::new(SqliteTransactionManager::new(db_pool.clone()));
    let http_client = Arc::new(ReqwestDeliveryClient::new(std::time::Duration::from_secs(
        5,
    )));

    let services = initialize_services(crate::bootstrap::services::ServiceInitContext {
        settings: &settings,
        repos: &repos,
        cache: &cache_service,
        event_publisher: event_bus.clone(),
        outbox_repo: repos.outbox_repo.clone(),
        token_service: &jwt_service,
        telemetry_repo: telemetry_repo.clone(),
        tx_manager: &tx_manager,
        http_client: http_client.clone(),
    });

    let delivery_replay_service = Arc::new(DeliveryReplayService::new(
        telemetry_service.clone(),
        services.webhook_service.clone(),
        telemetry_repo.clone(),
        repos.webhook_repo.clone(),
        http_client.clone(),
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
        services.theme_service.clone(),
        services.harbor_service.clone(),
    )
    .await?;

    let setup_state = Arc::new(RwLock::new(
        detect_setup_state(&services.realm_service, &services.user_service, &settings).await?,
    ));

    let settings_shared = Arc::new(RwLock::new(settings.clone()));
    if options.watch_config {
        spawn_config_watcher(settings_shared.clone());
    }
    if options.enable_telemetry_cleanup {
        spawn_telemetry_cleanup(settings_shared.clone(), telemetry_service.clone());
    }
    if options.enable_outbox_worker {
        OutboxWorker::new(db_pool.clone(), telemetry_db).spawn();
    }
    if options.enable_refresh_cleanup {
        spawn_refresh_token_cleanup(settings_shared.clone(), db_pool.clone());
    }
    if options.enable_harbor_cleanup {
        spawn_harbor_cleanup(settings_shared.clone(), db_pool.clone());
    }
    if options.enable_passkey_challenge_cleanup {
        spawn_passkey_challenge_cleanup(
            settings_shared.clone(),
            repos.passkey_challenge_repo.clone(),
        );
    }

    Ok(AppState {
        settings: settings_shared,
        user_service: services.user_service,
        user_credentials_service: services.user_credentials_service,
        rbac_service: services.rbac_service,
        auth_service: services.auth_service,
        audit_service: services.audit_service,
        telemetry_service,
        delivery_replay_service,
        metrics_service,
        realm_service: services.realm_service,
        realm_email_settings_service: services.realm_email_settings_service,
        realm_passkey_settings_service: services.realm_passkey_settings_service,
        realm_recovery_settings_service: services.realm_recovery_settings_service,
        realm_security_headers_service: services.realm_security_headers_service,
        passkey_assertion_service: services.passkey_assertion_service,
        passkey_analytics_service: services.passkey_analytics_service,
        email_delivery_service: services.email_delivery_service,
        webhook_service: services.webhook_service,
        theme_service: services.theme_service,
        harbor_service: services.harbor_service,
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
        setup_state,
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

async fn detect_setup_state(
    realm_service: &Arc<crate::application::realm_service::RealmService>,
    user_service: &Arc<crate::application::user_service::UserService>,
    settings: &Settings,
) -> anyhow::Result<SetupState> {
    let realm = realm_service.find_by_name(DEFAULT_REALM_NAME).await?;
    let needs_setup = match realm {
        Some(realm) => user_service.count_users_in_realm(realm.id).await? == 0,
        None => true,
    };

    if !needs_setup {
        return Ok(SetupState::sealed());
    }

    let token: String = Alphanumeric.sample_string(&mut rand::rng(), 32);
    let base_url = settings.server.public_url.trim_end_matches('/');
    let setup_url = format!("{}/setup", base_url);
    info!(
        "First run detected. Visit {} and use Token: {}",
        setup_url, token
    );

    Ok(SetupState::pending(token))
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

fn spawn_refresh_token_cleanup(settings: Arc<RwLock<Settings>>, db_pool: Database) {
    tokio::spawn(async move {
        loop {
            let interval_secs = {
                settings
                    .read()
                    .await
                    .auth
                    .refresh_token_cleanup_interval_secs
            };
            if interval_secs == 0 {
                info!("Refresh token cleanup disabled (refresh_token_cleanup_interval_secs=0).");
                return;
            }

            tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;

            let retention_secs = { settings.read().await.auth.refresh_token_retention_secs };

            let cutoff = if retention_secs <= 0 {
                Utc::now()
            } else {
                Utc::now() - Duration::seconds(retention_secs)
            };

            match sqlx::query("DELETE FROM refresh_tokens WHERE expires_at < ?")
                .bind(cutoff)
                .execute(&*db_pool)
                .await
            {
                Ok(result) => {
                    info!(
                        "Refresh token cleanup removed {} tokens (expired before {}).",
                        result.rows_affected(),
                        cutoff
                    );
                }
                Err(err) => {
                    warn!("Failed to cleanup refresh tokens: {}", err);
                }
            }
        }
    });
}

fn spawn_harbor_cleanup(settings: Arc<RwLock<Settings>>, db_pool: Database) {
    tokio::spawn(async move {
        loop {
            let interval_secs = { settings.read().await.harbor.cleanup_interval_secs };
            if interval_secs == 0 {
                info!("Harbor cleanup disabled (harbor.cleanup_interval_secs=0).");
                return;
            }

            tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;

            let (storage_dir, retention_hours) = {
                let guard = settings.read().await;
                (
                    guard.harbor.storage_dir.clone(),
                    guard.harbor.artifact_retention_hours,
                )
            };

            if retention_hours == 0 {
                continue;
            }

            match cleanup_harbor_artifacts(&storage_dir, retention_hours, &db_pool).await {
                Ok(removed) => {
                    if removed > 0 {
                        info!(
                            "Harbor cleanup removed {} artifact(s) older than {} hours.",
                            removed, retention_hours
                        );
                    }
                }
                Err(err) => {
                    warn!("Failed to cleanup Harbor artifacts: {}", err);
                }
            }
        }
    });
}

fn spawn_passkey_challenge_cleanup(
    settings: Arc<RwLock<Settings>>,
    passkey_challenge_repo: Arc<dyn PasskeyChallengeRepository>,
) {
    tokio::spawn(async move {
        loop {
            let interval_secs = {
                settings
                    .read()
                    .await
                    .auth
                    .passkey_challenge_cleanup_interval_secs
            };
            if interval_secs == 0 {
                info!(
                    "Passkey challenge cleanup disabled (passkey_challenge_cleanup_interval_secs=0)."
                );
                return;
            }

            tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;

            let batch_size = {
                settings
                    .read()
                    .await
                    .auth
                    .passkey_challenge_cleanup_batch_size
            };
            let cutoff = Utc::now();
            let mut total_removed = 0u64;

            loop {
                match passkey_challenge_repo
                    .delete_expired_before(cutoff, batch_size)
                    .await
                {
                    Ok(removed) => {
                        total_removed += removed;
                        if removed < batch_size.max(1) as u64 {
                            break;
                        }
                    }
                    Err(err) => {
                        warn!("Failed to cleanup passkey challenges: {}", err);
                        break;
                    }
                }
            }

            if total_removed > 0 {
                info!(
                    "Passkey challenge cleanup removed {} rows (cutoff {}).",
                    total_removed, cutoff
                );
            }
        }
    });
}

async fn cleanup_harbor_artifacts(
    storage_dir: &str,
    retention_hours: u64,
    db_pool: &Database,
) -> anyhow::Result<u64> {
    if storage_dir.trim().is_empty() {
        return Ok(0);
    }

    let cutoff = SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(retention_hours * 3600))
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let mut removed = 0u64;
    let mut entries = match tokio::fs::read_dir(storage_dir).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(err) => return Err(anyhow::anyhow!(err)),
    };

    let mut removed_paths = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let metadata = entry.metadata().await?;
        if !metadata.is_file() {
            continue;
        }

        let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        if modified < cutoff && tokio::fs::remove_file(&path).await.is_ok() {
            removed += 1;
            removed_paths.push(path);
        }
    }

    for path in removed_paths {
        let _ = sqlx::query(
            "UPDATE harbor_jobs
             SET artifact_path = NULL,
                 artifact_filename = NULL,
                 artifact_content_type = NULL,
                 status = 'expired',
                 updated_at = CURRENT_TIMESTAMP
             WHERE artifact_path = ? AND status = 'completed'",
        )
        .bind(path.to_string_lossy().to_string())
        .execute(&**db_pool)
        .await;
    }

    Ok(removed)
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
