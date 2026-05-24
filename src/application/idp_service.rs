use crate::application::audit_service::AuditService;
use crate::application::identity_provider_metadata::{
    force_refresh_jwks, force_refresh_oidc_discovery, maybe_refresh_oidc_discovery,
};
use crate::application::secret_service::SecretService;
use crate::domain::audit::AuditActionCount;
use crate::domain::identity_provider::{
    IdentityProvider, IdentityProviderPreset, IdentityProviderProtocol,
};
use crate::domain::realm::{RealmIdpDefaultEmailLinkPolicy, RealmIdpDefaultJitPolicy};
use crate::error::{Error, Result};
use crate::ports::federated_identity_repository::FederatedIdentityRepository;
use crate::ports::http_client::{HttpDeliveryClient, HttpDeliveryRequest};
use crate::ports::identity_provider_repository::IdentityProviderRepository;
use crate::ports::realm_repository::RealmRepository;
use crate::ports::user_repository::UserRepository;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateIdentityProviderRequest {
    pub preset: Option<String>,
    pub alias: String,
    pub display_name: String,
    pub protocol: IdentityProviderProtocol,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub issuer: Option<String>,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
    pub userinfo_endpoint: Option<String>,
    pub jwks_uri: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub claim_mapping: Option<Value>,
    pub pkce_required: Option<bool>,
    pub allow_login: Option<bool>,
    pub allow_link: Option<bool>,
    pub allow_jit_provisioning: Option<bool>,
    pub allow_email_auto_link: Option<bool>,
    pub require_verified_email: Option<bool>,
    pub icon_ref: Option<String>,
    pub button_color: Option<String>,
    pub sort_order: Option<i64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIdentityProviderRequest {
    pub alias: Option<String>,
    pub display_name: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub issuer: Option<String>,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
    pub userinfo_endpoint: Option<String>,
    pub jwks_uri: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub claim_mapping: Option<Value>,
    pub pkce_required: Option<bool>,
    pub allow_login: Option<bool>,
    pub allow_link: Option<bool>,
    pub allow_jit_provisioning: Option<bool>,
    pub allow_email_auto_link: Option<bool>,
    pub require_verified_email: Option<bool>,
    pub icon_ref: Option<String>,
    pub button_color: Option<String>,
    pub sort_order: Option<i64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct IdentityProviderResponse {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub alias: String,
    pub display_name: String,
    pub protocol: IdentityProviderProtocol,
    pub preset_key: Option<String>,
    pub enabled: bool,
    pub client_id: String,
    pub issuer: Option<String>,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
    pub userinfo_endpoint: Option<String>,
    pub jwks_uri: Option<String>,
    pub scopes: Vec<String>,
    pub claim_mapping: Value,
    pub pkce_required: bool,
    pub allow_login: bool,
    pub allow_link: bool,
    pub allow_jit_provisioning: bool,
    pub allow_email_auto_link: bool,
    pub require_verified_email: bool,
    pub icon_ref: Option<String>,
    pub button_color: Option<String>,
    pub sort_order: i64,
    pub metadata_cached_at: Option<chrono::DateTime<chrono::Utc>>,
    pub jwks_cached_at: Option<chrono::DateTime<chrono::Utc>>,
    pub client_secret_set: bool,
    pub client_secret_mask: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProviderLoginOption {
    pub alias: String,
    pub display_name: String,
    pub icon_ref: Option<String>,
    pub button_color: Option<String>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityProviderLinkedUser {
    pub federated_identity_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub subject: String,
    pub external_username: Option<String>,
    pub external_email: Option<String>,
    pub linked_via: String,
    pub linked_at: DateTime<Utc>,
    pub last_provider_login_at: Option<DateTime<Utc>>,
    pub last_user_sign_in_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteIdentityProviderResult {
    pub provider_id: Uuid,
    pub provider_alias: String,
    pub outcome: String,
    pub linked_identity_count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityProviderConnectionCheck {
    pub attempted: bool,
    pub ok: bool,
    pub status_code: Option<u16>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityProviderConnectionTestResult {
    pub provider_id: Uuid,
    pub provider_alias: String,
    pub protocol: IdentityProviderProtocol,
    pub ok: bool,
    pub discovery: IdentityProviderConnectionCheck,
    pub token_endpoint: IdentityProviderConnectionCheck,
    pub userinfo_endpoint: IdentityProviderConnectionCheck,
    pub jwks: IdentityProviderConnectionCheck,
    pub metadata_cached_at: Option<DateTime<Utc>>,
    pub jwks_cached_at: Option<DateTime<Utc>>,
    pub tested_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityProviderActivitySummary {
    pub total_events_last_24h: u64,
    pub failures_last_24h: u64,
    pub callback_success_last_24h: u64,
    pub links_last_24h: u64,
    pub jit_provisioned_last_24h: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityProviderActivityEvent {
    pub audit_event_id: Uuid,
    pub action: String,
    pub created_at: String,
    pub actor_user_id: Option<Uuid>,
    pub auth_session_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub subject: Option<String>,
    pub email: Option<String>,
    pub linked_via: Option<String>,
    pub message: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityProviderActivityFeed {
    pub provider_id: Uuid,
    pub provider_alias: String,
    pub summary: IdentityProviderActivitySummary,
    pub recent_events: Vec<IdentityProviderActivityEvent>,
}

pub struct IdentityProviderService {
    repo: Arc<dyn IdentityProviderRepository>,
    federated_identity_repo: Arc<dyn FederatedIdentityRepository>,
    realm_repo: Arc<dyn RealmRepository>,
    user_repo: Arc<dyn UserRepository>,
    audit_service: Arc<AuditService>,
    secret_service: Arc<SecretService>,
    http_client: Arc<dyn HttpDeliveryClient>,
}

impl IdentityProviderService {
    pub fn new(
        repo: Arc<dyn IdentityProviderRepository>,
        federated_identity_repo: Arc<dyn FederatedIdentityRepository>,
        realm_repo: Arc<dyn RealmRepository>,
        user_repo: Arc<dyn UserRepository>,
        audit_service: Arc<AuditService>,
        secret_service: Arc<SecretService>,
        http_client: Arc<dyn HttpDeliveryClient>,
    ) -> Self {
        Self {
            repo,
            federated_identity_repo,
            realm_repo,
            user_repo,
            audit_service,
            secret_service,
            http_client,
        }
    }

    pub async fn list_by_realm(&self, realm_id: Uuid) -> Result<Vec<IdentityProviderResponse>> {
        let providers = self.repo.list_by_realm(&realm_id).await?;
        providers.into_iter().map(Self::to_response).collect()
    }

    pub async fn list_enabled_login_options(
        &self,
        realm_id: Uuid,
    ) -> Result<Vec<IdentityProviderLoginOption>> {
        let realm = self
            .realm_repo
            .find_by_id(&realm_id)
            .await?
            .ok_or_else(|| Error::NotFound("Realm not found".to_string()))?;
        if !realm.idp_broker_enabled {
            return Ok(Vec::new());
        }
        let mut providers = self.repo.list_by_realm(&realm_id).await?;
        providers.retain(|provider| provider.enabled && provider.allow_login);
        providers.sort_by(|left, right| {
            left.sort_order
                .cmp(&right.sort_order)
                .then_with(|| left.display_name.cmp(&right.display_name))
        });
        Ok(providers
            .into_iter()
            .map(|provider| IdentityProviderLoginOption {
                alias: provider.alias,
                display_name: provider.display_name,
                icon_ref: provider.icon_ref,
                button_color: provider.button_color,
                sort_order: provider.sort_order,
            })
            .collect())
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<IdentityProviderResponse> {
        let provider = self
            .repo
            .find_by_id(&id)
            .await?
            .ok_or_else(|| Error::NotFound("Identity provider not found".to_string()))?;
        Self::to_response(provider)
    }

    pub async fn list_linked_users(&self, id: Uuid) -> Result<Vec<IdentityProviderLinkedUser>> {
        let provider = self
            .repo
            .find_by_id(&id)
            .await?
            .ok_or_else(|| Error::NotFound("Identity provider not found".to_string()))?;
        let identities = self
            .federated_identity_repo
            .list_by_provider(&provider.realm_id, &provider.id)
            .await?;

        let mut linked_users = Vec::with_capacity(identities.len());
        for identity in identities {
            let Some(user) = self.user_repo.find_by_id(&identity.user_id).await? else {
                continue;
            };
            if user.realm_id != provider.realm_id {
                continue;
            }
            linked_users.push(IdentityProviderLinkedUser {
                federated_identity_id: identity.id,
                user_id: user.id,
                username: user.username,
                email: user.email,
                subject: identity.subject,
                external_username: identity.external_username,
                external_email: identity.external_email,
                linked_via: identity.linked_via,
                linked_at: identity.created_at,
                last_provider_login_at: identity.last_login_at,
                last_user_sign_in_at: user.last_sign_in_at,
            });
        }

        Ok(linked_users)
    }

    pub async fn list_recent_activity(
        &self,
        id: Uuid,
        limit: usize,
    ) -> Result<IdentityProviderActivityFeed> {
        let provider = self
            .repo
            .find_by_id(&id)
            .await?
            .ok_or_else(|| Error::NotFound("Identity provider not found".to_string()))?;
        let action_counts = self
            .audit_service
            .count_by_target_and_actions_since(
                provider.realm_id,
                "identity_provider",
                &provider.id.to_string(),
                idp_audit_actions(),
                Some(Utc::now() - chrono::Duration::hours(24)),
            )
            .await?;
        let events = self
            .audit_service
            .list_recent_by_target_and_actions(
                provider.realm_id,
                "identity_provider",
                &provider.id.to_string(),
                idp_audit_actions(),
                limit,
            )
            .await?;

        Ok(IdentityProviderActivityFeed {
            provider_id: provider.id,
            provider_alias: provider.alias,
            summary: summarize_activity_counts(&action_counts),
            recent_events: events.into_iter().map(map_activity_event).collect(),
        })
    }

    pub async fn get_domain_by_alias(
        &self,
        realm_id: Uuid,
        alias: &str,
    ) -> Result<IdentityProvider> {
        self.repo
            .find_by_alias(&realm_id, alias)
            .await?
            .ok_or_else(|| Error::NotFound(format!("Identity provider '{}' not found", alias)))
    }

    pub fn list_presets(&self) -> Vec<IdentityProviderPreset> {
        built_in_presets()
    }

    pub async fn create(
        &self,
        realm_id: Uuid,
        request: CreateIdentityProviderRequest,
    ) -> Result<IdentityProviderResponse> {
        let realm = self
            .realm_repo
            .find_by_id(&realm_id)
            .await?
            .ok_or_else(|| Error::NotFound("Realm not found".to_string()))?;
        validate_alias(&request.alias)?;
        if self
            .repo
            .find_by_alias(&realm_id, &request.alias)
            .await?
            .is_some()
        {
            return Err(Error::Validation(
                "Identity provider alias already exists in this realm".to_string(),
            ));
        }

        let preset = request.preset.as_deref().and_then(find_preset);
        let now = Utc::now();
        let provider = IdentityProvider {
            id: Uuid::new_v4(),
            realm_id,
            alias: request.alias,
            display_name: request.display_name,
            protocol: request.protocol,
            preset_key: preset.as_ref().map(|value| value.key.clone()),
            enabled: request.enabled.unwrap_or(false),
            client_id: request.client_id,
            client_secret: request
                .client_secret
                .as_deref()
                .map(|value| self.secret_service.encrypt(value))
                .transpose()?,
            issuer: request
                .issuer
                .or_else(|| preset.as_ref().and_then(|value| value.issuer.clone())),
            authorization_endpoint: request.authorization_endpoint.or_else(|| {
                preset
                    .as_ref()
                    .and_then(|value| value.authorization_endpoint.clone())
            }),
            token_endpoint: request.token_endpoint.or_else(|| {
                preset
                    .as_ref()
                    .and_then(|value| value.token_endpoint.clone())
            }),
            userinfo_endpoint: request.userinfo_endpoint.or_else(|| {
                preset
                    .as_ref()
                    .and_then(|value| value.userinfo_endpoint.clone())
            }),
            jwks_uri: request
                .jwks_uri
                .or_else(|| preset.as_ref().and_then(|value| value.jwks_uri.clone())),
            scopes_json: serde_json::to_string(
                &request
                    .scopes
                    .unwrap_or_else(|| preset_scopes(&preset))
                    .into_iter()
                    .collect::<Vec<_>>(),
            )
            .map_err(|e| Error::System(format!("Failed to serialize provider scopes: {}", e)))?,
            claim_mapping_json: serde_json::to_string(
                &request
                    .claim_mapping
                    .unwrap_or_else(|| preset_claim_mapping(&preset)),
            )
            .map_err(|e| Error::System(format!("Failed to serialize claim mapping: {}", e)))?,
            pkce_required: request.pkce_required.unwrap_or(true),
            allow_login: request.allow_login.unwrap_or(true),
            allow_link: request.allow_link.unwrap_or(true),
            allow_jit_provisioning: request.allow_jit_provisioning.unwrap_or(
                default_allow_jit_provisioning(&realm.idp_default_jit_policy),
            ),
            allow_email_auto_link: request.allow_email_auto_link.unwrap_or(
                default_allow_email_auto_link(&realm.idp_default_email_link_policy),
            ),
            require_verified_email: request.require_verified_email.unwrap_or(
                default_require_verified_email(&realm.idp_default_email_link_policy),
            ),
            icon_ref: request
                .icon_ref
                .or_else(|| preset.as_ref().and_then(|value| value.icon_ref.clone())),
            button_color: request.button_color,
            sort_order: request.sort_order.unwrap_or(0),
            metadata_cached_at: None,
            metadata_cache_json: None,
            jwks_cached_at: None,
            jwks_cache_json: None,
            created_at: now,
            updated_at: now,
        };

        let mut provider = provider;
        maybe_refresh_oidc_discovery(self.http_client.clone(), &mut provider).await?;
        self.repo.create(&provider).await?;
        Self::to_response(provider)
    }

    pub async fn update(
        &self,
        id: Uuid,
        request: UpdateIdentityProviderRequest,
    ) -> Result<IdentityProviderResponse> {
        let mut provider = self
            .repo
            .find_by_id(&id)
            .await?
            .ok_or_else(|| Error::NotFound("Identity provider not found".to_string()))?;

        if let Some(alias) = request.alias {
            validate_alias(&alias)?;
            provider.alias = alias;
        }
        if let Some(value) = request.display_name {
            provider.display_name = value;
        }
        if let Some(value) = request.client_id {
            provider.client_id = value;
        }
        if let Some(value) = request.client_secret {
            provider.client_secret = Some(self.secret_service.encrypt(&value)?);
        }
        if let Some(value) = request.issuer {
            provider.issuer = Some(value);
        }
        if let Some(value) = request.authorization_endpoint {
            provider.authorization_endpoint = Some(value);
        }
        if let Some(value) = request.token_endpoint {
            provider.token_endpoint = Some(value);
        }
        if let Some(value) = request.userinfo_endpoint {
            provider.userinfo_endpoint = Some(value);
        }
        if let Some(value) = request.jwks_uri {
            provider.jwks_uri = Some(value);
        }
        if let Some(value) = request.scopes {
            provider.scopes_json = serde_json::to_string(&value).map_err(|e| {
                Error::System(format!("Failed to serialize provider scopes: {}", e))
            })?;
        }
        if let Some(value) = request.claim_mapping {
            provider.claim_mapping_json = serde_json::to_string(&value)
                .map_err(|e| Error::System(format!("Failed to serialize claim mapping: {}", e)))?;
        }
        if let Some(value) = request.pkce_required {
            provider.pkce_required = value;
        }
        if let Some(value) = request.allow_login {
            provider.allow_login = value;
        }
        if let Some(value) = request.allow_link {
            provider.allow_link = value;
        }
        if let Some(value) = request.allow_jit_provisioning {
            provider.allow_jit_provisioning = value;
        }
        if let Some(value) = request.allow_email_auto_link {
            provider.allow_email_auto_link = value;
        }
        if let Some(value) = request.require_verified_email {
            provider.require_verified_email = value;
        }
        if let Some(value) = request.icon_ref {
            provider.icon_ref = Some(value);
        }
        if let Some(value) = request.button_color {
            provider.button_color = Some(value);
        }
        if let Some(value) = request.sort_order {
            provider.sort_order = value;
        }
        if let Some(value) = request.enabled {
            provider.enabled = value;
        }
        provider.updated_at = Utc::now();
        maybe_refresh_oidc_discovery(self.http_client.clone(), &mut provider).await?;
        self.repo.update(&provider).await?;
        Self::to_response(provider)
    }

    pub async fn delete(
        &self,
        id: Uuid,
        hard_delete: bool,
    ) -> Result<DeleteIdentityProviderResult> {
        let mut provider = self
            .repo
            .find_by_id(&id)
            .await?
            .ok_or_else(|| Error::NotFound("Identity provider not found".to_string()))?;
        let linked_identity_count = self
            .federated_identity_repo
            .count_by_provider(&provider.realm_id, &provider.id)
            .await?;

        if linked_identity_count > 0 && !hard_delete {
            provider.enabled = false;
            provider.allow_login = false;
            provider.allow_link = false;
            provider.updated_at = Utc::now();
            self.repo.update(&provider).await?;
            return Ok(DeleteIdentityProviderResult {
                provider_id: provider.id,
                provider_alias: provider.alias,
                outcome: "soft_deleted".to_string(),
                linked_identity_count,
            });
        }

        if linked_identity_count > 0 {
            self.federated_identity_repo
                .delete_by_provider(&provider.realm_id, &provider.id)
                .await?;
        }
        self.repo.delete(&id).await?;
        Ok(DeleteIdentityProviderResult {
            provider_id: provider.id,
            provider_alias: provider.alias,
            outcome: "hard_deleted".to_string(),
            linked_identity_count,
        })
    }

    pub async fn refresh_metadata(&self, id: Uuid) -> Result<IdentityProviderResponse> {
        let mut provider = self
            .repo
            .find_by_id(&id)
            .await?
            .ok_or_else(|| Error::NotFound("Identity provider not found".to_string()))?;
        force_refresh_oidc_discovery(self.http_client.clone(), &mut provider).await?;
        provider.updated_at = Utc::now();
        self.repo.update(&provider).await?;
        Self::to_response(provider)
    }

    pub async fn test_connection(&self, id: Uuid) -> Result<IdentityProviderConnectionTestResult> {
        let mut provider = self
            .repo
            .find_by_id(&id)
            .await?
            .ok_or_else(|| Error::NotFound("Identity provider not found".to_string()))?;
        let mut provider_changed = false;

        let discovery = if provider.protocol == IdentityProviderProtocol::Oidc {
            if provider.issuer.is_some() {
                match force_refresh_oidc_discovery(self.http_client.clone(), &mut provider).await {
                    Ok(_) => {
                        provider_changed = true;
                        IdentityProviderConnectionCheck {
                            attempted: true,
                            ok: true,
                            status_code: Some(200),
                            detail: "OIDC discovery refreshed".to_string(),
                        }
                    }
                    Err(err) => IdentityProviderConnectionCheck {
                        attempted: true,
                        ok: false,
                        status_code: None,
                        detail: err.to_string(),
                    },
                }
            } else {
                IdentityProviderConnectionCheck {
                    attempted: false,
                    ok: false,
                    status_code: None,
                    detail: "Issuer URL is not configured".to_string(),
                }
            }
        } else {
            IdentityProviderConnectionCheck {
                attempted: false,
                ok: true,
                status_code: None,
                detail: "Not applicable for OAuth2 providers".to_string(),
            }
        };

        let token_endpoint = if let Some(url) = provider.token_endpoint.clone() {
            probe_endpoint(
                self.http_client.clone(),
                HttpDeliveryRequest {
                    method: "POST".to_string(),
                    url,
                    headers: HashMap::from([(
                        "content-type".to_string(),
                        "application/x-www-form-urlencoded".to_string(),
                    )]),
                    body: format!(
                        "grant_type=authorization_code&code=reauth-test&client_id={}",
                        urlencoding::encode(&provider.client_id)
                    ),
                },
                "Token endpoint is reachable",
            )
            .await
        } else {
            IdentityProviderConnectionCheck {
                attempted: false,
                ok: false,
                status_code: None,
                detail: "Token endpoint is not configured".to_string(),
            }
        };

        let userinfo_endpoint = if let Some(url) = provider.userinfo_endpoint.clone() {
            probe_endpoint(
                self.http_client.clone(),
                HttpDeliveryRequest {
                    method: "GET".to_string(),
                    url,
                    headers: HashMap::new(),
                    body: String::new(),
                },
                "Userinfo endpoint is reachable",
            )
            .await
        } else {
            IdentityProviderConnectionCheck {
                attempted: false,
                ok: true,
                status_code: None,
                detail: "Userinfo endpoint is not configured".to_string(),
            }
        };

        let jwks = if provider.protocol == IdentityProviderProtocol::Oidc {
            if provider.jwks_uri.is_some() {
                match force_refresh_jwks(self.http_client.clone(), &mut provider).await {
                    Ok(_) => {
                        provider_changed = true;
                        IdentityProviderConnectionCheck {
                            attempted: true,
                            ok: true,
                            status_code: Some(200),
                            detail: "JWKS refreshed".to_string(),
                        }
                    }
                    Err(err) => IdentityProviderConnectionCheck {
                        attempted: true,
                        ok: false,
                        status_code: None,
                        detail: err.to_string(),
                    },
                }
            } else {
                IdentityProviderConnectionCheck {
                    attempted: false,
                    ok: false,
                    status_code: None,
                    detail: "JWKS URI is not configured".to_string(),
                }
            }
        } else {
            IdentityProviderConnectionCheck {
                attempted: false,
                ok: true,
                status_code: None,
                detail: "Not applicable for OAuth2 providers".to_string(),
            }
        };

        if provider_changed {
            provider.updated_at = Utc::now();
            self.repo.update(&provider).await?;
        }

        let ok = [
            discovery.ok,
            token_endpoint.ok,
            userinfo_endpoint.ok,
            jwks.ok,
        ]
        .into_iter()
        .zip([
            discovery.attempted,
            token_endpoint.attempted,
            userinfo_endpoint.attempted,
            jwks.attempted,
        ])
        .all(|(ok, attempted)| !attempted || ok);

        Ok(IdentityProviderConnectionTestResult {
            provider_id: provider.id,
            provider_alias: provider.alias,
            protocol: provider.protocol,
            ok,
            discovery,
            token_endpoint,
            userinfo_endpoint,
            jwks,
            metadata_cached_at: provider.metadata_cached_at,
            jwks_cached_at: provider.jwks_cached_at,
            tested_at: Utc::now(),
        })
    }

    fn to_response(provider: IdentityProvider) -> Result<IdentityProviderResponse> {
        let scopes = serde_json::from_str(&provider.scopes_json)
            .map_err(|e| Error::System(format!("Invalid provider scopes: {}", e)))?;
        let claim_mapping = serde_json::from_str(&provider.claim_mapping_json)
            .map_err(|e| Error::System(format!("Invalid provider claim mapping: {}", e)))?;
        let client_secret_mask = provider.client_secret.as_deref().map(mask_secret_tail);
        Ok(IdentityProviderResponse {
            id: provider.id,
            realm_id: provider.realm_id,
            alias: provider.alias,
            display_name: provider.display_name,
            protocol: provider.protocol,
            preset_key: provider.preset_key,
            enabled: provider.enabled,
            client_id: provider.client_id,
            issuer: provider.issuer,
            authorization_endpoint: provider.authorization_endpoint,
            token_endpoint: provider.token_endpoint,
            userinfo_endpoint: provider.userinfo_endpoint,
            jwks_uri: provider.jwks_uri,
            scopes,
            claim_mapping,
            pkce_required: provider.pkce_required,
            allow_login: provider.allow_login,
            allow_link: provider.allow_link,
            allow_jit_provisioning: provider.allow_jit_provisioning,
            allow_email_auto_link: provider.allow_email_auto_link,
            require_verified_email: provider.require_verified_email,
            icon_ref: provider.icon_ref,
            button_color: provider.button_color,
            sort_order: provider.sort_order,
            metadata_cached_at: provider.metadata_cached_at,
            jwks_cached_at: provider.jwks_cached_at,
            client_secret_set: provider.client_secret.is_some(),
            client_secret_mask,
        })
    }
}

fn validate_alias(alias: &str) -> Result<()> {
    let trimmed = alias.trim();
    if trimmed.is_empty() {
        return Err(Error::Validation("Provider alias is required".to_string()));
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Err(Error::Validation(
            "Provider alias must be URL-safe lowercase text".to_string(),
        ));
    }
    Ok(())
}

fn mask_secret_tail(value: &str) -> String {
    let tail: String = value
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("***{}", tail)
}

fn preset_scopes(preset: &Option<IdentityProviderPreset>) -> Vec<String> {
    preset
        .as_ref()
        .map(|value| value.scopes.clone())
        .unwrap_or_else(|| {
            vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ]
        })
}

fn preset_claim_mapping(preset: &Option<IdentityProviderPreset>) -> Value {
    preset
        .as_ref()
        .map(|value| value.claim_mapping.clone())
        .unwrap_or_else(|| json!({ "username": "preferred_username", "email": "email" }))
}

fn find_preset(key: &str) -> Option<IdentityProviderPreset> {
    built_in_presets()
        .into_iter()
        .find(|preset| preset.key == key)
}

fn default_allow_jit_provisioning(policy: &RealmIdpDefaultJitPolicy) -> bool {
    matches!(policy, RealmIdpDefaultJitPolicy::Allow)
}

fn default_allow_email_auto_link(policy: &RealmIdpDefaultEmailLinkPolicy) -> bool {
    matches!(policy, RealmIdpDefaultEmailLinkPolicy::AllowVerified)
}

fn default_require_verified_email(policy: &RealmIdpDefaultEmailLinkPolicy) -> bool {
    !matches!(policy, RealmIdpDefaultEmailLinkPolicy::Deny)
}

async fn probe_endpoint(
    http_client: Arc<dyn HttpDeliveryClient>,
    request: HttpDeliveryRequest,
    success_detail: &str,
) -> IdentityProviderConnectionCheck {
    match http_client.send(request).await {
        Ok(response) => {
            let reachable = response.status_code < 500 && response.status_code != 404;
            IdentityProviderConnectionCheck {
                attempted: true,
                ok: reachable,
                status_code: Some(response.status_code),
                detail: if reachable {
                    success_detail.to_string()
                } else {
                    format!("Endpoint returned status {}", response.status_code)
                },
            }
        }
        Err(err) => IdentityProviderConnectionCheck {
            attempted: true,
            ok: false,
            status_code: None,
            detail: err.to_string(),
        },
    }
}

fn idp_audit_actions() -> &'static [&'static str] {
    &[
        "idp_redirect_issued",
        "idp_callback_received",
        "idp_callback_success",
        "idp_callback_failure",
        "idp_callback_invalid_request",
        "idp_callback_upstream_error",
        "idp_callback_session_mismatch",
        "idp_user_linked",
        "idp_user_unlinked",
        "idp_jit_provisioned",
        "idp_conflict_email_collision",
        "idp_state_mismatch",
        "idp_start_rate_limited",
        "idp_pkce_failure",
        "idp_token_exchange_failure",
        "idp_userinfo_failure",
    ]
}

fn summarize_activity_counts(counts: &[AuditActionCount]) -> IdentityProviderActivitySummary {
    let mut total_events_last_24h = 0_u64;
    let mut failures_last_24h = 0_u64;
    let mut callback_success_last_24h = 0_u64;
    let mut links_last_24h = 0_u64;
    let mut jit_provisioned_last_24h = 0_u64;

    for count in counts {
        total_events_last_24h += count.count;
        match count.action.as_str() {
            "idp_callback_success" => callback_success_last_24h += count.count,
            "idp_user_linked" | "idp_user_unlinked" => links_last_24h += count.count,
            "idp_jit_provisioned" => jit_provisioned_last_24h += count.count,
            "idp_callback_failure"
            | "idp_callback_invalid_request"
            | "idp_callback_upstream_error"
            | "idp_callback_session_mismatch"
            | "idp_conflict_email_collision"
            | "idp_state_mismatch"
            | "idp_start_rate_limited"
            | "idp_pkce_failure"
            | "idp_token_exchange_failure"
            | "idp_userinfo_failure" => failures_last_24h += count.count,
            _ => {}
        }
    }

    IdentityProviderActivitySummary {
        total_events_last_24h,
        failures_last_24h,
        callback_success_last_24h,
        links_last_24h,
        jit_provisioned_last_24h,
    }
}

fn map_activity_event(event: crate::domain::audit::AuditEvent) -> IdentityProviderActivityEvent {
    let auth_session_id = event
        .metadata
        .get("auth_session_id")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok());
    let user_id = event
        .metadata
        .get("user_id")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
        .or(event.actor_user_id);
    let subject = event
        .metadata
        .get("subject")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let email = event
        .metadata
        .get("email")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let linked_via = event
        .metadata
        .get("linked_via")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let message = event
        .metadata
        .get("message")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            event
                .metadata
                .get("reason")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        });

    IdentityProviderActivityEvent {
        audit_event_id: event.id,
        action: event.action,
        created_at: event.created_at,
        actor_user_id: event.actor_user_id,
        auth_session_id,
        user_id,
        subject,
        email,
        linked_via,
        message,
        metadata: event.metadata,
    }
}

struct OidcPresetSpec<'a> {
    key: &'a str,
    display_name: &'a str,
    issuer: Option<&'a str>,
    authorization_endpoint: Option<&'a str>,
    token_endpoint: Option<&'a str>,
    userinfo_endpoint: Option<&'a str>,
    jwks_uri: Option<&'a str>,
    icon_ref: Option<&'a str>,
}

fn built_in_presets() -> Vec<IdentityProviderPreset> {
    vec![
        preset(OidcPresetSpec {
            key: "google",
            display_name: "Google",
            issuer: Some("https://accounts.google.com"),
            authorization_endpoint: Some("https://accounts.google.com/o/oauth2/v2/auth"),
            token_endpoint: Some("https://oauth2.googleapis.com/token"),
            userinfo_endpoint: Some("https://openidconnect.googleapis.com/v1/userinfo"),
            jwks_uri: Some("https://www.googleapis.com/oauth2/v3/certs"),
            icon_ref: Some("google"),
        }),
        IdentityProviderPreset {
            key: "github".to_string(),
            display_name: "GitHub".to_string(),
            protocol: IdentityProviderProtocol::Oauth2,
            issuer: None,
            authorization_endpoint: Some("https://github.com/login/oauth/authorize".to_string()),
            token_endpoint: Some("https://github.com/login/oauth/access_token".to_string()),
            userinfo_endpoint: Some("https://api.github.com/user".to_string()),
            jwks_uri: None,
            scopes: vec!["read:user".to_string(), "user:email".to_string()],
            claim_mapping: json!({
                "username": "login",
                "email": "email",
                "subject": "id"
            }),
            icon_ref: Some("github".to_string()),
        },
        preset(OidcPresetSpec {
            key: "microsoft",
            display_name: "Microsoft",
            issuer: Some("https://login.microsoftonline.com/common/v2.0"),
            authorization_endpoint: Some(
                "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
            ),
            token_endpoint: Some("https://login.microsoftonline.com/common/oauth2/v2.0/token"),
            userinfo_endpoint: Some("https://graph.microsoft.com/oidc/userinfo"),
            jwks_uri: Some("https://login.microsoftonline.com/common/discovery/v2.0/keys"),
            icon_ref: Some("microsoft"),
        }),
        preset(OidcPresetSpec {
            key: "apple",
            display_name: "Apple",
            issuer: Some("https://appleid.apple.com"),
            authorization_endpoint: Some("https://appleid.apple.com/auth/authorize"),
            token_endpoint: Some("https://appleid.apple.com/auth/token"),
            userinfo_endpoint: None,
            jwks_uri: Some("https://appleid.apple.com/auth/keys"),
            icon_ref: Some("apple"),
        }),
        preset(OidcPresetSpec {
            key: "gitlab",
            display_name: "GitLab",
            issuer: None,
            authorization_endpoint: Some("https://gitlab.com/oauth/authorize"),
            token_endpoint: Some("https://gitlab.com/oauth/token"),
            userinfo_endpoint: Some("https://gitlab.com/api/v4/user"),
            jwks_uri: None,
            icon_ref: Some("gitlab"),
        }),
        preset(OidcPresetSpec {
            key: "okta",
            display_name: "Okta",
            issuer: None,
            authorization_endpoint: None,
            token_endpoint: None,
            userinfo_endpoint: None,
            jwks_uri: None,
            icon_ref: Some("okta"),
        }),
        preset(OidcPresetSpec {
            key: "auth0",
            display_name: "Auth0",
            issuer: None,
            authorization_endpoint: None,
            token_endpoint: None,
            userinfo_endpoint: None,
            jwks_uri: None,
            icon_ref: Some("shield"),
        }),
        preset(OidcPresetSpec {
            key: "custom-oidc",
            display_name: "Custom OIDC",
            issuer: None,
            authorization_endpoint: None,
            token_endpoint: None,
            userinfo_endpoint: None,
            jwks_uri: None,
            icon_ref: Some("globe"),
        }),
        IdentityProviderPreset {
            key: "custom-oauth2".to_string(),
            display_name: "Custom OAuth2".to_string(),
            protocol: IdentityProviderProtocol::Oauth2,
            issuer: None,
            authorization_endpoint: None,
            token_endpoint: None,
            userinfo_endpoint: None,
            jwks_uri: None,
            scopes: vec!["read:user".to_string()],
            claim_mapping: json!({ "username": "login", "email": "email" }),
            icon_ref: Some("globe".to_string()),
        },
    ]
}

fn preset(spec: OidcPresetSpec<'_>) -> IdentityProviderPreset {
    IdentityProviderPreset {
        key: spec.key.to_string(),
        display_name: spec.display_name.to_string(),
        protocol: IdentityProviderProtocol::Oidc,
        issuer: spec.issuer.map(ToString::to_string),
        authorization_endpoint: spec.authorization_endpoint.map(ToString::to_string),
        token_endpoint: spec.token_endpoint.map(ToString::to_string),
        userinfo_endpoint: spec.userinfo_endpoint.map(ToString::to_string),
        jwks_uri: spec.jwks_uri.map(ToString::to_string),
        scopes: vec![
            "openid".to_string(),
            "email".to_string(),
            "profile".to_string(),
        ],
        claim_mapping: json!({
            "username": "preferred_username",
            "email": "email",
            "subject": "sub"
        }),
        icon_ref: spec.icon_ref.map(ToString::to_string),
    }
}
