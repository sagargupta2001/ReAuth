#![allow(clippy::needless_option_as_deref)]

mod context;
mod flows;
pub mod history;
mod oidc;
mod realm;
mod theme;

use crate::adapters::persistence::transaction::SqliteTransactionManager;
use crate::application::flow_manager::FlowManager;
use crate::application::harbor::HarborService;
use crate::application::harbor::{
    read_bundle_from_path, ConflictPolicy, HarborBundle, HarborExportType, HarborScope,
};
use crate::application::oidc_service::OidcService;
use crate::application::rbac_service::RbacService;
use crate::application::realm_service::RealmService;
use crate::application::realm_service::UpdateRealmPayload;
use crate::application::theme_service::ThemeResolverService;
use crate::application::user_service::UserService;
use crate::config::Settings;
use crate::domain::realm::Realm;
use crate::ports::flow_repository::FlowRepository;
use crate::ports::flow_store::FlowStore;
use crate::ports::transaction_manager::{Transaction, TransactionManager};
use async_trait::async_trait;
use context::SeedContext;
use history::SeedHistory;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub async fn seed_database(
    db_pool: &sqlx::SqlitePool,
    realm_service: &Arc<RealmService>,
    _user_service: &Arc<UserService>,
    flow_repo: &Arc<dyn FlowRepository>,
    flow_store: &Arc<dyn FlowStore>,
    flow_manager: &Arc<FlowManager>,
    settings: &Settings,
    oidc_service: &Arc<OidcService>,
    _rbac_service: &Arc<RbacService>,
    theme_service: &Arc<ThemeResolverService>,
    harbor_service: &Arc<HarborService>,
) -> anyhow::Result<()> {
    let ctx = SeedContext {
        realm_service,
        flow_repo,
        flow_store,
        flow_manager,
        settings,
        oidc_service,
        theme_service,
        harbor_service,
    };

    let mut state = SeedState::default();
    let seeders: Vec<Box<dyn Seeder>> = vec![
        Box::new(RealmSeeder),
        Box::new(HarborBundleSeeder),
        Box::new(FlowsSeeder),
        Box::new(ThemeSeeder),
        Box::new(OidcSeeder),
    ];

    let tx_manager = SqliteTransactionManager::new(Arc::new(db_pool.clone()));
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

        if seeder.transactional() {
            let mut tx = tx_manager.begin().await?;
            {
                let mut tx_opt: Option<&mut dyn Transaction> = Some(&mut *tx);
                if let Err(err) = seeder.run(&ctx, &mut state, &mut tx_opt).await {
                    tx_manager.rollback(tx).await?;
                    return Err(err);
                }
                let tx_ref = tx_opt.as_deref_mut();
                history.upsert(name, version, &checksum, tx_ref).await?;
            }
            tx_manager.commit(tx).await?;
        } else {
            let mut tx_opt: Option<&mut dyn Transaction> = None;
            seeder.run(&ctx, &mut state, &mut tx_opt).await?;
            history.upsert(name, version, &checksum, None).await?;
        }

        info!("Seeder '{}' completed.", name);
    }

    Ok(())
}

#[derive(Default)]
struct SeedState {
    default_realm: Option<Realm>,
    harbor_seeded: bool,
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

    fn mark_harbor_seeded(&mut self) {
        self.harbor_seeded = true;
    }

    fn is_harbor_seeded(&self) -> bool {
        self.harbor_seeded
    }
}

#[async_trait]
trait Seeder: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> i32;
    fn transactional(&self) -> bool {
        false
    }
    fn always_run(&self) -> bool {
        false
    }
    fn checksum(&self, _ctx: &SeedContext<'_>) -> String {
        String::new()
    }
    async fn run(
        &self,
        ctx: &SeedContext<'_>,
        state: &mut SeedState,
        tx: &mut Option<&mut dyn Transaction>,
    ) -> anyhow::Result<()>;
}

struct RealmSeeder;
struct HarborBundleSeeder;
struct FlowsSeeder;
struct ThemeSeeder;
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

    async fn run(
        &self,
        ctx: &SeedContext<'_>,
        state: &mut SeedState,
        _tx: &mut Option<&mut dyn Transaction>,
    ) -> anyhow::Result<()> {
        let realm = realm::ensure_default_realm(ctx).await?;
        state.set_realm(realm);
        Ok(())
    }
}

#[derive(Deserialize)]
struct SeedFlowPayload {
    pub flow_type: String,
    #[serde(default)]
    pub flow_id: Option<String>,
}

#[async_trait]
impl Seeder for HarborBundleSeeder {
    fn name(&self) -> &'static str {
        "harbor_bundle"
    }

    fn version(&self) -> i32 {
        1
    }

    fn transactional(&self) -> bool {
        false
    }

    fn checksum(&self, _ctx: &SeedContext<'_>) -> String {
        resolve_seed_bundle_path()
            .map(|path| format!("{}", path.display()))
            .unwrap_or_default()
    }

    async fn run(
        &self,
        ctx: &SeedContext<'_>,
        state: &mut SeedState,
        _tx: &mut Option<&mut dyn Transaction>,
    ) -> anyhow::Result<()> {
        let Some(path) = resolve_seed_bundle_path() else {
            return Ok(());
        };

        let realm = state.require_realm()?;
        info!("Seeding Harbor bundle from {}", path.display());

        let bundle = read_bundle_from_path(&path)?;
        if bundle.manifest.export_type != HarborExportType::FullRealm {
            return Err(anyhow::anyhow!("Seed bundle must be a full realm export"));
        }

        let flow_ids = extract_flow_ids(&bundle);
        ctx.harbor_service
            .import_bundle(
                realm.id,
                HarborScope::FullRealm,
                bundle,
                false,
                ConflictPolicy::Overwrite,
            )
            .await?;

        if !flow_ids.is_empty() {
            let payload = build_realm_flow_payload(&flow_ids);
            ctx.realm_service.update_realm(realm.id, payload).await?;
        }

        state.mark_harbor_seeded();
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

    fn transactional(&self) -> bool {
        true
    }

    fn always_run(&self) -> bool {
        true
    }

    fn checksum(&self, _ctx: &SeedContext<'_>) -> String {
        "default_flows_v1".to_string()
    }

    async fn run(
        &self,
        ctx: &SeedContext<'_>,
        state: &mut SeedState,
        tx: &mut Option<&mut dyn Transaction>,
    ) -> anyhow::Result<()> {
        if state.is_harbor_seeded() {
            return Ok(());
        }
        let mut realm = state.require_realm()?;
        flows::ensure_default_flows(ctx, &mut realm, tx).await?;
        state.set_realm(realm);
        Ok(())
    }
}

#[async_trait]
impl Seeder for ThemeSeeder {
    fn name(&self) -> &'static str {
        "default_theme"
    }

    fn version(&self) -> i32 {
        1
    }

    fn always_run(&self) -> bool {
        true
    }

    fn checksum(&self, _ctx: &SeedContext<'_>) -> String {
        "default_theme_v1".to_string()
    }

    async fn run(
        &self,
        ctx: &SeedContext<'_>,
        state: &mut SeedState,
        _tx: &mut Option<&mut dyn Transaction>,
    ) -> anyhow::Result<()> {
        if state.is_harbor_seeded() {
            return Ok(());
        }
        let realm = state.require_realm()?;
        theme::ensure_default_theme(ctx, realm.id).await?;
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

    async fn run(
        &self,
        ctx: &SeedContext<'_>,
        state: &mut SeedState,
        _tx: &mut Option<&mut dyn Transaction>,
    ) -> anyhow::Result<()> {
        let realm = state.require_realm()?;
        oidc::seed_default_oidc_client(ctx, realm.id).await?;
        Ok(())
    }
}

fn resolve_seed_bundle_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("REAUTH_SEED_BUNDLE_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }

    let candidates = [
        "config/seed/system-init.reauth",
        "config/seed/system-init.zip",
        "config/seed/system-init.tar.gz",
        "config/seed/system-init.tgz",
        "config/seed/default.reauth",
    ];

    for candidate in candidates {
        let path = Path::new(candidate);
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }

    None
}

fn extract_flow_ids(bundle: &HarborBundle) -> HashMap<String, Uuid> {
    let mut map = HashMap::new();
    for resource in bundle.resources.iter().filter(|r| r.key == "flow") {
        let payload: SeedFlowPayload = match serde_json::from_value(resource.data.clone()) {
            Ok(payload) => payload,
            Err(_) => continue,
        };
        let Some(flow_id) = payload
            .flow_id
            .as_deref()
            .and_then(|value| Uuid::parse_str(value).ok())
        else {
            continue;
        };
        map.entry(payload.flow_type).or_insert(flow_id);
    }
    map
}

fn build_realm_flow_payload(flow_ids: &HashMap<String, Uuid>) -> UpdateRealmPayload {
    let browser = flow_ids.get("browser").copied();
    let registration = flow_ids.get("registration").copied();
    let direct = flow_ids
        .get("direct")
        .or_else(|| flow_ids.get("direct_grant"))
        .copied();
    let reset = flow_ids
        .get("reset")
        .or_else(|| flow_ids.get("reset_credentials"))
        .copied();

    UpdateRealmPayload {
        name: None,
        access_token_ttl_secs: None,
        refresh_token_ttl_secs: None,
        pkce_required_public_clients: None,
        lockout_threshold: None,
        lockout_duration_secs: None,
        browser_flow_id: browser.map(Some),
        registration_flow_id: registration.map(Some),
        direct_grant_flow_id: direct.map(Some),
        reset_credentials_flow_id: reset.map(Some),
    }
}
