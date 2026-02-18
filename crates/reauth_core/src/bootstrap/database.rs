use crate::adapters::persistence::connection::Database;
use crate::adapters::{init_db, run_migrations};
use crate::application::flow_manager::FlowManager;
use crate::application::oidc_service::OidcService;
use crate::application::rbac_service::RbacService;
use crate::application::realm_service::RealmService;
use crate::application::user_service::UserService;
use crate::bootstrap::seed::seed_database;
use crate::config::Settings;
use crate::ports::flow_repository::FlowRepository;
use crate::ports::flow_store::FlowStore;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tracing::{info, warn};

pub async fn initialize_database(settings: &Settings) -> anyhow::Result<Database> {
    if let Some(path) = settings.database.url.strip_prefix("sqlite:") {
        ensure_sqlite_file_exists(path).await?;
    }

    info!("Initializing database...");
    Ok(init_db(&settings.database).await?)
}

pub async fn ensure_sqlite_file_exists(path: &str) -> anyhow::Result<()> {
    let db_path = Path::new(path);

    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            info!("Creating DB folder: {:?}", parent);
            fs::create_dir_all(parent).await?;
        }
    }

    if !db_path.exists() {
        info!("Creating DB file: {:?}", db_path);
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(db_path)?;
    }

    Ok(())
}

pub async fn run_migrations_and_seed(
    db_pool: &sqlx::SqlitePool,
    realm_service: &Arc<RealmService>,
    user_service: &Arc<UserService>,
    flow_repo: Arc<dyn FlowRepository>,
    flow_store: Arc<dyn FlowStore>,
    flow_manager: Arc<FlowManager>,
    settings: &Settings,
    oidc_service: &Arc<OidcService>,
    rbac_service: Arc<RbacService>,
) -> anyhow::Result<()> {
    if let Err(e) = run_migrations(db_pool).await {
        warn!("Migration warning: {}", e);
    }

    info!("Running database seeding...");
    seed_database(
        db_pool,
        realm_service,
        user_service,
        &flow_repo,
        &flow_store,
        &flow_manager,
        &settings,
        oidc_service,
        &rbac_service,
    )
    .await?;

    Ok(())
}
