use crate::adapters::web::middleware::request_logging::RequestContext;
use crate::constants::ACCESS_TOKEN_COOKIE;
use crate::{domain::user::User, AppState};
use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use serde_json::json;
use tracing::{field, instrument, Span};

/// A struct to hold the authenticated user, which we will attach to the request.
#[derive(Clone)]
pub struct AuthUser(pub User);

#[instrument(skip_all, fields(telemetry = "span"))]
pub async fn auth_guard(
    State(state): State<AppState>,
    // We ADD CookieJar to read browser cookies
    cookie_jar: CookieJar,
    // We REMOVE TypedHeader from args to prevent auto-rejection
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // 1. Try to extract token from the "Authorization" Header first
    let token_from_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer ").map(|token| token.to_string()));

    // 2. Fallback: Try to extract from the "access_token" Cookie
    let token = match token_from_header {
        Some(t) => t,
        None => match cookie_jar.get(ACCESS_TOKEN_COOKIE) {
            Some(c) => c.value().to_string(),
            None => {
                // If neither exists, return 401 immediately
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "Missing Authentication Token",
                        "code": "auth.missing_token"
                    })),
                )
                    .into_response();
            }
        },
    };

    // 3. Validate via Service
    match state.auth_service.validate_token_and_get_user(&token).await {
        Ok(user) => {
            let user_id = user.id;
            // [CRITICAL] Insert ONLY the UUID if your permission_guard expects Uuid
            req.extensions_mut().insert(user_id);

            // Optional: Insert the full User struct if other handlers need it
            req.extensions_mut().insert(AuthUser(user));

            if let Some(context) = req.extensions().get::<RequestContext>() {
                context.set_user_id(user_id);
            }

            Span::current().record("user_id", field::display(user_id));

            next.run(req).await
        }
        Err(e) => {
            // Convert your domain error into an Axum response
            e.into_response()
        }
    }
}
