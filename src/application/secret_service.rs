use aes_gcm::aead::{consts::U12, Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngExt;
use sha2::{Digest, Sha256};
use std::env;
use tracing::warn;

use crate::config::Settings;
use crate::error::{Error, Result};

const SECRET_PREFIX: &str = "enc:v1:";

type AesNonce = Nonce<U12>;

pub struct SecretService {
    cipher: Aes256Gcm,
}

fn build_nonce(bytes: &[u8], error_message: &str) -> Result<AesNonce> {
    #[allow(deprecated)]
    let nonce = Nonce::from_exact_iter(bytes.iter().copied())
        .ok_or_else(|| Error::SecurityViolation(error_message.to_string()))?;
    Ok(nonce)
}

impl SecretService {
    pub fn from_settings(settings: &Settings) -> Self {
        let key_source = env::var("REAUTH_SECRET_KEY")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| {
                warn!("REAUTH_SECRET_KEY not set; falling back to auth.jwt_secret");
                settings.auth.jwt_secret.clone()
            });
        Self::from_key(&key_source)
    }

    pub fn from_key(key: &str) -> Self {
        let digest = Sha256::digest(key.as_bytes());
        let cipher = Aes256Gcm::new_from_slice(&digest).expect("valid key length");
        Self { cipher }
    }

    pub fn is_encrypted(&self, value: &str) -> bool {
        value.starts_with(SECRET_PREFIX)
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; 12];
        rand::rng().fill(&mut nonce_bytes);
        let nonce = build_nonce(&nonce_bytes, "Secret encryption failed")?;
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| Error::SecurityViolation("Secret encryption failed".to_string()))?;
        let mut blob = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        blob.extend_from_slice(&nonce_bytes);
        blob.extend_from_slice(&ciphertext);
        let encoded = URL_SAFE_NO_PAD.encode(blob);
        Ok(format!("{}{}", SECRET_PREFIX, encoded))
    }

    pub fn encrypt_if_plain(&self, value: &str) -> Result<String> {
        if self.is_encrypted(value) {
            Ok(value.to_string())
        } else {
            self.encrypt(value)
        }
    }

    pub fn decrypt(&self, value: &str) -> Result<String> {
        if !self.is_encrypted(value) {
            return Ok(value.to_string());
        }

        let encoded = value
            .strip_prefix(SECRET_PREFIX)
            .ok_or_else(|| Error::SecurityViolation("Secret decrypt failed".to_string()))?;
        let decoded = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| Error::SecurityViolation("Secret decrypt failed".to_string()))?;
        if decoded.len() <= 12 {
            return Err(Error::SecurityViolation(
                "Secret decrypt failed".to_string(),
            ));
        }
        let (nonce_bytes, ciphertext) = decoded.split_at(12);
        let nonce = build_nonce(nonce_bytes, "Secret decrypt failed")?;
        let plaintext = self
            .cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|_| Error::SecurityViolation("Secret decrypt failed".to_string()))?;
        String::from_utf8(plaintext)
            .map_err(|_| Error::SecurityViolation("Secret decrypt failed".to_string()))
    }
}
