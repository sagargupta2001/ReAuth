use crate::application::audit_service::AuditService;
use crate::application::identity_provider_metadata::{
    load_jwks_with_refresh, maybe_refresh_oidc_discovery,
};
use crate::application::secret_service::SecretService;
use crate::domain::crypto::HashedPassword;
use crate::domain::identity_provider::{
    FederatedIdentity, IdentityProvider, OAuthBrokerResult, OAuthBrokerState, OAuthUpstreamIdentity,
};
use crate::domain::user::User;
use crate::domain::user_email::UserEmail;
use crate::error::{Error, Result};
use crate::ports::auth_session_repository::AuthSessionRepository;
use crate::ports::federated_identity_repository::FederatedIdentityRepository;
use crate::ports::http_client::{HttpDeliveryClient, HttpDeliveryRequest};
use crate::ports::oauth_broker_state_repository::OAuthBrokerStateRepository;
use crate::ports::realm_repository::RealmRepository;
use crate::ports::user_email_repository::UserEmailRepository;
use crate::ports::user_repository::UserRepository;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use rand::distr::{Alphanumeric, SampleString};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;
use uuid::Uuid;

pub struct OAuthBrokerService {
    provider_repo: Arc<dyn crate::ports::identity_provider_repository::IdentityProviderRepository>,
    federation_repo: Arc<dyn FederatedIdentityRepository>,
    broker_state_repo: Arc<dyn OAuthBrokerStateRepository>,
    auth_session_repo: Arc<dyn AuthSessionRepository>,
    realm_repo: Arc<dyn RealmRepository>,
    user_repo: Arc<dyn UserRepository>,
    user_email_repo: Arc<dyn UserEmailRepository>,
    audit_service: Arc<AuditService>,
    secret_service: Arc<SecretService>,
    http_client: Arc<dyn HttpDeliveryClient>,
    public_url: String,
}

pub struct OAuthRedirect {
    pub redirect_url: String,
    pub provider_alias: String,
    pub provider_id: Uuid,
    pub state_id: Uuid,
}

pub struct OAuthCallbackResult {
    pub auth_session_id: Uuid,
    pub broker_result: OAuthBrokerResult,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProviderClaimFetchStrategy {
    Default,
    GithubEmailsEndpoint,
}

impl OAuthBrokerService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider_repo: Arc<
            dyn crate::ports::identity_provider_repository::IdentityProviderRepository,
        >,
        federation_repo: Arc<dyn FederatedIdentityRepository>,
        broker_state_repo: Arc<dyn OAuthBrokerStateRepository>,
        auth_session_repo: Arc<dyn AuthSessionRepository>,
        realm_repo: Arc<dyn RealmRepository>,
        user_repo: Arc<dyn UserRepository>,
        user_email_repo: Arc<dyn UserEmailRepository>,
        audit_service: Arc<AuditService>,
        secret_service: Arc<SecretService>,
        http_client: Arc<dyn HttpDeliveryClient>,
        public_url: String,
    ) -> Self {
        Self {
            provider_repo,
            federation_repo,
            broker_state_repo,
            auth_session_repo,
            realm_repo,
            user_repo,
            user_email_repo,
            audit_service,
            secret_service,
            http_client,
            public_url,
        }
    }

    pub async fn create_redirect(
        &self,
        realm_id: Uuid,
        realm_path: &str,
        auth_session_id: Uuid,
        provider_alias: &str,
    ) -> Result<OAuthRedirect> {
        let realm = self
            .realm_repo
            .find_by_id(&realm_id)
            .await?
            .ok_or_else(|| Error::NotFound("Realm not found".to_string()))?;
        if !realm.idp_broker_enabled {
            return Err(Error::Validation(
                "Identity brokering is disabled for this realm.".to_string(),
            ));
        }
        let mut provider = self
            .provider_repo
            .find_by_alias(&realm_id, provider_alias)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!("Identity provider '{}' not found", provider_alias))
            })?;
        if !provider.enabled || !provider.allow_login {
            return Err(Error::Validation(
                "Identity provider is disabled".to_string(),
            ));
        }
        if maybe_refresh_oidc_discovery(self.http_client.clone(), &mut provider).await? {
            provider.updated_at = Utc::now();
            self.provider_repo.update(&provider).await?;
        }

        let redirect_uri = self.callback_url(realm_path, provider_alias)?;
        let raw_verifier = Alphanumeric.sample_string(&mut rand::rng(), 64);
        let pkce_hash = sha256_text(&raw_verifier);
        let nonce = Some(Alphanumeric.sample_string(&mut rand::rng(), 32));
        let state_id = Uuid::new_v4();
        let now = Utc::now();
        let state = OAuthBrokerState {
            id: state_id,
            realm_id,
            provider_id: provider.id,
            auth_session_id,
            pkce_verifier_hash: pkce_hash,
            redirect_uri: redirect_uri.clone(),
            nonce: nonce.clone(),
            expires_at: now + Duration::minutes(10),
            consumed_at: None,
            created_at: now,
            updated_at: now,
        };
        self.broker_state_repo.create(&state).await?;

        let mut session = self
            .auth_session_repo
            .find_by_id(&auth_session_id)
            .await?
            .ok_or(Error::InvalidLoginSession)?;
        set_encrypted_verifier(
            &mut session.context,
            state_id,
            &self.secret_service.encrypt(&raw_verifier)?,
        );
        self.auth_session_repo.update(&session).await?;

        let redirect_url = build_authorization_url(
            &provider,
            &redirect_uri,
            &state.id.to_string(),
            nonce,
            provider
                .pkce_required
                .then(|| pkce_challenge(&raw_verifier)),
        )?;

        self.audit_service
            .record(crate::domain::audit::NewAuditEvent {
                realm_id,
                actor_user_id: None,
                action: "idp_redirect_issued".to_string(),
                target_type: "identity_provider".to_string(),
                target_id: Some(provider.id.to_string()),
                metadata: json!({
                    "provider_alias": provider.alias,
                    "auth_session_id": auth_session_id,
                    "state_id": state.id
                }),
            })
            .await?;

        Ok(OAuthRedirect {
            redirect_url,
            provider_alias: provider.alias,
            provider_id: provider.id,
            state_id,
        })
    }

    pub async fn handle_callback(
        &self,
        realm_id: Uuid,
        provider_alias: &str,
        code: &str,
        state: &str,
    ) -> Result<OAuthCallbackResult> {
        let realm = self
            .realm_repo
            .find_by_id(&realm_id)
            .await?
            .ok_or_else(|| Error::NotFound("Realm not found".to_string()))?;
        if !realm.idp_broker_enabled {
            return Err(Error::Validation(
                "Identity brokering is disabled for this realm.".to_string(),
            ));
        }
        let state_id = Uuid::parse_str(state)
            .map_err(|_| Error::Validation("Invalid OAuth broker state".to_string()))?;
        let broker_state = self
            .broker_state_repo
            .find_by_id(&state_id)
            .await?
            .ok_or_else(|| Error::Validation("OAuth broker state not found".to_string()))?;
        if broker_state.realm_id != realm_id {
            return Err(Error::Validation(
                "OAuth broker state realm mismatch".to_string(),
            ));
        }
        if !self
            .broker_state_repo
            .mark_consumed_if_active(&state_id, Utc::now())
            .await?
        {
            self.audit_service
                .record(crate::domain::audit::NewAuditEvent {
                    realm_id,
                    actor_user_id: None,
                    action: "idp_state_mismatch".to_string(),
                    target_type: "identity_provider".to_string(),
                    target_id: Some(broker_state.provider_id.to_string()),
                    metadata: json!({
                        "auth_session_id": broker_state.auth_session_id,
                        "state_id": state_id
                    }),
                })
                .await?;
            return Err(Error::Validation(
                "OAuth broker state expired or already consumed".to_string(),
            ));
        }

        let mut provider = self
            .provider_repo
            .find_by_id(&broker_state.provider_id)
            .await?
            .ok_or_else(|| Error::NotFound("Identity provider not found".to_string()))?;
        if provider.realm_id != realm_id || provider.alias != provider_alias || !provider.enabled {
            return Err(Error::Validation(
                "Identity provider is unavailable".to_string(),
            ));
        }
        if maybe_refresh_oidc_discovery(self.http_client.clone(), &mut provider).await? {
            provider.updated_at = Utc::now();
            self.provider_repo.update(&provider).await?;
        }

        let mut session = self
            .auth_session_repo
            .find_by_id(&broker_state.auth_session_id)
            .await?
            .ok_or(Error::InvalidLoginSession)?;
        let verifier = take_encrypted_verifier(&mut session.context, state_id)
            .and_then(|value| self.secret_service.decrypt(&value).ok());
        let verifier = match verifier {
            Some(verifier) => verifier,
            None => {
                self.record_provider_event(
                    provider.realm_id,
                    &provider,
                    Some(broker_state.auth_session_id),
                    None,
                    "idp_pkce_failure",
                    json!({
                        "provider_alias": provider.alias,
                        "state_id": state_id,
                        "reason": "verifier_missing"
                    }),
                )
                .await?;
                return Err(Error::Validation("OAuth PKCE verifier missing".to_string()));
            }
        };
        if sha256_text(&verifier) != broker_state.pkce_verifier_hash {
            self.record_provider_event(
                provider.realm_id,
                &provider,
                Some(broker_state.auth_session_id),
                None,
                "idp_pkce_failure",
                json!({
                    "provider_alias": provider.alias,
                    "state_id": state_id,
                    "reason": "verifier_mismatch"
                }),
            )
            .await?;
            return Err(Error::Validation(
                "OAuth PKCE verifier mismatch".to_string(),
            ));
        }

        let upstream = self
            .exchange_and_fetch_profile(
                &mut provider,
                code,
                &broker_state.redirect_uri,
                &verifier,
                broker_state.nonce.as_deref(),
                broker_state.auth_session_id,
            )
            .await?;
        self.auth_session_repo.update(&session).await?;

        let broker_result = self.resolve_user_link(provider.clone(), upstream).await?;

        self.audit_service
            .record(crate::domain::audit::NewAuditEvent {
                realm_id,
                actor_user_id: broker_result.user_id,
                action: "idp_callback_success".to_string(),
                target_type: "identity_provider".to_string(),
                target_id: Some(provider.id.to_string()),
                metadata: json!({
                    "provider_alias": provider.alias,
                    "auth_session_id": broker_state.auth_session_id,
                    "user_id": broker_result.user_id,
                    "subject": broker_result.subject,
                    "output": broker_result.output
                }),
            })
            .await?;

        Ok(OAuthCallbackResult {
            auth_session_id: broker_state.auth_session_id,
            broker_result,
        })
    }

    async fn exchange_and_fetch_profile(
        &self,
        provider: &mut IdentityProvider,
        code: &str,
        redirect_uri: &str,
        verifier: &str,
        expected_nonce: Option<&str>,
        auth_session_id: Uuid,
    ) -> Result<OAuthUpstreamIdentity> {
        let token_endpoint = provider.token_endpoint.as_deref().ok_or_else(|| {
            Error::Validation("Identity provider token endpoint is missing".to_string())
        })?;
        let request_body = {
            let mut params = url::form_urlencoded::Serializer::new(String::new());
            params.append_pair("grant_type", "authorization_code");
            params.append_pair("code", code);
            params.append_pair("redirect_uri", redirect_uri);
            params.append_pair("client_id", &provider.client_id);
            if provider.pkce_required {
                params.append_pair("code_verifier", verifier);
            }
            if let Some(secret) = provider.client_secret.as_deref() {
                params.append_pair("client_secret", &self.secret_service.decrypt(secret)?);
            }
            params.finish()
        };
        let token_response = self
            .http_client
            .send(HttpDeliveryRequest {
                method: "POST".to_string(),
                url: token_endpoint.to_string(),
                headers: HashMap::from([
                    (
                        "content-type".to_string(),
                        "application/x-www-form-urlencoded".to_string(),
                    ),
                    ("accept".to_string(), "application/json".to_string()),
                ]),
                body: request_body,
            })
            .await
            .map_err(|err| Error::System(format!("Token exchange failed: {}", err.message)));
        let token_response = match token_response {
            Ok(response) => response,
            Err(err) => {
                self.record_provider_event(
                    provider.realm_id,
                    provider,
                    Some(auth_session_id),
                    None,
                    "idp_token_exchange_failure",
                    json!({
                        "provider_alias": provider.alias,
                        "message": err.to_string()
                    }),
                )
                .await?;
                return Err(err);
            }
        };
        if token_response.status_code >= 400 {
            self.record_provider_event(
                provider.realm_id,
                provider,
                Some(auth_session_id),
                None,
                "idp_token_exchange_failure",
                json!({
                    "provider_alias": provider.alias,
                    "status_code": token_response.status_code
                }),
            )
            .await?;
            return Err(Error::Validation(format!(
                "Token exchange failed with status {}",
                token_response.status_code
            )));
        }
        let token_body: TokenResponse = match serde_json::from_str(&token_response.body) {
            Ok(value) => value,
            Err(err) => {
                self.record_provider_event(
                    provider.realm_id,
                    provider,
                    Some(auth_session_id),
                    None,
                    "idp_token_exchange_failure",
                    json!({
                        "provider_alias": provider.alias,
                        "message": err.to_string()
                    }),
                )
                .await?;
                return Err(Error::System(format!("Invalid token response: {}", err)));
            }
        };
        let id_token_claims = match provider.protocol {
            crate::domain::identity_provider::IdentityProviderProtocol::Oidc => Some(
                self.validate_id_token(provider, &token_body, expected_nonce)
                    .await?,
            ),
            crate::domain::identity_provider::IdentityProviderProtocol::Oauth2 => None,
        };
        let userinfo_claims = self
            .fetch_userinfo_claims(provider, &token_body.access_token, auth_session_id)
            .await?;
        let provider_claims = self
            .fetch_provider_specific_claims(provider, &token_body.access_token, auth_session_id)
            .await?;
        let claims = merge_claim_sets(
            merge_claim_sets(id_token_claims, userinfo_claims).map(Some)?,
            provider_claims,
        )?;
        let subject = claim_string(&claims, &["sub", "id"]).ok_or_else(|| {
            Error::Validation("Upstream identity response did not include a subject".to_string())
        })?;
        let email = claim_string(&claims, &["email"]);
        let username = claim_string(&claims, &["preferred_username", "login", "name"]);
        let email_verified = claim_bool(&claims, "email_verified").unwrap_or(false);
        Ok(OAuthUpstreamIdentity {
            subject,
            email,
            email_verified,
            username,
            claims,
        })
    }

    async fn validate_id_token(
        &self,
        provider: &mut IdentityProvider,
        token_body: &TokenResponse,
        expected_nonce: Option<&str>,
    ) -> Result<Value> {
        let id_token = token_body.id_token.as_deref().ok_or_else(|| {
            Error::Validation("OIDC identity provider did not return an id_token".to_string())
        })?;
        let issuer = provider.issuer.clone().ok_or_else(|| {
            Error::Validation("OIDC identity provider issuer is missing".to_string())
        })?;
        let header = decode_header(id_token)?;
        if !matches!(header.alg, Algorithm::RS256 | Algorithm::ES256) {
            return Err(Error::Validation(format!(
                "OIDC id_token used an unsupported signing algorithm: {:?}",
                header.alg
            )));
        }
        let (jwks, refreshed) =
            load_jwks_with_refresh(self.http_client.clone(), provider, header.kid.as_deref())
                .await?;
        if refreshed {
            provider.updated_at = Utc::now();
            self.provider_repo.update(provider).await?;
        }
        let jwk = select_jwk(&jwks, header.kid.as_deref())?;
        let decoding_key = DecodingKey::from_jwk(jwk)
            .map_err(|err| Error::System(format!("Invalid JWKS decoding key: {}", err)))?;

        let mut validation = Validation::new(header.alg);
        validation.set_required_spec_claims(&["exp", "aud", "iss", "sub"]);
        validation.set_audience(&[provider.client_id.as_str()]);
        validation.set_issuer(&[issuer.as_str()]);

        let token_data = decode::<Value>(id_token, &decoding_key, &validation)?;
        if let Some(expected_nonce) = expected_nonce {
            let actual_nonce = claim_string(&token_data.claims, &["nonce"]);
            if actual_nonce.as_deref() != Some(expected_nonce) {
                return Err(Error::Validation(
                    "OIDC id_token nonce mismatch".to_string(),
                ));
            }
        }

        Ok(token_data.claims)
    }

    async fn fetch_userinfo_claims(
        &self,
        provider: &IdentityProvider,
        access_token: &str,
        auth_session_id: Uuid,
    ) -> Result<Option<Value>> {
        let Some(userinfo_endpoint) = provider.userinfo_endpoint.as_deref() else {
            return match provider.protocol {
                crate::domain::identity_provider::IdentityProviderProtocol::Oidc => Ok(None),
                crate::domain::identity_provider::IdentityProviderProtocol::Oauth2 => Err(
                    Error::Validation("Identity provider userinfo endpoint is missing".to_string()),
                ),
            };
        };

        let mut headers = HashMap::from([(
            "authorization".to_string(),
            format!("Bearer {}", access_token),
        )]);
        match claim_fetch_strategy(provider) {
            ProviderClaimFetchStrategy::Default => {
                headers.insert("accept".to_string(), "application/json".to_string());
            }
            ProviderClaimFetchStrategy::GithubEmailsEndpoint => {
                headers.insert(
                    "accept".to_string(),
                    "application/vnd.github+json".to_string(),
                );
                headers.insert("user-agent".to_string(), "ReAuth".to_string());
            }
        }

        let userinfo_response = self
            .http_client
            .send(HttpDeliveryRequest {
                method: "GET".to_string(),
                url: userinfo_endpoint.to_string(),
                headers,
                body: String::new(),
            })
            .await
            .map_err(|err| Error::System(format!("Userinfo request failed: {}", err.message)));
        let userinfo_response = match userinfo_response {
            Ok(response) => response,
            Err(err) => {
                self.record_provider_event(
                    provider.realm_id,
                    provider,
                    Some(auth_session_id),
                    None,
                    "idp_userinfo_failure",
                    json!({
                        "provider_alias": provider.alias,
                        "message": err.to_string()
                    }),
                )
                .await?;
                return Err(err);
            }
        };
        if userinfo_response.status_code >= 400 {
            self.record_provider_event(
                provider.realm_id,
                provider,
                Some(auth_session_id),
                None,
                "idp_userinfo_failure",
                json!({
                    "provider_alias": provider.alias,
                    "status_code": userinfo_response.status_code
                }),
            )
            .await?;
            return Err(Error::Validation(format!(
                "Userinfo request failed with status {}",
                userinfo_response.status_code
            )));
        }

        let claims: Value = match serde_json::from_str(&userinfo_response.body) {
            Ok(value) => value,
            Err(err) => {
                self.record_provider_event(
                    provider.realm_id,
                    provider,
                    Some(auth_session_id),
                    None,
                    "idp_userinfo_failure",
                    json!({
                        "provider_alias": provider.alias,
                        "message": err.to_string()
                    }),
                )
                .await?;
                return Err(Error::System(format!("Invalid userinfo response: {}", err)));
            }
        };
        Ok(Some(claims))
    }

    async fn fetch_provider_specific_claims(
        &self,
        provider: &IdentityProvider,
        access_token: &str,
        auth_session_id: Uuid,
    ) -> Result<Option<Value>> {
        match claim_fetch_strategy(provider) {
            ProviderClaimFetchStrategy::Default => Ok(None),
            ProviderClaimFetchStrategy::GithubEmailsEndpoint => {
                self.fetch_github_email_claims(provider, access_token, auth_session_id)
                    .await
            }
        }
    }

    async fn fetch_github_email_claims(
        &self,
        provider: &IdentityProvider,
        access_token: &str,
        auth_session_id: Uuid,
    ) -> Result<Option<Value>> {
        let userinfo_endpoint = provider.userinfo_endpoint.as_deref().ok_or_else(|| {
            Error::Validation("Identity provider userinfo endpoint is missing".to_string())
        })?;
        let emails_endpoint = format!("{}/emails", userinfo_endpoint.trim_end_matches('/'));
        let response = self
            .http_client
            .send(HttpDeliveryRequest {
                method: "GET".to_string(),
                url: emails_endpoint,
                headers: HashMap::from([
                    (
                        "authorization".to_string(),
                        format!("Bearer {}", access_token),
                    ),
                    (
                        "accept".to_string(),
                        "application/vnd.github+json".to_string(),
                    ),
                    ("user-agent".to_string(), "ReAuth".to_string()),
                ]),
                body: String::new(),
            })
            .await
            .map_err(|err| Error::System(format!("GitHub email request failed: {}", err.message)));
        let response = match response {
            Ok(response) => response,
            Err(err) => {
                self.record_provider_event(
                    provider.realm_id,
                    provider,
                    Some(auth_session_id),
                    None,
                    "idp_userinfo_failure",
                    json!({
                        "provider_alias": provider.alias,
                        "message": err.to_string(),
                        "endpoint": "github_user_emails"
                    }),
                )
                .await?;
                return Err(err);
            }
        };
        if response.status_code >= 400 {
            self.record_provider_event(
                provider.realm_id,
                provider,
                Some(auth_session_id),
                None,
                "idp_userinfo_failure",
                json!({
                    "provider_alias": provider.alias,
                    "status_code": response.status_code,
                    "endpoint": "github_user_emails"
                }),
            )
            .await?;
            return Err(Error::Validation(format!(
                "GitHub email request failed with status {}",
                response.status_code
            )));
        }

        let emails: Vec<GithubEmailRecord> = match serde_json::from_str(&response.body) {
            Ok(value) => value,
            Err(err) => {
                self.record_provider_event(
                    provider.realm_id,
                    provider,
                    Some(auth_session_id),
                    None,
                    "idp_userinfo_failure",
                    json!({
                        "provider_alias": provider.alias,
                        "message": err.to_string(),
                        "endpoint": "github_user_emails"
                    }),
                )
                .await?;
                return Err(Error::System(format!(
                    "Invalid GitHub email response: {}",
                    err
                )));
            }
        };
        let selected = emails
            .iter()
            .find(|entry| entry.primary)
            .or_else(|| emails.iter().find(|entry| entry.verified))
            .or_else(|| emails.first());
        Ok(selected.map(|entry| {
            json!({
                "email": entry.email,
                "email_verified": entry.verified
            })
        }))
    }

    async fn record_provider_event(
        &self,
        realm_id: Uuid,
        provider: &IdentityProvider,
        auth_session_id: Option<Uuid>,
        user_id: Option<Uuid>,
        action: &str,
        metadata: Value,
    ) -> Result<()> {
        self.audit_service
            .record(crate::domain::audit::NewAuditEvent {
                realm_id,
                actor_user_id: user_id,
                action: action.to_string(),
                target_type: "identity_provider".to_string(),
                target_id: Some(provider.id.to_string()),
                metadata: metadata_with_auth_session(metadata, auth_session_id),
            })
            .await
    }

    async fn resolve_user_link(
        &self,
        provider: IdentityProvider,
        upstream: OAuthUpstreamIdentity,
    ) -> Result<OAuthBrokerResult> {
        if let Some(existing) = self
            .federation_repo
            .find_by_provider_subject(&provider.realm_id, &provider.id, &upstream.subject)
            .await?
        {
            let mut existing = existing;
            existing.external_email = upstream.email.clone();
            existing.external_username = upstream.username.clone();
            existing.raw_claims_json = Some(upstream.claims.to_string());
            existing.last_login_at = Some(Utc::now());
            existing.updated_at = Utc::now();
            self.federation_repo.update(&existing).await?;
            let user = self
                .user_repo
                .find_by_id(&existing.user_id)
                .await?
                .ok_or(Error::UserNotFound)?;
            if let Some(reason) = user.sign_in_block_reason(Utc::now()) {
                return Err(Error::Validation(reason.to_string()));
            }
            return Ok(self.build_result(
                &provider,
                "logged_in",
                Some(existing.user_id),
                &upstream,
                None,
            ));
        }

        if provider.allow_email_auto_link && upstream.email_verified {
            if let Some(email) = upstream.email.as_deref() {
                if let Some(user) = self
                    .user_repo
                    .find_by_email(&provider.realm_id, email)
                    .await?
                {
                    let now = Utc::now();
                    if let Some(reason) = user.sign_in_block_reason(now) {
                        return Err(Error::Validation(reason.to_string()));
                    }
                    self.federation_repo
                        .create(&FederatedIdentity {
                            id: Uuid::new_v4(),
                            realm_id: provider.realm_id,
                            provider_id: provider.id,
                            user_id: user.id,
                            subject: upstream.subject.clone(),
                            external_username: upstream.username.clone(),
                            external_email: upstream.email.clone(),
                            raw_claims_json: Some(upstream.claims.to_string()),
                            linked_via: "auto_email".to_string(),
                            last_login_at: Some(now),
                            created_at: now,
                            updated_at: now,
                        })
                        .await?;
                    self.audit_service
                        .record(crate::domain::audit::NewAuditEvent {
                            realm_id: provider.realm_id,
                            actor_user_id: Some(user.id),
                            action: "idp_user_linked".to_string(),
                            target_type: "identity_provider".to_string(),
                            target_id: Some(provider.id.to_string()),
                            metadata: json!({
                                "provider_alias": provider.alias,
                                "subject": upstream.subject,
                                "linked_via": "auto_email"
                            }),
                        })
                        .await?;
                    return Ok(self.build_result(
                        &provider,
                        "logged_in",
                        Some(user.id),
                        &upstream,
                        None,
                    ));
                }
            }
        }

        if let Some(email) = upstream.email.as_deref() {
            if self
                .user_repo
                .find_by_email(&provider.realm_id, email)
                .await?
                .is_some()
            {
                self.audit_service
                    .record(crate::domain::audit::NewAuditEvent {
                        realm_id: provider.realm_id,
                        actor_user_id: None,
                        action: "idp_conflict_email_collision".to_string(),
                        target_type: "identity_provider".to_string(),
                        target_id: Some(provider.id.to_string()),
                        metadata: json!({
                            "provider_alias": provider.alias,
                            "email": email,
                            "subject": upstream.subject,
                            "allow_link": provider.allow_link,
                            "email_verified": upstream.email_verified
                        }),
                    })
                    .await?;

                let message = if provider.allow_link {
                    if upstream.email_verified {
                        format!(
                            "Sign in with your local account to link {} to {}.",
                            provider.display_name, email
                        )
                    } else {
                        format!(
                            "{} returned an unverified email. Sign in with your local account to confirm the link to {}.",
                            provider.display_name, email
                        )
                    }
                } else {
                    format!(
                        "An account for {} already exists, but this provider cannot be linked automatically.",
                        email
                    )
                };

                return Ok(self.build_result(
                    &provider,
                    if provider.allow_link {
                        "link_required"
                    } else {
                        "conflict"
                    },
                    None,
                    &upstream,
                    Some(message),
                ));
            }
        }

        if provider.allow_jit_provisioning {
            let user = self.create_jit_user(&provider, &upstream).await?;
            return Ok(self.build_result(
                &provider,
                "jit_provisioned",
                Some(user.id),
                &upstream,
                None,
            ));
        }

        Ok(self.build_result(
            &provider,
            "conflict",
            None,
            &upstream,
            Some(
                "No eligible local account mapping was available for this identity provider."
                    .to_string(),
            ),
        ))
    }

    pub async fn complete_manual_link(
        &self,
        realm_id: Uuid,
        broker_result: &OAuthBrokerResult,
        username: &str,
        password: &str,
    ) -> Result<OAuthBrokerResult> {
        if broker_result.output != "link_required" {
            return Err(Error::Validation(
                "OAuth link confirmation is not active for this session".to_string(),
            ));
        }

        let provider = self
            .provider_repo
            .find_by_id(&broker_result.provider_id)
            .await?
            .ok_or_else(|| Error::NotFound("Identity provider not found".to_string()))?;
        if provider.realm_id != realm_id {
            return Err(Error::SecurityViolation(
                "Identity provider does not belong to this realm".to_string(),
            ));
        }
        if !provider.allow_link {
            return Err(Error::Validation(
                "This identity provider cannot be linked to local accounts".to_string(),
            ));
        }

        let user = self
            .user_repo
            .find_by_username(&realm_id, username)
            .await?
            .ok_or(Error::InvalidCredentials)?;
        if let Some(reason) = user.sign_in_block_reason(Utc::now()) {
            return Err(Error::Validation(reason.to_string()));
        }
        if user.password_login_disabled {
            return Err(Error::Validation(
                "Password login is disabled for this account. Use another local sign-in method."
                    .to_string(),
            ));
        }
        let hashed = HashedPassword::from_hash(&user.hashed_password)?;
        if !hashed.verify(password)? {
            return Err(Error::InvalidCredentials);
        }
        if let Some(expected_email) = broker_result.external_email.as_deref() {
            let primary_email = self
                .user_email_repo
                .find_primary(&user.id)
                .await?
                .map(|e| e.email_normalized);
            let expected_normalized = expected_email.trim().to_lowercase();
            if primary_email.as_deref() != Some(expected_normalized.as_str()) {
                return Err(Error::Validation(format!(
                    "Sign in with the local account that uses {} to confirm this link.",
                    expected_email
                )));
            }
        }

        if let Some(existing) = self
            .federation_repo
            .find_by_provider_subject(&realm_id, &provider.id, &broker_result.subject)
            .await?
        {
            if existing.user_id == user.id {
                return Ok(self.build_result(
                    &provider,
                    "logged_in",
                    Some(user.id),
                    &OAuthUpstreamIdentity {
                        subject: broker_result.subject.clone(),
                        email: broker_result.external_email.clone(),
                        email_verified: true,
                        username: broker_result.external_username.clone(),
                        claims: json!({}),
                    },
                    None,
                ));
            }
            return Err(Error::Validation(
                "This external identity is already linked to a different local account."
                    .to_string(),
            ));
        }

        let now = Utc::now();
        self.federation_repo
            .create(&FederatedIdentity {
                id: Uuid::new_v4(),
                realm_id,
                provider_id: provider.id,
                user_id: user.id,
                subject: broker_result.subject.clone(),
                external_username: broker_result.external_username.clone(),
                external_email: broker_result.external_email.clone(),
                raw_claims_json: None,
                linked_via: "manual".to_string(),
                last_login_at: Some(now),
                created_at: now,
                updated_at: now,
            })
            .await?;
        self.audit_service
            .record(crate::domain::audit::NewAuditEvent {
                realm_id,
                actor_user_id: Some(user.id),
                action: "idp_user_linked".to_string(),
                target_type: "identity_provider".to_string(),
                target_id: Some(provider.id.to_string()),
                metadata: json!({
                    "provider_alias": provider.alias,
                    "subject": broker_result.subject,
                    "linked_via": "manual"
                }),
            })
            .await?;

        Ok(self.build_result(
            &provider,
            "logged_in",
            Some(user.id),
            &OAuthUpstreamIdentity {
                subject: broker_result.subject.clone(),
                email: broker_result.external_email.clone(),
                email_verified: true,
                username: broker_result.external_username.clone(),
                claims: json!({}),
            },
            None,
        ))
    }

    async fn create_jit_user(
        &self,
        provider: &IdentityProvider,
        upstream: &OAuthUpstreamIdentity,
    ) -> Result<User> {
        let username = upstream
            .username
            .clone()
            .or_else(|| upstream.email.clone())
            .unwrap_or_else(|| {
                format!(
                    "{}-{}",
                    provider.alias,
                    &upstream.subject[..8.min(upstream.subject.len())]
                )
            })
            .to_lowercase()
            .replace(' ', "-");
        let candidate_email = upstream.email.clone();
        if let Some(email) = candidate_email.as_deref() {
            if self
                .user_repo
                .find_by_email(&provider.realm_id, email)
                .await?
                .is_some()
            {
                self.audit_service
                    .record(crate::domain::audit::NewAuditEvent {
                        realm_id: provider.realm_id,
                        actor_user_id: None,
                        action: "idp_conflict_email_collision".to_string(),
                        target_type: "identity_provider".to_string(),
                        target_id: Some(provider.id.to_string()),
                        metadata: json!({
                            "provider_alias": provider.alias,
                            "email": email,
                            "subject": upstream.subject,
                        }),
                    })
                    .await?;
                return Err(Error::Validation(
                    "An existing account already uses this email address".to_string(),
                ));
            }
        }

        let now = Utc::now();
        let password_seed = Alphanumeric.sample_string(&mut rand::rng(), 32);
        let hashed_password = HashedPassword::new(&password_seed)?;
        let user = User {
            id: Uuid::new_v4(),
            realm_id: provider.realm_id,
            username: unique_username(&*self.user_repo, provider.realm_id, &username).await?,
            first_name: None,
            last_name: None,
            hashed_password: hashed_password.as_str().to_string(),
            public_metadata_json: crate::domain::user::EMPTY_METADATA_JSON.to_string(),
            private_metadata_json: crate::domain::user::EMPTY_METADATA_JSON.to_string(),
            unsafe_metadata_json: crate::domain::user::EMPTY_METADATA_JSON.to_string(),
            force_password_reset: false,
            password_login_disabled: true,
            created_at: Some(now),
            updated_at: None,
            last_sign_in_at: Some(now),
            locked_until: None,
            banned_at: None,
        };

        let federation = FederatedIdentity {
            id: Uuid::new_v4(),
            realm_id: provider.realm_id,
            provider_id: provider.id,
            user_id: user.id,
            subject: upstream.subject.clone(),
            external_username: upstream.username.clone(),
            external_email: upstream.email.clone(),
            raw_claims_json: Some(upstream.claims.to_string()),
            linked_via: "jit".to_string(),
            last_login_at: Some(now),
            created_at: now,
            updated_at: now,
        };

        self.user_repo.save(&user, None).await?;
        if let Some(email) = candidate_email {
            let user_email = UserEmail::new(user.id, provider.realm_id, email, true, true);
            self.user_email_repo.save(&user_email, None).await?;
        }
        self.federation_repo.create(&federation).await?;

        self.audit_service
            .record(crate::domain::audit::NewAuditEvent {
                realm_id: provider.realm_id,
                actor_user_id: Some(user.id),
                action: "idp_jit_provisioned".to_string(),
                target_type: "identity_provider".to_string(),
                target_id: Some(provider.id.to_string()),
                metadata: json!({
                    "provider_alias": provider.alias,
                    "subject": upstream.subject
                }),
            })
            .await?;

        Ok(user)
    }

    fn callback_url(&self, realm_path: &str, alias: &str) -> Result<String> {
        let mut base = Url::parse(self.public_url.trim())
            .map_err(|_| Error::System("server.public_url is invalid".to_string()))?;
        base.set_path(&format!(
            "/api/realms/{}/auth/oauth/{}/callback",
            realm_path, alias
        ));
        base.set_query(None);
        Ok(base.to_string())
    }

    fn build_result(
        &self,
        provider: &IdentityProvider,
        output: &str,
        user_id: Option<Uuid>,
        upstream: &OAuthUpstreamIdentity,
        message: Option<String>,
    ) -> OAuthBrokerResult {
        OAuthBrokerResult {
            user_id,
            output: output.to_string(),
            provider_id: provider.id,
            provider_alias: provider.alias.clone(),
            provider_display_name: provider.display_name.clone(),
            subject: upstream.subject.clone(),
            external_email: upstream.email.clone(),
            external_username: upstream.username.clone(),
            message,
        }
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    id_token: Option<String>,
}

#[derive(Deserialize)]
struct GithubEmailRecord {
    email: String,
    #[serde(default)]
    primary: bool,
    #[serde(default)]
    verified: bool,
}

fn merge_claim_sets(base: Option<Value>, overlay: Option<Value>) -> Result<Value> {
    let mut claims = base.unwrap_or_else(|| json!({}));
    if let Some(overlay) = overlay {
        let overlay_map = overlay
            .as_object()
            .ok_or_else(|| Error::System("Upstream claims must be a JSON object".to_string()))?;
        let claims_map = ensure_object(&mut claims);
        for (key, value) in overlay_map {
            claims_map.insert(key.clone(), value.clone());
        }
    }
    Ok(claims)
}

fn claim_fetch_strategy(provider: &IdentityProvider) -> ProviderClaimFetchStrategy {
    match provider.preset_key.as_deref() {
        Some("github") => ProviderClaimFetchStrategy::GithubEmailsEndpoint,
        _ => ProviderClaimFetchStrategy::Default,
    }
}

fn metadata_with_auth_session(mut metadata: Value, auth_session_id: Option<Uuid>) -> Value {
    if let Some(auth_session_id) = auth_session_id {
        ensure_object(&mut metadata).insert(
            "auth_session_id".to_string(),
            Value::String(auth_session_id.to_string()),
        );
    }
    metadata
}

fn select_jwk<'a>(
    jwks: &'a jsonwebtoken::jwk::JwkSet,
    required_kid: Option<&str>,
) -> Result<&'a jsonwebtoken::jwk::Jwk> {
    match required_kid {
        Some(kid) => jwks.find(kid).ok_or_else(|| {
            Error::Validation("OIDC JWKS did not include the expected signing key".to_string())
        }),
        None => jwks.keys.first().ok_or_else(|| {
            Error::Validation("OIDC JWKS did not include any signing keys".to_string())
        }),
    }
}

fn sha256_text(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    URL_SAFE_NO_PAD.encode(hasher.finalize())
}

fn pkce_challenge(verifier: &str) -> String {
    sha256_text(verifier)
}

fn build_authorization_url(
    provider: &IdentityProvider,
    redirect_uri: &str,
    state: &str,
    nonce: Option<String>,
    pkce_challenge: Option<String>,
) -> Result<String> {
    let authorization_endpoint = provider.authorization_endpoint.as_deref().ok_or_else(|| {
        Error::Validation("Identity provider authorization endpoint is missing".to_string())
    })?;
    let mut url = Url::parse(authorization_endpoint).map_err(|_| {
        Error::Validation("Identity provider authorization endpoint is invalid".to_string())
    })?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &provider.client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("scope", &provider.scopes_json_to_space_delimited()?)
        .append_pair("state", state);
    if let Some(nonce) = nonce.as_deref() {
        url.query_pairs_mut().append_pair("nonce", nonce);
    }
    if let Some(challenge) = pkce_challenge.as_deref() {
        url.query_pairs_mut()
            .append_pair("code_challenge", challenge)
            .append_pair("code_challenge_method", "S256");
    }
    Ok(url.to_string())
}

impl IdentityProvider {
    fn scopes_json_to_space_delimited(&self) -> Result<String> {
        let scopes: Vec<String> = serde_json::from_str(&self.scopes_json)
            .map_err(|e| Error::System(format!("Invalid provider scopes: {}", e)))?;
        Ok(scopes.join(" "))
    }
}

fn claim_string(claims: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| {
            claims
                .get(*key)
                .and_then(|value| value.as_str())
                .map(ToString::to_string)
        })
        .or_else(|| {
            keys.iter().find_map(|key| {
                claims
                    .get(*key)
                    .and_then(|value| value.as_i64())
                    .map(|value| value.to_string())
            })
        })
}

fn claim_bool(claims: &Value, key: &str) -> Option<bool> {
    claims.get(key).and_then(|value| value.as_bool())
}

fn set_encrypted_verifier(context: &mut Value, state_id: Uuid, encrypted_verifier: &str) {
    let map = ensure_object(context);
    let store = map
        .entry("oauth_broker_verifiers".to_string())
        .or_insert_with(|| json!({}));
    let store_map = ensure_object(store);
    store_map.insert(
        state_id.to_string(),
        Value::String(encrypted_verifier.to_string()),
    );
}

fn take_encrypted_verifier(context: &mut Value, state_id: Uuid) -> Option<String> {
    let map = context.as_object_mut()?;
    let store = map.get_mut("oauth_broker_verifiers")?.as_object_mut()?;
    store
        .remove(&state_id.to_string())
        .and_then(|value| value.as_str().map(ToString::to_string))
}

fn ensure_object(value: &mut Value) -> &mut serde_json::Map<String, Value> {
    if !value.is_object() {
        *value = json!({});
    }
    value.as_object_mut().expect("object")
}

async fn unique_username(repo: &dyn UserRepository, realm_id: Uuid, base: &str) -> Result<String> {
    let mut candidate = sanitize_username(base);
    if candidate.is_empty() {
        candidate = "brokered-user".to_string();
    }
    if repo
        .find_by_username(&realm_id, &candidate)
        .await?
        .is_none()
    {
        return Ok(candidate);
    }
    for idx in 1..=1000 {
        let next = format!("{}-{}", candidate, idx);
        if repo.find_by_username(&realm_id, &next).await?.is_none() {
            return Ok(next);
        }
    }
    Err(Error::Validation(
        "Could not allocate a unique username".to_string(),
    ))
}

fn sanitize_username(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
