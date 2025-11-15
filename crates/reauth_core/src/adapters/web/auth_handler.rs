use crate::{
    adapters::web::server::AppState,
    application::auth_service::LoginPayload,
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

pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse> {
    let (login_response, refresh_token) = state.auth_service.login(payload).await?;

    let cookie = create_refresh_cookie(&refresh_token);
    let mut headers = HeaderMap::new();

    let cookie_value =
        HeaderValue::from_str(&cookie.to_string()).map_err(|e| Error::Unexpected(e.into()))?;

    headers.insert(header::SET_COOKIE, cookie_value);

    Ok((StatusCode::OK, headers, Json(login_response)))
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
