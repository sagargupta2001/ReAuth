use super::Realm;
use uuid::Uuid;

#[test]
fn realm_round_trip_and_browser_flow_uuid() {
    let flow_id = Uuid::new_v4();
    let realm = Realm {
        id: Uuid::new_v4(),
        name: "default".to_string(),
        access_token_ttl_secs: 3600,
        refresh_token_ttl_secs: 7200,
        browser_flow_id: Some(flow_id.to_string()),
        registration_flow_id: None,
        direct_grant_flow_id: Some(Uuid::new_v4().to_string()),
        reset_credentials_flow_id: None,
    };

    assert_eq!(realm.browser_flow_uuid(), Some(flow_id));

    let json = serde_json::to_string(&realm).expect("serialize");
    let decoded: Realm = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(decoded.id, realm.id);
    assert_eq!(decoded.name, realm.name);
    assert_eq!(decoded.access_token_ttl_secs, realm.access_token_ttl_secs);
    assert_eq!(decoded.refresh_token_ttl_secs, realm.refresh_token_ttl_secs);
    assert_eq!(decoded.browser_flow_id, realm.browser_flow_id);
}

#[test]
fn realm_browser_flow_uuid_returns_none_for_invalid_id() {
    let realm = Realm {
        id: Uuid::new_v4(),
        name: "default".to_string(),
        access_token_ttl_secs: 3600,
        refresh_token_ttl_secs: 7200,
        browser_flow_id: Some("not-a-uuid".to_string()),
        registration_flow_id: None,
        direct_grant_flow_id: None,
        reset_credentials_flow_id: None,
    };

    assert!(realm.browser_flow_uuid().is_none());
}
