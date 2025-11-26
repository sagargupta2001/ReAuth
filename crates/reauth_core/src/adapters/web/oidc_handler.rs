use crate::constants::{LOGIN_SESSION_COOKIE, REFRESH_TOKEN_COOKIE};
use crate::domain::oidc::{OidcClient, OidcContext};
use crate::domain::pagination::PageRequest;
use crate::domain::session::RefreshToken;
use crate::{
    adapters::web::server::AppState,
    error::{Error, Result},
};
use axum::extract::Path;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Form, Json,
};
use cookie::{Cookie, CookieBuilder, SameSite};
use http::{header, HeaderMap, HeaderValue};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AuthorizeParams {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

#[derive(Deserialize)]
pub struct TokenParams {
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub code_verifier: Option<String>,
}

fn create_refresh_cookie(token: &RefreshToken) -> Cookie<'static> {
    let expires_time = time::OffsetDateTime::from_unix_timestamp(token.expires_at.timestamp())
        .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);

    Cookie::build((REFRESH_TOKEN_COOKIE, token.id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .expires(expires_time)
        .into()
}
/// GET /api/oidc/authorize
/// Starts the OIDC flow. For this MVP, we will simulate a successful login flow
/// initialization and return the "Challenge" to the UI.
///
/// In a full implementation, this would create a LoginSession with OIDC context.
pub async fn authorize_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(params): Query<AuthorizeParams>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    // 2. Validate Client (Now we scope it to the Realm!)
    // You will need to update `validate_client` in OidcService to accept realm_id
    let _client = state
        .oidc_service
        .validate_client(&realm.id, &params.client_id, &params.redirect_uri)
        .await?;

    // 3. Start Login Flow (Pass the resolved Realm ID)
    let (mut login_session, challenge) = state.flow_engine.start_login_flow(realm.id).await?;

    // 3. Attach OIDC Context
    let oidc_context = OidcContext {
        client_id: params.client_id,
        redirect_uri: params.redirect_uri,
        state: params.state,
        nonce: params.nonce,
        code_challenge: params.code_challenge,
        code_challenge_method: params.code_challenge_method,
    };

    login_session.state_data =
        Some(serde_json::to_string(&oidc_context).map_err(|e| Error::Unexpected(e.into()))?);

    state
        .flow_engine
        .update_login_session(&login_session)
        .await?;

    // 4. Return Challenge & Set Cookietly
    let expires_time =
        time::OffsetDateTime::from_unix_timestamp(login_session.expires_at.timestamp())
            .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);

    // FIX: Type should be `Cookie<'static>`, not `CookieBuilder`
    let cookie: Cookie<'static> =
        Cookie::build((LOGIN_SESSION_COOKIE, login_session.id.to_string()))
            .path("/") // Use "/" so it works for all routes
            .http_only(true)
            .same_site(SameSite::Strict)
            .expires(expires_time)
            .into(); // This converts Builder -> Cookie

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).map_err(|e| Error::Unexpected(e.into()))?,
    );

    Ok((StatusCode::OK, headers, Json(challenge)))
}

/// POST /api/oidc/token
/// Exchanges an Authorization Code for an Access Token.
pub async fn token_handler(
    State(state): State<AppState>,
    Path(_realm_name): Path<String>,
    Form(params): Form<TokenParams>,
) -> Result<impl IntoResponse> {
    if params.grant_type != "authorization_code" {
        return Err(Error::OidcInvalidRequest(
            "Unsupported grant_type".to_string(),
        ));
    }

    // 1. Call the service (now returns a tuple)
    let (token_response, refresh_token) = state
        .oidc_service
        .exchange_code_for_token(&params.code, params.code_verifier.as_deref().unwrap_or(""))
        .await?;
    // FIX: Convert timestamp correc

    // 2. Create the HttpOnly Cookie
    // (This uses the helper function defined earlier in this file)
    let cookie = create_refresh_cookie(&refresh_token);

    // 3. Set the header
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).map_err(|e| Error::Unexpected(e.into()))?,
    );

    // 4. Return the tuple (Status, Headers, JSON Body)
    Ok((StatusCode::OK, headers, Json(token_response)))
}

/// Get /.well-known/jwks.json
pub async fn jwks_handler(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let jwks = state.oidc_service.get_jwks()?;
    Ok((StatusCode::OK, Json(jwks)))
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
}

pub async fn create_client_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Json(payload): Json<CreateClientRequest>,
) -> Result<impl IntoResponse> {
    // Resolve Realm
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    // Validate Client ID (Ensure it's unique, handled by DB constraint usually, but good to check)
    // For now, we rely on the DB unique constraint error.

    // Serialize Redirect URIs to String (for DB storage)
    let redirect_uris_json =
        serde_json::to_string(&payload.redirect_uris).map_err(|e| Error::Unexpected(e.into()))?;

    // Create Domain Entity
    let client = OidcClient {
        id: Uuid::new_v4(),
        realm_id: realm.id,
        client_id: payload.client_id,
        client_secret: None, // Public client for now
        redirect_uris: redirect_uris_json,
        scopes: "openid profile email".to_string(), // Default scopes
    };

    // 5. Save to DB
    state.oidc_service.register_client(&client).await?;

    // 6. Return Success
    Ok((StatusCode::CREATED, Json(client)))
}
