use crate::application::rbac_service::{CreateGroupPayload, CreateRolePayload};
use crate::error::{Error, Result};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum::extract::Query;
use serde::Deserialize;
use uuid::Uuid;
use crate::domain::pagination::PageRequest;

// POST /api/realms/{realm}/rbac/roles
pub async fn create_role_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Json(payload): Json<CreateRolePayload>,
) -> Result<impl IntoResponse> {
    // 1. Resolve Realm ID
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    // 2. Pass realm ID to service
    let role = state.rbac_service.create_role(realm.id, payload).await?;

    Ok((StatusCode::CREATED, Json(role)))
}

// POST /api/realms/{realm}/rbac/groups
pub async fn create_group_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Json(payload): Json<CreateGroupPayload>,
) -> Result<impl IntoResponse> {
    // 1. Resolve Realm ID
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    // 2. Pass realm ID to service
    let group = state.rbac_service.create_group(realm.id, payload).await?;

    Ok((StatusCode::CREATED, Json(group)))
}

// GET /api/realms/{realm}/rbac/roles
pub async fn list_roles_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(req): Query<PageRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let response = state.rbac_service.list_roles(realm.id, req).await?;

    Ok((StatusCode::OK, Json(response)))
}

// GET /api/realms/{realm}/clients/{client_id}/roles
pub async fn list_client_roles_handler(
    State(state): State<AppState>,
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    Query(req): Query<PageRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let response = state
        .rbac_service
        .list_client_roles(realm.id, client_id, req)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

#[derive(Deserialize)]
pub struct AssignPermissionPayload {
    pub permission: String,
}

// POST /api/realms/{realm}/rbac/roles/{role_id}/permissions
pub async fn assign_permission_handler(
    State(state): State<AppState>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
    Json(payload): Json<AssignPermissionPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .rbac_service
        .assign_permission_to_role(realm.id, payload.permission, role_id)
        .await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct AssignRolePayload {
    pub role_id: Uuid,
}

// POST /api/realms/{realm}/users/{user_id}/roles
pub async fn assign_user_role_handler(
    State(state): State<AppState>,
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    Json(payload): Json<AssignRolePayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .rbac_service
        .assign_role_to_user(realm.id, user_id, payload.role_id)
        .await?;

    Ok(StatusCode::OK)
}

// DELETE /api/realms/{realm}/rbac/roles/{id}
pub async fn delete_role_handler(
    State(state): State<AppState>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;
    state.rbac_service.delete_role(realm.id, role_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// GET /api/realms/{realm}/rbac/roles/{id}
pub async fn get_role_handler(
    State(state): State<AppState>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let role = state.rbac_service.get_role(realm.id, role_id).await?;
    Ok((StatusCode::OK, Json(role)))
}