//! Defines the core business events that can occur in the application.

use serde::Serialize;
use uuid::Uuid;

/// An enum representing all possible events in the core domain.
/// This single enum is the "message" that flows through the event bus.
#[derive(Clone, Debug)]
pub enum DomainEvent {
    UserCreated(UserCreated),
    UserAssignedToGroup(UserGroupChanged),
    RoleAssignedToGroup(RoleGroupChanged),
    RolePermissionChanged(RolePermissionChanged),
    UserRoleAssigned(UserRoleChanged),
    RoleDeleted(RoleDeleted),
}

#[derive(Clone, Debug, Serialize)]
pub struct UserCreated {
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserGroupChanged {
    pub user_id: Uuid,
    pub group_id: Uuid,
}

#[derive(Clone, Debug, Serialize)]
pub struct RoleGroupChanged {
    pub role_id: Uuid,
    pub group_id: Uuid,
}

#[derive(Clone, Debug, Serialize)]
pub struct RolePermissionChanged {
    pub role_id: Uuid,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserRoleChanged {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Clone, Debug, Serialize)]
pub struct RoleDeleted {
    pub role_id: Uuid,
    pub affected_user_ids: Vec<Uuid>,
}
