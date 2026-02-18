use crate::domain::pagination::{PageRequest, PageResponse};
use crate::ports::token_service::TokenService;
use crate::{
    application::auth_service::AuthService,
    domain::{
        auth_session::{AuthenticationSession, SessionStatus},
        execution::ExecutionPlan,
        oidc::{AuthCode, OidcClient, OidcContext, OidcRequest},
        session::RefreshToken,
    },
    error::{Error, Result},
    ports::{
        auth_session_repository::AuthSessionRepository, flow_store::FlowStore,
        oidc_repository::OidcRepository, realm_repository::RealmRepository,
        user_repository::UserRepository,
    },
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

#[derive(Deserialize)]
pub struct UpdateClientRequest {
    pub client_id: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    pub web_origins: Option<Vec<String>>,
}

pub struct OidcService {
    oidc_repo: Arc<dyn OidcRepository>,
    user_repo: Arc<dyn UserRepository>,
    auth_service: Arc<AuthService>,
    token_service: Arc<dyn TokenService>,
    // --- NEW DEPENDENCIES ---
    auth_session_repo: Arc<dyn AuthSessionRepository>,
    flow_store: Arc<dyn FlowStore>,
    realm_repo: Arc<dyn RealmRepository>,
}

impl OidcService {
    pub fn new(
        oidc_repo: Arc<dyn OidcRepository>,
        user_repo: Arc<dyn UserRepository>,
        auth_service: Arc<AuthService>,
        token_service: Arc<dyn TokenService>,
        auth_session_repo: Arc<dyn AuthSessionRepository>,
        flow_store: Arc<dyn FlowStore>,
        realm_repo: Arc<dyn RealmRepository>,
    ) -> Self {
        Self {
            oidc_repo,
            user_repo,
            auth_service,
            token_service,
            auth_session_repo,
            flow_store,
            realm_repo,
        }
    }

    /// NEW: The entry point for the browser flow (GET /authorize).
    /// Creates a proper graph session in 'auth_sessions' with OIDC context preserved.
    pub async fn initiate_browser_login(
        &self,
        realm_id: Uuid,
        req: OidcRequest,
    ) -> Result<AuthenticationSession> {
        // 1. Validate Client & Redirect URI immediately
        // This ensures we don't start a flow for a bad client.
        self.validate_client(&realm_id, &req.client_id, &req.redirect_uri)
            .await?;

        // 2. Fetch Realm to find the configured Browser Flow
        let realm = self
            .realm_repo
            .find_by_id(&realm_id)
            .await?
            .ok_or(Error::NotFound("Realm not found".to_string()))?;

        // 3. Identify the Flow ID
        let flow_id_str = realm.browser_flow_id.ok_or(Error::Validation(
            "Realm has no browser flow configured".to_string(),
        ))?;
        let flow_id = Uuid::parse_str(&flow_id_str).unwrap_or_default();

        // 4. Get the Active Version of that Flow (To find Start Node)
        let version = self
            .flow_store
            .get_active_version(&flow_id)
            .await?
            .or(self.flow_store.get_latest_version(&flow_id).await?)
            .ok_or(Error::NotFound("Flow version not found".to_string()))?;

        let plan: ExecutionPlan = serde_json::from_str(&version.execution_artifact)
            .map_err(|e| Error::Unexpected(anyhow::anyhow!("Corrupt execution artifact: {}", e)))?;

        // 5. Construct OIDC Context (Data to preserve across the login flow)
        let oidc_context = OidcContext {
            client_id: req.client_id,
            redirect_uri: req.redirect_uri,
            response_type: req.response_type,
            scope: req.scope,
            state: req.state,
            nonce: req.nonce,
            code_challenge: req.code_challenge,
            code_challenge_method: req.code_challenge_method,
        };

        // 6. Create the Authentication Session
        let session = AuthenticationSession {
            id: Uuid::new_v4(),
            realm_id,
            flow_version_id: Uuid::parse_str(&version.id).unwrap_or_default(),
            current_node_id: plan.start_node_id, // Start at the correct node

            // CRITICAL: Save OIDC data here in the unified JSON context
            context: serde_json::json!({
                "oidc": oidc_context
            }),

            status: SessionStatus::Active,
            user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(30),
        };

        // 7. Persist directly to 'auth_sessions' table
        self.auth_session_repo.create(&session).await?;

        Ok(session)
    }

    /// Handles the creation of the authorization code AFTER login success.
    #[allow(clippy::too_many_arguments)]
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
        // Double check client validation just to be safe
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
        };

        self.oidc_repo.save_auth_code(&auth_code).await?;
        Ok(auth_code)
    }

    /// Handles the `/token` endpoint logic.
    pub async fn exchange_code_for_token(
        &self,
        code: &str,
        code_verifier: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
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

        // 3. Delete the code
        self.oidc_repo.delete_auth_code(code).await?;

        // 4. Get the User
        let user = self
            .user_repo
            .find_by_id(&auth_code.user_id)
            .await?
            .ok_or(Error::UserNotFound)?;

        // 5. Create Session
        let (login_response, refresh_token) = self
            .auth_service
            .create_session(
                &user,
                Some(auth_code.client_id.clone()),
                ip_address,
                user_agent,
            )
            .await?;

        // 6. Map Response
        let token_response = TokenResponse {
            access_token: login_response.access_token,
            id_token: login_response.id_token.unwrap_or_default(),
            token_type: "Bearer".to_string(),
            expires_in: 900,
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

        // Parse allowed URIs
        let allowed_uris: Vec<String> =
            serde_json::from_str(&client.redirect_uris).map_err(|_| {
                Error::Unexpected(anyhow::anyhow!("Invalid redirect_uris format in DB"))
            })?;

        if !allowed_uris.contains(&redirect_uri.to_string()) {
            return Err(Error::OidcInvalidRedirect(redirect_uri.to_string()));
        }

        Ok(client)
    }

    // --- CRUD and Helpers ---

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

    pub async fn find_client_by_client_id(
        &self,
        realm_id: &Uuid,
        client_id: &str,
    ) -> Result<Option<OidcClient>> {
        self.oidc_repo.find_client_by_id(realm_id, client_id).await
    }

    pub async fn update_client_record(&self, client: &OidcClient) -> Result<()> {
        self.oidc_repo.update_client(client).await
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

        if let Some(origins) = payload.web_origins {
            client.web_origins =
                serde_json::to_string(&origins).map_err(|e| Error::Unexpected(e.into()))?;
        }

        self.oidc_repo.update_client(&client).await?;
        Ok(client)
    }

    pub fn get_jwks(&self) -> Result<serde_json::Value> {
        self.token_service.get_jwks()
    }

    pub async fn is_origin_allowed(&self, origin: &str) -> Result<bool> {
        self.oidc_repo.is_origin_allowed(origin).await
    }
}

#[cfg(test)]
mod oidc_service_tests;
