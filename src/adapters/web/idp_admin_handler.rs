use crate::application::idp_service::{
    CreateIdentityProviderRequest, UpdateIdentityProviderRequest,
};
use crate::error::{Error, Result};
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Default, Deserialize)]
pub struct DeleteIdentityProviderQuery {
    pub hard: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
pub struct IdentityProviderActivityQuery {
    pub limit: Option<usize>,
}

const DEFAULT_ACTIVITY_LIMIT: usize = 20;
const MAX_ACTIVITY_LIMIT: usize = 100;

pub async fn list_identity_providers_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;
    let providers = state
        .identity_provider_service
        .list_by_realm(realm.id)
        .await?;
    Ok((StatusCode::OK, Json(providers)))
}

pub async fn list_identity_provider_presets_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    Ok((
        StatusCode::OK,
        Json(state.identity_provider_service.list_presets()),
    ))
}

pub async fn create_identity_provider_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Json(payload): Json<CreateIdentityProviderRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;
    let provider = state
        .identity_provider_service
        .create(realm.id, payload)
        .await?;
    Ok((StatusCode::CREATED, Json(provider)))
}

pub async fn get_identity_provider_handler(
    State(state): State<AppState>,
    Path((_realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let provider = state.identity_provider_service.get_by_id(id).await?;
    Ok((StatusCode::OK, Json(provider)))
}

pub async fn list_identity_provider_linked_users_handler(
    State(state): State<AppState>,
    Path((_realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let linked_users = state
        .identity_provider_service
        .list_linked_users(id)
        .await?;
    Ok((StatusCode::OK, Json(linked_users)))
}

pub async fn list_identity_provider_activity_handler(
    State(state): State<AppState>,
    Query(query): Query<IdentityProviderActivityQuery>,
    Path((_realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let activity = state
        .identity_provider_service
        .list_recent_activity(
            id,
            query
                .limit
                .unwrap_or(DEFAULT_ACTIVITY_LIMIT)
                .min(MAX_ACTIVITY_LIMIT),
        )
        .await?;
    Ok((StatusCode::OK, Json(activity)))
}

pub async fn update_identity_provider_handler(
    State(state): State<AppState>,
    Path((_realm_name, id)): Path<(String, Uuid)>,
    Json(payload): Json<UpdateIdentityProviderRequest>,
) -> Result<impl IntoResponse> {
    let provider = state.identity_provider_service.update(id, payload).await?;
    Ok((StatusCode::OK, Json(provider)))
}

pub async fn delete_identity_provider_handler(
    State(state): State<AppState>,
    Query(query): Query<DeleteIdentityProviderQuery>,
    Path((_realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let result = state
        .identity_provider_service
        .delete(id, query.hard.unwrap_or(false))
        .await?;
    Ok((StatusCode::OK, Json(result)))
}

pub async fn refresh_identity_provider_metadata_handler(
    State(state): State<AppState>,
    Path((_realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let provider = state.identity_provider_service.refresh_metadata(id).await?;
    Ok((StatusCode::OK, Json(provider)))
}

pub async fn test_identity_provider_connection_handler(
    State(state): State<AppState>,
    Path((_realm_name, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let result = state.identity_provider_service.test_connection(id).await?;
    Ok((StatusCode::OK, Json(result)))
}
