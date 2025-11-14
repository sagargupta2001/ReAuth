use crate::{domain::user::User, error::Result};
use serde::{Deserialize, Serialize};
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
}
