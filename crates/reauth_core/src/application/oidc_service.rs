use crate::domain::pagination::{PageRequest, PageResponse};
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
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub id_token: String,
    pub token_type: String, // e.g., "Bearer"
    pub expires_in: i64,    // e.g., 900 (seconds until expiry)
}

pub struct OidcService {
    oidc_repo: Arc<dyn OidcRepository>,
    user_repo: Arc<dyn UserRepository>,
    auth_service: Arc<AuthService>,
    token_service: Arc<dyn TokenService>,
}

#[derive(Deserialize)]
pub struct UpdateClientRequest {
    pub client_id: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
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
            // Note: todo In a robust multi-tenant system, you might also want to store
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
        let (login_response, refresh_token) = self
            .auth_service
            .create_session(&user, Some(auth_code.client_id.clone()))
            .await?;

        // 6. Map to OIDC response format
        let token_response = TokenResponse {
            access_token: login_response.access_token,
            id_token: login_response.id_token.unwrap_or_default(),
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
    pub async fn register_client(&self, client: &mut OidcClient) -> Result<()> {
        if client.client_secret.is_none() {
            let secret: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(32)
                .map(char::from)
                .collect();

            client.client_secret = Some(secret);
        }

        self.oidc_repo.create_client(client).await
    }

    pub async fn list_clients(
        &self,
        realm_id: Uuid,
        page_req: PageRequest,
    ) -> Result<PageResponse<OidcClient>> {
        self.oidc_repo
            .find_clients_by_realm(&realm_id, &page_req)
            .await
    }

    pub async fn get_client(&self, id: Uuid) -> Result<OidcClient> {
        self.oidc_repo
            .find_client_by_uuid(&id)
            .await?
            .ok_or(Error::OidcClientNotFound(id.to_string()))
    }

    pub async fn update_client(
        &self,
        id: Uuid,
        payload: UpdateClientRequest,
    ) -> Result<OidcClient> {
        let mut client = self.get_client(id).await?;

        if let Some(new_id) = payload.client_id {
            client.client_id = new_id;
        }

        if let Some(uris) = payload.redirect_uris {
            client.redirect_uris =
                serde_json::to_string(&uris).map_err(|e| Error::Unexpected(e.into()))?;
        }

        self.oidc_repo.update_client(&client).await?;
        Ok(client)
    }

    pub fn get_jwks(&self) -> Result<serde_json::Value> {
        self.token_service.get_jwks()
    }
}
