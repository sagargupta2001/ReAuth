use crate::{
    constants::{DEFAULT_REALM_NAME, LOGIN_SESSION_COOKIE, REFRESH_TOKEN_COOKIE},
    domain::{
        auth_session::AuthenticationSession, auth_session::SessionStatus,
        execution::ExecutionResult, session::RefreshToken,
    },
    error::{Error, Result},
    AppState,
};
use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{Duration, Utc};
use std::net::SocketAddr;
use uuid::Uuid;

fn create_refresh_cookie(token: &RefreshToken) -> Cookie<'static> {
    let expires_time = time::OffsetDateTime::from_unix_timestamp(token.expires_at.timestamp())
        .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);
    Cookie::build((REFRESH_TOKEN_COOKIE, token.id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(false)
        .expires(expires_time)
        .into()
}

fn create_clear_cookie() -> Cookie<'static> {
    Cookie::build(REFRESH_TOKEN_COOKIE)
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(false)
        .max_age(time::Duration::seconds(0))
        .into()
}

fn create_clear_login_cookie() -> Cookie<'static> {
    Cookie::build(LOGIN_SESSION_COOKIE)
        .path("/api")
        .expires(time::OffsetDateTime::UNIX_EPOCH)
        .same_site(SameSite::Lax)
        .secure(false)
        .max_age(time::Duration::seconds(0))
        .into()
}

fn create_login_cookie(session_id: Uuid) -> Cookie<'static> {
    // 15 min expiry for login session
    let expires = time::OffsetDateTime::now_utc() + time::Duration::minutes(15);
    Cookie::build((LOGIN_SESSION_COOKIE, session_id.to_string()))
        .path("/api")
        .http_only(true)
        .same_site(SameSite::Strict)
        .expires(expires)
        .into()
}

// --- HANDLERS ---

// GET /api/auth/login
pub async fn start_login_flow_handler(
    State(state): State<AppState>,
    jar: CookieJar, // <--- Add CookieJar to read existing cookies
) -> Result<impl IntoResponse> {
    // --- 1. RESUME LOGIC (Check for existing session) ---
    if let Some(cookie) = jar.get(LOGIN_SESSION_COOKIE) {
        if let Ok(session_id) = Uuid::parse_str(cookie.value()) {
            // Check if session exists and is active
            if let Ok(Some(existing_session)) =
                state.auth_session_repo.find_by_id(&session_id).await
            {
                if existing_session.status == SessionStatus::active {
                    // RESUME: Re-execute the current node to get the UI JSON
                    let result = state.flow_executor.execute(session_id, None).await?;

                    // We don't need to set a new cookie, just return the JSON
                    let response_body = match result {
                        ExecutionResult::Challenge { screen_id, context } => serde_json::json!({
                            "status": "challenge",
                            "challengeName": screen_id,
                            "context": context
                        }),
                        ExecutionResult::Success { redirect_url } => serde_json::json!({
                            "status": "redirect",
                            "url": redirect_url
                        }),
                        ExecutionResult::Failure { reason } => serde_json::json!({
                            "status": "failure",
                            "message": reason
                        }),
                    };

                    // Return immediately so we don't overwrite with a new session
                    return Ok((StatusCode::OK, Json(response_body)).into_response());
                }
            }
        }
    }

    // --- 2. NEW SESSION LOGIC (Fallback if no valid cookie) ---

    let realm = state
        .realm_service
        .find_by_name(DEFAULT_REALM_NAME)
        .await?
        .ok_or(Error::RealmNotFound(DEFAULT_REALM_NAME.to_string()))?;

    // Resolve Flow
    let flow_id_str = realm
        .browser_flow_id
        .ok_or(Error::System("No browser flow configured for realm".into()))?;
    let flow_id = Uuid::parse_str(&flow_id_str)
        .map_err(|_| Error::System("Invalid flow ID format".into()))?;

    // Resolve Active Version
    let version_num = state
        .flow_store
        .get_deployed_version_number(&realm.id, "browser", &flow_id)
        .await?
        .ok_or(Error::System("No active version deployed".into()))?;

    let version = state
        .flow_store
        .get_version_by_number(&flow_id, version_num)
        .await?
        .ok_or(Error::System("Active version not found".into()))?;

    // Create New Session
    let session_id = Uuid::new_v4();
    let session = AuthenticationSession {
        id: session_id,
        realm_id: realm.id,
        flow_version_id: Uuid::parse_str(&version.id).unwrap_or_default(),
        current_node_id: "start".to_string(),
        user_id: None,
        status: SessionStatus::active,
        context: serde_json::json!({}),
        expires_at: Utc::now() + Duration::minutes(15),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    state.auth_session_repo.create(&session).await?;

    // Execute Initial Step
    let result = state.flow_executor.execute(session_id, None).await?;

    // Return Response + NEW Cookie
    let cookie = create_login_cookie(session_id);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).unwrap(),
    );

    let response_body = match &result {
        ExecutionResult::Challenge { screen_id, context } => serde_json::json!({
            "status": "challenge",
            "challengeName": screen_id,
            "context": context
        }),
        ExecutionResult::Success { redirect_url } => serde_json::json!({
            "status": "redirect",
            "url": redirect_url
        }),
        ExecutionResult::Failure { reason } => serde_json::json!({
            "status": "failure",
            "message": reason
        }),
    };

    let status = if matches!(result, ExecutionResult::Failure { .. }) {
        StatusCode::UNAUTHORIZED
    } else {
        StatusCode::OK
    };

    Ok((status, headers, Json(response_body)).into_response())
}

// POST /api/auth/login/execute
// Accepts generic JSON input (mapped to credentials or other data)
pub async fn execute_login_step_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<serde_json::Value>, // Generic JSON payload
) -> Result<impl IntoResponse> {
    // 1. Extract Session ID
    let session_id = jar
        .get(LOGIN_SESSION_COOKIE)
        .map(|c| Uuid::parse_str(c.value()))
        .transpose()?
        .ok_or(Error::InvalidLoginSession)?;

    // 2. Execute Graph
    let result = state
        .flow_executor
        .execute(session_id, Some(payload))
        .await?;

    // 3. Handle Result
    match result {
        // --- UI Challenge (e.g. "Wrong Password", try again) ---
        ExecutionResult::Challenge { screen_id, context } => {
            // Keep the cookie
            Ok((
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "challenge",
                    "challengeName": screen_id,
                    "context": context
                })),
            )
                .into_response())
        }

        // --- Success (Flow Complete) ---
        ExecutionResult::Success { redirect_url } => {
            // Clear Login Cookie
            let clear_cookie = create_clear_login_cookie();
            let mut res_headers = HeaderMap::new();
            res_headers.append(
                header::SET_COOKIE,
                HeaderValue::from_str(&clear_cookie.to_string())?,
            );

            // A. Check if we need to issue a Session (e.g. this wasn't just an OIDC loop)
            // Retrieve session to get user_id
            let final_session = state
                .auth_session_repo
                .find_by_id(&session_id)
                .await?
                .ok_or(Error::InvalidLoginSession)?;

            // B. If User ID is present, issue Global Session Cookies
            if let Some(user_id) = final_session.user_id {
                // Fetch full user object
                let user = state.user_service.get_user(user_id).await?;

                // Handle OIDC Redirect vs Direct Login
                // If the redirect_url is "/", treat as direct login to dashboard
                if redirect_url == "/" {
                    let ip = headers
                        .get("x-forwarded-for")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or(addr.ip().to_string().as_str())
                        .to_string();

                    let (login_resp, refresh_token) = state
                        .auth_service
                        .create_session(&user, None, Some(ip), None)
                        .await?;
                    let refresh_cookie = create_refresh_cookie(&refresh_token);
                    res_headers.append(
                        header::SET_COOKIE,
                        HeaderValue::from_str(&refresh_cookie.to_string())?,
                    );

                    return Ok((
                        StatusCode::OK,
                        res_headers,
                        Json(serde_json::json!({
                           "status": "redirect",
                           "url": "/"
                        })),
                    )
                        .into_response());
                }
            }

            // C. Generic Redirect (e.g. OIDC)
            Ok((
                StatusCode::OK,
                res_headers,
                Json(serde_json::json!({
                    "status": "redirect",
                    "url": redirect_url
                })),
            )
                .into_response())
        }

        // --- Failure ---
        ExecutionResult::Failure { reason } => Ok((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "status": "failure",
                "message": reason
            })),
        )
            .into_response()),
    }
}

// Refresh and Logout handlers remain largely the same, just standard auth_service calls.
pub async fn refresh_handler(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse> {
    let refresh_token_id = jar
        .get(REFRESH_TOKEN_COOKIE)
        .map(|c| Uuid::parse_str(c.value()))
        .transpose()
        .map_err(|_| Error::InvalidRefreshToken)?
        .ok_or(Error::InvalidRefreshToken)?;

    match state.auth_service.refresh_session(refresh_token_id).await {
        Ok((resp, new_token)) => {
            let cookie = create_refresh_cookie(&new_token);
            let mut headers = HeaderMap::new();
            headers.insert(
                header::SET_COOKIE,
                HeaderValue::from_str(&cookie.to_string())?,
            );
            Ok((StatusCode::OK, headers, Json(resp)).into_response())
        }
        Err(_) => {
            let cookie = create_clear_cookie();
            let mut headers = HeaderMap::new();
            headers.insert(
                header::SET_COOKIE,
                HeaderValue::from_str(&cookie.to_string())?,
            );
            Ok((StatusCode::UNAUTHORIZED, headers, "Invalid Token").into_response())
        }
    }
}

pub async fn logout_handler(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse> {
    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&create_clear_cookie().to_string())?,
    );
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&create_clear_login_cookie().to_string())?,
    );

    if let Some(c) = jar.get(REFRESH_TOKEN_COOKIE) {
        if let Ok(id) = Uuid::parse_str(c.value()) {
            let _ = state.auth_service.logout(id).await;
        }
    }
    Ok((StatusCode::OK, headers, Json("Logged out")))
}
