use super::record_audit;
use super::*;
use crate::adapters::web::auth_middleware::AuthUser;
use crate::application::rbac_service::CreateGroupPayload;
use crate::domain::rbac::{GroupMemberFilter, GroupRoleFilter};
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
pub async fn create_group_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
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

    let group_name = group.name.clone();
    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.group.created",
        "group",
        Some(group.id.to_string()),
        json!({ "name": group_name }),
    )
    .await;

    Ok((StatusCode::CREATED, Json(group)))
}
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

    let response = state
        .rbac_service
        .list_group_roots(realm.id, req.page)
        .await?;
    Ok((StatusCode::OK, Json(response)))
}
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

pub async fn update_group_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
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

    let group_name = updated_group.name.clone();
    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.group.updated",
        "group",
        Some(group_id.to_string()),
        json!({ "name": group_name }),
    )
    .await;

    Ok((StatusCode::OK, Json(updated_group)))
}

pub async fn delete_group_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
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

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.group.deleted",
        "group",
        Some(group_id.to_string()),
        json!({ "cascade": cascade }),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}
pub async fn move_group_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
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

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.group.moved",
        "group",
        Some(group_id.to_string()),
        json!({
            "parent_id": payload.parent_id.map(|id| id.to_string()),
            "before_id": payload.before_id.map(|id| id.to_string()),
            "after_id": payload.after_id.map(|id| id.to_string()),
        }),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}
pub async fn assign_user_to_group_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
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

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.user.group.assigned",
        "user",
        Some(payload.user_id.to_string()),
        json!({ "group_id": group_id.to_string() }),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_user_from_group_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
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

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.user.group.removed",
        "user",
        Some(user_id.to_string()),
        json!({ "group_id": group_id.to_string() }),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}
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
pub async fn assign_role_to_group_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
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

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.group.role.assigned",
        "group",
        Some(group_id.to_string()),
        json!({ "role_id": payload.role_id.to_string() }),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_role_from_group_handler(
    State(state): State<AppState>,
    Extension(AuthUser(actor)): Extension<AuthUser>,
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

    record_audit(
        &state,
        realm.id,
        actor.id,
        "rbac.group.role.removed",
        "group",
        Some(group_id.to_string()),
        json!({ "role_id": role_id.to_string() }),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}
pub async fn list_group_roles_handler(
    State(state): State<AppState>,
    Path((realm_name, group_id)): Path<(String, Uuid)>,
    Query(query): Query<GroupRolesScopeQuery>,
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
                .get_group_role_ids(realm.id, group_id)
                .await?
        }
        "effective" => {
            state
                .rbac_service
                .get_effective_group_role_ids(realm.id, group_id)
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
        "assigned" => GroupRoleFilter::Direct,
        "direct" => GroupRoleFilter::Direct,
        "effective" => GroupRoleFilter::Effective,
        "unassigned" => GroupRoleFilter::Unassigned,
        _ => GroupRoleFilter::All,
    };

    let response = state
        .rbac_service
        .list_group_roles(realm.id, group_id, filter, query.page)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}
