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
    response::{IntoResponse, Redirect, Response}, // Redirect is needed for authorize
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite}; // Use axum_extra cookie types
use cookie::CookieBuilder;
use http::{header, HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use url::Url;
use uuid::Uuid;

// Note: AuthorizeParams is replaced by domain::oidc::OidcRequest to match service signature

#[derive(Deserialize)]
pub struct TokenParams {
    pub grant_type: Option<String>,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: Option<String>,
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
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_request",
                    "error_description": rejection.to_string()
                })),
            )),
        }
    }
}

#[derive(Serialize)]
struct OidcErrorResponse<'a> {
    error: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_description: Option<&'a str>,
}

fn oidc_error_response(status: StatusCode, error: &str, description: Option<&str>) -> Response {
    (
        status,
        Json(OidcErrorResponse {
            error,
            error_description: description,
        }),
    )
        .into_response()
}

fn oidc_bearer_error(status: StatusCode, error: &str, description: &str) -> Response {
    let mut response = oidc_error_response(status, error, Some(description));
    let header_value = format!(
        "Bearer error=\"{}\", error_description=\"{}\"",
        error, description
    );
    if let Ok(value) = HeaderValue::from_str(&header_value) {
        response
            .headers_mut()
            .insert(header::WWW_AUTHENTICATE, value);
    }
    response
}

fn normalize_authorize_error(error: &Error) -> (&'static str, StatusCode, String) {
    match error {
        Error::OidcInvalidRequest(message) => {
            let error_code = if message.contains("response_type") {
                "unsupported_response_type"
            } else {
                "invalid_request"
            };
            (error_code, StatusCode::BAD_REQUEST, message.clone())
        }
        Error::OidcInvalidRedirect(message) => {
            ("invalid_request", StatusCode::BAD_REQUEST, message.clone())
        }
        Error::OidcClientNotFound(message) => (
            "unauthorized_client",
            StatusCode::BAD_REQUEST,
            message.clone(),
        ),
        Error::Validation(message) => ("invalid_request", StatusCode::BAD_REQUEST, message.clone()),
        Error::SecurityViolation(message) => {
            ("access_denied", StatusCode::FORBIDDEN, message.clone())
        }
        _ => (
            "server_error",
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected server error".to_string(),
        ),
    }
}

fn normalize_token_error(error: &Error) -> (&'static str, StatusCode, String) {
    match error {
        Error::OidcInvalidCode => (
            "invalid_grant",
            StatusCode::UNAUTHORIZED,
            "Invalid authorization code".to_string(),
        ),
        Error::OidcInvalidRedirect(message) => {
            ("invalid_grant", StatusCode::BAD_REQUEST, message.clone())
        }
        Error::OidcInvalidRequest(message) => {
            let error_code = if message.contains("grant_type") {
                "unsupported_grant_type"
            } else {
                "invalid_request"
            };
            (error_code, StatusCode::BAD_REQUEST, message.clone())
        }
        Error::OidcClientNotFound(message) => {
            ("invalid_client", StatusCode::UNAUTHORIZED, message.clone())
        }
        Error::Validation(message) => ("invalid_request", StatusCode::BAD_REQUEST, message.clone()),
        _ => (
            "server_error",
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected server error".to_string(),
        ),
    }
}

fn normalize_userinfo_error(error: &Error) -> (&'static str, StatusCode, String) {
    match error {
        Error::Jwt(_) => (
            "invalid_token",
            StatusCode::UNAUTHORIZED,
            "Invalid or expired access token".to_string(),
        ),
        Error::OidcInvalidRequest(message) => {
            ("invalid_request", StatusCode::BAD_REQUEST, message.clone())
        }
        Error::Validation(message) => ("invalid_request", StatusCode::BAD_REQUEST, message.clone()),
        _ => (
            "server_error",
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected server error".to_string(),
        ),
    }
}

fn build_authorize_redirect(
    redirect_uri: &str,
    error: &str,
    description: &str,
    state: Option<&str>,
) -> Option<Redirect> {
    let mut url = Url::parse(redirect_uri).ok()?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("error", error);
        pairs.append_pair("error_description", description);
        if let Some(state) = state {
            pairs.append_pair("state", state);
        }
    }
    Some(Redirect::to(url.as_str()))
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
) -> Result<Response> {
    // 1. Resolve Realm
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name.clone()))?;

    // 2. Initiate the Graph Session via OidcService
    // This handles client validation, flow lookup, and unified session creation.
    let session = match state
        .oidc_service
        .initiate_browser_login(realm.id, params.clone())
        .await
    {
        Ok(session) => session,
        Err(err) => {
            let (error_code, status, description) = normalize_authorize_error(&err);

            let redirect_ok = state
                .oidc_service
                .validate_client(&realm.id, &params.client_id, &params.redirect_uri)
                .await
                .is_ok();

            if redirect_ok {
                if let Some(redirect) = build_authorize_redirect(
                    &params.redirect_uri,
                    error_code,
                    &description,
                    params.state.as_deref(),
                ) {
                    return Ok(redirect.into_response());
                }
            }

            return Ok(oidc_error_response(status, error_code, Some(&description)));
        }
    };

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

    Ok((headers, Redirect::to(&frontend_login_url)).into_response())
}

/// POST /api/oidc/token
/// Exchanges an Authorization Code for an Access Token.
pub async fn token_handler(
    State(state): State<AppState>,
    Path(_realm_name): Path<String>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    JsonForm(params): JsonForm<TokenParams>,
) -> Result<Response> {
    let grant_type = params.grant_type.as_deref().unwrap_or_default().trim();
    if grant_type.is_empty() {
        return Ok(oidc_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_request",
            Some("grant_type is required"),
        ));
    }
    if grant_type != "authorization_code" {
        return Ok(oidc_error_response(
            StatusCode::BAD_REQUEST,
            "unsupported_grant_type",
            Some("Unsupported grant_type"),
        ));
    }

    let code = params.code.as_deref().unwrap_or_default().trim();
    if code.is_empty() {
        return Ok(oidc_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_request",
            Some("code is required"),
        ));
    }

    let client_id = params.client_id.as_deref().unwrap_or_default().trim();
    if client_id.is_empty() {
        return Ok(oidc_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_request",
            Some("client_id is required"),
        ));
    }

    let redirect_uri = params.redirect_uri.as_deref().unwrap_or_default().trim();
    if redirect_uri.is_empty() {
        return Ok(oidc_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_request",
            Some("redirect_uri is required"),
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
    let (token_response, refresh_token) = match state
        .oidc_service
        .exchange_code_for_token(
            code,
            redirect_uri,
            params.code_verifier.as_deref().unwrap_or(""),
            Some(ip_address),
            user_agent,
        )
        .await
    {
        Ok(values) => values,
        Err(err) => {
            let (error_code, status, description) = normalize_token_error(&err);
            return Ok(oidc_error_response(status, error_code, Some(&description)));
        }
    };

    // Create the HttpOnly Cookie
    let cookie = create_refresh_cookie(&refresh_token);

    // Set the header
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).map_err(|e| Error::Unexpected(e.into()))?,
    );

    // Return the tuple (Status, Headers, JSON Body)
    Ok((StatusCode::OK, headers, Json(token_response)).into_response())
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
) -> Result<Response> {
    let Some(auth_header) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    else {
        return Ok(oidc_bearer_error(
            StatusCode::UNAUTHORIZED,
            "invalid_token",
            "Missing Authorization header",
        ));
    };

    let Some(token) = auth_header.strip_prefix("Bearer ") else {
        return Ok(oidc_error_response(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            Some("Invalid Authorization header"),
        ));
    };

    match state.oidc_service.userinfo(token).await {
        Ok(response) => Ok((StatusCode::OK, Json(response)).into_response()),
        Err(err) => {
            let (error_code, status, description) = normalize_userinfo_error(&err);
            if error_code == "invalid_token" {
                Ok(oidc_bearer_error(status, error_code, &description))
            } else {
                Ok(oidc_error_response(status, error_code, Some(&description)))
            }
        }
    }
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
    let scopes_json = serde_json::to_string(&vec!["openid", "profile", "email"])
        .map_err(|e| Error::Unexpected(e.into()))?;

    // Create Domain Entity
    let mut client = OidcClient {
        id: Uuid::new_v4(),
        realm_id: realm.id,
        client_id: payload.client_id,
        client_secret: None, // Public client for now
        redirect_uris: redirect_uris_json,
        web_origins: web_origins_json,
        scopes: scopes_json,
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
