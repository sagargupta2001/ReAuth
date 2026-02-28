use crate::application::rbac_service::CreateRolePayload;
use crate::bootstrap::seed::context::SeedContext;
use crate::domain::permissions;
use tracing::{info, warn};
use uuid::Uuid;

/// Ensure the Admin User exists and has the Super Admin Role.
pub async fn seed_admin_user(ctx: &SeedContext<'_>, realm_id: Uuid) -> anyhow::Result<()> {
    let user = match ctx
        .user_service
        .find_by_username(&realm_id, &ctx.settings.default_admin.username)
        .await?
    {
        Some(user) => user,
        None => {
            info!(
                "No admin user found. Creating admin user '{}'...",
                &ctx.settings.default_admin.username
            );

            let user = ctx
                .user_service
                .create_user(
                    realm_id,
                    &ctx.settings.default_admin.username,
                    &ctx.settings.default_admin.password,
                )
                .await?;

            info!("Admin user created successfully.");
            warn!("SECURITY: Admin user created with the default password. Please log in and change it immediately.");
            user
        }
    };

    let role_name = "super_admin";

    let role = match ctx
        .rbac_service
        .find_role_by_name(realm_id, role_name)
        .await?
    {
        Some(role) => role,
        None => {
            ctx.rbac_service
                .create_role(
                    realm_id,
                    CreateRolePayload {
                        name: role_name.to_string(),
                        description: Some("System Administrator with full roles".to_string()),
                        client_id: None,
                    },
                )
                .await?
        }
    };

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
        "*",
    ];

    for perm in all_permissions {
        let _ = ctx
            .rbac_service
            .assign_permission_to_role(realm_id, role.id, perm.to_string())
            .await;
    }

    ctx.rbac_service
        .assign_role_to_user(realm_id, user.id, role.id)
        .await?;

    info!("Assigned 'super_admin' role to default admin user.");

    Ok(())
}
