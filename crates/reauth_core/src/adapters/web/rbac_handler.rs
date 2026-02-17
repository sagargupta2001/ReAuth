use crate::application::rbac_service::{CreateGroupPayload, CreateRolePayload};
use crate::domain::rbac::{GroupMemberFilter, GroupRoleFilter, RoleMemberFilter};
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

#[derive(Deserialize)]
pub struct GroupListQuery {
    #[serde(flatten)]
    pub page: PageRequest,
}

#[derive(Deserialize)]
pub struct GroupTreeQuery {
    #[serde(flatten)]
    pub page: PageRequest,
}

#[derive(Deserialize)]
pub struct GroupDeleteQuery {
    pub cascade: Option<bool>,
}

// GET /api/realms/{realm}/rbac/groups
pub async fn list_groups_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(req): Query<GroupListQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let response = state.rbac_service.list_groups(realm.id, req.page).await?;
    Ok((StatusCode::OK, Json(response)))
}

// GET /api/realms/{realm}/rbac/groups/tree
pub async fn list_group_roots_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(req): Query<GroupTreeQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let response = state.rbac_service.list_group_roots(realm.id, req.page).await?;
    Ok((StatusCode::OK, Json(response)))
}

// GET /api/realms/{realm}/rbac/groups/{id}/children
pub async fn list_group_children_handler(
    State(state): State<AppState>,
    Path((realm_name, group_id)): Path<(String, Uuid)>,
    Query(req): Query<GroupTreeQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let response = state
        .rbac_service
        .list_group_children(realm.id, group_id, req.page)
        .await?;
    Ok((StatusCode::OK, Json(response)))
}

// GET /api/realms/{realm}/rbac/groups/{id}
pub async fn get_group_handler(
    State(state): State<AppState>,
    Path((realm_name, group_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let group = state.rbac_service.get_group(realm.id, group_id).await?;
    Ok((StatusCode::OK, Json(group)))
}

// GET /api/realms/{realm}/rbac/groups/{id}/delete-summary
pub async fn get_group_delete_summary_handler(
    State(state): State<AppState>,
    Path((realm_name, group_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let summary = state
        .rbac_service
        .get_group_delete_summary(realm.id, group_id)
        .await?;
    Ok((StatusCode::OK, Json(summary)))
}

// PUT /api/realms/{realm}/rbac/groups/{id}
pub async fn update_group_handler(
    State(state): State<AppState>,
    Path((realm_name, group_id)): Path<(String, Uuid)>,
    Json(payload): Json<CreateGroupPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let updated_group = state
        .rbac_service
        .update_group(realm.id, group_id, payload)
        .await?;

    Ok((StatusCode::OK, Json(updated_group)))
}

// DELETE /api/realms/{realm}/rbac/groups/{id}
pub async fn delete_group_handler(
    State(state): State<AppState>,
    Path((realm_name, group_id)): Path<(String, Uuid)>,
    Query(query): Query<GroupDeleteQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let cascade = query.cascade.unwrap_or(false);
    state
        .rbac_service
        .delete_group(realm.id, group_id, cascade)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

// POST /api/realms/{realm}/rbac/groups/{id}/move
pub async fn move_group_handler(
    State(state): State<AppState>,
    Path((realm_name, group_id)): Path<(String, Uuid)>,
    Json(payload): Json<MoveGroupPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .rbac_service
        .move_group(
            realm.id,
            group_id,
            payload.parent_id,
            payload.before_id,
            payload.after_id,
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
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

#[derive(Deserialize)]
pub struct AssignGroupMemberPayload {
    pub user_id: Uuid,
}

#[derive(Deserialize)]
pub struct AssignGroupRolePayload {
    pub role_id: Uuid,
}

#[derive(Deserialize)]
pub struct MoveGroupPayload {
    pub parent_id: Option<Uuid>,
    pub before_id: Option<Uuid>,
    pub after_id: Option<Uuid>,
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

    Ok(StatusCode::NO_CONTENT)
}

// POST /api/realms/{realm}/rbac/groups/{group_id}/members
pub async fn assign_user_to_group_handler(
    State(state): State<AppState>,
    Path((realm_name, group_id)): Path<(String, Uuid)>,
    Json(payload): Json<AssignGroupMemberPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .rbac_service
        .assign_user_to_group(realm.id, payload.user_id, group_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

// DELETE /api/realms/{realm}/rbac/groups/{group_id}/members/{user_id}
pub async fn remove_user_from_group_handler(
    State(state): State<AppState>,
    Path((realm_name, group_id, user_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .rbac_service
        .remove_user_from_group(realm.id, user_id, group_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

// GET /api/realms/{realm}/rbac/groups/{group_id}/members
pub async fn list_group_members_handler(
    State(state): State<AppState>,
    Path((realm_name, group_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let users = state
        .rbac_service
        .get_group_member_ids(realm.id, group_id)
        .await?;

    Ok((StatusCode::OK, Json(users)))
}

#[derive(Deserialize)]
pub struct GroupMembersListQuery {
    #[serde(flatten)]
    pub page: PageRequest,
    pub filter: Option<String>, // all | members | non-members
}

// GET /api/realms/{realm}/rbac/groups/{group_id}/members/list
pub async fn list_group_members_page_handler(
    State(state): State<AppState>,
    Path((realm_name, group_id)): Path<(String, Uuid)>,
    Query(query): Query<GroupMembersListQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let filter = match query.filter.as_deref().unwrap_or("all") {
        "all" => GroupMemberFilter::All,
        "members" => GroupMemberFilter::Members,
        "non-members" => GroupMemberFilter::NonMembers,
        _ => GroupMemberFilter::All,
    };

    let response = state
        .rbac_service
        .list_group_members(realm.id, group_id, filter, query.page)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

// POST /api/realms/{realm}/rbac/groups/{group_id}/roles
pub async fn assign_role_to_group_handler(
    State(state): State<AppState>,
    Path((realm_name, group_id)): Path<(String, Uuid)>,
    Json(payload): Json<AssignGroupRolePayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .rbac_service
        .assign_role_to_group(realm.id, payload.role_id, group_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

// DELETE /api/realms/{realm}/rbac/groups/{group_id}/roles/{role_id}
pub async fn remove_role_from_group_handler(
    State(state): State<AppState>,
    Path((realm_name, group_id, role_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .rbac_service
        .remove_role_from_group(realm.id, role_id, group_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

// GET /api/realms/{realm}/rbac/groups/{group_id}/roles
pub async fn list_group_roles_handler(
    State(state): State<AppState>,
    Path((realm_name, group_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let roles = state
        .rbac_service
        .get_group_role_ids(realm.id, group_id)
        .await?;

    Ok((StatusCode::OK, Json(roles)))
}

#[derive(Deserialize)]
pub struct GroupRolesListQuery {
    #[serde(flatten)]
    pub page: PageRequest,
    pub filter: Option<String>, // all | assigned | unassigned
}

// GET /api/realms/{realm}/rbac/groups/{group_id}/roles/list
pub async fn list_group_roles_page_handler(
    State(state): State<AppState>,
    Path((realm_name, group_id)): Path<(String, Uuid)>,
    Query(query): Query<GroupRolesListQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let filter = match query.filter.as_deref().unwrap_or("all") {
        "all" => GroupRoleFilter::All,
        "assigned" => GroupRoleFilter::Assigned,
        "unassigned" => GroupRoleFilter::Unassigned,
        _ => GroupRoleFilter::All,
    };

    let response = state
        .rbac_service
        .list_group_roles(realm.id, group_id, filter, query.page)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

// DELETE /api/realms/{realm}/users/{user_id}/roles/{role_id}
pub async fn remove_user_role_handler(
    State(state): State<AppState>,
    Path((realm_name, user_id, role_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .rbac_service
        .remove_role_from_user(realm.id, user_id, role_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
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

#[derive(Deserialize)]
pub struct RoleMembersQuery {
    pub scope: Option<String>,
}

// GET /api/realms/{realm}/rbac/roles/{id}/members?scope=direct|effective
pub async fn list_role_members_handler(
    State(state): State<AppState>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
    Query(query): Query<RoleMembersQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let scope = query.scope.unwrap_or_else(|| "direct".to_string());

    let users = match scope.as_str() {
        "direct" => state
            .rbac_service
            .get_direct_user_ids_for_role(realm.id, role_id)
            .await?,
        "effective" => state
            .rbac_service
            .get_effective_user_ids_for_role(realm.id, role_id)
            .await?,
        _ => {
            return Err(Error::Validation(
                "Invalid scope. Use 'direct' or 'effective'.".into(),
            ))
        }
    };

    Ok((StatusCode::OK, Json(users)))
}

#[derive(Deserialize)]
pub struct RoleMembersListQuery {
    #[serde(flatten)]
    pub page: PageRequest,
    pub filter: Option<String>, // all | direct | effective | unassigned
}

// GET /api/realms/{realm}/rbac/roles/{id}/members/list
pub async fn list_role_members_page_handler(
    State(state): State<AppState>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
    Query(query): Query<RoleMembersListQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let filter = match query.filter.as_deref().unwrap_or("all") {
        "all" => RoleMemberFilter::All,
        "direct" => RoleMemberFilter::Direct,
        "effective" => RoleMemberFilter::Effective,
        "unassigned" => RoleMemberFilter::Unassigned,
        _ => RoleMemberFilter::All,
    };

    let response = state
        .rbac_service
        .list_role_members(realm.id, role_id, filter, query.page)
        .await?;

    Ok((StatusCode::OK, Json(response)))
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
