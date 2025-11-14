use crate::{
    adapters::web::server::AppState,
    application::auth_service::LoginPayload,
    error::{Error, Result},
};
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};

pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse> {
    // Call the application service, which returns both tokens
    let (login_response, refresh_token) = state.auth_service.login(payload).await?;

    // Create the HttpOnly cookie for the refresh token
    // This is a secure-by-default, HttpOnly cookie that the browser
    // will automatically send on requests to /api/auth/refresh (which we'll build later).
    // JavaScript can't access it, which prevents XSS attacks.
    let cookie_value = format!(
        "refresh_token={}; HttpOnly; SameSite=Strict; Path=/; Expires={}",
        refresh_token.id,
        refresh_token.expires_at.to_rfc2822() // Format as a standard cookie expiration date
    );

    // Create the response headers and add the cookie
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie_value).map_err(|e| Error::Unexpected(e.into()))?, // Convert error to our app Error
    );

    // Return the (Status, Headers, JSON Body) tuple
    // The JSON body (login_response) contains the access_token.
    Ok((StatusCode::OK, headers, Json(login_response)))
}
