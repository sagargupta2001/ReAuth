use super::*;
use sqlx::SqlitePool;
use uuid::Uuid;

#[test]
fn verify_pkce_challenge_accepts_valid_pair() {
    // Example from RFC 7636 (Section 4.2).
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

    assert!(verify_pkce_challenge(challenge, verifier));
}

#[test]
fn verify_pkce_challenge_rejects_invalid_pair() {
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    let challenge = "invalid-challenge";

    assert!(!verify_pkce_challenge(challenge, verifier));
}

#[tokio::test]
async fn oidc_models_from_row_parse_fields() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect");
    let now = chrono::Utc::now();

    let client_id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();
    let client: OidcClient = sqlx::query_as(
        "SELECT ? as id, ? as realm_id, ? as client_id, ? as client_secret, ? as redirect_uris, ? as scopes, ? as web_origins, ? as managed_by_config",
    )
    .bind(client_id.to_string())
    .bind(realm_id.to_string())
    .bind("client")
    .bind("secret")
    .bind("[]")
    .bind("[\"openid\"]")
    .bind("[\"http://localhost:3000\"]")
    .bind(true)
    .fetch_one(&pool)
    .await
    .expect("client row");

    assert_eq!(client.id, client_id);
    assert_eq!(client.realm_id, realm_id);
    assert_eq!(client.client_id, "client");
    assert!(client.managed_by_config);

    let user_id = Uuid::new_v4();
    let auth_code: AuthCode = sqlx::query_as(
        "SELECT ? as code, ? as user_id, ? as client_id, ? as redirect_uri, ? as nonce, ? as code_challenge, ? as code_challenge_method, ? as expires_at",
    )
    .bind("code")
    .bind(user_id.to_string())
    .bind("client")
    .bind("https://example.com/callback")
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind("plain")
    .bind(now)
    .fetch_one(&pool)
    .await
    .expect("auth code row");

    assert_eq!(auth_code.code, "code");
    assert_eq!(auth_code.user_id, user_id);
    assert_eq!(auth_code.client_id, "client");
}
