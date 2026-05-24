use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IdentityProviderProtocol {
    Oidc,
    Oauth2,
}

impl std::fmt::Display for IdentityProviderProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Oidc => write!(f, "oidc"),
            Self::Oauth2 => write!(f, "oauth2"),
        }
    }
}

impl TryFrom<String> for IdentityProviderProtocol {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "oidc" => Ok(Self::Oidc),
            "oauth2" => Ok(Self::Oauth2),
            other => Err(format!("Unsupported identity provider protocol: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProvider {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub alias: String,
    pub display_name: String,
    pub protocol: IdentityProviderProtocol,
    pub preset_key: Option<String>,
    pub enabled: bool,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub issuer: Option<String>,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
    pub userinfo_endpoint: Option<String>,
    pub jwks_uri: Option<String>,
    pub scopes_json: String,
    pub claim_mapping_json: String,
    pub pkce_required: bool,
    pub allow_login: bool,
    pub allow_link: bool,
    pub allow_jit_provisioning: bool,
    pub allow_email_auto_link: bool,
    pub require_verified_email: bool,
    pub icon_ref: Option<String>,
    pub button_color: Option<String>,
    pub sort_order: i64,
    pub metadata_cached_at: Option<DateTime<Utc>>,
    pub metadata_cache_json: Option<String>,
    pub jwks_cached_at: Option<DateTime<Utc>>,
    pub jwks_cache_json: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedIdentity {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub provider_id: Uuid,
    pub user_id: Uuid,
    pub subject: String,
    pub external_username: Option<String>,
    pub external_email: Option<String>,
    pub raw_claims_json: Option<String>,
    pub linked_via: String,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthBrokerState {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub provider_id: Uuid,
    pub auth_session_id: Uuid,
    pub pkce_verifier_hash: String,
    pub redirect_uri: String,
    pub nonce: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProviderPreset {
    pub key: String,
    pub display_name: String,
    pub protocol: IdentityProviderProtocol,
    pub issuer: Option<String>,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
    pub userinfo_endpoint: Option<String>,
    pub jwks_uri: Option<String>,
    pub scopes: Vec<String>,
    pub claim_mapping: serde_json::Value,
    pub icon_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthBrokerResult {
    pub user_id: Option<Uuid>,
    pub output: String,
    pub provider_id: Uuid,
    pub provider_alias: String,
    pub provider_display_name: String,
    pub subject: String,
    pub external_email: Option<String>,
    pub external_username: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUpstreamIdentity {
    pub subject: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub username: Option<String>,
    pub claims: serde_json::Value,
}
