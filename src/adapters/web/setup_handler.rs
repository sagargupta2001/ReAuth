use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::adapters::web::validation::ValidatedJson;
use crate::application::rbac_service::CreateRolePayload;
use crate::application::realm_service::CreateRealmPayload;
use crate::constants::DEFAULT_REALM_NAME;
use crate::domain::permissions;
use crate::error::{Error, Result};
use crate::AppState;

#[derive(Serialize)]
pub struct SetupStatusResponse {
    pub required: bool,
}

#[derive(Serialize)]
pub struct SetupCompleteResponse {
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Deserialize, Validate)]
pub struct SetupPayload {
    #[validate(length(min = 8, message = "Setup token is required"))]
    token: String,
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    username: String,
    #[validate(length(
        min = 8,
        max = 100,
        message = "Password must be between 8 and 100 characters"
    ))]
    password: String,
}

pub async fn setup_status_handler(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let setup = state.setup_state.read().await;
    Ok((
        StatusCode::OK,
        Json(SetupStatusResponse {
            required: setup.required,
        }),
    ))
}

pub async fn setup_handler(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<SetupPayload>,
) -> Result<impl IntoResponse> {
    {
        let setup = state.setup_state.read().await;
        if !setup.required {
            return Err(Error::SecurityViolation(
                "Setup has already been completed.".to_string(),
            ));
        }
        if setup.token.as_deref() != Some(payload.token.as_str()) {
            return Err(Error::SecurityViolation("Invalid setup token.".to_string()));
        }
    }

    let realm = match state.realm_service.find_by_name(DEFAULT_REALM_NAME).await? {
        Some(realm) => realm,
        None => {
            let payload = CreateRealmPayload {
                name: DEFAULT_REALM_NAME.to_string(),
            };
            state.realm_service.create_realm(payload).await?
        }
    };

    if state.user_service.count_users_in_realm(realm.id).await? > 0 {
        return Err(Error::SecurityViolation(
            "Setup has already been completed.".to_string(),
        ));
    }

    let user = state
        .user_service
        .create_user(realm.id, &payload.username, &payload.password)
        .await?;

    let role_name = "super_admin";
    let role = match state
        .rbac_service
        .find_role_by_name(realm.id, role_name)
        .await?
    {
        Some(role) => role,
        None => {
            state
                .rbac_service
                .create_role(
                    realm.id,
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
        let _ = state
            .rbac_service
            .assign_permission_to_role(realm.id, role.id, perm.to_string())
            .await;
    }

    state
        .rbac_service
        .assign_role_to_user(realm.id, user.id, role.id)
        .await?;

    {
        let mut setup = state.setup_state.write().await;
        setup.required = false;
        setup.token = None;
    }

    tracing::info!("Setup completed. System sealed.");

    Ok((
        StatusCode::CREATED,
        Json(SetupCompleteResponse {
            user_id: user.id,
            username: user.username,
        }),
    ))
}
