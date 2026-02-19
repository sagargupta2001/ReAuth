use super::RefreshToken;
use chrono::{TimeZone, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

#[tokio::test]
async fn refresh_token_from_row_works() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect");
    let id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();
    let now = Utc::now();

    let token: RefreshToken = sqlx::query_as(
        "SELECT ? as id, ? as user_id, ? as realm_id, ? as client_id, ? as expires_at, ? as ip_address, ? as user_agent, ? as created_at, ? as last_used_at",
    )
    .bind(id.to_string())
    .bind(user_id.to_string())
    .bind(realm_id.to_string())
    .bind("client")
    .bind(now)
    .bind("127.0.0.1")
    .bind("agent")
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await
    .expect("fetch token");

    assert_eq!(token.id, id);
    assert_eq!(token.user_id, user_id);
    assert_eq!(token.realm_id, realm_id);
    assert_eq!(token.client_id, Some("client".to_string()));
    // SQLite might lose some precision in datetime, but should be close enough
    assert_eq!(token.expires_at.timestamp(), now.timestamp());
    assert_eq!(token.ip_address, Some("127.0.0.1".to_string()));
    assert_eq!(token.user_agent, Some("agent".to_string()));
}

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

#[test]
fn refresh_token_new_logic() {
    let user_id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();
    let duration = chrono::Duration::hours(1);

    let token = RefreshToken::new(user_id, realm_id, Some("client".to_string()), duration);

    assert!(!token.id.is_nil());
    assert_eq!(token.user_id, user_id);
    assert_eq!(token.realm_id, realm_id);
    assert_eq!(token.client_id, Some("client".to_string()));
    assert!(token.expires_at > token.created_at);
    assert!(!token.is_expired());
}

#[test]
fn refresh_token_expiration() {
    let user_id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();

    // Expired token
    let expired_token = RefreshToken::new(user_id, realm_id, None, chrono::Duration::seconds(-1));
    assert!(expired_token.is_expired());

    // Valid token
    let valid_token = RefreshToken::new(user_id, realm_id, None, chrono::Duration::minutes(1));
    assert!(!valid_token.is_expired());
}
