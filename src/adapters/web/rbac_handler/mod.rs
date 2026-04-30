use crate::domain::audit::NewAuditEvent;
use crate::domain::pagination::PageRequest;
use crate::AppState;
use serde::Deserialize;
use tracing::error;
use uuid::Uuid;

pub(super) async fn record_audit(
    state: &AppState,
    realm_id: Uuid,
    actor_id: Uuid,
    action: &str,
    target_type: &str,
    target_id: Option<String>,
    metadata: serde_json::Value,
) {
    let event = NewAuditEvent {
        realm_id,
        actor_user_id: Some(actor_id),
        action: action.to_string(),
        target_type: target_type.to_string(),
        target_id,
        metadata,
    };

    if let Err(err) = state.audit_service.record(event).await {
        error!("Failed to write audit event: {:?}", err);
    }
}

pub mod group_handlers;
pub mod role_handlers;

pub use group_handlers::*;
pub use role_handlers::*;
// POST /api/realms/{realm}

// POST /api/realms/{realm}

// GET /api/realms/{realm}

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

// GET /api/realms/{realm}

// GET /api/realms/{realm}

// GET /api/realms/{realm}

// GET /api/realms/{realm}

// GET /api/realms/{realm}

// PUT /api/realms/{realm}

// DELETE /api/realms/{realm}

// POST /api/realms/{realm}

// GET /api/realms/{realm}

#[derive(Deserialize)]
pub struct AssignPermissionPayload {
    pub permission: String,
}

// POST /api/realms/{realm}

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

// POST /api/realms/{realm}

// POST /api/realms/{realm}

// DELETE /api/realms/{realm}

// GET /api/realms/{realm}

#[derive(Deserialize)]
pub struct GroupMembersListQuery {
    #[serde(flatten)]
    pub page: PageRequest,
    pub filter: Option<String>, // all | members | non-members
}

// GET /api/realms/{realm}

// POST /api/realms/{realm}

// DELETE /api/realms/{realm}

#[derive(Deserialize)]
pub struct GroupRolesScopeQuery {
    pub scope: Option<String>,
}

// GET /api/realms/{realm}

#[derive(Deserialize)]
pub struct GroupRolesListQuery {
    #[serde(flatten)]
    pub page: PageRequest,
    pub filter: Option<String>, // all | direct | effective | unassigned
}

// GET /api/realms/{realm}

#[derive(Deserialize)]
pub struct UserRolesScopeQuery {
    pub scope: Option<String>,
}

// GET /api/realms/{realm}

#[derive(Deserialize)]
pub struct UserRolesListQuery {
    #[serde(flatten)]
    pub page: PageRequest,
    pub filter: Option<String>, // all | direct | effective | unassigned
}

// GET /api/realms/{realm}

#[derive(Deserialize)]
pub struct RoleCompositesScopeQuery {
    pub scope: Option<String>,
}

// GET /api/realms/{realm}

#[derive(Deserialize)]
pub struct RoleCompositesListQuery {
    #[serde(flatten)]
    pub page: PageRequest,
    pub filter: Option<String>, // all | direct | effective | unassigned
}

// GET /api/realms/{realm}

// POST /api/realms/{realm}

// DELETE /api/realms/{realm}

// DELETE /api/realms/{realm}

// DELETE /api/realms/{realm}

// GET /api/realms/{realm}

// PUT /api/realms/{realm}

#[derive(Deserialize)]
pub struct PermissionsQuery {
    pub client_id: Option<Uuid>,
}

// GET /api/realms/{realm}

// POST /api/realms/{realm}

// PUT /api/realms/{realm}

// DELETE /api/realms/{realm}

#[derive(Deserialize)]
pub struct PermissionPayload {
    pub permission: String,
}

#[derive(Deserialize)]
pub struct BulkPermissionPayload {
    pub permissions: Vec<String>,
    pub action: String, // "add" or "remove"
}

#[derive(Deserialize)]
pub struct RoleMembersQuery {
    pub scope: Option<String>,
}

// GET /api/realms/{realm}

#[derive(Deserialize)]
pub struct RoleMembersListQuery {
    #[serde(flatten)]
    pub page: PageRequest,
    pub filter: Option<String>, // all | direct | effective | unassigned
}

// GET /api/realms/{realm}

// DELETE /roles/:id/permissions (Body: { permission: "..." }
