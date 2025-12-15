use crate::domain::oidc::OidcContext;
use crate::{
    constants::{DEFAULT_REALM_NAME, LOGIN_SESSION_COOKIE, REFRESH_TOKEN_COOKIE},
    domain::{
        auth_session::AuthenticationSession, auth_session::SessionStatus,
        execution::ExecutionResult, session::RefreshToken,
    },
    error::{Error, Result},
    AppState,
};
use axum::extract::Query;
use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::net::SocketAddr;
use tracing::{info, instrument, warn};
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

fn append_clear_cookies(headers: &mut HeaderMap) {
    // Clear on API path
    let c1 = Cookie::build(LOGIN_SESSION_COOKIE)
        .path("/api")
        .max_age(time::Duration::seconds(0))
        .finish();
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&c1.to_string()).unwrap(),
    );

    // Clear on Root path (Just in case)
    let c2 = Cookie::build(LOGIN_SESSION_COOKIE)
        .path("/")
        .max_age(time::Duration::seconds(0))
        .finish();
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&c2.to_string()).unwrap(),
    );
}

// GET /api/auth/login
#[instrument(skip(state, jar))]
pub async fn start_login_flow_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse> {
    info!("[StartFlow] Incoming Request.");

    // --- 1. RESUME LOGIC (Trust the Cookie) ---
    // If the browser sends a cookie, and that session is ACTIVE, we resume it.
    // We assume the user is continuing a flow they started (either Dashboard or OIDC).

    let mut valid_session_id = None;

    let cookies: Vec<_> = jar
        .iter()
        .filter(|c| c.name() == LOGIN_SESSION_COOKIE)
        .collect();

    for cookie in cookies {
        if let Ok(parse_id) = Uuid::parse_str(cookie.value()) {
            match state.auth_session_repo.find_by_id(&parse_id).await {
                Ok(Some(session)) => {
                    // Only resume if strictly ACTIVE.
                    // Completed/Failed sessions are "Zombies" and should be ignored.
                    if session.status == SessionStatus::active {
                        info!("[StartFlow] Found ACTIVE session {}. Resuming.", parse_id);
                        valid_session_id = Some(parse_id);
                        break;
                    } else {
                        info!(
                            "[StartFlow] Session {} is {:?}. Ignoring.",
                            parse_id, session.status
                        );
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(session_id) = valid_session_id {
        // EXECUTE RESUME
        let result = state.flow_executor.execute(session_id, None).await?;
        return map_execution_result(result, HeaderMap::new());
    }

    // --- 2. NEW SESSION LOGIC ---
    // No active session found? Create a new one.
    // Since we are here, and no params were provided, this defaults to a standard Dashboard flow.
    // (If this was supposed to be OIDC, the /authorize endpoint would have created the session already).

    info!("[StartFlow] No active session found. Starting New Dashboard Session.");

    let realm = state
        .realm_service
        .find_by_name(DEFAULT_REALM_NAME)
        .await?
        .ok_or(Error::RealmNotFound(DEFAULT_REALM_NAME.to_string()))?;

    let flow_id_str = realm
        .browser_flow_id
        .ok_or(Error::System("No browser flow".into()))?;
    let flow_id =
        Uuid::parse_str(&flow_id_str).map_err(|_| Error::System("Invalid flow ID".into()))?;

    let version_num = state
        .flow_store
        .get_deployed_version_number(&realm.id, "browser", &flow_id)
        .await?
        .ok_or(Error::System("No deployed version".into()))?;
    let version = state
        .flow_store
        .get_version_by_number(&flow_id, version_num)
        .await?
        .ok_or(Error::System("Version not found".into()))?;

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
    let result = state.flow_executor.execute(session_id, None).await?;

    // Cookie Hygiene: Clear old roots, set new API cookie
    let mut headers = HeaderMap::new();
    let kill_root = Cookie::build(LOGIN_SESSION_COOKIE)
        .path("/")
        .max_age(time::Duration::seconds(0))
        .finish();
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&kill_root.to_string()).unwrap(),
    );

    let new_cookie = create_login_cookie(session_id);
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&new_cookie.to_string()).unwrap(),
    );

    map_execution_result(result, headers)
}

// POST /api/auth/login/execute
#[instrument(skip(state, jar))]
pub async fn execute_login_step_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse> {
    info!("[ExecuteStep] Processing step submission");

    let mut target_session_id = None;
    let mut fallback_session_id = None;

    let cookies: Vec<_> = jar
        .iter()
        .filter(|c| c.name() == LOGIN_SESSION_COOKIE)
        .collect();
    info!("[ExecuteStep] Found {} cookies", cookies.len());

    for cookie in cookies {
        if let Ok(parse_id) = Uuid::parse_str(cookie.value()) {
            if let Ok(Some(session)) = state.auth_session_repo.find_by_id(&parse_id).await {
                if session.status == SessionStatus::active {
                    info!("[ExecuteStep] Found ACTIVE session {}", parse_id);
                    // Use the first active one we find
                    // Improvement: We could check creation time, but first valid is usually okay
                    // if StartFlow did its job of cleaning up.
                    target_session_id = Some(parse_id);
                    break;
                } else {
                    info!("[ExecuteStep] Found inactive session {}", parse_id);
                    fallback_session_id = Some(parse_id);
                }
            }
        }
    }

    let session_id = target_session_id.or(fallback_session_id).ok_or_else(|| {
        warn!("[ExecuteStep] No valid session found in cookies");
        Error::InvalidLoginSession
    })?;

    info!("[ExecuteStep] Executing against Session {}", session_id);
    let result = state
        .flow_executor
        .execute(session_id, Some(payload))
        .await?;

    // Handle Output
    match result {
        ExecutionResult::Challenge { screen_id, context } => {
            info!("[ExecuteStep] Result: Challenge ({})", screen_id);
            Ok((
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "challenge", "challengeName": screen_id, "context": context
                })),
            )
                .into_response())
        }
        ExecutionResult::Success { redirect_url } => {
            info!(
                "[ExecuteStep] Result: Success (Redirect -> {})",
                redirect_url
            );

            // 1. Cleanup
            let mut res_headers = HeaderMap::new();
            res_headers.append(
                header::SET_COOKIE,
                HeaderValue::from_str(&create_clear_login_cookie().to_string()).unwrap(),
            );

            // 2. Fetch Session for Final Decision
            let final_session = state
                .auth_session_repo
                .find_by_id(&session_id)
                .await?
                .ok_or(Error::InvalidLoginSession)?;
            let user_id = final_session
                .user_id
                .ok_or(Error::System("Authenticated user not found".into()))?;

            // 3. PRIORITY 1: OIDC
            if let Some(oidc_value) = final_session.context.get("oidc") {
                info!("[ExecuteStep] Detected OIDC context. Generating Code.");
                if let Ok(oidc_ctx) = serde_json::from_value::<OidcContext>(oidc_value.clone()) {
                    let auth_code = state
                        .oidc_service
                        .create_authorization_code(
                            final_session.realm_id,
                            user_id,
                            oidc_ctx.client_id,
                            oidc_ctx.redirect_uri.clone(),
                            oidc_ctx.nonce,
                            oidc_ctx.code_challenge,
                            oidc_ctx
                                .code_challenge_method
                                .unwrap_or_else(|| "plain".to_string()),
                        )
                        .await?;

                    let mut url = url::Url::parse(&oidc_ctx.redirect_uri)
                        .map_err(|_| Error::OidcInvalidRedirect(oidc_ctx.redirect_uri.clone()))?;

                    url.query_pairs_mut().append_pair("code", &auth_code.code);
                    if let Some(s) = oidc_ctx.state {
                        url.query_pairs_mut().append_pair("state", &s);
                    }

                    return Ok((
                        StatusCode::OK,
                        res_headers,
                        Json(serde_json::json!({
                           "status": "redirect", "url": url.to_string()
                        })),
                    )
                        .into_response());
                }
            }

            // 4. PRIORITY 2: Dashboard
            if redirect_url == "/" {
                info!("[ExecuteStep] Direct Dashboard Login. Issuing Session Cookies.");
                let user = state.user_service.get_user(user_id).await?;
                let ip = headers
                    .get("x-forwarded-for")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or(addr.ip().to_string().as_str())
                    .to_string();

                let (_login_resp, refresh_token) = state
                    .auth_service
                    .create_session(&user, None, Some(ip), None)
                    .await?;
                let refresh_cookie = create_refresh_cookie(&refresh_token);
                res_headers.append(
                    header::SET_COOKIE,
                    HeaderValue::from_str(&refresh_cookie.to_string()).unwrap(),
                );

                return Ok((
                    StatusCode::OK,
                    res_headers,
                    Json(serde_json::json!({
                       "status": "redirect", "url": "/"
                    })),
                )
                    .into_response());
            }

            Ok((
                StatusCode::OK,
                res_headers,
                Json(serde_json::json!({
                    "status": "redirect", "url": redirect_url
                })),
            )
                .into_response())
        }
        ExecutionResult::Failure { reason } => {
            warn!("[ExecuteStep] Result: Failure ({})", reason);
            Ok((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "status": "failure", "message": reason
                })),
            )
                .into_response())
        }
    }
}

// Helper to map result to response (Deduped logic)
fn map_execution_result(result: ExecutionResult, headers: HeaderMap) -> Result<impl IntoResponse> {
    match result {
        ExecutionResult::Challenge { screen_id, context } => {
            let body = serde_json::json!({ "status": "challenge", "challengeName": screen_id, "context": context });
            Ok((StatusCode::OK, headers, Json(body)).into_response())
        }
        ExecutionResult::Success { redirect_url } => {
            let body = serde_json::json!({ "status": "redirect", "url": redirect_url });
            Ok((StatusCode::OK, headers, Json(body)).into_response())
        }
        ExecutionResult::Failure { reason } => {
            let body = serde_json::json!({ "status": "failure", "message": reason });
            Ok((StatusCode::UNAUTHORIZED, headers, Json(body)).into_response())
        }
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
