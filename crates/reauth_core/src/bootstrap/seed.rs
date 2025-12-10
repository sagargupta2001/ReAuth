use crate::application::oidc_service::OidcService;
use crate::application::realm_service::{CreateRealmPayload, RealmService, UpdateRealmPayload};
use crate::application::user_service::UserService;
use crate::config::Settings;
use crate::constants::DEFAULT_REALM_NAME;
use crate::domain::auth_flow::{AuthFlow, AuthFlowStep};
use crate::domain::flow::FlowDraft;
use crate::domain::oidc::OidcClient;
use crate::ports::flow_repository::FlowRepository;
use crate::ports::flow_store::FlowStore;
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

pub async fn seed_database(
    realm_service: &Arc<RealmService>,
    user_service: &Arc<UserService>,
    flow_repo: &Arc<dyn FlowRepository>,
    flow_store: &Arc<dyn FlowStore>,
    settings: &Settings,
    oidc_service: &Arc<OidcService>,
) -> anyhow::Result<()> {
    // 1. Check for the default realm
    if realm_service
        .find_by_name(DEFAULT_REALM_NAME)
        .await?
        .is_none()
    {
        info!(
            "No default realm found. Creating '{}' realm...",
            DEFAULT_REALM_NAME
        );
        let payload = CreateRealmPayload {
            name: DEFAULT_REALM_NAME.to_string(),
        };
        realm_service.create_realm(payload).await?;
        info!("Default realm created successfully.");
    }

    let mut realm = if let Some(r) = realm_service.find_by_name(DEFAULT_REALM_NAME).await? {
        r
    } else {
        info!(
            "No default realm found. Creating '{}' realm...",
            DEFAULT_REALM_NAME
        );
        let payload = CreateRealmPayload {
            name: DEFAULT_REALM_NAME.to_string(),
        };
        let r = realm_service.create_realm(payload).await?;
        info!("Default realm created successfully.");
        r
    };

    // 2. Seed Built-in Flows
    let browser_flow_id = ensure_flow(
        flow_repo,
        flow_store,
        &realm.id,
        "browser-login",
        "Browser Login",
        "browser",
        vec!["builtin-password-auth"],
    )
    .await?;

    // Direct Grant -> Needs Password Auth (usually same authenticator logic for MVP)
    let direct_flow_id = ensure_flow(
        flow_repo,
        flow_store,
        &realm.id,
        "direct-grant",
        "Direct Grant",
        "direct",
        vec!["builtin-password-auth"],
    )
    .await?;

    // Registration -> Needs Registration Profile (Placeholder for now)
    let registration_flow_id = ensure_flow(
        flow_repo,
        flow_store,
        &realm.id,
        "registration",
        "Registration",
        "registration",
        vec![], // Empty for now
    )
    .await?;

    // Reset Credentials -> Needs Email verification (Placeholder for now)
    let reset_flow_id = ensure_flow(
        flow_repo,
        flow_store,
        &realm.id,
        "reset-credentials",
        "Reset Credentials",
        "reset",
        vec![], // Empty for now
    )
    .await?;

    // 3. Link Defaults to Realm
    let mut needs_update = false;

    // We use a separate struct to track updates because we can't just mutate `realm`
    // and pass it to `update_realm` directly if the service expects a Payload struct.
    let mut update_payload = UpdateRealmPayload {
        name: None,
        access_token_ttl_secs: None,
        refresh_token_ttl_secs: None,
        // We will add these fields to UpdateRealmPayload in step 3 below
        browser_flow_id: None,
        registration_flow_id: None,
        direct_grant_flow_id: None,
        reset_credentials_flow_id: None,
    };

    if realm.browser_flow_id.is_none() {
        update_payload.browser_flow_id = Some(Some(browser_flow_id));
        needs_update = true;
    }
    if realm.direct_grant_flow_id.is_none() {
        update_payload.direct_grant_flow_id = Some(Some(direct_flow_id));
        needs_update = true;
    }
    if realm.registration_flow_id.is_none() {
        update_payload.registration_flow_id = Some(Some(registration_flow_id));
        needs_update = true;
    }
    if realm.reset_credentials_flow_id.is_none() {
        update_payload.reset_credentials_flow_id = Some(Some(reset_flow_id));
        needs_update = true;
    }

    if needs_update {
        realm_service.update_realm(realm.id, update_payload).await?;
        info!("Updated realm with default flow bindings.");
        // Reload realm to get updated state for later steps if needed
        realm = realm_service.find_by_id(realm.id).await?.unwrap();
    }

    if user_service
        .find_by_username(&realm.id, &settings.default_admin.username)
        .await?
        .is_none()
    {
        info!(
            "No admin user found. Creating admin user '{}'...",
            &settings.default_admin.username
        );
        user_service
            .create_user(
                realm.id,
                &settings.default_admin.username,
                &settings.default_admin.password,
            )
            .await?;

        info!("Admin user created successfully.");
        warn!("SECURITY: Admin user created with the default password. Please log in and change it immediately.");
    }

    //  SEED DEFAULT OIDC CLIENT
    let client_id = "reauth-admin";
    // Allow both dev and prod URLs
    let check_uri = &settings
        .default_oidc_client
        .redirect_uris
        .first()
        .map(|s| s.as_str())
        .unwrap_or("");

    if oidc_service
        .validate_client(
            &realm.id,
            &settings.default_oidc_client.client_id,
            check_uri,
        )
        .await
        .is_err()
    {
        info!("Seeding default OIDC client '{}'...", client_id);

        // Note: OidcService doesn't have a `create_client` method exposed yet,
        // so we might need to use the repo directly or add a method to the service.
        // For Clean Architecture, let's add a helper to OidcService.
        // For this snippet, I'll assume we added `register_client` to OidcService.

        // *TEMPORARY FIX*: Use the repo directly via a new Service method or just
        // assume the service has a create method. Let's add it to OidcService below.

        let secret: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let mut client = OidcClient {
            id: uuid::Uuid::new_v4(),
            realm_id: realm.id,
            client_id: client_id.to_string(),
            client_secret: Some(secret), // Public client (SPA)
            redirect_uris: serde_json::to_string(&settings.default_oidc_client.redirect_uris)?,
            scopes: "openid profile email".to_string(),
        };

        // You need to expose a create method in OidcService, see Step 5 below.
        oidc_service.register_client(&mut client).await?;
        info!("Default OIDC client created.");
    }

    Ok(())
}

async fn ensure_flow(
    flow_repo: &Arc<dyn FlowRepository>,
    flow_store: &Arc<dyn FlowStore>, // ✅ Added FlowStore
    realm_id: &Uuid,
    name: &str,
    alias: &str,
    type_: &str,
    default_steps: Vec<&str>,
) -> anyhow::Result<Uuid> {
    // 1. Resolve Flow ID (Find existing or Create new)
    let flow_id = if let Some(flow) = flow_repo.find_flow_by_name(realm_id, name).await? {
        flow.id
    } else {
        info!("Seeding built-in flow: {}", alias);
        let new_id = Uuid::new_v4();
        let flow = AuthFlow {
            id: new_id,
            realm_id: *realm_id,
            name: name.to_string(),
            alias: alias.to_string(),
            description: Some(format!("Default {} flow", alias)),
            r#type: type_.to_string(),
            built_in: true,
        };
        flow_repo.create_flow(&flow, None).await?;

        // Create Default Runtime Steps (Legacy execution logic)
        for (index, authenticator_name) in default_steps.iter().enumerate() {
            let step = AuthFlowStep {
                id: Uuid::new_v4(),
                flow_id: new_id,
                authenticator_name: authenticator_name.to_string(),
                priority: index as i64 * 10,
                requirement: "REQUIRED".to_string(),
                config: None,
                parent_step_id: None,
            };
            flow_repo.add_step_to_flow(&step, None).await?;
        }
        new_id
    };

    // 2. ✅ Ensure Draft Exists (For Visual Builder / Details Page)
    // Check if we already have a draft for this ID
    if flow_store.get_draft_by_id(&flow_id).await?.is_none() {
        info!("Seeding visual graph for flow: {}", alias);

        let graph_json = generate_default_graph(type_);

        let draft = FlowDraft {
            id: flow_id, // Use SAME ID as the runtime flow
            realm_id: *realm_id,
            name: alias.to_string(),
            description: Some(format!("Visual draft for {}", alias)),
            graph_json,
            flow_type: type_.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        flow_store.create_draft(&draft).await?;
    }

    Ok(flow_id)
}

fn generate_default_graph(flow_type: &str) -> String {
    // Basic templates
    match flow_type {
        "browser" => r#"{
            "nodes": [
                { "id": "start", "type": "terminal", "position": { "x": 250, "y": 0 }, "data": { "label": "Start", "config": {} } },
                { "id": "auth-1", "type": "authenticator", "position": { "x": 250, "y": 150 }, "data": { "label": "Username Password", "config": {} } },
                { "id": "end", "type": "terminal", "position": { "x": 250, "y": 300 }, "data": { "label": "Success", "config": {} } }
            ],
            "edges": [
                { "id": "e1", "source": "start", "target": "auth-1" },
                { "id": "e2", "source": "auth-1", "target": "end" }
            ]
        }"#.to_string(),

        "direct" => r#"{
            "nodes": [
                { "id": "auth-1", "type": "authenticator", "position": { "x": 250, "y": 50 }, "data": { "label": "Direct Grant Auth", "config": {} } },
                { "id": "end", "type": "terminal", "position": { "x": 250, "y": 200 }, "data": { "label": "Success", "config": {} } }
            ],
            "edges": [
                { "id": "e1", "source": "auth-1", "target": "end" }
            ]
        }"#.to_string(),

        // Default empty structure
        _ => r#"{ "nodes": [], "edges": [] }"#.to_string(),
    }
}
