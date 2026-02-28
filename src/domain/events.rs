//! Defines the core business events that can occur in the application.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

/// An enum representing all possible events in the core domain.
/// This single enum is the "message" that flows through the event bus.
#[derive(Clone, Debug)]
pub enum DomainEvent {
    UserCreated(UserCreated),
    UserAssignedToGroup(UserGroupChanged),
    UserRemovedFromGroup(UserGroupChanged),
    RoleAssignedToGroup(RoleGroupChanged),
    RoleRemovedFromGroup(RoleGroupChanged),
    RolePermissionChanged(RolePermissionChanged),
    UserRoleAssigned(UserRoleChanged),
    UserRoleRemoved(UserRoleChanged),
    RoleCompositeChanged(RoleCompositeChanged),
    RoleDeleted(RoleDeleted),
    GroupDeleted(GroupDeleted),
}

pub const EVENT_VERSION_V1: &str = "v1";

#[derive(Clone, Debug, Serialize)]
pub struct EventActor {
    pub user_id: Option<Uuid>,
    pub client_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EventEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub event_version: String,
    pub occurred_at: String,
    pub realm_id: Option<Uuid>,
    pub actor: Option<EventActor>,
    pub data: Value,
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
    pub permission: String,
    pub action: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserRoleChanged {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Clone, Debug, Serialize)]
pub struct RoleCompositeChanged {
    pub parent_role_id: Uuid,
    pub child_role_id: Uuid,
    pub action: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct RoleDeleted {
    pub role_id: Uuid,
    pub affected_user_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GroupDeleted {
    pub group_ids: Vec<Uuid>,
    pub affected_user_ids: Vec<Uuid>,
}

impl DomainEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            DomainEvent::UserCreated(_) => "user.created",
            DomainEvent::UserAssignedToGroup(_) => "user.assigned",
            DomainEvent::UserRemovedFromGroup(_) => "user.removed",
            DomainEvent::RoleAssignedToGroup(_) => "role.assigned",
            DomainEvent::RoleRemovedFromGroup(_) => "role.removed",
            DomainEvent::RolePermissionChanged(_) => "role.updated",
            DomainEvent::UserRoleAssigned(_) => "role.assigned",
            DomainEvent::UserRoleRemoved(_) => "role.removed",
            DomainEvent::RoleCompositeChanged(_) => "role.updated",
            DomainEvent::RoleDeleted(_) => "role.deleted",
            DomainEvent::GroupDeleted(_) => "group.deleted",
        }
    }

    pub fn payload_value(&self) -> Value {
        match self {
            DomainEvent::UserCreated(e) => serde_json::to_value(e),
            DomainEvent::UserAssignedToGroup(e) => serde_json::to_value(e),
            DomainEvent::UserRemovedFromGroup(e) => serde_json::to_value(e),
            DomainEvent::RoleAssignedToGroup(e) => serde_json::to_value(e),
            DomainEvent::RoleRemovedFromGroup(e) => serde_json::to_value(e),
            DomainEvent::RolePermissionChanged(e) => serde_json::to_value(e),
            DomainEvent::UserRoleAssigned(e) => serde_json::to_value(e),
            DomainEvent::UserRoleRemoved(e) => serde_json::to_value(e),
            DomainEvent::RoleCompositeChanged(e) => serde_json::to_value(e),
            DomainEvent::RoleDeleted(e) => serde_json::to_value(e),
            DomainEvent::GroupDeleted(e) => serde_json::to_value(e),
        }
        .unwrap_or_else(|_| Value::Object(Default::default()))
    }

    pub fn payload_json(&self) -> String {
        self.payload_value().to_string()
    }

    pub fn to_envelope(
        &self,
        event_id: Uuid,
        occurred_at: DateTime<Utc>,
        realm_id: Option<Uuid>,
        actor: Option<EventActor>,
    ) -> EventEnvelope {
        EventEnvelope {
            event_id: event_id.to_string(),
            event_type: self.event_type().to_string(),
            event_version: EVENT_VERSION_V1.to_string(),
            occurred_at: occurred_at.to_rfc3339(),
            realm_id,
            actor,
            data: self.payload_value(),
        }
    }

    pub fn envelope_json(
        &self,
        event_id: Uuid,
        occurred_at: DateTime<Utc>,
        realm_id: Option<Uuid>,
        actor: Option<EventActor>,
    ) -> String {
        serde_json::to_string(&self.to_envelope(event_id, occurred_at, realm_id, actor))
            .unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use super::*;
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn event_structs_serialize_fields() {
        let user_id = Uuid::new_v4();
        let group_id = Uuid::new_v4();
        let role_id = Uuid::new_v4();

        let created = UserCreated {
            user_id,
            username: "alice".to_string(),
        };
        let value = serde_json::to_value(&created).expect("serialize");
        assert_eq!(value["user_id"], json!(user_id));
        assert_eq!(value["username"], json!("alice"));

        let changed = UserGroupChanged { user_id, group_id };
        let value = serde_json::to_value(&changed).expect("serialize");
        assert_eq!(value["user_id"], json!(user_id));
        assert_eq!(value["group_id"], json!(group_id));

        let role_group = RoleGroupChanged { role_id, group_id };
        let value = serde_json::to_value(&role_group).expect("serialize");
        assert_eq!(value["role_id"], json!(role_id));
        assert_eq!(value["group_id"], json!(group_id));

        let role_permission = RolePermissionChanged {
            role_id,
            permission: "perm.read".to_string(),
            action: "add".to_string(),
        };
        let value = serde_json::to_value(&role_permission).expect("serialize");
        assert_eq!(value["permission"], json!("perm.read"));
        assert_eq!(value["action"], json!("add"));

        let user_role = UserRoleChanged { user_id, role_id };
        let value = serde_json::to_value(&user_role).expect("serialize");
        assert_eq!(value["user_id"], json!(user_id));
        assert_eq!(value["role_id"], json!(role_id));

        let composite = RoleCompositeChanged {
            parent_role_id: role_id,
            child_role_id: group_id,
            action: "remove".to_string(),
        };
        let value = serde_json::to_value(&composite).expect("serialize");
        assert_eq!(value["action"], json!("remove"));

        let deleted = RoleDeleted {
            role_id,
            affected_user_ids: vec![user_id],
        };
        let value = serde_json::to_value(&deleted).expect("serialize");
        assert_eq!(value["role_id"], json!(role_id));
        assert_eq!(value["affected_user_ids"], json!([user_id]));

        let group_deleted = GroupDeleted {
            group_ids: vec![group_id],
            affected_user_ids: vec![user_id],
        };
        let value = serde_json::to_value(&group_deleted).expect("serialize");
        assert_eq!(value["group_ids"], json!([group_id]));
        assert_eq!(value["affected_user_ids"], json!([user_id]));
    }

    #[test]
    fn domain_event_variants_hold_payloads() {
        let user_id = Uuid::new_v4();
        let group_id = Uuid::new_v4();
        let role_id = Uuid::new_v4();

        let event = DomainEvent::UserCreated(UserCreated {
            user_id,
            username: "alice".to_string(),
        });
        if let DomainEvent::UserCreated(payload) = event {
            assert_eq!(payload.user_id, user_id);
        } else {
            panic!("expected UserCreated");
        }

        let event = DomainEvent::UserAssignedToGroup(UserGroupChanged { user_id, group_id });
        if let DomainEvent::UserAssignedToGroup(payload) = event {
            assert_eq!(payload.group_id, group_id);
        } else {
            panic!("expected UserAssignedToGroup");
        }

        let event = DomainEvent::RoleAssignedToGroup(RoleGroupChanged { role_id, group_id });
        if let DomainEvent::RoleAssignedToGroup(payload) = event {
            assert_eq!(payload.role_id, role_id);
        } else {
            panic!("expected RoleAssignedToGroup");
        }

        let event = DomainEvent::RolePermissionChanged(RolePermissionChanged {
            role_id,
            permission: "perm.read".to_string(),
            action: "add".to_string(),
        });
        if let DomainEvent::RolePermissionChanged(payload) = event {
            assert_eq!(payload.permission, "perm.read");
        } else {
            panic!("expected RolePermissionChanged");
        }

        let event = DomainEvent::UserRoleAssigned(UserRoleChanged { user_id, role_id });
        if let DomainEvent::UserRoleAssigned(payload) = event {
            assert_eq!(payload.role_id, role_id);
        } else {
            panic!("expected UserRoleAssigned");
        }

        let event = DomainEvent::RoleCompositeChanged(RoleCompositeChanged {
            parent_role_id: role_id,
            child_role_id: group_id,
            action: "remove".to_string(),
        });
        if let DomainEvent::RoleCompositeChanged(payload) = event {
            assert_eq!(payload.action, "remove");
        } else {
            panic!("expected RoleCompositeChanged");
        }

        let event = DomainEvent::RoleDeleted(RoleDeleted {
            role_id,
            affected_user_ids: vec![user_id],
        });
        if let DomainEvent::RoleDeleted(payload) = event {
            assert_eq!(payload.affected_user_ids.len(), 1);
        } else {
            panic!("expected RoleDeleted");
        }

        let event = DomainEvent::GroupDeleted(GroupDeleted {
            group_ids: vec![group_id],
            affected_user_ids: vec![user_id],
        });
        if let DomainEvent::GroupDeleted(payload) = event {
            assert_eq!(payload.group_ids.len(), 1);
        } else {
            panic!("expected GroupDeleted");
        }
    }
}
