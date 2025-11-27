use crate::application::oidc_service::OidcService;
use crate::application::realm_service::{CreateRealmPayload, RealmService};
use crate::application::user_service::UserService;
use crate::config::Settings;
use crate::constants::DEFAULT_REALM_NAME;
use crate::domain::auth_flow::{AuthFlow, AuthFlowStep};
use crate::domain::oidc::OidcClient;
use crate::ports::flow_repository::FlowRepository;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::sync::Arc;
use tracing::{info, warn};

pub async fn seed_database(
    realm_service: &Arc<RealmService>,
    user_service: &Arc<UserService>,
    flow_repo: &Arc<dyn FlowRepository>,
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

    // 2. Check for the admin user
    if user_service
        .find_by_username(&settings.default_admin.username)
        .await?
        .is_none()
    {
        info!(
            "No admin user found. Creating admin user '{}'...",
            &settings.default_admin.username
        );
        user_service
            .create_user(
                &settings.default_admin.username,
                &settings.default_admin.password,
            )
            .await?;
        info!("Admin user created successfully.");
        warn!(
            "SECURITY: Admin user created with the default password. Please log in and change it immediately."
        );
    }

    let realm = if let Some(r) = realm_service.find_by_name(DEFAULT_REALM_NAME).await? {
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

    // 3. Check/Create Default Flow (NEW LOGIC)
    if flow_repo
        .find_flow_by_name(&realm.id, "browser-login")
        .await?
        .is_none()
    {
        info!("Seeding default 'browser-login' flow...");

        let flow_id = uuid::Uuid::new_v4();
        let flow = AuthFlow {
            id: flow_id,
            realm_id: realm.id,
            name: "browser-login".to_string(),
        };
        flow_repo.create_flow(&flow).await?;

        // Add the password step
        let step = AuthFlowStep {
            id: uuid::Uuid::new_v4(),
            flow_id,
            authenticator_name: "builtin-password-auth".to_string(),
            priority: 0,
        };
        flow_repo.add_step_to_flow(&step).await?;

        info!("Default 'browser-login' flow created.");
    }

    // 4. --- SEED DEFAULT OIDC CLIENT ---
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
