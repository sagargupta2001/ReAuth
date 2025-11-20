use crate::config::AuthConfig;
use crate::{
    domain::user::User,
    error::Result,
    ports::token_service::{AccessTokenClaims, TokenService},
};
use async_trait::async_trait;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::collections::HashSet;
use uuid::Uuid;

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_ttl_secs: i64,
    key_id: String,
}

impl JwtService {
    pub fn new(config: AuthConfig) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(config.jwt_secret.as_ref()),
            decoding_key: DecodingKey::from_secret(config.jwt_secret.as_ref()),
            access_token_ttl_secs: config.access_token_ttl_secs,
            key_id: config.jwt_key_id,
        }
    }
}

#[async_trait]
impl TokenService for JwtService {
    async fn create_access_token(
        &self,
        user: &User,
        session_id: Uuid,
        permissions: &HashSet<String>,
    ) -> Result<String> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::seconds(self.access_token_ttl_secs))
            .expect("Failed to create expiration")
            .timestamp() as usize;

        let claims = AccessTokenClaims {
            sub: user.id,
            sid: session_id,
            perms: permissions.clone(),
            exp: expiration,
        };

        // Set the Key ID in the header
        let header = Header {
            kid: Some(self.key_id.clone()),
            ..Header::default()
        };

        Ok(encode(&header, &claims, &self.encoding_key)?)
    }

    async fn validate_access_token(&self, token: &str) -> Result<AccessTokenClaims> {
        let token_data =
            decode::<AccessTokenClaims>(token, &self.decoding_key, &Validation::default())?;
        Ok(token_data.claims)
    }

    fn get_key_id(&self) -> &str {
        &self.key_id
    }
}
