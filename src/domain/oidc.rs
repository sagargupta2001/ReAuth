use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct OidcClient {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uris: String, // Stored as JSON array string
    pub scopes: String,        // Stored as JSON array string
    pub web_origins: String,   // JSON array string (e.g. ["http://localhost:6565"])
    pub managed_by_config: bool,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AuthCode {
    pub code: String,
    #[sqlx(try_from = "String")]
    pub user_id: Uuid,
    pub client_id: String,
    pub redirect_uri: String,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Verifies the PKCE code challenge.
pub fn verify_pkce_challenge(challenge: &str, verifier: &str) -> bool {
    // 1. Create the SHA-256 hash of the verifier
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash_result = hasher.finalize();

    // 2. Encode the hash using Base64 URL-Safe (No Padding)
    let calculated_challenge = URL_SAFE_NO_PAD.encode(hash_result);

    // 3. Compare with the provided challenge
    // We use constant-time comparison if possible, but for PKCE strings == is generally accepted
    challenge == calculated_challenge
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct OidcContext {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OidcRequest {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    // use super::*;
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
}
