use crate::constants::{LOGIN_SESSION_COOKIE, REFRESH_TOKEN_COOKIE};
use crate::domain::oidc::{OidcClient, OidcRequest}; // Use OidcRequest from domain
use crate::domain::pagination::PageRequest;
use crate::domain::session::RefreshToken;
use crate::{
    error::{Error, Result},
    AppState,
};
use axum::extract::{ConnectInfo, FromRequest, OriginalUri, Path, Request};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect}, // Redirect is needed for authorize
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite}; // Use axum_extra cookie types
use cookie::CookieBuilder;
use http::{header, HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::net::SocketAddr;
use uuid::Uuid;

// Note: AuthorizeParams is replaced by domain::oidc::OidcRequest to match service signature

#[derive(Deserialize)]
pub struct TokenParams {
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub code_verifier: Option<String>,
}

pub struct JsonForm<T>(pub T);

impl<S, T> FromRequest<S> for JsonForm<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: Request, state: &S) -> std::result::Result<Self, Self::Rejection> {
        match axum::extract::Form::<T>::from_request(req, state).await {
            Ok(axum::extract::Form(value)) => Ok(JsonForm(value)),
            Err(rejection) => Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": rejection.to_string() })),
            )),
        }
    }
}

fn create_refresh_cookie(token: &RefreshToken) -> Cookie<'static> {
    let expires_time = time::OffsetDateTime::from_unix_timestamp(token.expires_at.timestamp())
        .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);

    // Force insecure for localhost dev
    let is_production = false;

    Cookie::build((REFRESH_TOKEN_COOKIE, token.id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax) // Lax is generally safer for refresh tokens in modern browsers
        .secure(is_production)
        .expires(expires_time)
        .into()
}

/// GET /api/realms/{realm}/protocol/openid-connect/authorize
/// Starts the OIDC flow.
///
/// 1. Validates Client.
/// 2. Creates an AuthenticationSession in DB with OIDC context preserved.
/// 3. Sets a cookie.
/// 4. Redirects browser to Frontend Login UI.
pub async fn authorize_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(params): Query<OidcRequest>,
    uri: OriginalUri,
) -> Result<impl IntoResponse> {
    // 1. Resolve Realm
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name.clone()))?;

    // 2. Initiate the Graph Session via OidcService
    // This handles client validation, flow lookup, and unified session creation.
    let session = state
        .oidc_service
        .initiate_browser_login(realm.id, params)
        .await?;

    // 3. Set the Session Cookie
    let mut headers = HeaderMap::new();

    let expires_time = time::OffsetDateTime::from_unix_timestamp(session.expires_at.timestamp())
        .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);

    // Force insecure for localhost dev
    let is_production = false;

    let cookie: CookieBuilder = Cookie::build((LOGIN_SESSION_COOKIE, session.id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(is_production)
        .expires(expires_time);

    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).map_err(|e| Error::Unexpected(e.into()))?,
    );

    // 4. Redirect to Frontend Login WITH PARAMS
    // We fetch the original query string (client_id=..., response_type=..., etc.)
    let query_string = uri.query().unwrap_or("");

    // We append it to the frontend URL so AuthGuard sees it!
    let frontend_login_url = format!("/#/login?realm={}&{}", realm_name, query_string);

    Ok((headers, Redirect::to(&frontend_login_url)))
}

/// POST /api/oidc/token
/// Exchanges an Authorization Code for an Access Token.
pub async fn token_handler(
    State(state): State<AppState>,
    Path(_realm_name): Path<String>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    JsonForm(params): JsonForm<TokenParams>,
) -> Result<impl IntoResponse> {
    if params.grant_type != "authorization_code" {
        return Err(Error::OidcInvalidRequest(
            "Unsupported grant_type".to_string(),
        ));
    }

    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    // Simple IP extraction
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .unwrap_or_else(|| addr.ip().to_string());

    // Call the service
    let (token_response, refresh_token) = state
        .oidc_service
        .exchange_code_for_token(
            &params.code,
            &params.redirect_uri,
            params.code_verifier.as_deref().unwrap_or(""),
            Some(ip_address),
            user_agent,
        )
        .await?;

    // Create the HttpOnly Cookie
    let cookie = create_refresh_cookie(&refresh_token);

    // Set the header
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).map_err(|e| Error::Unexpected(e.into()))?,
    );

    // Return the tuple (Status, Headers, JSON Body)
    Ok((StatusCode::OK, headers, Json(token_response)))
}

/// Get /.well-known/jwks.json
pub async fn jwks_handler(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let jwks = state.oidc_service.get_jwks()?;
    Ok((StatusCode::OK, Json(jwks)))
}

/// Get /.well-known/openid-configuration
pub async fn discovery_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
) -> Result<impl IntoResponse> {
    let settings = state.settings.read().await;
    let base = settings.server.public_url.trim_end_matches('/').to_string();

    let response = serde_json::json!({
        "issuer": settings.auth.issuer,
        "authorization_endpoint": format!("{}/api/realms/{}/oidc/authorize", base, realm_name),
        "token_endpoint": format!("{}/api/realms/{}/oidc/token", base, realm_name),
        "userinfo_endpoint": format!("{}/api/realms/{}/oidc/userinfo", base, realm_name),
        "jwks_uri": format!("{}/api/realms/{}/oidc/.well-known/jwks.json", base, realm_name),
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": ["none"],
        "scopes_supported": ["openid", "profile"]
    });

    Ok((StatusCode::OK, Json(response)))
}

/// Get /userinfo
pub async fn userinfo_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| Error::OidcInvalidRequest("Missing Authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| Error::OidcInvalidRequest("Invalid Authorization header".to_string()))?;

    let response = state.oidc_service.userinfo(token).await?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn list_clients_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(page_req): Query<PageRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let response = state.oidc_service.list_clients(realm.id, page_req).await?;

    Ok((StatusCode::OK, Json(response)))
}

#[derive(Deserialize)]
pub struct CreateClientRequest {
    pub client_id: String,
    pub redirect_uris: Vec<String>,
    pub web_origins: Option<Vec<String>>,
}

pub async fn create_client_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Json(payload): Json<CreateClientRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    // Serialize Redirect URIs to String (for DB storage)
    let redirect_uris_json =
        serde_json::to_string(&payload.redirect_uris).map_err(|e| Error::Unexpected(e.into()))?;

    // Serialize Web Origins
    let web_origins_json = serde_json::to_string(&payload.web_origins.unwrap_or_default())
        .map_err(|e| Error::Unexpected(e.into()))?;

    // Create Domain Entity
    let mut client = OidcClient {
        id: Uuid::new_v4(),
        realm_id: realm.id,
        client_id: payload.client_id,
        client_secret: None, // Public client for now
        redirect_uris: redirect_uris_json,
        web_origins: web_origins_json,
        scopes: "openid profile email".to_string(), // Default scopes
        managed_by_config: false,
    };

    state.oidc_service.register_client(&mut client).await?;
    Ok((StatusCode::CREATED, Json(client)))
}

pub async fn get_client_handler(
    State(state): State<AppState>,
    Path((_realm, id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let client = state.oidc_service.get_client(id).await?;
    Ok((StatusCode::OK, Json(client)))
}

pub async fn update_client_handler(
    State(state): State<AppState>,
    Path((_realm, id)): Path<(String, Uuid)>,
    Json(payload): Json<crate::application::oidc_service::UpdateClientRequest>,
) -> Result<impl IntoResponse> {
    let client = state.oidc_service.update_client(id, payload).await?;
    Ok((StatusCode::OK, Json(client)))
}
