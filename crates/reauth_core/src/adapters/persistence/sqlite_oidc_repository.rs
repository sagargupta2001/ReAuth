use crate::adapters::persistence::connection::Database;
use crate::{
    domain::oidc::{AuthCode, OidcClient},
    error::{Error, Result},
    ports::oidc_repository::OidcRepository,
};
use async_trait::async_trait;
use chrono::Utc;

///This repository handles the stateful, temporary data required by the OIDC protocol (Clients and Auth Codes).
pub struct SqliteOidcRepository {
    pool: Database,
}

impl SqliteOidcRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OidcRepository for SqliteOidcRepository {
    // --- Client Management ---

    async fn find_client_by_id(&self, client_id: &str) -> Result<Option<OidcClient>> {
        let client = sqlx::query_as("SELECT * FROM oidc_clients WHERE client_id = ?")
            .bind(client_id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(client)
    }

    async fn create_client(&self, client: &OidcClient) -> Result<()> {
        sqlx::query(
            "INSERT INTO oidc_clients (id, realm_id, client_id, client_secret, redirect_uris, scopes)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
            .bind(client.id.to_string())
            .bind(client.realm_id.to_string())
            .bind(&client.client_id)
            .bind(&client.client_secret)
            .bind(&client.redirect_uris)
            .bind(&client.scopes)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    // --- Auth Code Management ---

    async fn save_auth_code(&self, code: &AuthCode) -> Result<()> {
        sqlx::query(
            "INSERT INTO authorization_codes (code, user_id, client_id, redirect_uri, nonce, code_challenge, code_challenge_method, expires_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
            .bind(&code.code)
            .bind(code.user_id.to_string())
            .bind(&code.client_id)
            .bind(&code.redirect_uri)
            .bind(&code.nonce)
            .bind(&code.code_challenge)
            .bind(&code.code_challenge_method)
            .bind(code.expires_at)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn find_auth_code_by_code(&self, code: &str) -> Result<Option<AuthCode>> {
        let code =
            sqlx::query_as("SELECT * FROM authorization_codes WHERE code = ? AND expires_at > ?")
                .bind(code)
                .bind(Utc::now())
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(code)
    }

    async fn delete_auth_code(&self, code: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM authorization_codes WHERE code = ?")
            .bind(code)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        // Best practice: Check rows affected to detect concurrent usage
        if result.rows_affected() == 0 {
            return Err(Error::Unexpected(anyhow::anyhow!(
                "Authorization code not found for deletion or already expired."
            )));
        }
        Ok(())
    }
}
