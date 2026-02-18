use crate::application::realm_service::CreateRealmPayload;
use crate::bootstrap::seed::context::SeedContext;
use crate::constants::DEFAULT_REALM_NAME;
use crate::domain::realm::Realm;
use tracing::info;

pub async fn ensure_default_realm(ctx: &SeedContext<'_>) -> anyhow::Result<Realm> {
    if let Some(realm) = ctx.realm_service.find_by_name(DEFAULT_REALM_NAME).await? {
        return Ok(realm);
    }

    info!(
        "No default realm found. Creating '{}' realm...",
        DEFAULT_REALM_NAME
    );
    let payload = CreateRealmPayload {
        name: DEFAULT_REALM_NAME.to_string(),
    };
    let realm = ctx.realm_service.create_realm(payload).await?;
    info!("Default realm created successfully.");
    Ok(realm)
}
