use crate::error::{Error, Result};
use rsa::{
    pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
    RsaPrivateKey, RsaPublicKey,
};
use std::fs;
use std::path::Path;
use tracing::info;

pub struct KeyPair {
    pub private_key_pem: String,
    pub public_key_pem: String,
}

pub struct KeyManager;

impl KeyManager {
    /// Ensures an RSA keypair exists at the specified path.
    /// If it doesn't, it generates one. Returns the PEM contents.
    pub fn get_or_create_keys(data_dir: &str) -> Result<KeyPair> {
        let private_key_path = Path::new(data_dir).join("private_key.pem");
        let public_key_path = Path::new(data_dir).join("public_key.pem");

        if private_key_path.exists() && public_key_path.exists() {
            info!("Loading existing RSA keys from {:?}", data_dir);
            let private_key_pem = fs::read_to_string(&private_key_path).map_err(|e| {
                Error::Unexpected(anyhow::anyhow!("Failed to read private key: {}", e))
            })?;
            let public_key_pem = fs::read_to_string(&public_key_path).map_err(|e| {
                Error::Unexpected(anyhow::anyhow!("Failed to read public key: {}", e))
            })?;

            return Ok(KeyPair {
                private_key_pem,
                public_key_pem,
            });
        }

        info!("No RSA keys found. Generating new 2048-bit keypair...");

        // 1. Generate Private Key
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).map_err(|e| {
            Error::Unexpected(anyhow::anyhow!("Failed to generate private key: {}", e))
        })?;

        // 2. Derive Public Key
        let public_key = RsaPublicKey::from(&private_key);

        // 3. Convert to PEM format
        let private_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|e| Error::Unexpected(anyhow::anyhow!("Failed to encode private key: {}", e)))?
            .to_string();

        let public_pem = public_key.to_public_key_pem(LineEnding::LF).map_err(|e| {
            Error::Unexpected(anyhow::anyhow!("Failed to encode public key: {}", e))
        })?;

        // 4. Save to disk
        // Ensure dir exists
        if let Some(parent) = private_key_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        fs::write(&private_key_path, &private_pem)
            .map_err(|e| Error::Unexpected(anyhow::anyhow!("Failed to save private key: {}", e)))?;
        fs::write(&public_key_path, &public_pem)
            .map_err(|e| Error::Unexpected(anyhow::anyhow!("Failed to save public key: {}", e)))?;

        info!("New RSA keys saved to {:?}", data_dir);

        Ok(KeyPair {
            private_key_pem: private_pem,
            public_key_pem: public_pem,
        })
    }
}
