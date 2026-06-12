use crate::adapters::web::auth_middleware::AuthUser;
use crate::adapters::web::validation::ValidatedJson;
use crate::application::user_credentials_service::UserCredentialsSummary;
use crate::application::user_service::{admin_metadata_response, UserMetadataVisibility};
use crate::domain::pagination::PageRequest;
use crate::domain::user::{User, UserDateTimeRangeFilter, UserListFilters};
use crate::domain::user_email::UserEmail;
use crate::domain::user_phone_number::UserPhoneNumber;
use crate::error::{Error, Result};
use crate::AppState;
use axum::extract::{Path, Query};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use chrono::{DateTime, Days, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
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
    /// The primary phone number, or the first available one.
    pub phone_number: Option<String>,
    pub phone_numbers: Vec<UserPhoneNumber>,
    pub public_metadata: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_metadata: Option<Value>,
    pub unsafe_metadata: Value,
}

impl UserResponse {
    pub fn new_admin(
        user: User,
        emails: Vec<UserEmail>,
        phone_numbers: Vec<UserPhoneNumber>,
    ) -> Self {
        Self::from_parts(user, emails, phone_numbers, true)
    }

    pub fn new_self(
        user: User,
        emails: Vec<UserEmail>,
        phone_numbers: Vec<UserPhoneNumber>,
    ) -> Self {
        Self::from_parts(user, emails, phone_numbers, false)
    }

    fn from_parts(
        user: User,
        emails: Vec<UserEmail>,
        phone_numbers: Vec<UserPhoneNumber>,
        include_private_metadata: bool,
    ) -> Self {
        let primary = emails
            .iter()
            .find(|e| e.is_primary)
            .or_else(|| emails.first())
            .map(|e| e.email.clone());
        let primary_phone_number = phone_numbers
            .iter()
            .find(|phone_number| phone_number.is_primary)
            .or_else(|| phone_numbers.first())
            .map(|phone_number| phone_number.phone_number.clone());
        let metadata = admin_metadata_response(&user, include_private_metadata);
        Self {
            user,
            email: primary,
            emails,
            phone_number: primary_phone_number,
            phone_numbers,
            public_metadata: metadata.public_metadata,
            private_metadata: include_private_metadata.then_some(metadata.private_metadata),
            unsafe_metadata: metadata.unsafe_metadata,
        }
    }

    /// Lightweight variant used in list responses where only the primary email
    /// is needed (no full emails array).
    pub fn list_item(user: User, primary_email: Option<String>) -> Self {
        Self {
            user,
            email: primary_email,
            emails: vec![],
            phone_number: None,
            phone_numbers: vec![],
            public_metadata: serde_json::json!({}),
            private_metadata: None,
            unsafe_metadata: serde_json::json!({}),
        }
    }
}

async fn user_response_from_user(state: &AppState, user: User) -> Result<UserResponse> {
    let emails = state
        .user_email_service
        .list_emails(user.id)
        .await
        .unwrap_or_default();
    let phone_numbers = state
        .user_phone_number_service
        .list_phone_numbers(user.id)
        .await
        .unwrap_or_default();
    Ok(UserResponse::new_admin(user, emails, phone_numbers))
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
    let phone_numbers = state
        .user_phone_number_service
        .list_phone_numbers(user.id)
        .await
        .unwrap_or_default();
    Ok((
        StatusCode::CREATED,
        Json(UserResponse::new_self(user, emails, phone_numbers)),
    ))
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
    let phone_numbers = state
        .user_phone_number_service
        .list_phone_numbers(user.id)
        .await
        .unwrap_or_default();
    Ok((
        StatusCode::OK,
        Json(UserResponse::new_self(user, emails, phone_numbers)),
    ))
}

#[derive(Deserialize)]
pub struct UpdateUserMetadataPayload {
    pub metadata: Value,
}

pub async fn get_me_metadata_handler(
    Extension(AuthUser(user)): Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let metadata = state
        .user_service
        .get_self_metadata(user.realm_id, user.id)
        .await?;
    Ok((StatusCode::OK, Json(metadata)))
}

pub async fn update_me_unsafe_metadata_handler(
    Extension(AuthUser(user)): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateUserMetadataPayload>,
) -> Result<impl IntoResponse> {
    state
        .user_service
        .update_metadata(
            user.realm_id,
            user.id,
            UserMetadataVisibility::Unsafe,
            payload.metadata,
        )
        .await?;

    let metadata = state
        .user_service
        .get_self_metadata(user.realm_id, user.id)
        .await?;
    Ok((StatusCode::OK, Json(metadata)))
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
    Ok((
        StatusCode::OK,
        Json(user_response_from_user(&state, user).await?),
    ))
}

pub async fn get_user_metadata_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let metadata = state
        .user_service
        .get_admin_metadata(realm.id, id, true)
        .await?;
    Ok((StatusCode::OK, Json(metadata)))
}

pub async fn update_user_public_metadata_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
    Json(payload): Json<UpdateUserMetadataPayload>,
) -> Result<impl IntoResponse> {
    update_user_metadata(
        state,
        realm_name,
        id,
        UserMetadataVisibility::Public,
        payload.metadata,
    )
    .await
}

pub async fn update_user_private_metadata_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
    Json(payload): Json<UpdateUserMetadataPayload>,
) -> Result<impl IntoResponse> {
    update_user_metadata(
        state,
        realm_name,
        id,
        UserMetadataVisibility::Private,
        payload.metadata,
    )
    .await
}

pub async fn update_user_unsafe_metadata_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
    Json(payload): Json<UpdateUserMetadataPayload>,
) -> Result<impl IntoResponse> {
    update_user_metadata(
        state,
        realm_name,
        id,
        UserMetadataVisibility::Unsafe,
        payload.metadata,
    )
    .await
}

async fn update_user_metadata(
    state: AppState,
    realm_name: String,
    user_id: Uuid,
    visibility: UserMetadataVisibility,
    metadata: Value,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let metadata = state
        .user_service
        .update_metadata(realm.id, user_id, visibility, metadata)
        .await?;
    Ok((StatusCode::OK, Json(metadata)))
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

pub async fn lock_user_handler(
    State(state): State<AppState>,
    Extension(AuthUser(current_user)): Extension<AuthUser>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    if current_user.id == id {
        return Err(Error::Validation(
            "You cannot lock your own account.".to_string(),
        ));
    }

    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let user = state
        .user_service
        .lock_user(realm.id, id, realm.lockout_duration_secs)
        .await?;
    state
        .session_repo
        .revoke_all_for_user(&realm.id, &id)
        .await?;

    Ok((
        StatusCode::OK,
        Json(user_response_from_user(&state, user).await?),
    ))
}

pub async fn ban_user_handler(
    State(state): State<AppState>,
    Extension(AuthUser(current_user)): Extension<AuthUser>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    if current_user.id == id {
        return Err(Error::Validation(
            "You cannot ban your own account.".to_string(),
        ));
    }

    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let user = state.user_service.ban_user(realm.id, id).await?;
    state
        .session_repo
        .revoke_all_for_user(&realm.id, &id)
        .await?;

    Ok((
        StatusCode::OK,
        Json(user_response_from_user(&state, user).await?),
    ))
}

// ---------------------------------------------------------------------------
// Update user profile (username only; emails go through /emails sub-resource)
// ---------------------------------------------------------------------------

#[derive(Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    pub username: Option<String>,
    #[serde(default)]
    pub first_name: Option<Option<String>>,
    #[serde(default)]
    pub last_name: Option<Option<String>>,
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

    if username.is_none() && payload.first_name.is_none() && payload.last_name.is_none() {
        return Err(Error::Validation("No updates provided".to_string()));
    }

    let user = state
        .user_service
        .update_profile(
            realm.id,
            id,
            username,
            payload.first_name,
            payload.last_name,
        )
        .await?;

    Ok((
        StatusCode::OK,
        Json(user_response_from_user(&state, user).await?),
    ))
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
// Phone-number sub-resource: list
// ---------------------------------------------------------------------------

pub async fn list_user_phone_numbers_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state.user_service.get_user_in_realm(realm.id, id).await?;

    let phone_numbers = state
        .user_phone_number_service
        .list_phone_numbers(id)
        .await?;
    Ok((StatusCode::OK, Json(phone_numbers)))
}

// ---------------------------------------------------------------------------
// Phone-number sub-resource: add
// ---------------------------------------------------------------------------

#[derive(Deserialize, Validate)]
pub struct AddUserPhoneNumberPayload {
    #[validate(length(min = 1, message = "Phone number is required"))]
    pub phone_number: String,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub is_verified: bool,
}

pub async fn add_user_phone_number_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
    ValidatedJson(payload): ValidatedJson<AddUserPhoneNumberPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state.user_service.get_user_in_realm(realm.id, id).await?;

    let phone_number = match state
        .user_phone_number_service
        .add_phone_number(
            id,
            realm.id,
            &payload.phone_number,
            payload.is_primary,
            payload.is_verified,
        )
        .await
    {
        Ok(phone_number) => phone_number,
        Err(Error::PhoneNumberAlreadyExists) => {
            let mut fields = std::collections::HashMap::new();
            fields.insert(
                "phone_number".to_string(),
                "Phone number is already in use".to_string(),
            );
            return Err(Error::FieldsValidation {
                message: "Validation failed".to_string(),
                fields,
            });
        }
        Err(err) => return Err(err),
    };

    Ok((StatusCode::CREATED, Json(phone_number)))
}

// ---------------------------------------------------------------------------
// Phone-number sub-resource: remove
// ---------------------------------------------------------------------------

pub async fn remove_user_phone_number_handler(
    State(state): State<AppState>,
    Path((realm_name, id, phone_number_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state.user_service.get_user_in_realm(realm.id, id).await?;
    state
        .user_phone_number_service
        .remove_phone_number(id, phone_number_id)
        .await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "removed" })),
    ))
}

// ---------------------------------------------------------------------------
// Phone-number sub-resource: set primary
// ---------------------------------------------------------------------------

pub async fn set_primary_phone_number_handler(
    State(state): State<AppState>,
    Path((realm_name, id, phone_number_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state.user_service.get_user_in_realm(realm.id, id).await?;
    state
        .user_phone_number_service
        .set_primary(id, phone_number_id)
        .await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "updated" })),
    ))
}

// ---------------------------------------------------------------------------
// Phone-number sub-resource: set verified
// ---------------------------------------------------------------------------

pub async fn set_phone_number_verified_handler(
    State(state): State<AppState>,
    Path((realm_name, id, phone_number_id)): Path<(String, Uuid, Uuid)>,
    Json(payload): Json<SetVerifiedPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state.user_service.get_user_in_realm(realm.id, id).await?;
    state
        .user_phone_number_service
        .set_verified(id, phone_number_id, payload.is_verified)
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
    pub sign_out_all_sessions: Option<bool>,
    pub skip_password_checks: Option<bool>,
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
    let skip_password_checks = payload.skip_password_checks.unwrap_or(false);
    let invalid_password = if skip_password_checks {
        payload.password.is_empty() || payload.password.len() > 100
    } else {
        payload.password.len() < 8 || payload.password.len() > 100
    };

    if invalid_password {
        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "password".to_string(),
            if skip_password_checks {
                "Password is required and must be no more than 100 characters".to_string()
            } else {
                "Password must be between 8 and 100 characters".to_string()
            },
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
        .update_password(
            realm.id,
            id,
            &payload.password,
            payload.sign_out_all_sessions.unwrap_or(false),
        )
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
