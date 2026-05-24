use crate::domain::identity_provider::{IdentityProvider, IdentityProviderProtocol};
use crate::error::{Error, Result};
use crate::ports::http_client::{HttpDeliveryClient, HttpDeliveryRequest};
use chrono::{Duration, Utc};
use jsonwebtoken::jwk::JwkSet;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::warn;

const PROVIDER_METADATA_CACHE_TTL_HOURS: i64 = 24;

#[derive(Deserialize)]
struct DiscoveryDocument {
    authorization_endpoint: Option<String>,
    token_endpoint: Option<String>,
    userinfo_endpoint: Option<String>,
    jwks_uri: Option<String>,
}

pub async fn maybe_refresh_oidc_discovery(
    http_client: Arc<dyn HttpDeliveryClient>,
    provider: &mut IdentityProvider,
) -> Result<bool> {
    if !should_attempt_discovery_refresh(provider) {
        return Ok(false);
    }

    match refresh_oidc_discovery(http_client, provider).await {
        Ok(()) => Ok(true),
        Err(err) if has_usable_oidc_metadata(provider) => {
            warn!(
                provider_id = %provider.id,
                provider_alias = %provider.alias,
                error = %err,
                "OIDC discovery refresh failed; serving cached provider metadata."
            );
            Ok(false)
        }
        Err(err) => Err(err),
    }
}

pub async fn force_refresh_oidc_discovery(
    http_client: Arc<dyn HttpDeliveryClient>,
    provider: &mut IdentityProvider,
) -> Result<bool> {
    if provider.protocol != IdentityProviderProtocol::Oidc {
        return Ok(false);
    }
    refresh_oidc_discovery(http_client, provider).await?;
    Ok(true)
}

pub async fn force_refresh_jwks(
    http_client: Arc<dyn HttpDeliveryClient>,
    provider: &mut IdentityProvider,
) -> Result<bool> {
    let jwks_uri = provider.jwks_uri.as_deref().ok_or_else(|| {
        Error::Validation("OIDC identity provider JWKS URI is missing".to_string())
    })?;
    let response = http_client
        .send(HttpDeliveryRequest {
            method: "GET".to_string(),
            url: jwks_uri.to_string(),
            headers: HashMap::from([("accept".to_string(), "application/json".to_string())]),
            body: String::new(),
        })
        .await
        .map_err(|err| Error::System(format!("JWKS request failed: {}", err.message)))?;
    if response.status_code >= 400 {
        return Err(Error::Validation(format!(
            "JWKS request failed with status {}",
            response.status_code
        )));
    }

    let jwks: JwkSet = serde_json::from_str(&response.body)
        .map_err(|err| Error::System(format!("Invalid JWKS response: {}", err)))?;
    if jwks.keys.is_empty() {
        return Err(Error::Validation(
            "OIDC JWKS response did not contain any signing keys".to_string(),
        ));
    }

    provider.jwks_cached_at = Some(Utc::now());
    provider.jwks_cache_json = Some(response.body);
    Ok(true)
}

pub async fn load_jwks_with_refresh(
    http_client: Arc<dyn HttpDeliveryClient>,
    provider: &mut IdentityProvider,
    required_kid: Option<&str>,
) -> Result<(JwkSet, bool)> {
    if let Some((cached, stale)) = parse_cached_jwks(provider, required_kid) {
        if !stale {
            return Ok((cached, false));
        }

        match fetch_jwks(http_client, provider, required_kid).await {
            Ok(jwks) => return Ok((jwks, true)),
            Err(err) => {
                warn!(
                    provider_id = %provider.id,
                    provider_alias = %provider.alias,
                    error = %err,
                    "OIDC JWKS refresh failed; serving cached signing keys."
                );
                return Ok((cached, false));
            }
        }
    }

    let jwks = fetch_jwks(http_client, provider, required_kid).await?;
    Ok((jwks, true))
}

fn should_attempt_discovery_refresh(provider: &IdentityProvider) -> bool {
    provider.protocol == IdentityProviderProtocol::Oidc
        && provider.issuer.is_some()
        && (missing_required_oidc_metadata(provider) || discovery_cache_stale(provider))
}

fn missing_required_oidc_metadata(provider: &IdentityProvider) -> bool {
    provider.authorization_endpoint.is_none()
        || provider.token_endpoint.is_none()
        || provider.jwks_uri.is_none()
}

fn has_usable_oidc_metadata(provider: &IdentityProvider) -> bool {
    provider.authorization_endpoint.is_some()
        && provider.token_endpoint.is_some()
        && provider.jwks_uri.is_some()
}

fn discovery_cache_stale(provider: &IdentityProvider) -> bool {
    if provider.metadata_cache_json.is_none() {
        return false;
    }
    let ttl_cutoff = Utc::now() - Duration::hours(PROVIDER_METADATA_CACHE_TTL_HOURS);
    provider
        .metadata_cached_at
        .map(|cached_at| cached_at <= ttl_cutoff)
        .unwrap_or(true)
}

fn jwks_cache_stale(provider: &IdentityProvider) -> bool {
    let ttl_cutoff = Utc::now() - Duration::hours(PROVIDER_METADATA_CACHE_TTL_HOURS);
    provider
        .jwks_cached_at
        .map(|cached_at| cached_at <= ttl_cutoff)
        .unwrap_or(true)
}

async fn refresh_oidc_discovery(
    http_client: Arc<dyn HttpDeliveryClient>,
    provider: &mut IdentityProvider,
) -> Result<()> {
    let issuer = provider
        .issuer
        .as_deref()
        .ok_or_else(|| Error::Validation("OIDC discovery requires an issuer URL".to_string()))?;
    let issuer = issuer.trim_end_matches('/');
    let url = format!("{}/.well-known/openid-configuration", issuer);
    let response = http_client
        .send(HttpDeliveryRequest {
            method: "GET".to_string(),
            url,
            headers: HashMap::new(),
            body: String::new(),
        })
        .await
        .map_err(|err| Error::System(format!("OIDC discovery failed: {}", err.message)))?;
    if response.status_code >= 400 {
        return Err(Error::System(format!(
            "OIDC discovery failed with status {}",
            response.status_code
        )));
    }

    let discovery: DiscoveryDocument = serde_json::from_str(&response.body)
        .map_err(|e| Error::System(format!("Invalid OIDC discovery response: {}", e)))?;
    provider.authorization_endpoint = discovery
        .authorization_endpoint
        .or_else(|| provider.authorization_endpoint.clone());
    provider.token_endpoint = discovery
        .token_endpoint
        .or_else(|| provider.token_endpoint.clone());
    if let Some(userinfo_endpoint) = discovery.userinfo_endpoint {
        provider.userinfo_endpoint = Some(userinfo_endpoint);
    }
    provider.jwks_uri = discovery.jwks_uri.or_else(|| provider.jwks_uri.clone());
    provider.metadata_cached_at = Some(Utc::now());
    provider.metadata_cache_json = Some(response.body);
    Ok(())
}

fn parse_cached_jwks(
    provider: &IdentityProvider,
    required_kid: Option<&str>,
) -> Option<(JwkSet, bool)> {
    let cached = provider.jwks_cache_json.as_deref()?;
    let jwks = serde_json::from_str::<JwkSet>(cached).ok()?;
    if !has_usable_jwk(&jwks, required_kid) {
        return None;
    }
    Some((jwks, jwks_cache_stale(provider)))
}

async fn fetch_jwks(
    http_client: Arc<dyn HttpDeliveryClient>,
    provider: &mut IdentityProvider,
    required_kid: Option<&str>,
) -> Result<JwkSet> {
    let jwks_uri = provider.jwks_uri.as_deref().ok_or_else(|| {
        Error::Validation("OIDC identity provider JWKS URI is missing".to_string())
    })?;
    let response = http_client
        .send(HttpDeliveryRequest {
            method: "GET".to_string(),
            url: jwks_uri.to_string(),
            headers: HashMap::from([("accept".to_string(), "application/json".to_string())]),
            body: String::new(),
        })
        .await
        .map_err(|err| Error::System(format!("JWKS request failed: {}", err.message)))?;
    if response.status_code >= 400 {
        return Err(Error::Validation(format!(
            "JWKS request failed with status {}",
            response.status_code
        )));
    }

    let jwks: JwkSet = serde_json::from_str(&response.body)
        .map_err(|err| Error::System(format!("Invalid JWKS response: {}", err)))?;
    if !has_usable_jwk(&jwks, required_kid) {
        return Err(Error::Validation(
            "OIDC JWKS did not include a matching signing key".to_string(),
        ));
    }

    provider.jwks_cached_at = Some(Utc::now());
    provider.jwks_cache_json = Some(response.body);
    Ok(jwks)
}

fn has_usable_jwk(jwks: &JwkSet, required_kid: Option<&str>) -> bool {
    match required_kid {
        Some(kid) => jwks.find(kid).is_some(),
        None => jwks.keys.len() == 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::identity_provider::IdentityProvider;
    use crate::ports::http_client::{HttpDeliveryError, HttpDeliveryResponse};
    use async_trait::async_trait;
    use chrono::Duration;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use uuid::Uuid;

    #[derive(Clone)]
    struct FakeHttpClient {
        responses: Arc<
            std::sync::Mutex<Vec<std::result::Result<HttpDeliveryResponse, HttpDeliveryError>>>,
        >,
        calls: Arc<AtomicUsize>,
    }

    impl FakeHttpClient {
        fn ok_json(body: serde_json::Value) -> Self {
            Self {
                responses: Arc::new(std::sync::Mutex::new(vec![Ok(HttpDeliveryResponse {
                    status_code: 200,
                    body: body.to_string(),
                })])),
                calls: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn error(message: &str) -> Self {
            Self {
                responses: Arc::new(std::sync::Mutex::new(vec![Err(HttpDeliveryError {
                    message: message.to_string(),
                    error_chain: vec![],
                })])),
                calls: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl HttpDeliveryClient for FakeHttpClient {
        async fn send(
            &self,
            _request: HttpDeliveryRequest,
        ) -> std::result::Result<HttpDeliveryResponse, HttpDeliveryError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.responses.lock().expect("responses lock").remove(0)
        }
    }

    fn oidc_provider() -> IdentityProvider {
        let now = Utc::now();
        IdentityProvider {
            id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            alias: "google".to_string(),
            display_name: "Google".to_string(),
            protocol: IdentityProviderProtocol::Oidc,
            preset_key: Some("google".to_string()),
            enabled: true,
            client_id: "client-google".to_string(),
            client_secret: None,
            issuer: Some("https://issuer.example.com".to_string()),
            authorization_endpoint: Some("https://issuer.example.com/authorize".to_string()),
            token_endpoint: Some("https://issuer.example.com/token".to_string()),
            userinfo_endpoint: Some("https://issuer.example.com/userinfo".to_string()),
            jwks_uri: Some("https://issuer.example.com/jwks".to_string()),
            scopes_json: "[]".to_string(),
            claim_mapping_json: "{}".to_string(),
            pkce_required: true,
            allow_login: true,
            allow_link: true,
            allow_jit_provisioning: false,
            allow_email_auto_link: false,
            require_verified_email: true,
            icon_ref: None,
            button_color: None,
            sort_order: 0,
            metadata_cached_at: Some(now),
            metadata_cache_json: Some("{}".to_string()),
            jwks_cached_at: Some(now),
            jwks_cache_json: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn discovery_doc() -> serde_json::Value {
        json!({
            "authorization_endpoint": "https://issuer.example.com/authorize-new",
            "token_endpoint": "https://issuer.example.com/token-new",
            "userinfo_endpoint": "https://issuer.example.com/userinfo-new",
            "jwks_uri": "https://issuer.example.com/jwks-new"
        })
    }

    fn jwks_doc(kid: &str) -> serde_json::Value {
        json!({
            "keys": [{
                "kty": "RSA",
                "kid": kid,
                "alg": "RS256",
                "use": "sig",
                "n": "yRE6rHuNR0QbHO3H3Kt2pOKGVhQqGZXInOduQNxXzuKlvQTLUTv4l4sggh5_CYYi_cvI-SXVT9kPWSKXxJXBXd_4LkvcPuUakBoAkfh-eiFVMh2VrUyWyj3MFl0HTVF9KwRXLAcwkREiS3npThHRyIxuy0ZMeZfxVL5arMhw1SRELB8HoGfG_AtH89BIE9jDBHZ9dLelK9a184zAf8LwoPLxvJb3Il5nncqPcSfKDDodMFBIMc4lQzDKL5gvmiXLXB1AGLm8KBjfE8s3L5xqi-yUod-j8MtvIj812dkS4QMiRVN_by2h3ZY8LYVGrqZXZTcgn2ujn8uKjXLZVD5TdQ",
                "e": "AQAB"
            }]
        })
    }

    #[tokio::test]
    async fn maybe_refresh_discovery_refreshes_stale_oidc_metadata() {
        let http = Arc::new(FakeHttpClient::ok_json(discovery_doc()));
        let mut provider = oidc_provider();
        provider.metadata_cached_at = Some(Utc::now() - Duration::hours(25));

        let refreshed = maybe_refresh_oidc_discovery(http.clone(), &mut provider)
            .await
            .expect("refresh result");

        assert!(refreshed);
        assert_eq!(http.calls(), 1);
        assert_eq!(
            provider.authorization_endpoint.as_deref(),
            Some("https://issuer.example.com/authorize-new")
        );
        assert_eq!(
            provider.jwks_uri.as_deref(),
            Some("https://issuer.example.com/jwks-new")
        );
    }

    #[tokio::test]
    async fn maybe_refresh_discovery_keeps_cached_metadata_on_refresh_failure() {
        let http = Arc::new(FakeHttpClient::error("discovery unavailable"));
        let mut provider = oidc_provider();
        let original_authorize = provider.authorization_endpoint.clone();
        provider.metadata_cached_at = Some(Utc::now() - Duration::hours(25));

        let refreshed = maybe_refresh_oidc_discovery(http.clone(), &mut provider)
            .await
            .expect("fallback result");

        assert!(!refreshed);
        assert_eq!(http.calls(), 1);
        assert_eq!(provider.authorization_endpoint, original_authorize);
    }

    #[tokio::test]
    async fn force_refresh_discovery_returns_error_on_failure() {
        let http = Arc::new(FakeHttpClient::error("discovery unavailable"));
        let mut provider = oidc_provider();

        let result = force_refresh_oidc_discovery(http, &mut provider).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn load_jwks_refreshes_stale_cache_even_when_cached_key_matches() {
        let http = Arc::new(FakeHttpClient::ok_json(jwks_doc("kid-1")));
        let mut provider = oidc_provider();
        provider.jwks_cached_at = Some(Utc::now() - Duration::hours(25));
        provider.jwks_cache_json = Some(jwks_doc("kid-1").to_string());

        let (_jwks, refreshed) = load_jwks_with_refresh(http.clone(), &mut provider, Some("kid-1"))
            .await
            .expect("jwks result");

        assert!(refreshed);
        assert_eq!(http.calls(), 1);
        assert!(provider.jwks_cached_at.is_some());
    }

    #[tokio::test]
    async fn load_jwks_uses_stale_cache_when_refresh_fails() {
        let http = Arc::new(FakeHttpClient::error("jwks unavailable"));
        let mut provider = oidc_provider();
        provider.jwks_cached_at = Some(Utc::now() - Duration::hours(25));
        provider.jwks_cache_json = Some(jwks_doc("kid-1").to_string());

        let (jwks, refreshed) = load_jwks_with_refresh(http.clone(), &mut provider, Some("kid-1"))
            .await
            .expect("jwks fallback");

        assert!(!refreshed);
        assert_eq!(http.calls(), 1);
        assert!(jwks.find("kid-1").is_some());
    }
}
