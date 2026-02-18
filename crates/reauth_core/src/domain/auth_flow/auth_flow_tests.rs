use super::AuthFlow;
use uuid::Uuid;

#[test]
fn auth_flow_round_trip() {
    let flow = AuthFlow {
        id: Uuid::new_v4(),
        realm_id: Uuid::new_v4(),
        name: "Browser Login".to_string(),
        description: Some("Default browser flow".to_string()),
        alias: "browser".to_string(),
        r#type: "browser".to_string(),
        built_in: true,
    };

    let json = serde_json::to_string(&flow).expect("serialize");
    let decoded: AuthFlow = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(decoded.id, flow.id);
    assert_eq!(decoded.realm_id, flow.realm_id);
    assert_eq!(decoded.name, flow.name);
    assert_eq!(decoded.description, flow.description);
    assert_eq!(decoded.alias, flow.alias);
    assert_eq!(decoded.r#type, flow.r#type);
    assert_eq!(decoded.built_in, flow.built_in);
}
