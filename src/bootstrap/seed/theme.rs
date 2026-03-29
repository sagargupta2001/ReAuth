use crate::bootstrap::seed::context::SeedContext;
use crate::constants::DEFAULT_THEME_NAME;
use tracing::warn;
use uuid::Uuid;

pub async fn ensure_default_theme(ctx: &SeedContext<'_>, realm_id: Uuid) -> anyhow::Result<()> {
    let configured_name = ctx.settings.theme.default_theme_name.trim();
    let default_name = if configured_name.is_empty() {
        DEFAULT_THEME_NAME
    } else {
        configured_name
    };
    ctx.theme_service
        .ensure_default_theme_named(realm_id, default_name)
        .await?;

    let binding_name = ctx.settings.theme.default_binding_name.trim();
    if !binding_name.is_empty() && binding_name != default_name {
        let activated = ctx
            .theme_service
            .activate_theme_by_name(realm_id, binding_name)
            .await?;
        if !activated {
            warn!(
                "Configured default binding theme '{}' was not found or has no versions; keeping '{}' active.",
                binding_name, default_name
            );
        }
    }
    Ok(())
}
