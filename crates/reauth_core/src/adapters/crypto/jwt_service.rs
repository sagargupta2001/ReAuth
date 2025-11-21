use crate::adapters::crypto::key_manager::KeyPair;
use crate::config::AuthConfig;
use crate::error::Error;
use crate::{
    domain::user::User,
    error::Result,
    ports::token_service::{AccessTokenClaims, TokenService},
};
use async_trait::async_trait;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use std::collections::HashSet;
use uuid::Uuid;

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_ttl_secs: i64,
    key_id: String,
    pub public_key_pem: String,
}

impl JwtService {
    pub fn new(config: AuthConfig, keys: KeyPair) -> Result<Self> {
        // Create EncodingKey from Private PEM (for signing)
        let encoding_key = EncodingKey::from_rsa_pem(keys.private_key_pem.as_bytes())
            .map_err(|e| Error::Unexpected(anyhow::anyhow!("Invalid Private Key: {}", e)))?;

        // Create DecodingKey from Public PEM (for verification)
        let decoding_key = DecodingKey::from_rsa_pem(keys.public_key_pem.as_bytes())
            .map_err(|e| Error::Unexpected(anyhow::anyhow!("Invalid Public Key: {}", e)))?;

        Ok(Self {
            encoding_key,
            decoding_key,
            access_token_ttl_secs: config.access_token_ttl_secs,
            key_id: config.jwt_key_id,
            public_key_pem: keys.public_key_pem,
        })
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
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.key_id.clone());

        Ok(
            encode(&header, &claims, &self.encoding_key)
                .map_err(|e| Error::Unexpected(e.into()))?,
        )
    }

    async fn validate_access_token(&self, token: &str) -> Result<AccessTokenClaims> {
        let validation = Validation::new(Algorithm::RS256);

        let token_data = decode::<AccessTokenClaims>(token, &self.decoding_key, &validation)
            .map_err(|_| Error::InvalidCredentials)?;

        Ok(token_data.claims)
    }

    fn get_key_id(&self) -> &str {
        &self.key_id
    }
}
