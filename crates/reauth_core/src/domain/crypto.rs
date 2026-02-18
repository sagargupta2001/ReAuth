use crate::error::{Error, Result};
use argon2::{Algorithm, Argon2, Params, Version};
use password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use rand_core::OsRng;

#[derive(Debug, Clone)]
pub struct HashedPassword(String);

impl HashedPassword {
    pub fn new(password: &str) -> Result<Self> {
        let salt = SaltString::generate(&mut OsRng);

        // Explicit parameters (no default() anymore)
        let params = Params::new(15_000, 2, 1, None)
            .map_err(|e| Error::Unexpected(anyhow::Error::msg(e.to_string())))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| Error::Unexpected(anyhow::Error::msg(e.to_string())))?
            .to_string();

        Ok(Self(hash))
    }

    pub fn from_hash(hash: &str) -> Result<Self> {
        // We parse the hash just to validate its format.
        PasswordHash::new(hash).map_err(|e| Error::Unexpected(e.into()))?;

        Ok(Self(hash.to_string()))
    }

    pub fn verify(&self, password: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(self.0.as_str())
            .map_err(|e| Error::Unexpected(anyhow::Error::msg(e.to_string())))?;

        let params = Params::new(15_000, 2, 1, None)
            .map_err(|e| Error::Unexpected(anyhow::Error::msg(e.to_string())))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub fn as_str(&self) -> &String {
        &self.0
    }
}

#[cfg(test)]
mod crypto_tests;
