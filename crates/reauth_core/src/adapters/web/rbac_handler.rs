use crate::application::rbac_service::{CreateGroupPayload, CreateRolePayload};
use crate::domain::permissions;
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
        .assign_permission_to_role(realm.id, role_id, payload.permission)
        .await?;

    Ok((StatusCode::OK, Json({})))
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

// PUT /api/realms/{realm}/rbac/roles/{id}
pub async fn update_role_handler(
    State(state): State<AppState>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
    Json(payload): Json<CreateRolePayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let updated_role = state
        .rbac_service
        .update_role(realm.id, role_id, payload)
        .await?;

    Ok((StatusCode::OK, Json(updated_role)))
}

// GET /api/realms/{realm}/rbac/permissions
pub async fn list_permissions_handler() -> impl IntoResponse {
    // 1. Get System Permissions
    let groups = permissions::get_system_permissions();

    // 2. [Future] Fetch Custom Permissions (Client Scopes) from DB
    // let custom_groups = client_service.get_all_scopes_as_groups().await?;
    // groups.extend(custom_groups);

    // 3. Return JSON
    (StatusCode::OK, Json(groups))
}

#[derive(Deserialize)]
pub struct PermissionPayload {
    pub permission: String,
}

#[derive(Deserialize)]
pub struct BulkPermissionPayload {
    pub permissions: Vec<String>,
    pub action: String, // "add" or "remove"
}

// GET /roles/:id/permissions
pub async fn list_role_permissions_handler(
    State(state): State<AppState>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let perms = state.rbac_service.get_permissions_for_role(realm.id, role_id).await?;
    Ok(Json(perms)) // Returns ["user:read", "client:write"]
}

// DELETE /roles/:id/permissions (Body: { permission: "..." })
// OR Query param ?permission=... (Cleaner for DELETE). Let's use Body for consistency with Assign.
// Axum allows body in DELETE but it's sometimes frowned upon.
// A better REST pattern: DELETE /roles/:id/permissions/:permission
// For simplicity, let's use the payload approach or a Bulk action with "remove".
pub async fn revoke_permission_handler(
    State(state): State<AppState>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
    Json(payload): Json<PermissionPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state.rbac_service.revoke_permission(realm.id, role_id, payload.permission).await?;
    Ok(StatusCode::NO_CONTENT)
}

// POST /roles/:id/permissions/bulk
pub async fn bulk_permissions_handler(
    State(state): State<AppState>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
    Json(payload): Json<BulkPermissionPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state.rbac_service.bulk_update_permissions(realm.id, role_id, payload.permissions, payload.action).await?;

    Ok((StatusCode::OK, Json({})))
}
