use crate::adapters::crypto::key_manager::KeyPair;
use crate::config::AuthConfig;
use crate::error::Error;
use crate::ports::token_service::IdTokenClaims;
use crate::{
    domain::user::User,
    error::Result,
    ports::token_service::{AccessTokenClaims, TokenService},
};
use async_trait::async_trait;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rsa::pkcs8::DecodePublicKey;
use rsa::traits::PublicKeyParts;
use rsa::RsaPublicKey;
use serde_json::json;
use std::collections::HashSet;
use uuid::Uuid;

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_ttl_secs: i64,
    key_id: String,
    pub public_key_pem: String,
    issuer: String,
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
            issuer: config.issuer,
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
        roles: &[String],
        groups: &[String],
    ) -> Result<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::seconds(self.access_token_ttl_secs))
            .expect("Failed to create expiration")
            .timestamp() as usize;

        let claims = AccessTokenClaims {
            sub: user.id,
            sid: session_id,
            perms: permissions.clone(),
            roles: roles.to_vec(),
            groups: groups.to_vec(),
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

    async fn create_id_token(
        &self,
        user: &User,
        client_id: &str,
        groups: &[String],
    ) -> Result<String> {
        let now = Utc::now();
        let expiration = (now + Duration::seconds(self.access_token_ttl_secs)).timestamp();

        let claims = IdTokenClaims {
            iss: self.issuer.clone(),
            sub: user.id.to_string(),
            aud: client_id.to_string(),
            exp: expiration,
            iat: now.timestamp(),
            preferred_username: user.username.clone(),
            groups: groups.to_vec(),
        };

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

    fn get_jwks(&self) -> Result<serde_json::Value> {
        // 1. Parse the PEM to get the raw RSA components
        let public_key = RsaPublicKey::from_public_key_pem(&self.public_key_pem)
            .map_err(|e| Error::Unexpected(anyhow::anyhow!("Failed to parse public key: {}", e)))?;

        // 2. Extract Modulus (n) and Exponent (e)
        let n = public_key.n();
        let e = public_key.e();

        // 3. Convert to Base64URL (Big Endian)
        let n_b64 = URL_SAFE_NO_PAD.encode(n.to_bytes_be());
        let e_b64 = URL_SAFE_NO_PAD.encode(e.to_bytes_be());

        // 4. Construct the JWKS JSON
        Ok(json!({
            "keys": [{
                "kty": "RSA",
                "use": "sig",
                "kid": self.key_id,
                "alg": "RS256",
                "n": n_b64,
                "e": e_b64
            }]
        }))
    }
}
