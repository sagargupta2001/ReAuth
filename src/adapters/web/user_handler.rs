use crate::adapters::web::auth_middleware::AuthUser;
use crate::adapters::web::validation::ValidatedJson;
use crate::application::user_credentials_service::UserCredentialsSummary;
use crate::domain::pagination::PageRequest;
use crate::error::{Error, Result};
use crate::AppState;
use axum::extract::{Path, Query};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::Deserialize;
use tracing::warn;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateUserPayload {
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    username: String,
    #[validate(email(message = "Email address is invalid"))]
    email: Option<String>,
    #[validate(length(
        max = 100,
        message = "Password must be between 8 and 100 characters"
    ))]
    password: String,
    ignore_password_policies: Option<bool>,
}

pub async fn create_user_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    ValidatedJson(payload): ValidatedJson<CreateUserPayload>,
) -> Result<impl IntoResponse> {
    if state.is_setup_required().await {
        return Err(Error::SecurityViolation(
            "Initial setup is required before creating users.".to_string(),
        ));
    }

    let ignore_policies = payload.ignore_password_policies.unwrap_or(false);
    if !ignore_policies && payload.password.len() < 8 {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "password".to_string(),
            "Password must be between 8 and 100 characters".to_string(),
        );
        return Err(Error::FieldsValidation {
            message: "Validation failed".to_string(),
            fields,
        });
    }

    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;
    let capabilities = crate::application::realm_policy::RealmCapabilities::from_realm(&realm);

    let email = payload
        .email
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
        
    let ignore_policies = payload.ignore_password_policies.unwrap_or(false);

    let user = state
        .user_service
        .create_user(
            realm.id,
            &payload.username,
            &payload.password,
            email.as_deref(),
            ignore_policies,
        )
        .await?;

    for role_id in capabilities.default_registration_role_ids {
        if let Err(err) = state
            .rbac_service
            .assign_role_to_user(realm.id, user.id, role_id)
            .await
        {
            warn!(
                "Failed to assign default registration role {}: {}",
                role_id, err
            );
        }
    }

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn get_me_handler(
    // Get the `AuthUser` that the middleware inserted
    Extension(AuthUser(user)): Extension<AuthUser>,
) -> Result<impl IntoResponse> {
    // The user is already authenticated and fetched. Just return it.
    Ok((StatusCode::OK, Json(user)))
}

pub async fn list_users_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(req): Query<PageRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let response = state.user_service.list_users(realm.id, req).await?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_user_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let user = state.user_service.get_user_in_realm(realm.id, id).await?;

    Ok((StatusCode::OK, Json(user)))
}

#[derive(Deserialize, Validate)]
pub struct DeleteUsersRequest {
    pub user_ids: Vec<Uuid>,
}

pub async fn delete_users_handler(
    State(state): State<AppState>,
    Extension(AuthUser(current_user)): Extension<AuthUser>,
    Path(realm_name): Path<String>,
    Json(payload): Json<DeleteUsersRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    if payload.user_ids.contains(&current_user.id) {
        return Err(Error::Validation(
            "You cannot delete your own account.".to_string(),
        ));
    }

    let count = state
        .user_service
        .delete_users(&realm.id, &payload.user_ids)
        .await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "deleted", "count": count })),
    ))
}

#[derive(Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    pub username: Option<String>,
    #[validate(email(message = "Email address is invalid"))]
    pub email: Option<String>,
}

pub async fn update_user_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
    ValidatedJson(payload): ValidatedJson<UpdateUserRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let username = payload.username.map(|value| value.trim().to_string());
    let email = payload.email.map(|value| value.trim().to_string());
    if username.as_deref().is_some_and(|value| value.is_empty()) {
        return Err(Error::Validation("Username cannot be empty".to_string()));
    }

    let email_update = email.map(|value| if value.is_empty() { None } else { Some(value) });

    if username.is_none() && email_update.is_none() {
        return Err(Error::Validation("No updates provided".to_string()));
    }

    let user = state
        .user_service
        .update_profile(realm.id, id, username, email_update)
        .await?;

    Ok((StatusCode::OK, Json(user)))
}

#[derive(Deserialize)]
pub struct UpdateUserPasswordRequest {
    pub password: String,
}

#[derive(Deserialize)]
pub struct UpdatePasskeyMetadataRequest {
    pub friendly_name: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePasswordPolicyRequest {
    pub force_reset_on_next_login: Option<bool>,
    pub password_login_disabled: Option<bool>,
}

pub async fn list_user_credentials_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let credentials: UserCredentialsSummary = state
        .user_credentials_service
        .list_credentials(realm.id, id)
        .await?;
    Ok((StatusCode::OK, Json(credentials)))
}

pub async fn update_user_password_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
    Json(payload): Json<UpdateUserPasswordRequest>,
) -> Result<impl IntoResponse> {
    if payload.password.len() < 8 || payload.password.len() > 100 {
        return Err(Error::Validation(
            "Password must be between 8 and 100 characters".to_string(),
        ));
    }

    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .user_credentials_service
        .update_password(realm.id, id, &payload.password)
        .await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "updated" })),
    ))
}

pub async fn revoke_user_passkey_handler(
    State(state): State<AppState>,
    Path((realm_name, id, credential_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .user_credentials_service
        .revoke_passkey(realm.id, id, credential_id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "revoked" })),
    ))
}

pub async fn update_user_passkey_metadata_handler(
    State(state): State<AppState>,
    Path((realm_name, id, credential_id)): Path<(String, Uuid, Uuid)>,
    Json(payload): Json<UpdatePasskeyMetadataRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let friendly_name = payload
        .friendly_name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    state
        .user_credentials_service
        .rename_passkey(realm.id, id, credential_id, friendly_name)
        .await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "updated" })),
    ))
}

pub async fn update_user_password_policy_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
    Json(payload): Json<UpdatePasswordPolicyRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .user_credentials_service
        .update_password_policy(
            realm.id,
            id,
            payload.force_reset_on_next_login,
            payload.password_login_disabled,
        )
        .await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "updated" })),
    ))
}
