use crate::ports::token_service::TokenService;
use crate::{
    application::auth_service::AuthService,
    domain::{
        oidc::{AuthCode, OidcClient},
        session::RefreshToken,
    },
    error::{Error, Result},
    ports::{oidc_repository::OidcRepository, user_repository::UserRepository},
};
use chrono::{Duration, Utc};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String, // e.g., "Bearer"
    pub expires_in: i64,    // e.g., 900 (seconds until expiry)
}

pub struct OidcService {
    oidc_repo: Arc<dyn OidcRepository>,
    user_repo: Arc<dyn UserRepository>,
    auth_service: Arc<AuthService>,
    token_service: Arc<dyn TokenService>,
}

impl OidcService {
    pub fn new(
        oidc_repo: Arc<dyn OidcRepository>,
        user_repo: Arc<dyn UserRepository>,
        auth_service: Arc<AuthService>,
        token_service: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            oidc_repo,
            user_repo,
            auth_service,
            token_service,
        }
    }

    /// Handles the `/authorize` endpoint logic (creating the authorization code).
    pub async fn create_authorization_code(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        client_id: String,
        redirect_uri: String,
        nonce: Option<String>,
        code_challenge: Option<String>,
        code_challenge_method: String,
    ) -> Result<AuthCode> {
        // 2. Pass realm_id to validation
        // This ensures we are checking for a client that actually exists IN THIS REALM
        let _client = self
            .validate_client(&realm_id, &client_id, &redirect_uri)
            .await?;

        let code = Uuid::new_v4().to_string();
        let auth_code = AuthCode {
            code,
            user_id,
            client_id,
            redirect_uri,
            nonce,
            code_challenge,
            code_challenge_method,
            expires_at: Utc::now() + Duration::seconds(300),
            // Note: In a robust multi-tenant system, you might also want to store
            // `realm_id` on the AuthCode itself, but for now this is sufficient
            // to validate the creation request.
        };

        self.oidc_repo.save_auth_code(&auth_code).await?;
        Ok(auth_code)
    }

    /// Handles the `/token` endpoint logic.
    /// Exchanges code for tokens AND creates a persistent session via AuthService.
    pub async fn exchange_code_for_token(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<(TokenResponse, RefreshToken)> {
        // 1. Find the code
        let auth_code = self
            .oidc_repo
            .find_auth_code_by_code(code)
            .await?
            .ok_or(Error::OidcInvalidCode)?;

        // 2. Verify PKCE
        if !crate::domain::oidc::verify_pkce_challenge(
            auth_code.code_challenge.as_deref().unwrap_or(""),
            code_verifier,
        ) {
            return Err(Error::OidcInvalidCode);
        }

        // 3. Delete the code (Atomic consumption)
        self.oidc_repo.delete_auth_code(code).await?;

        // 4. Get the User
        let user = self
            .user_repo
            .find_by_id(&auth_code.user_id)
            .await?
            .ok_or(Error::UserNotFound)?;

        // 5. Create the Real Session
        // We delegate to AuthService. This calculates permissions, saves the RefreshToken
        // to the DB, and generates the JWT.
        let (login_response, refresh_token) = self.auth_service.create_session(&user).await?;

        // 6. Map to OIDC response format
        let token_response = TokenResponse {
            access_token: login_response.access_token,
            token_type: "Bearer".to_string(),
            expires_in: 900, // Should match your config/AuthService settings
        };

        Ok((token_response, refresh_token))
    }

    /// Validates that a client exists and the redirect URI is allowed.
    pub async fn validate_client(
        &self,
        realm_id: &Uuid,
        client_id: &str,
        redirect_uri: &str,
    ) -> Result<OidcClient> {
        let client = self
            .oidc_repo
            .find_client_by_id(realm_id, client_id)
            .await?
            .ok_or_else(|| Error::OidcClientNotFound(client_id.to_string()))?;

        // Parse the JSON array of allowed URIs
        let allowed_uris: Vec<String> =
            serde_json::from_str(&client.redirect_uris).map_err(|_| {
                Error::Unexpected(anyhow::anyhow!("Invalid redirect_uris format in DB"))
            })?;

        if !allowed_uris.contains(&redirect_uri.to_string()) {
            return Err(Error::OidcInvalidRedirect(redirect_uri.to_string()));
        }

        Ok(client)
    }

    /// Registers a new OIDC client (used by seeder).
    pub async fn register_client(&self, client: &OidcClient) -> Result<()> {
        self.oidc_repo.create_client(client).await
    }

    pub fn get_jwks(&self) -> Result<serde_json::Value> {
        self.token_service.get_jwks()
    }
}
