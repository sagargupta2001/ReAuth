use crate::adapters::web::auth_middleware::AuthUser;
use crate::adapters::web::validation::ValidatedJson;
use crate::application::user_credentials_service::UserCredentialsSummary;
use crate::domain::pagination::PageRequest;
use crate::domain::user::{User, UserDateTimeRangeFilter, UserListFilters};
use crate::domain::user_email::UserEmail;
use crate::error::{Error, Result};
use crate::AppState;
use axum::extract::{Path, Query};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use chrono::{DateTime, Days, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use tracing::warn;
use uuid::Uuid;
use validator::Validate;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// API response envelope for a single user — includes the primary email for
/// backward compatibility and the full emails list for the profile UI.
#[derive(Serialize)]
pub struct UserResponse {
    #[serde(flatten)]
    pub user: User,
    /// The primary email address, or the first available one (backward compat).
    pub email: Option<String>,
    pub emails: Vec<UserEmail>,
}

impl UserResponse {
    pub fn new(user: User, emails: Vec<UserEmail>) -> Self {
        let primary = emails
            .iter()
            .find(|e| e.is_primary)
            .or_else(|| emails.first())
            .map(|e| e.email.clone());
        Self {
            user,
            email: primary,
            emails,
        }
    }

    /// Lightweight variant used in list responses where only the primary email
    /// is needed (no full emails array).
    pub fn list_item(user: User, primary_email: Option<String>) -> Self {
        Self {
            user,
            email: primary_email,
            emails: vec![],
        }
    }
}

// ---------------------------------------------------------------------------
// Create user
// ---------------------------------------------------------------------------

#[derive(Deserialize, Validate)]
pub struct CreateUserPayload {
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    username: String,
    #[validate(email(message = "Email address is invalid"))]
    email: Option<String>,
    #[validate(length(max = 100, message = "Password must be between 8 and 100 characters"))]
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

    let user_result = state
        .user_service
        .create_user(
            realm.id,
            &payload.username,
            &payload.password,
            email.as_deref(),
            ignore_policies,
        )
        .await;

    let user = match user_result {
        Ok(user) => user,
        Err(Error::UsernameAlreadyExists) => {
            let mut fields = std::collections::HashMap::new();
            fields.insert(
                "username".to_string(),
                "Username is already taken".to_string(),
            );
            return Err(Error::FieldsValidation {
                message: "Validation failed".to_string(),
                fields,
            });
        }
        Err(Error::EmailAlreadyExists) => {
            let mut fields = std::collections::HashMap::new();
            fields.insert(
                "email".to_string(),
                "Email address is already in use".to_string(),
            );
            return Err(Error::FieldsValidation {
                message: "Validation failed".to_string(),
                fields,
            });
        }
        Err(err) => return Err(err),
    };

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

    let emails = state
        .user_email_service
        .list_emails(user.id)
        .await
        .unwrap_or_default();
    Ok((StatusCode::CREATED, Json(UserResponse::new(user, emails))))
}

// ---------------------------------------------------------------------------
// Get /me
// ---------------------------------------------------------------------------

pub async fn get_me_handler(
    Extension(AuthUser(user)): Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let emails = state
        .user_email_service
        .list_emails(user.id)
        .await
        .unwrap_or_default();
    Ok((StatusCode::OK, Json(UserResponse::new(user, emails))))
}

// ---------------------------------------------------------------------------
// List users
// ---------------------------------------------------------------------------

pub async fn list_users_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(query): Query<ListUsersQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let filters = query.filters();
    let page = state
        .user_service
        .list_users(realm.id, query.page, filters)
        .await?;

    // Bulk-fetch primary emails for all listed users in one query per user.
    // For list views this is acceptable (page sizes are small, ≤100).
    let mut items: Vec<UserResponse> = Vec::with_capacity(page.data.len());
    for user in page.data {
        let primary = state
            .user_email_service
            .get_primary_email(user.id)
            .await
            .ok()
            .flatten()
            .map(|e| e.email);
        items.push(UserResponse::list_item(user, primary));
    }

    let response = crate::domain::pagination::PageResponse::new(
        items,
        page.meta.total,
        page.meta.page,
        page.meta.per_page,
    );
    Ok((StatusCode::OK, Json(response)))
}

#[derive(Deserialize)]
pub struct ListUsersQuery {
    #[serde(flatten)]
    pub page: PageRequest,
    #[serde(default, deserialize_with = "deserialize_optional_text_filter")]
    pub filter_email: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_date_range_filter")]
    pub filter_created_at: Option<UserDateTimeRangeFilter>,
    #[serde(default, deserialize_with = "deserialize_optional_date_range_filter")]
    pub filter_last_sign_in_at: Option<UserDateTimeRangeFilter>,
}

impl ListUsersQuery {
    fn filters(&self) -> UserListFilters {
        UserListFilters {
            email: self.filter_email.clone(),
            created_at: self.filter_created_at.clone().unwrap_or_default(),
            last_sign_in_at: self.filter_last_sign_in_at.clone().unwrap_or_default(),
        }
    }
}

fn deserialize_optional_text_filter<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty()))
}

#[derive(Deserialize)]
struct DateRangeQueryParam {
    from: Option<String>,
    to: Option<String>,
}

fn deserialize_optional_date_range_filter<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<UserDateTimeRangeFilter>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    let Some(value) = value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    let Ok(range) = serde_json::from_str::<DateRangeQueryParam>(&value) else {
        return Ok(None);
    };

    let filter = UserDateTimeRangeFilter {
        from: range.from.as_deref().and_then(parse_filter_start),
        to_exclusive: range.to.as_deref().and_then(parse_filter_end_exclusive),
    };

    Ok((!filter.is_empty()).then_some(filter))
}

fn parse_filter_start(value: &str) -> Option<DateTime<Utc>> {
    parse_date_only(value)
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|datetime| Utc.from_utc_datetime(&datetime))
        .or_else(|| parse_rfc3339(value))
}

fn parse_filter_end_exclusive(value: &str) -> Option<DateTime<Utc>> {
    parse_date_only(value)
        .and_then(|date| date.checked_add_days(Days::new(1)))
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|datetime| Utc.from_utc_datetime(&datetime))
        .or_else(|| parse_rfc3339(value))
}

fn parse_date_only(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").ok()
}

fn parse_rfc3339(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value.trim())
        .ok()
        .map(|datetime| datetime.with_timezone(&Utc))
}

// ---------------------------------------------------------------------------
// Get single user
// ---------------------------------------------------------------------------

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
    let emails = state
        .user_email_service
        .list_emails(user.id)
        .await
        .unwrap_or_default();
    Ok((StatusCode::OK, Json(UserResponse::new(user, emails))))
}

// ---------------------------------------------------------------------------
// Delete users
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Update user profile (username only; emails go through /emails sub-resource)
// ---------------------------------------------------------------------------

#[derive(Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    pub username: Option<String>,
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
    if username.as_deref().is_some_and(|value| value.is_empty()) {
        return Err(Error::Validation("Username cannot be empty".to_string()));
    }

    if username.is_none() {
        return Err(Error::Validation("No updates provided".to_string()));
    }

    let user = state
        .user_service
        .update_profile(realm.id, id, username)
        .await?;

    let emails = state
        .user_email_service
        .list_emails(user.id)
        .await
        .unwrap_or_default();
    Ok((StatusCode::OK, Json(UserResponse::new(user, emails))))
}

// ---------------------------------------------------------------------------
// Email sub-resource: list
// ---------------------------------------------------------------------------

pub async fn list_user_emails_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    // Confirm the user belongs to this realm
    state.user_service.get_user_in_realm(realm.id, id).await?;

    let emails = state.user_email_service.list_emails(id).await?;
    Ok((StatusCode::OK, Json(emails)))
}

// ---------------------------------------------------------------------------
// Email sub-resource: add
// ---------------------------------------------------------------------------

#[derive(Deserialize, Validate)]
pub struct AddUserEmailPayload {
    #[validate(email(message = "Email address is invalid"))]
    pub email: String,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub is_verified: bool,
}

pub async fn add_user_email_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
    ValidatedJson(payload): ValidatedJson<AddUserEmailPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state.user_service.get_user_in_realm(realm.id, id).await?;

    let email = match state
        .user_email_service
        .add_email(
            id,
            realm.id,
            &payload.email,
            payload.is_primary,
            payload.is_verified,
        )
        .await
    {
        Ok(e) => e,
        Err(Error::EmailAlreadyExists) => {
            let mut fields = std::collections::HashMap::new();
            fields.insert(
                "email".to_string(),
                "Email address is already in use".to_string(),
            );
            return Err(Error::FieldsValidation {
                message: "Validation failed".to_string(),
                fields,
            });
        }
        Err(err) => return Err(err),
    };

    Ok((StatusCode::CREATED, Json(email)))
}

// ---------------------------------------------------------------------------
// Email sub-resource: remove
// ---------------------------------------------------------------------------

pub async fn remove_user_email_handler(
    State(state): State<AppState>,
    Path((realm_name, id, email_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state.user_service.get_user_in_realm(realm.id, id).await?;
    state.user_email_service.remove_email(id, email_id).await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "removed" })),
    ))
}

// ---------------------------------------------------------------------------
// Email sub-resource: set primary
// ---------------------------------------------------------------------------

pub async fn set_primary_email_handler(
    State(state): State<AppState>,
    Path((realm_name, id, email_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state.user_service.get_user_in_realm(realm.id, id).await?;
    state.user_email_service.set_primary(id, email_id).await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "updated" })),
    ))
}

// ---------------------------------------------------------------------------
// Email sub-resource: set verified
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SetVerifiedPayload {
    pub is_verified: bool,
}

pub async fn set_email_verified_handler(
    State(state): State<AppState>,
    Path((realm_name, id, email_id)): Path<(String, Uuid, Uuid)>,
    Json(payload): Json<SetVerifiedPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state.user_service.get_user_in_realm(realm.id, id).await?;
    state
        .user_email_service
        .set_verified(id, email_id, payload.is_verified)
        .await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "updated" })),
    ))
}

// ---------------------------------------------------------------------------
// Credentials handlers (unchanged, kept here for co-location)
// ---------------------------------------------------------------------------

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

pub async fn unlink_user_federated_identity_handler(
    State(state): State<AppState>,
    Extension(AuthUser(current_user)): Extension<AuthUser>,
    Path((realm_name, id, federated_identity_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .user_credentials_service
        .unlink_federated_identity(realm.id, Some(current_user.id), id, federated_identity_id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "unlinked" })),
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
