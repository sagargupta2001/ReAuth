use crate::bootstrap::seed::context::SeedContext;
use uuid::Uuid;

pub async fn ensure_default_theme(ctx: &SeedContext<'_>, realm_id: Uuid) -> anyhow::Result<()> {
    ctx.theme_service.ensure_default_theme(realm_id).await?;
    Ok(())
}
