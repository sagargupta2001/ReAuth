use crate::bootstrap::seed::context::SeedContext;
use crate::domain::oidc::OidcClient;
use rand::distributions::Alphanumeric;
use rand::Rng;
use tracing::info;
use uuid::Uuid;

pub async fn seed_default_oidc_client(ctx: &SeedContext<'_>, realm_id: Uuid) -> anyhow::Result<()> {
    let client_id = ctx.settings.default_oidc_client.client_id.clone();
    let desired_redirect_uris =
        serde_json::to_string(&ctx.settings.default_oidc_client.redirect_uris)?;
    let desired_web_origins = serde_json::to_string(&ctx.settings.default_oidc_client.web_origins)?;

    match ctx
        .oidc_service
        .find_client_by_client_id(&realm_id, &client_id)
        .await?
    {
        Some(mut client) => {
            let mut needs_update = false;

            if !client.managed_by_config {
                client.managed_by_config = true;
                needs_update = true;
            }

            if client.managed_by_config {
                if client.redirect_uris != desired_redirect_uris {
                    client.redirect_uris = desired_redirect_uris.clone();
                    needs_update = true;
                }

                if client.web_origins != desired_web_origins {
                    client.web_origins = desired_web_origins.clone();
                    needs_update = true;
                }
            }

            if needs_update {
                ctx.oidc_service.update_client_record(&client).await?;
                info!("Default OIDC client synced with config.");
            }
        }
        None => {
            info!("Seeding default OIDC client '{}'...", client_id);

            let secret: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(32)
                .map(char::from)
                .collect();

            let mut client = OidcClient {
                id: uuid::Uuid::new_v4(),
                realm_id,
                client_id: client_id.to_string(),
                client_secret: Some(secret),
                redirect_uris: desired_redirect_uris,
                scopes: "openid profile email".to_string(),
                web_origins: desired_web_origins,
                managed_by_config: true,
            };

            ctx.oidc_service.register_client(&mut client).await?;
            info!("Default OIDC client created.");
        }
    }

    Ok(())
}
