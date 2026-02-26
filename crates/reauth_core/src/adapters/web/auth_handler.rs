use crate::domain::oidc::OidcContext;
use crate::{
    constants::{LOGIN_SESSION_COOKIE, REFRESH_TOKEN_COOKIE},
    domain::{
        auth_session::AuthenticationSession, auth_session::SessionStatus,
        execution::ExecutionResult, session::RefreshToken,
    },
    error::{Error, Result},
    AppState,
};
use axum::extract::{Path, Query};
use axum::response::Response;
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
use tracing::{error, instrument, warn};
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

// GET /api/auth/login
// This handles generating OIDC codes OR Dashboard Cookies upon flow completion
async fn handle_flow_success(
    state: &AppState,
    session_id: Uuid,
    redirect_url: String,
    mut headers: HeaderMap,
    ip_address: String,
) -> Result<Response> {
    // 1. Cleanup Login Cookie (Flow is done)
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&create_clear_login_cookie().to_string())?,
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

    // 3. PRIORITY 1: OIDC (Dummy App / External Clients)
    if let Some(oidc_value) = final_session.context.get("oidc") {
        if let Ok(oidc_ctx) = serde_json::from_value::<OidcContext>(oidc_value.clone()) {
            // [OPTIMIZATION] Root Session Management
            // We only create a NEW persistent Root (SSO) Session if the user
            // did NOT come from an existing SSO session (i.e., they typed a password).
            // If they resumed via SSO, we trust the existing cookie and do not issue a new one.
            let sso_cookie_update_needed = final_session.context.get("sso_token_id").is_none();

            if sso_cookie_update_needed {
                let user = state.user_service.get_user(user_id).await?;
                // Create a "Root" session (client_id = None) for global SSO
                let (_, refresh_token) = state
                    .auth_service
                    .create_session(&user, None, Some(ip_address), None)
                    .await?;

                let refresh_cookie = create_refresh_cookie(&refresh_token);
                headers.append(
                    header::SET_COOKIE,
                    HeaderValue::from_str(&refresh_cookie.to_string())?,
                );
            }

            // Generate Authorization Code for the specific App
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
                headers,
                Json(serde_json::json!({
                   "status": "redirect", "url": url.to_string()
                })),
            )
                .into_response());
        }
    }

    // 4. PRIORITY 2: Dashboard (Direct Login)
    if redirect_url == "/" {
        // Dashboard login always refreshes the Root Session
        let user = state.user_service.get_user(user_id).await?;
        let (_login_resp, refresh_token) = state
            .auth_service
            .create_session(&user, None, Some(ip_address), None)
            .await?;

        let refresh_cookie = create_refresh_cookie(&refresh_token);
        headers.append(
            header::SET_COOKIE,
            HeaderValue::from_str(&refresh_cookie.to_string())?,
        );

        return Ok((
            StatusCode::OK,
            headers,
            Json(serde_json::json!({
               "status": "redirect", "url": "/"
            })),
        )
            .into_response());
    }

    // 5. Generic Redirect (Fallback)
    Ok((
        StatusCode::OK,
        headers,
        Json(serde_json::json!({
            "status": "redirect", "url": redirect_url
        })),
    )
        .into_response())
}

// GET /api/auth/login
#[instrument(skip_all)]
pub async fn start_login_flow_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(realm_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse> {
    // 1. Resolve Realm
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or_else(|| Error::RealmNotFound(realm_name.clone()))?;

    // IP extraction for later use
    let ip = addr.ip().to_string();

    let force_login = params.get("prompt").map(|v| v == "login").unwrap_or(false);
    let sso_token_id = if !force_login {
        jar.get(REFRESH_TOKEN_COOKIE).map(|c| c.value().to_string())
    } else {
        None
    };

    // --- 1. RESUME LOGIC ---
    let mut valid_session_id = None;
    if !force_login {
        let cookies: Vec<_> = jar
            .iter()
            .filter(|c| c.name() == LOGIN_SESSION_COOKIE)
            .collect();

        for cookie in cookies {
            if let Ok(parse_id) = Uuid::parse_str(cookie.value()) {
                if let Ok(Some(mut session)) = state.auth_session_repo.find_by_id(&parse_id).await {
                    // Verify the existing session belongs to the requested Realm.
                    // If I am logged into "Tenant A" but request "Tenant B", ignore the cookie.
                    if session.realm_id != realm.id {
                        warn!(
                            "[StartFlow] Ignoring session {} (Realm mismatch).",
                            parse_id
                        );
                        continue;
                    }

                    if session.status == SessionStatus::Active {
                        // [CRITICAL] Inject Context into Resumed Session
                        let mut updated = false;
                        if let Some(token) = &sso_token_id {
                            session.context["sso_token_id"] =
                                serde_json::Value::String(token.clone());
                            updated = true;
                        }
                        if let Some(client_id) = params.get("client_id") {
                            session.context["oidc"] = serde_json::json!({
                                "client_id": client_id,
                                "redirect_uri": params.get("redirect_uri"),
                                "response_type": params.get("response_type"),
                                "scope": params.get("scope"),
                                "state": params.get("state"),
                                "nonce": params.get("nonce"),
                                "code_challenge": params.get("code_challenge"),
                                "code_challenge_method": params.get("code_challenge_method"),
                            });
                            updated = true;
                        }

                        if updated {
                            state.auth_session_repo.update(&session).await?;
                        }

                        valid_session_id = Some(parse_id);
                        break;
                    }
                }
            }
        }
    }

    // Determine Session ID (Resume or Create)
    let session_id = if let Some(sid) = valid_session_id {
        sid
    } else {
        let flow_id_str = realm
            .browser_flow_id
            .ok_or(Error::System("Realm has no browser flow configured".into()))?;
        let flow_id =
            Uuid::parse_str(&flow_id_str).map_err(|_| Error::System("Invalid Flow ID".into()))?;

        // Version Resolution
        let version_num = state
            .flow_store
            .get_deployed_version_number(&realm.id, "browser", &flow_id)
            .await?
            .ok_or(Error::System("Flow not deployed".into()))?;
        let version = state
            .flow_store
            .get_version_by_number(&flow_id, version_num)
            .await?
            .ok_or(Error::System("Flow version not found".into()))?;

        let mut context = serde_json::json!({});
        // Inject OIDC
        if let Some(client_id) = params.get("client_id") {
            context["oidc"] = serde_json::json!({
                "client_id": client_id,
                "redirect_uri": params.get("redirect_uri"),
                "response_type": params.get("response_type"),
                "scope": params.get("scope"),
                "state": params.get("state"),
                "nonce": params.get("nonce"),
                "code_challenge": params.get("code_challenge"),
                "code_challenge_method": params.get("code_challenge_method"),
            });
        }
        // Inject SSO
        if let Some(token) = sso_token_id {
            context["sso_token_id"] = serde_json::Value::String(token);
        }

        let new_sid = Uuid::new_v4();
        let session = AuthenticationSession {
            id: new_sid,
            realm_id: realm.id,
            flow_version_id: Uuid::parse_str(&version.id).unwrap_or_default(),
            current_node_id: "start".to_string(),
            user_id: None,
            status: SessionStatus::Active,
            context,
            expires_at: Utc::now() + Duration::minutes(15),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        state.auth_session_repo.create(&session).await?;
        new_sid
    };

    // Execute Flow
    let result = state.flow_executor.execute(session_id, None).await?;

    // Prepare Cookies
    let mut headers = HeaderMap::new();
    // Always clear root login cookie to prevent clashes
    let kill_root = Cookie::build(LOGIN_SESSION_COOKIE)
        .path("/")
        .max_age(time::Duration::seconds(0))
        .build();
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&kill_root.to_string())?,
    );

    // Set API-scoped login cookie
    let new_cookie = create_login_cookie(session_id);
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&new_cookie.to_string())?,
    );

    // Handle Result
    match result {
        // [FIX] Use shared logic for Success
        ExecutionResult::Success { redirect_url } => {
            handle_flow_success(&state, session_id, redirect_url, headers, ip).await
        }
        _ => map_execution_result(result, headers),
    }
}

// POST /api/auth/login/execute
#[instrument(skip_all)]
pub async fn execute_login_step_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(realm_name): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse> {
    // We fetch the realm just to ensure it exists (Validation)
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or_else(|| Error::RealmNotFound(realm_name.clone()))?;

    let ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(addr.ip().to_string().as_str())
        .to_string();

    // Session Selection Logic (same as before)
    let mut target_session_id = None;
    let cookies: Vec<_> = jar
        .iter()
        .filter(|c| c.name() == LOGIN_SESSION_COOKIE)
        .collect();

    for cookie in cookies {
        if let Ok(parse_id) = Uuid::parse_str(cookie.value()) {
            if let Ok(Some(session)) = state.auth_session_repo.find_by_id(&parse_id).await {
                if session.realm_id == realm.id && session.status == SessionStatus::Active {
                    target_session_id = Some(parse_id);
                    break;
                }
            }
        }
    }

    let session_id = target_session_id.ok_or(Error::InvalidLoginSession)?;

    // Execute
    let result = state
        .flow_executor
        .execute(session_id, Some(payload))
        .await?;

    // Handle Result
    match result {
        ExecutionResult::Success { redirect_url } => {
            handle_flow_success(&state, session_id, redirect_url, HeaderMap::new(), ip).await
        }
        _ => map_execution_result(result, HeaderMap::new()),
    }
}

// Helper to map result to response (Deduped logic)
fn map_execution_result(result: ExecutionResult, headers: HeaderMap) -> Result<Response> {
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
        ExecutionResult::Continue => {
            error!("[MapResult] Internal 'Continue' state reached web layer.");
            let body =
                serde_json::json!({ "status": "failure", "message": "Internal System Error" });
            Ok((StatusCode::INTERNAL_SERVER_ERROR, headers, Json(body)).into_response())
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
    Path(_realm): Path<String>,
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
