use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use http::{header, HeaderMap, HeaderValue, StatusCode};
use serde_json::Value;
use uuid::Uuid;

use crate::bootstrap::app_state::AppState;
use crate::constants::{LOGIN_SESSION_COOKIE, REFRESH_TOKEN_COOKIE};
use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::{ExecutionPlan, ExecutionResult};
use crate::domain::oidc::OidcContext;
use crate::domain::session::RefreshToken;
use crate::error::{Error, Result};
use axum_extra::extract::cookie::{Cookie, SameSite};
use cookie::CookieBuilder;

/// 1. START LOGIN: Logic to bootstrap a new session
///
/// GET /api/realms/{realm}/login
pub async fn start_login(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    jar: CookieJar,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::NotFound("Realm not found".to_string()))?;

    let flow_id_str = realm
        .browser_flow_id
        .ok_or(Error::Validation("No flow".to_string()))?;
    let flow_id = Uuid::parse_str(&flow_id_str).unwrap_or_default();

    // 1. Resolve Session
    let mut session = None;
    if let Some(cookie) = jar.get(LOGIN_SESSION_COOKIE) {
        if let Ok(id) = Uuid::parse_str(cookie.value()) {
            // We just pass whatever we find. The Executor will reset it if it's dead.
            if let Ok(Some(s)) = state.auth_session_repo.find_by_id(&id).await {
                session = Some(s);
            }
        }
    }

    // 2. Create if Missing
    let final_session = if let Some(s) = session {
        s
    } else {
        let version = state
            .flow_store
            .get_active_version(&flow_id)
            .await?
            .or(state.flow_store.get_latest_version(&flow_id).await?)
            .ok_or(Error::NotFound("Flow version missing".to_string()))?;

        let plan: ExecutionPlan = serde_json::from_str(&version.execution_artifact).unwrap();
        let new_s = AuthenticationSession::new(
            realm.id,
            Uuid::parse_str(&version.id).unwrap_or_default(),
            plan.start_node_id,
        );

        state.auth_session_repo.create(&new_s).await?;
        new_s
    };

    // 3. Execute
    let result = state.flow_executor.execute(final_session.id, None).await?;

    // 4. Set Cookie
    let mut headers = HeaderMap::new();
    let is_production = false; // Localhost
    let cookie: CookieBuilder = Cookie::build((LOGIN_SESSION_COOKIE, final_session.id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(is_production)
        .expires(
            time::OffsetDateTime::from_unix_timestamp(final_session.expires_at.timestamp())
                .unwrap(),
        );

    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).unwrap(),
    );

    Ok((
        headers,
        Json(serde_json::json!({
            "session_id": final_session.id,
            "execution": result
        })),
    ))
}

fn create_refresh_cookie(token: &RefreshToken) -> Cookie<'static> {
    let expires_time = time::OffsetDateTime::from_unix_timestamp(token.expires_at.timestamp())
        .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);

    let is_production = false;

    Cookie::build((REFRESH_TOKEN_COOKIE, token.id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(is_production)
        .expires(expires_time)
        .into()
}

pub async fn submit_execution(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    _jar: CookieJar, // Kept in signature if needed for reading existing cookies later
    Json(body): Json<Value>,
) -> Result<impl IntoResponse> {
    // 1. Run the Flow Executor
    // The executor handles graph traversal, credential validation, and updates the session state.
    let result = state.flow_executor.execute(session_id, Some(body)).await?;
    let mut headers = HeaderMap::new();

    // 2. Handle Success (Session Creation & Redirect Logic)
    if let ExecutionResult::Success {
        redirect_url: default_redirect,
    } = &result
    {
        // A. Retrieve Session & User
        let auth_session = state
            .auth_session_repo
            .find_by_id(&session_id)
            .await?
            .ok_or(Error::NotFound("Session lost during execution".to_string()))?;

        let user_id = auth_session.user_id.ok_or(Error::System(
            "Authentication succeeded but User ID is missing".to_string(),
        ))?;

        // Use the realm-aware user lookup for security
        let user = state
            .user_service
            .get_user_in_realm(auth_session.realm_id, user_id)
            .await?;

        // B. Create Persistent User Session (Refresh Token)
        // TODO: Extract IP/UserAgent from request headers if available in handler signature
        let (_login_response, refresh_token) = state
            .auth_service
            .create_session(&user, None, None, None)
            .await?;

        // C. Set the Refresh Token Cookie
        let cookie = create_refresh_cookie(&refresh_token);
        headers.insert(
            header::SET_COOKIE,
            HeaderValue::from_str(&cookie.to_string()).map_err(|e| Error::Unexpected(e.into()))?,
        );

        // D. Determine Final Redirect Destination (OIDC vs Dashboard)
        let final_redirect_url =
            resolve_redirect_target(&state, &auth_session, &user.id, default_redirect).await?;

        // E. Return Modified Success Result
        // We override the 'redirect_url' in the JSON so the frontend knows exactly where to go.
        // If it's OIDC, this will be the client callback. If not, it's the dashboard.
        return Ok((
            StatusCode::OK,
            headers,
            Json(serde_json::json!({
                "session_id": session_id,
                "execution": {
                    "type": "Success", // Matches your TypeScript Enum (ExecutionResult)
                    "payload": {
                        "redirect_url": final_redirect_url
                    }
                }
            })),
        ));
    }

    // 3. Handle Ongoing Flow (Challenge / Failure)
    // Just pass the result through to the frontend
    Ok((
        StatusCode::OK,
        headers,
        Json(serde_json::json!({
            "session_id": session_id,
            "execution": result
        })),
    ))
}

// --- Helper: Resolve OIDC vs Standard Redirect ---
async fn resolve_redirect_target(
    state: &AppState,
    session: &AuthenticationSession,
    user_id: &Uuid,
    default_url: &str,
) -> Result<String> {
    // Check if "oidc" context exists in the session (this should have been saved by /authorize)
    if let Some(oidc_value) = session.context.get("oidc") {
        // Try to deserialize context
        if let Ok(oidc_ctx) = serde_json::from_value::<OidcContext>(oidc_value.clone()) {
            // 1. Generate Authorization Code
            let auth_code = state
                .oidc_service
                .create_authorization_code(
                    session.realm_id,
                    *user_id,
                    oidc_ctx.client_id,
                    oidc_ctx.redirect_uri.clone(),
                    oidc_ctx.nonce,
                    oidc_ctx.code_challenge,
                    oidc_ctx
                        .code_challenge_method
                        .unwrap_or_else(|| "plain".to_string()),
                )
                .await?;

            // 2. Build Callback URL
            // Format: https://client.com/cb?code=XYZ&state=ABC
            let mut url = url::Url::parse(&oidc_ctx.redirect_uri).map_err(|_| {
                Error::Validation(format!(
                    "Invalid OIDC redirect URI: {}",
                    oidc_ctx.redirect_uri
                ))
            })?;

            url.query_pairs_mut().append_pair("code", &auth_code.code);

            if let Some(s) = oidc_ctx.state {
                url.query_pairs_mut().append_pair("state", &s);
            }

            return Ok(url.to_string());
        }
    }

    // Fallback: Return the dashboard URL (or whatever the flow defined in the graph)
    Ok(default_url.to_string())
}
