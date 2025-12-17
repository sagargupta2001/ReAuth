use crate::application::flow_manager::FlowManager;
use crate::application::oidc_service::OidcService;
use crate::application::rbac_service::{CreateRolePayload, RbacService};
use crate::application::realm_service::{CreateRealmPayload, RealmService, UpdateRealmPayload};
use crate::application::user_service::UserService;
use crate::config::Settings;
use crate::constants::DEFAULT_REALM_NAME;
use crate::domain::auth_flow::{AuthFlow, AuthFlowStep};
use crate::domain::flow::models::FlowDraft;
use crate::domain::oidc::OidcClient;
use crate::domain::permissions::permissions;
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
    flow_manager: &Arc<FlowManager>,
    settings: &Settings,
    oidc_service: &Arc<OidcService>,
    rbac_service: &Arc<RbacService>,
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
        flow_manager,
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
        flow_manager,
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
        flow_manager,
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
        flow_manager,
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

    // Seed Admin User & RBAC [EXTRACTED]
    seed_admin_user(realm.id, settings, user_service, rbac_service).await?;

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

        let web_origins_json = serde_json::to_string(&settings.default_oidc_client.web_origins)
            .expect("Failed to serialize default web origins");

        let mut client = OidcClient {
            id: uuid::Uuid::new_v4(),
            realm_id: realm.id,
            client_id: client_id.to_string(),
            client_secret: Some(secret), // Public client (SPA)
            redirect_uris: serde_json::to_string(&settings.default_oidc_client.redirect_uris)?,
            scopes: "openid profile email".to_string(),
            web_origins: web_origins_json,
        };

        // You need to expose a create method in OidcService, see Step 5 below.
        oidc_service.register_client(&mut client).await?;
        info!("Default OIDC client created.");
    }

    Ok(())
}

async fn ensure_flow(
    flow_repo: &Arc<dyn FlowRepository>,
    flow_store: &Arc<dyn FlowStore>,
    flow_manager: &Arc<FlowManager>,
    realm_id: &Uuid,
    name: &str,
    alias: &str,
    type_: &str,
    default_steps: Vec<&str>,
) -> anyhow::Result<Uuid> {
    // Ensure Runtime Flow Exists
    let flow_id = if let Some(flow) = flow_repo.find_flow_by_name(realm_id, name).await? {
        flow.id
    } else {
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

        // Create Steps
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

    // Ensure Visual Draft Exists
    let draft_exists = flow_store.get_draft_by_id(&flow_id).await?.is_some();

    // Prepare the Draft Object
    let graph_json = FlowManager::generate_default_graph(type_);
    let draft_obj = FlowDraft {
        id: flow_id,
        realm_id: *realm_id,
        name: alias.to_string(),
        description: Some(format!("Visual draft for {}", alias)),
        graph_json: graph_json.clone(),
        flow_type: type_.to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    if !draft_exists {
        flow_store.create_draft(&draft_obj).await?;
    } else {
        info!("Step 2: Draft already exists for {}", alias);
    }

    // Ensure Active Version Exists
    let latest_version = flow_store.get_latest_version_number(&flow_id).await?;

    // Only consider it "published" if the version is > 0.
    // This treats 'None' and 'Some(0)' as "Not Published Yet".
    let has_valid_version = latest_version.unwrap_or(0) > 0;

    if !has_valid_version {
        match flow_manager.publish_flow(*realm_id, flow_id).await {
            Ok(v) => {
                flow_store.create_draft(&draft_obj).await?;
            }
            Err(e) => {
                tracing::error!("Step 4: FAILURE - Could not publish {}: {:?}", alias, e);
            }
        }
    }

    Ok(flow_id)
}

/// Helper to ensure the Admin User exists and has the Super Admin Role
async fn seed_admin_user(
    realm_id: Uuid,
    settings: &Settings,
    user_service: &UserService,
    rbac_service: &RbacService,
) -> anyhow::Result<()> {
    // 1. Check if admin exists
    if user_service
        .find_by_username(&realm_id, &settings.default_admin.username)
        .await?
        .is_some()
    {
        // Admin already exists, assume seeded.
        return Ok(());
    }

    info!(
        "No admin user found. Creating admin user '{}'...",
        &settings.default_admin.username
    );

    // 2. Create the User
    let user = user_service
        .create_user(
            realm_id,
            &settings.default_admin.username,
            &settings.default_admin.password,
        )
        .await?;

    info!("Admin user created successfully.");
    warn!("SECURITY: Admin user created with the default password. Please log in and change it immediately.");

    // 3. Create 'Super Admin' Role
    let role_name = "super_admin";

    // Attempt to create. If it fails (exists), we try to find it.
    let role = match rbac_service
        .create_role(
            realm_id,
            CreateRolePayload {
                name: role_name.to_string(),
                description: Some("System Administrator with full access".to_string()),
            },
        )
        .await
    {
        Ok(r) => r,
        Err(_) => {
            // If creation failed, it likely exists.
            // Note: Ideally, you'd add find_role_by_name to RbacService.
            // For now, assuming you might not have it exposed,
            // you might need to query repo or just skip.
            // Best Practice: Expose find_role_by_name in RbacService.
            info!("Role '{}' likely already exists.", role_name);

            // To be safe, we stop here if we can't find the ID.
            // Ideally: rbac_service.find_role_by_name(realm_id, role_name).await?.unwrap()
            return Ok(());
        }
    };

    // 4. Assign ALL System Permissions to this Role
    let all_permissions = vec![
        permissions::CLIENT_READ,
        permissions::CLIENT_CREATE,
        permissions::CLIENT_UPDATE,
        permissions::REALM_READ,
        permissions::REALM_WRITE,
        permissions::RBAC_READ,
        permissions::RBAC_WRITE,
        permissions::USER_READ,
        permissions::USER_WRITE,
        // Add "*" super-wildcard
        "*",
    ];

    for perm in all_permissions {
        // We ignore errors here (e.g. if permission already assigned)
        let _ = rbac_service
            .assign_permission_to_role(realm_id, perm.to_string(), role.id)
            .await;
    }

    // 5. Assign the Role to the User
    rbac_service
        .assign_role_to_user(realm_id, user.id, role.id)
        .await?;

    info!("Assigned 'super_admin' role to default admin user.");

    Ok(())
}
