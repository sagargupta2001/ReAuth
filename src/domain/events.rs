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
    UserUpdated(UserChanged),
    UserDisabled(UserChanged),
    UserDeleted(UserDeleted),
    UserAssignedToGroup(UserGroupChanged),
    UserRemovedFromGroup(UserGroupChanged),
    RoleCreated(RoleCreated),
    RoleUpdated(RoleUpdated),
    RoleAssignedToGroup(RoleGroupChanged),
    RoleRemovedFromGroup(RoleGroupChanged),
    RolePermissionChanged(RolePermissionChanged),
    UserRoleAssigned(UserRoleChanged),
    UserRoleRemoved(UserRoleChanged),
    RoleCompositeChanged(RoleCompositeChanged),
    GroupCreated(GroupChanged),
    GroupUpdated(GroupChanged),
    GroupAssigned(UserGroupChanged),
    GroupRemoved(UserGroupChanged),
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
pub struct UserChanged {
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserDeleted {
    pub user_ids: Vec<Uuid>,
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
pub struct RoleCreated {
    pub role_id: Uuid,
    pub name: String,
    pub client_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RoleUpdated {
    pub role_id: Uuid,
    pub name: String,
    pub client_id: Option<Uuid>,
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
pub struct GroupChanged {
    pub group_id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
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
            DomainEvent::UserUpdated(_) => "user.updated",
            DomainEvent::UserDisabled(_) => "user.disabled",
            DomainEvent::UserDeleted(_) => "user.deleted",
            DomainEvent::UserAssignedToGroup(_) => "user.assigned",
            DomainEvent::UserRemovedFromGroup(_) => "user.removed",
            DomainEvent::RoleCreated(_) => "role.created",
            DomainEvent::RoleUpdated(_) => "role.updated",
            DomainEvent::RoleAssignedToGroup(_) => "role.assigned",
            DomainEvent::RoleRemovedFromGroup(_) => "role.removed",
            DomainEvent::RolePermissionChanged(_) => "role.updated",
            DomainEvent::UserRoleAssigned(_) => "role.assigned",
            DomainEvent::UserRoleRemoved(_) => "role.removed",
            DomainEvent::RoleCompositeChanged(_) => "role.updated",
            DomainEvent::GroupCreated(_) => "group.created",
            DomainEvent::GroupUpdated(_) => "group.updated",
            DomainEvent::GroupAssigned(_) => "group.assigned",
            DomainEvent::GroupRemoved(_) => "group.removed",
            DomainEvent::RoleDeleted(_) => "role.deleted",
            DomainEvent::GroupDeleted(_) => "group.deleted",
        }
    }

    pub fn payload_value(&self) -> Value {
        match self {
            DomainEvent::UserCreated(e) => serde_json::to_value(e),
            DomainEvent::UserUpdated(e) => serde_json::to_value(e),
            DomainEvent::UserDisabled(e) => serde_json::to_value(e),
            DomainEvent::UserDeleted(e) => serde_json::to_value(e),
            DomainEvent::UserAssignedToGroup(e) => serde_json::to_value(e),
            DomainEvent::UserRemovedFromGroup(e) => serde_json::to_value(e),
            DomainEvent::RoleCreated(e) => serde_json::to_value(e),
            DomainEvent::RoleUpdated(e) => serde_json::to_value(e),
            DomainEvent::RoleAssignedToGroup(e) => serde_json::to_value(e),
            DomainEvent::RoleRemovedFromGroup(e) => serde_json::to_value(e),
            DomainEvent::RolePermissionChanged(e) => serde_json::to_value(e),
            DomainEvent::UserRoleAssigned(e) => serde_json::to_value(e),
            DomainEvent::UserRoleRemoved(e) => serde_json::to_value(e),
            DomainEvent::RoleCompositeChanged(e) => serde_json::to_value(e),
            DomainEvent::GroupCreated(e) => serde_json::to_value(e),
            DomainEvent::GroupUpdated(e) => serde_json::to_value(e),
            DomainEvent::GroupAssigned(e) => serde_json::to_value(e),
            DomainEvent::GroupRemoved(e) => serde_json::to_value(e),
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

#[derive(Clone, Debug, Serialize)]
pub struct WebhookEventDefinition {
    pub event_type: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct WebhookEventGroup {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub events: &'static [WebhookEventDefinition],
}

pub const DEFAULT_WEBHOOK_EVENT_TYPES: &[&str] = &["user.created", "user.updated"];

pub const WEBHOOK_EVENT_CATALOG: &[WebhookEventGroup] = &[
    WebhookEventGroup {
        id: "users",
        label: "Users",
        description: "Authentication and lifecycle changes",
        events: &[
            WebhookEventDefinition {
                event_type: "user.created",
                label: "User created",
                description: "A user account was created.",
            },
            WebhookEventDefinition {
                event_type: "user.updated",
                label: "User updated",
                description: "A user profile, credential, or metadata field changed.",
            },
            WebhookEventDefinition {
                event_type: "user.disabled",
                label: "User disabled",
                description: "A user account was banned or password login was disabled.",
            },
            WebhookEventDefinition {
                event_type: "user.deleted",
                label: "User deleted",
                description: "One or more user accounts were deleted.",
            },
            WebhookEventDefinition {
                event_type: "user.assigned",
                label: "User assigned",
                description: "A user was added to a group.",
            },
            WebhookEventDefinition {
                event_type: "user.removed",
                label: "User removed",
                description: "A user was removed from a group.",
            },
        ],
    },
    WebhookEventGroup {
        id: "roles",
        label: "Roles",
        description: "Role lifecycle, assignments, and permission changes",
        events: &[
            WebhookEventDefinition {
                event_type: "role.created",
                label: "Role created",
                description: "A role was created.",
            },
            WebhookEventDefinition {
                event_type: "role.updated",
                label: "Role updated",
                description: "A role, role permission, or composite role changed.",
            },
            WebhookEventDefinition {
                event_type: "role.assigned",
                label: "Role assigned",
                description: "A role was assigned to a group or user.",
            },
            WebhookEventDefinition {
                event_type: "role.removed",
                label: "Role removed",
                description: "A role was removed from a group or user.",
            },
            WebhookEventDefinition {
                event_type: "role.deleted",
                label: "Role deleted",
                description: "A role was deleted.",
            },
        ],
    },
    WebhookEventGroup {
        id: "groups",
        label: "Groups",
        description: "Group lifecycle and membership changes",
        events: &[
            WebhookEventDefinition {
                event_type: "group.created",
                label: "Group created",
                description: "A group was created.",
            },
            WebhookEventDefinition {
                event_type: "group.updated",
                label: "Group updated",
                description: "A group profile or hierarchy field changed.",
            },
            WebhookEventDefinition {
                event_type: "group.assigned",
                label: "Group assigned",
                description: "A user was assigned to a group.",
            },
            WebhookEventDefinition {
                event_type: "group.removed",
                label: "Group removed",
                description: "A user was removed from a group.",
            },
            WebhookEventDefinition {
                event_type: "group.deleted",
                label: "Group deleted",
                description: "A group or group subtree was deleted.",
            },
        ],
    },
];

pub fn is_supported_webhook_event_type(event_type: &str) -> bool {
    WEBHOOK_EVENT_CATALOG
        .iter()
        .flat_map(|group| group.events.iter())
        .any(|event| event.event_type == event_type)
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

        let updated = UserChanged {
            user_id,
            username: "alice2".to_string(),
        };
        let value = serde_json::to_value(&updated).expect("serialize");
        assert_eq!(value["user_id"], json!(user_id));
        assert_eq!(value["username"], json!("alice2"));

        let user_deleted = UserDeleted {
            user_ids: vec![user_id],
        };
        let value = serde_json::to_value(&user_deleted).expect("serialize");
        assert_eq!(value["user_ids"], json!([user_id]));

        let changed = UserGroupChanged { user_id, group_id };
        let value = serde_json::to_value(&changed).expect("serialize");
        assert_eq!(value["user_id"], json!(user_id));
        assert_eq!(value["group_id"], json!(group_id));

        let role_group = RoleGroupChanged { role_id, group_id };
        let value = serde_json::to_value(&role_group).expect("serialize");
        assert_eq!(value["role_id"], json!(role_id));
        assert_eq!(value["group_id"], json!(group_id));

        let role_created = RoleCreated {
            role_id,
            name: "admin".to_string(),
            client_id: Some(group_id),
        };
        let value = serde_json::to_value(&role_created).expect("serialize");
        assert_eq!(value["role_id"], json!(role_id));
        assert_eq!(value["name"], json!("admin"));
        assert_eq!(value["client_id"], json!(group_id));

        let role_updated = RoleUpdated {
            role_id,
            name: "admin2".to_string(),
            client_id: None,
        };
        let value = serde_json::to_value(&role_updated).expect("serialize");
        assert_eq!(value["role_id"], json!(role_id));
        assert_eq!(value["name"], json!("admin2"));
        assert_eq!(value["client_id"], Value::Null);

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

        let group_changed = GroupChanged {
            group_id,
            name: "engineering".to_string(),
            parent_id: Some(role_id),
        };
        let value = serde_json::to_value(&group_changed).expect("serialize");
        assert_eq!(value["group_id"], json!(group_id));
        assert_eq!(value["name"], json!("engineering"));
        assert_eq!(value["parent_id"], json!(role_id));

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

        let event = DomainEvent::UserUpdated(UserChanged {
            user_id,
            username: "alice2".to_string(),
        });
        if let DomainEvent::UserUpdated(payload) = event {
            assert_eq!(payload.username, "alice2");
        } else {
            panic!("expected UserUpdated");
        }

        let event = DomainEvent::UserDisabled(UserChanged {
            user_id,
            username: "alice".to_string(),
        });
        if let DomainEvent::UserDisabled(payload) = event {
            assert_eq!(payload.user_id, user_id);
        } else {
            panic!("expected UserDisabled");
        }

        let event = DomainEvent::UserDeleted(UserDeleted {
            user_ids: vec![user_id],
        });
        if let DomainEvent::UserDeleted(payload) = event {
            assert_eq!(payload.user_ids, vec![user_id]);
        } else {
            panic!("expected UserDeleted");
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

        let event = DomainEvent::RoleCreated(RoleCreated {
            role_id,
            name: "admin".to_string(),
            client_id: None,
        });
        if let DomainEvent::RoleCreated(payload) = event {
            assert_eq!(payload.role_id, role_id);
            assert_eq!(payload.name, "admin");
        } else {
            panic!("expected RoleCreated");
        }

        let event = DomainEvent::RoleUpdated(RoleUpdated {
            role_id,
            name: "admin2".to_string(),
            client_id: None,
        });
        if let DomainEvent::RoleUpdated(payload) = event {
            assert_eq!(payload.name, "admin2");
        } else {
            panic!("expected RoleUpdated");
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

        let event = DomainEvent::GroupCreated(GroupChanged {
            group_id,
            name: "engineering".to_string(),
            parent_id: None,
        });
        if let DomainEvent::GroupCreated(payload) = event {
            assert_eq!(payload.group_id, group_id);
        } else {
            panic!("expected GroupCreated");
        }

        let event = DomainEvent::GroupUpdated(GroupChanged {
            group_id,
            name: "engineering2".to_string(),
            parent_id: Some(role_id),
        });
        if let DomainEvent::GroupUpdated(payload) = event {
            assert_eq!(payload.parent_id, Some(role_id));
        } else {
            panic!("expected GroupUpdated");
        }

        let event = DomainEvent::GroupAssigned(UserGroupChanged { user_id, group_id });
        if let DomainEvent::GroupAssigned(payload) = event {
            assert_eq!(payload.user_id, user_id);
        } else {
            panic!("expected GroupAssigned");
        }

        let event = DomainEvent::GroupRemoved(UserGroupChanged { user_id, group_id });
        if let DomainEvent::GroupRemoved(payload) = event {
            assert_eq!(payload.group_id, group_id);
        } else {
            panic!("expected GroupRemoved");
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

    #[test]
    fn webhook_event_catalog_contains_supported_event_types() {
        for group in WEBHOOK_EVENT_CATALOG {
            assert!(!group.id.is_empty());
            assert!(!group.events.is_empty());
            for event in group.events {
                assert_eq!(
                    event.event_type,
                    DomainEvent::event_type(&catalog_event(event.event_type))
                );
                assert!(is_supported_webhook_event_type(event.event_type));
            }
        }

        assert!(is_supported_webhook_event_type("role.created"));
        assert!(!is_supported_webhook_event_type("billing.invoice.created"));
    }

    fn catalog_event(event_type: &str) -> DomainEvent {
        let user_id = Uuid::new_v4();
        let role_id = Uuid::new_v4();
        let group_id = Uuid::new_v4();
        match event_type {
            "user.created" => DomainEvent::UserCreated(UserCreated {
                user_id,
                username: "alice".to_string(),
            }),
            "user.updated" => DomainEvent::UserUpdated(UserChanged {
                user_id,
                username: "alice".to_string(),
            }),
            "user.disabled" => DomainEvent::UserDisabled(UserChanged {
                user_id,
                username: "alice".to_string(),
            }),
            "user.deleted" => DomainEvent::UserDeleted(UserDeleted {
                user_ids: vec![user_id],
            }),
            "user.assigned" => {
                DomainEvent::UserAssignedToGroup(UserGroupChanged { user_id, group_id })
            }
            "user.removed" => {
                DomainEvent::UserRemovedFromGroup(UserGroupChanged { user_id, group_id })
            }
            "role.created" => DomainEvent::RoleCreated(RoleCreated {
                role_id,
                name: "admin".to_string(),
                client_id: None,
            }),
            "role.updated" => DomainEvent::RoleUpdated(RoleUpdated {
                role_id,
                name: "admin".to_string(),
                client_id: None,
            }),
            "role.assigned" => {
                DomainEvent::RoleAssignedToGroup(RoleGroupChanged { role_id, group_id })
            }
            "role.removed" => {
                DomainEvent::RoleRemovedFromGroup(RoleGroupChanged { role_id, group_id })
            }
            "role.deleted" => DomainEvent::RoleDeleted(RoleDeleted {
                role_id,
                affected_user_ids: vec![user_id],
            }),
            "group.created" => DomainEvent::GroupCreated(GroupChanged {
                group_id,
                name: "engineering".to_string(),
                parent_id: None,
            }),
            "group.updated" => DomainEvent::GroupUpdated(GroupChanged {
                group_id,
                name: "engineering".to_string(),
                parent_id: None,
            }),
            "group.assigned" => DomainEvent::GroupAssigned(UserGroupChanged { user_id, group_id }),
            "group.removed" => DomainEvent::GroupRemoved(UserGroupChanged { user_id, group_id }),
            "group.deleted" => DomainEvent::GroupDeleted(GroupDeleted {
                group_ids: vec![group_id],
                affected_user_ids: vec![user_id],
            }),
            _ => panic!("unsupported event type: {event_type}"),
        }
    }
}
