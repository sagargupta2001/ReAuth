use super::*;
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
