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
