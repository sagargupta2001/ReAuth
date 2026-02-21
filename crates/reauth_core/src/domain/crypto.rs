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
        // Use lower parameters during tests to speed up the suite
        #[cfg(not(test))]
        let (m_cost, t_cost, p_cost) = (15_000, 2, 1);
        #[cfg(test)]
        let (m_cost, t_cost, p_cost) = (500, 1, 1);

        let params = Params::new(m_cost, t_cost, p_cost, None)
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

        #[cfg(not(test))]
        let (m_cost, t_cost, p_cost) = (15_000, 2, 1);
        #[cfg(test)]
        let (m_cost, t_cost, p_cost) = (500, 1, 1);

        let params = Params::new(m_cost, t_cost, p_cost, None)
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
mod tests {
    use super::*;
    // use super::*;
    use crate::error::Error;

    #[test]
    fn hashed_password_round_trip_verifies() {
        let hash =
            HashedPassword::new("correct-horse-battery-staple").expect("hash should be created");

        let parsed = HashedPassword::from_hash(hash.as_str()).expect("hash should parse");

        assert!(parsed
            .verify("correct-horse-battery-staple")
            .expect("verify should succeed"));
        assert!(!parsed
            .verify("wrong-password")
            .expect("verify should succeed"));
    }

    #[test]
    fn hashed_password_rejects_invalid_hash() {
        let err =
            HashedPassword::from_hash("not-a-valid-hash").expect_err("invalid hash should fail");
        assert!(matches!(err, Error::Unexpected(_)));
    }
}
