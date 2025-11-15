use crate::application::realm_service::{CreateRealmPayload, RealmService};
use crate::application::user_service::UserService;
use crate::config::DefaultAdminConfig;
use crate::constants::DEFAULT_REALM_NAME;
use std::sync::Arc;
use tracing::{info, warn};

pub async fn seed_database(
    realm_service: &Arc<RealmService>,
    user_service: &Arc<UserService>,
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

    Ok(())
}
