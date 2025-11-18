use crate::domain::auth_flow::AuthStepResult;
use crate::{
    adapters::web::server::AppState,
    constants::REFRESH_TOKEN_COOKIE,
    domain::session::RefreshToken,
    error::{Error, Result},
};
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use cookie::CookieBuilder;
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

fn create_refresh_cookie(token: &RefreshToken) -> Cookie<'static> {
    let expires_time = {
        let system_time = std::time::SystemTime::from(token.expires_at);
        time::OffsetDateTime::from(system_time)
    };

    Cookie::build((REFRESH_TOKEN_COOKIE, token.id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(cookie::SameSite::Strict)
        .expires(expires_time)
        .into()
}

fn create_clear_cookie() -> Cookie<'static> {
    Cookie::build(REFRESH_TOKEN_COOKIE)
        .path("/")
        .http_only(true)
        .same_site(cookie::SameSite::Strict)
        .expires(time::OffsetDateTime::UNIX_EPOCH)
        .into()
}

const LOGIN_SESSION_COOKIE: &str = "reauth_login_session";

// ---
// NEW: `GET /api/auth/login`
// Starts the login flow
// ---
pub async fn start_login_flow_handler(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let (login_session, first_challenge) = state.flow_engine.start_login_flow().await?;

    let expires_time =
        time::OffsetDateTime::from_unix_timestamp(login_session.expires_at.timestamp())
            .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);

    // Set a cookie to track this login attempt
    let cookie: CookieBuilder = Cookie::build((LOGIN_SESSION_COOKIE, login_session.id.to_string()))
        .path("/api/auth") // Only send to auth endpoints
        .http_only(true)
        .same_site(cookie::SameSite::Strict)
        .expires(expires_time); // Use the converted time

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string())?,
    );

    Ok((StatusCode::OK, headers, Json(first_challenge)))
}

// ---
// NEW: `POST /api/auth/login/execute`
// Executes the current step of the flow
// ---
#[derive(Deserialize)]
pub struct ExecutePayload {
    credentials: HashMap<String, String>,
}

pub async fn execute_login_step_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<ExecutePayload>,
) -> Result<impl IntoResponse> {
    let login_session_id = jar
        .get(LOGIN_SESSION_COOKIE)
        .map(|cookie| Uuid::parse_str(cookie.value()))
        .transpose()?
        .ok_or(Error::InvalidLoginSession)?;

    match state
        .flow_engine
        .process_login_step(login_session_id, payload.credentials)
        .await?
    {
        // --- Flow is 100% complete ---
        (None, AuthStepResult::Success, Some(user)) => {
            // Flow succeeded, so we create the real user session
            let (login_response, refresh_token) = state.auth_service.create_session(&user).await?;

            let refresh_cookie = create_refresh_cookie(&refresh_token);
            let clear_login_cookie: CookieBuilder = Cookie::build(LOGIN_SESSION_COOKIE)
                .path("/api/auth")
                .expires(time::OffsetDateTime::UNIX_EPOCH);

            let mut headers = HeaderMap::new();
            headers.append(
                header::SET_COOKIE,
                HeaderValue::from_str(&refresh_cookie.to_string())?,
            );
            headers.append(
                header::SET_COOKIE,
                HeaderValue::from_str(&clear_login_cookie.to_string())?,
            );

            Ok((StatusCode::OK, headers, Json(login_response)).into_response())
        }
        // --- Flow is advancing to the next step ---
        (Some(new_login_session), result @ AuthStepResult::Challenge { .. }, None) => {
            let expires_time =
                time::OffsetDateTime::from_unix_timestamp(new_login_session.expires_at.timestamp())
                    .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);
            // The flow continues, update the session cookie
            let cookie: CookieBuilder =
                Cookie::build((LOGIN_SESSION_COOKIE, new_login_session.id.to_string()))
                    .path("/api/auth")
                    .http_only(true)
                    .same_site(cookie::SameSite::Strict)
                    .expires(expires_time);

            let mut headers = HeaderMap::new();
            headers.insert(
                header::SET_COOKIE,
                HeaderValue::from_str(&cookie.to_string())?,
            );
            Ok((StatusCode::OK, headers, Json(result)).into_response())
        }

        // --- Step failed (e.g., wrong password) ---
        (_, result @ AuthStepResult::Failure { .. }, None) => {
            Ok((StatusCode::UNAUTHORIZED, Json(result)).into_response())
        }

        // --- Catch-all for invalid states ---
        _ => Err(Error::InvalidLoginStep),
    }
}

pub async fn refresh_handler(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse> {
    let refresh_token_id = jar
        .get(REFRESH_TOKEN_COOKIE)
        .map(|cookie| Uuid::parse_str(cookie.value()))
        .transpose()
        .map_err(|_| Error::InvalidRefreshToken)?
        .ok_or(Error::InvalidRefreshToken)?;

    match state.auth_service.refresh_session(refresh_token_id).await {
        Ok((login_response, new_refresh_token)) => {
            let cookie = create_refresh_cookie(&new_refresh_token);
            let mut headers = HeaderMap::new();

            let cookie_value = HeaderValue::from_str(&cookie.to_string())
                .map_err(|e| Error::Unexpected(e.into()))?;

            headers.insert(header::SET_COOKIE, cookie_value);
            Ok((StatusCode::OK, headers, Json(login_response)).into_response())
        }
        Err(e @ Error::InvalidRefreshToken) => {
            let cookie = create_clear_cookie();
            let mut headers = HeaderMap::new();

            let cookie_value = HeaderValue::from_str(&cookie.to_string())
                .map_err(|e| Error::Unexpected(e.into()))?;

            headers.insert(header::SET_COOKIE, cookie_value);
            Ok((StatusCode::UNAUTHORIZED, headers, e.to_string()).into_response())
        }
        Err(e) => Err(e),
    }
}
