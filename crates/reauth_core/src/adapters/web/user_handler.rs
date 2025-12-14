use crate::adapters::web::auth_middleware::AuthUser;
use crate::adapters::web::validation::ValidatedJson;
use crate::domain::pagination::PageRequest;
use crate::error::{Error, Result};
use crate::AppState;
use axum::extract::{Path, Query};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateUserPayload {
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    username: String,
    #[validate(length(
        min = 8,
        max = 100,
        message = "Password must be between 8 and 100 characters"
    ))]
    password: String,
}

pub async fn create_user_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    ValidatedJson(payload): ValidatedJson<CreateUserPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let user = state
        .user_service
        .create_user(realm.id, &payload.username, &payload.password)
        .await?;

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

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub username: String,
}

pub async fn update_user_handler(
    State(state): State<AppState>,
    Path((realm_name, id)): Path<(String, Uuid)>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let user = state
        .user_service
        .update_username(realm.id, id, payload.username)
        .await?;

    Ok((StatusCode::OK, Json(user)))
}
