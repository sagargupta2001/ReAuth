use super::{AuthenticationSession, SessionStatus};
use serde_json::json;
use uuid::Uuid;

#[test]
fn authentication_session_new_sets_defaults() {
    let realm_id = Uuid::new_v4();
    let flow_id = Uuid::new_v4();
    let session = AuthenticationSession::new(realm_id, flow_id, "start".to_string());

    assert_eq!(session.realm_id, realm_id);
    assert_eq!(session.flow_version_id, flow_id);
    assert_eq!(session.current_node_id, "start");
    assert_eq!(session.status, SessionStatus::Active);
    assert!(session.user_id.is_none());
    assert!(session.context.as_object().unwrap().is_empty());
}

#[test]
fn authentication_session_update_context_inserts_value() {
    let mut session =
        AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "start".to_string());

    session.update_context("key", json!("value"));

    assert_eq!(session.context.get("key").unwrap(), "value");
}

#[test]
fn session_status_display_and_from_string() {
    assert_eq!(SessionStatus::Active.to_string(), "active");
    assert_eq!(SessionStatus::Completed.to_string(), "completed");
    assert_eq!(SessionStatus::Failed.to_string(), "failed");

    assert!(matches!(
        SessionStatus::from("completed".to_string()),
        SessionStatus::Completed
    ));
    assert!(matches!(
        SessionStatus::from("failed".to_string()),
        SessionStatus::Failed
    ));
    assert!(matches!(
        SessionStatus::from("other".to_string()),
        SessionStatus::Active
    ));
}
