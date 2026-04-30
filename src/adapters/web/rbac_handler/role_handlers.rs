use super::record_audit;
use super::*;
use crate::adapters::web::auth_middleware::AuthUser;
use crate::application::rbac_service::{
    CreateCustomPermissionPayload, CreateRolePayload, UpdateCustomPermissionPayload,
};
use crate::domain::pagination::PageRequest;
use crate::domain::permissions::{self, PermissionDef, ResourceGroup};
use crate::domain::rbac::{RoleCompositeFilter, RoleMemberFilter, UserRoleFilter};
use crate::error::{Error, Result};
use crate::AppState;
use axum::extract::Query;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde_json::json;
use uuid::Uuid;
pub async fn create_role_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
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

    let role_name = role.name.clone();
    let client_id = role.client_id.map(|id| id.to_string());
    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.role.created",
        "role",
        Some(role.id.to_string()),
        json!({
            "name": role_name,
            "client_id": client_id,
        }),
    )
    .await;

    Ok((StatusCode::CREATED, Json(role)))
}
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
pub async fn assign_permission_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
    Json(payload): Json<AssignPermissionPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let permission = payload.permission.clone();
    state
        .rbac_service
        .assign_permission_to_role(realm.id, role_id, payload.permission)
        .await?;

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.role.permission.assigned",
        "role",
        Some(role_id.to_string()),
        json!({ "permission": permission }),
    )
    .await;

    Ok((StatusCode::OK, Json(json!({}))))
}
pub async fn assign_user_role_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
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

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.user.role.assigned",
        "user",
        Some(user_id.to_string()),
        json!({ "role_id": payload.role_id.to_string() }),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}
pub async fn list_user_roles_handler(
    State(state): State<AppState>,
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    Query(query): Query<UserRolesScopeQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let scope = query.scope.unwrap_or_else(|| "direct".to_string());

    let roles = match scope.as_str() {
        "direct" => {
            state
                .rbac_service
                .get_direct_role_ids_for_user(realm.id, user_id)
                .await?
        }
        "effective" => {
            state
                .rbac_service
                .get_effective_role_ids_for_user(realm.id, user_id)
                .await?
        }
        _ => {
            return Err(Error::Validation(
                "Invalid scope. Use 'direct' or 'effective'.".into(),
            ))
        }
    };

    Ok((StatusCode::OK, Json(roles)))
}
pub async fn list_user_roles_page_handler(
    State(state): State<AppState>,
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    Query(query): Query<UserRolesListQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let filter = match query.filter.as_deref().unwrap_or("all") {
        "all" => UserRoleFilter::All,
        "direct" => UserRoleFilter::Direct,
        "effective" => UserRoleFilter::Effective,
        "unassigned" => UserRoleFilter::Unassigned,
        _ => UserRoleFilter::All,
    };

    let response = state
        .rbac_service
        .list_user_roles(realm.id, user_id, filter, query.page)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}
pub async fn list_role_composites_handler(
    State(state): State<AppState>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
    Query(query): Query<RoleCompositesScopeQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let scope = query.scope.unwrap_or_else(|| "direct".to_string());

    let roles = match scope.as_str() {
        "direct" => {
            state
                .rbac_service
                .get_role_composite_ids(realm.id, role_id)
                .await?
        }
        "effective" => {
            state
                .rbac_service
                .get_effective_role_composite_ids(realm.id, role_id)
                .await?
        }
        _ => {
            return Err(Error::Validation(
                "Invalid scope. Use 'direct' or 'effective'.".into(),
            ))
        }
    };

    Ok((StatusCode::OK, Json(roles)))
}
pub async fn list_role_composites_page_handler(
    State(state): State<AppState>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
    Query(query): Query<RoleCompositesListQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let filter = match query.filter.as_deref().unwrap_or("all") {
        "all" => RoleCompositeFilter::All,
        "direct" => RoleCompositeFilter::Direct,
        "effective" => RoleCompositeFilter::Effective,
        "unassigned" => RoleCompositeFilter::Unassigned,
        _ => RoleCompositeFilter::All,
    };

    let response = state
        .rbac_service
        .list_role_composites(realm.id, role_id, filter, query.page)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}
pub async fn assign_composite_role_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
    Json(payload): Json<AssignRolePayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .rbac_service
        .assign_composite_role(realm.id, role_id, payload.role_id)
        .await?;

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.role.composite.added",
        "role",
        Some(role_id.to_string()),
        json!({ "child_role_id": payload.role_id.to_string() }),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_composite_role_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
    Path((realm_name, role_id, child_role_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .rbac_service
        .remove_composite_role(realm.id, role_id, child_role_id)
        .await?;

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.role.composite.removed",
        "role",
        Some(role_id.to_string()),
        json!({ "child_role_id": child_role_id.to_string() }),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_user_role_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
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

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.user.role.removed",
        "user",
        Some(user_id.to_string()),
        json!({ "role_id": role_id.to_string() }),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_role_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;
    state.rbac_service.delete_role(realm.id, role_id).await?;

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.role.deleted",
        "role",
        Some(role_id.to_string()),
        json!({}),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

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

pub async fn update_role_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
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

    let role_name = updated_role.name.clone();
    let client_id = updated_role.client_id.map(|id| id.to_string());
    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.role.updated",
        "role",
        Some(role_id.to_string()),
        json!({
            "name": role_name,
            "client_id": client_id,
        }),
    )
    .await;

    Ok((StatusCode::OK, Json(updated_role)))
}
pub async fn list_permissions_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(query): Query<PermissionsQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let mut groups: Vec<ResourceGroup> = if query.client_id.is_some() {
        Vec::new()
    } else {
        permissions::get_system_permissions()
    };

    let custom_permissions = state
        .rbac_service
        .list_custom_permissions(realm.id, query.client_id)
        .await?;

    let custom_group = ResourceGroup {
        id: "custom".to_string(),
        label: "Custom Permissions".to_string(),
        description: if query.client_id.is_some() {
            "Permissions defined for this client application.".to_string()
        } else {
            "Permissions defined at the realm level.".to_string()
        },
        permissions: custom_permissions
            .into_iter()
            .map(|perm| PermissionDef {
                id: perm.permission,
                name: perm.name,
                description: perm.description.unwrap_or_default(),
                custom_id: Some(perm.id.to_string()),
            })
            .collect(),
    };

    groups.push(custom_group);

    Ok((StatusCode::OK, Json(groups)))
}
pub async fn create_custom_permission_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
    Path(realm_name): Path<String>,
    Json(payload): Json<CreateCustomPermissionPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let created = state
        .rbac_service
        .create_custom_permission(realm.id, payload)
        .await?;

    let permission_id = created.permission.clone();
    let permission_name = created.name.clone();
    let response = PermissionDef {
        id: created.permission,
        name: created.name,
        description: created.description.unwrap_or_default(),
        custom_id: Some(created.id.to_string()),
    };

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.permission.custom.created",
        "permission",
        Some(created.id.to_string()),
        json!({
            "permission": permission_id,
            "name": permission_name,
        }),
    )
    .await;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_custom_permission_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
    Path((realm_name, permission_id)): Path<(String, Uuid)>,
    Json(payload): Json<UpdateCustomPermissionPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let updated = state
        .rbac_service
        .update_custom_permission(realm.id, permission_id, payload)
        .await?;

    let permission_value = updated.permission.clone();
    let permission_name = updated.name.clone();
    let response = PermissionDef {
        id: updated.permission,
        name: updated.name,
        description: updated.description.unwrap_or_default(),
        custom_id: Some(updated.id.to_string()),
    };

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.permission.custom.updated",
        "permission",
        Some(permission_id.to_string()),
        json!({
            "permission": permission_value,
            "name": permission_name,
        }),
    )
    .await;

    Ok((StatusCode::OK, Json(response)))
}

pub async fn delete_custom_permission_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
    Path((realm_name, permission_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    state
        .rbac_service
        .delete_custom_permission(realm.id, permission_id)
        .await?;

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.permission.custom.deleted",
        "permission",
        Some(permission_id.to_string()),
        json!({}),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
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

    let perms = state
        .rbac_service
        .get_permissions_for_role(realm.id, role_id)
        .await?;
    Ok(Json(perms)) // Returns ["user:read", "client:write"]
}
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
        "direct" => {
            state
                .rbac_service
                .get_direct_user_ids_for_role(realm.id, role_id)
                .await?
        }
        "effective" => {
            state
                .rbac_service
                .get_effective_user_ids_for_role(realm.id, role_id)
                .await?
        }
        _ => {
            return Err(Error::Validation(
                "Invalid scope. Use 'direct' or 'effective'.".into(),
            ))
        }
    };

    Ok((StatusCode::OK, Json(users)))
}
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

// OR Query param ?permission=... (Cleaner for DELETE). Let's use Body for consistency with Assign.
// Axum allows body in DELETE but it's sometimes frowned upon.
// A better REST pattern: DELETE /roles/:id/permissions/:permission
// For simplicity, let's use the payload approach or a Bulk action with "remove".
pub async fn revoke_permission_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
    Json(payload): Json<PermissionPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let permission = payload.permission.clone();
    state
        .rbac_service
        .revoke_permission(realm.id, role_id, payload.permission)
        .await?;

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.role.permission.revoked",
        "role",
        Some(role_id.to_string()),
        json!({ "permission": permission }),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

// POST /roles/:id/permissions/bulk
pub async fn bulk_permissions_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
    Path((realm_name, role_id)): Path<(String, Uuid)>,
    Json(payload): Json<BulkPermissionPayload>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let action = payload.action.clone();
    let count = payload.permissions.len();
    state
        .rbac_service
        .bulk_update_permissions(realm.id, role_id, payload.permissions, payload.action)
        .await?;

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.role.permission.bulk",
        "role",
        Some(role_id.to_string()),
        json!({
            "action": action,
            "count": count,
        }),
    )
    .await;

    Ok((StatusCode::OK, Json(json!({}))))
}
