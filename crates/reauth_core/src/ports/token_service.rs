use crate::error::Error;
use crate::{domain::user::User, error::Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rsa::RsaPublicKey;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use uuid::Uuid;

/// The claims (payload) for our Access Token (JWT)
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: Uuid,              // Subject (the User ID)
    pub sid: Uuid,              // Session ID (the Refresh Token ID)
    pub perms: HashSet<String>, // Flattened permissions
    pub exp: usize,             // Expiration
}

#[async_trait::async_trait]
pub trait TokenService: Send + Sync {
    /// Creates a new, signed Access Token (JWT)
    async fn create_access_token(
        &self,
        user: &User,
        session_id: Uuid,
        permissions: &HashSet<String>,
    ) -> Result<String>;

    /// Validates an Access Token and returns its claims
    async fn validate_access_token(&self, token: &str) -> Result<AccessTokenClaims>;

    /// Gets the unique ID used to sign tokens (for JWKS endpoint)
    fn get_key_id(&self) -> &str;
    fn get_jwks(&self) -> Result<serde_json::Value>;
}
