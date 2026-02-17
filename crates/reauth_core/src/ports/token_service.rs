use crate::{domain::user::User, error::Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub iss: String, // Issuer
    pub sub: String, // Subject (User ID)
    pub aud: String, // Audience (Client ID)
    pub exp: i64,    // Expiration
    pub iat: i64,    // Issued At

    // Profile Claims
    pub preferred_username: String,
    pub groups: Vec<String>,
    // You can add email, picture, etc. here later
}

/// The claims (payload) for our Access Token (JWT)
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: Uuid,              // Subject (the User ID)
    pub sid: Uuid,              // Session ID (the Refresh Token ID)
    pub perms: HashSet<String>, // Flattened permissions
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub exp: usize, // Expiration
}

#[async_trait::async_trait]
pub trait TokenService: Send + Sync {
    /// Creates a new, signed Access Token (JWT)
    async fn create_access_token(
        &self,
        user: &User,
        session_id: Uuid,
        permissions: &HashSet<String>,
        roles: &[String],
        groups: &[String],
    ) -> Result<String>;

    async fn create_id_token(
        &self,
        user: &User,
        client_id: &str, // ID Token needs to know who it's for
        groups: &[String],
    ) -> Result<String>;

    /// Validates an Access Token and returns its claims
    async fn validate_access_token(&self, token: &str) -> Result<AccessTokenClaims>;

    /// Gets the unique ID used to sign tokens (for JWKS endpoint)
    fn get_key_id(&self) -> &str;
    fn get_jwks(&self) -> Result<serde_json::Value>;
}
