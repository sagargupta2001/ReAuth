mod admin;
mod context;
mod flows;
mod history;
mod oidc;
mod realm;

use crate::application::flow_manager::FlowManager;
use crate::application::oidc_service::OidcService;
use crate::application::rbac_service::RbacService;
use crate::application::realm_service::RealmService;
use crate::application::user_service::UserService;
use crate::config::Settings;
use crate::domain::realm::Realm;
use crate::ports::flow_repository::FlowRepository;
use crate::ports::flow_store::FlowStore;
use async_trait::async_trait;
use context::SeedContext;
use history::SeedHistory;
use std::sync::Arc;
use tracing::info;

pub async fn seed_database(
    db_pool: &sqlx::SqlitePool,
    realm_service: &Arc<RealmService>,
    user_service: &Arc<UserService>,
    flow_repo: &Arc<dyn FlowRepository>,
    flow_store: &Arc<dyn FlowStore>,
    flow_manager: &Arc<FlowManager>,
    settings: &Settings,
    oidc_service: &Arc<OidcService>,
    rbac_service: &Arc<RbacService>,
) -> anyhow::Result<()> {
    let ctx = SeedContext {
        realm_service,
        user_service,
        flow_repo,
        flow_store,
        flow_manager,
        settings,
        oidc_service,
        rbac_service,
    };

    let mut state = SeedState::default();
    let seeders: Vec<Box<dyn Seeder>> = vec![
        Box::new(RealmSeeder),
        Box::new(FlowsSeeder),
        Box::new(AdminSeeder),
        Box::new(OidcSeeder),
    ];

    let history = SeedHistory::new(db_pool);

    for seeder in seeders {
        let name = seeder.name();
        let version = seeder.version();
        let checksum = seeder.checksum(&ctx);

        let should_run = if seeder.always_run() {
            true
        } else {
            match history.get(name).await? {
                None => true,
                Some(record) => record.version != version || record.checksum != checksum,
            }
        };

        if !should_run {
            info!("Seeder '{}' is up to date; skipping.", name);
            continue;
        }

        info!("Running seeder '{}'...", name);
        seeder.run(&ctx, &mut state).await?;
        history.upsert(name, version, &checksum).await?;
        info!("Seeder '{}' completed.", name);
    }

    Ok(())
}

#[derive(Default)]
struct SeedState {
    default_realm: Option<Realm>,
}

impl SeedState {
    fn set_realm(&mut self, realm: Realm) {
        self.default_realm = Some(realm);
    }

    fn require_realm(&self) -> anyhow::Result<Realm> {
        self.default_realm
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Default realm must be seeded before this step"))
    }
}

#[async_trait]
trait Seeder: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> i32;
    fn always_run(&self) -> bool {
        false
    }
    fn checksum(&self, _ctx: &SeedContext<'_>) -> String {
        String::new()
    }
    async fn run(&self, ctx: &SeedContext<'_>, state: &mut SeedState) -> anyhow::Result<()>;
}

struct RealmSeeder;
struct FlowsSeeder;
struct AdminSeeder;
struct OidcSeeder;

#[async_trait]
impl Seeder for RealmSeeder {
    fn name(&self) -> &'static str {
        "default_realm"
    }

    fn version(&self) -> i32 {
        1
    }

    fn always_run(&self) -> bool {
        true
    }

    fn checksum(&self, _ctx: &SeedContext<'_>) -> String {
        crate::constants::DEFAULT_REALM_NAME.to_string()
    }

    async fn run(&self, ctx: &SeedContext<'_>, state: &mut SeedState) -> anyhow::Result<()> {
        let realm = realm::ensure_default_realm(ctx).await?;
        state.set_realm(realm);
        Ok(())
    }
}

#[async_trait]
impl Seeder for FlowsSeeder {
    fn name(&self) -> &'static str {
        "default_flows"
    }

    fn version(&self) -> i32 {
        1
    }

    fn always_run(&self) -> bool {
        true
    }

    fn checksum(&self, _ctx: &SeedContext<'_>) -> String {
        "default_flows_v1".to_string()
    }

    async fn run(&self, ctx: &SeedContext<'_>, state: &mut SeedState) -> anyhow::Result<()> {
        let mut realm = state.require_realm()?;
        flows::ensure_default_flows(ctx, &mut realm).await?;
        state.set_realm(realm);
        Ok(())
    }
}

#[async_trait]
impl Seeder for AdminSeeder {
    fn name(&self) -> &'static str {
        "default_admin"
    }

    fn version(&self) -> i32 {
        1
    }

    fn always_run(&self) -> bool {
        true
    }

    fn checksum(&self, ctx: &SeedContext<'_>) -> String {
        format!("username={}", ctx.settings.default_admin.username)
    }

    async fn run(&self, ctx: &SeedContext<'_>, state: &mut SeedState) -> anyhow::Result<()> {
        let realm = state.require_realm()?;
        admin::seed_admin_user(ctx, realm.id).await?;
        Ok(())
    }
}

#[async_trait]
impl Seeder for OidcSeeder {
    fn name(&self) -> &'static str {
        "default_oidc_client"
    }

    fn version(&self) -> i32 {
        1
    }

    fn always_run(&self) -> bool {
        true
    }

    fn checksum(&self, ctx: &SeedContext<'_>) -> String {
        let redirect_uris = ctx.settings.default_oidc_client.redirect_uris.join("|");
        let web_origins = ctx.settings.default_oidc_client.web_origins.join("|");
        format!(
            "client_id={};redirects={};origins={}",
            ctx.settings.default_oidc_client.client_id, redirect_uris, web_origins
        )
    }

    async fn run(&self, ctx: &SeedContext<'_>, state: &mut SeedState) -> anyhow::Result<()> {
        let realm = state.require_realm()?;
        oidc::seed_default_oidc_client(ctx, realm.id).await?;
        Ok(())
    }
}
