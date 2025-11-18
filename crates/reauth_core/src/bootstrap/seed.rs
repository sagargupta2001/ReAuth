use crate::application::realm_service::{CreateRealmPayload, RealmService};
use crate::application::user_service::UserService;
use crate::config::DefaultAdminConfig;
use crate::constants::DEFAULT_REALM_NAME;
use crate::domain::auth_flow::{AuthFlow, AuthFlowStep};
use crate::ports::flow_repository::FlowRepository;
use std::sync::Arc;
use tracing::{info, warn};

pub async fn seed_database(
    realm_service: &Arc<RealmService>,
    user_service: &Arc<UserService>,
    flow_repo: &Arc<dyn FlowRepository>,
    admin_config: &DefaultAdminConfig,
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
        .find_by_username(&admin_config.username)
        .await?
        .is_none()
    {
        info!(
            "No admin user found. Creating admin user '{}'...",
            &admin_config.username
        );
        user_service
            .create_user(&admin_config.username, &admin_config.password)
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

    Ok(())
}
