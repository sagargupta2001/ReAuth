use super::RefreshToken;
use chrono::{TimeZone, Utc};
use uuid::Uuid;

#[test]
fn refresh_token_round_trip() {
    let now = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
    let token = RefreshToken {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        realm_id: Uuid::new_v4(),
        client_id: Some("client".to_string()),
        expires_at: now,
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("test-agent".to_string()),
        created_at: now,
        last_used_at: now,
    };

    let json = serde_json::to_string(&token).expect("serialize");
    let decoded: RefreshToken = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(decoded.id, token.id);
    assert_eq!(decoded.user_id, token.user_id);
    assert_eq!(decoded.realm_id, token.realm_id);
    assert_eq!(decoded.client_id, token.client_id);
    assert_eq!(decoded.expires_at, token.expires_at);
    assert_eq!(decoded.ip_address, token.ip_address);
    assert_eq!(decoded.user_agent, token.user_agent);
    assert_eq!(decoded.created_at, token.created_at);
    assert_eq!(decoded.last_used_at, token.last_used_at);
}
