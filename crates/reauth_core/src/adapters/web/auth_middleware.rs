use crate::{adapters::web::server::AppState, domain::user::User};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

/// A struct to hold the authenticated user, which we will attach to the request.
#[derive(Clone)]
pub struct AuthUser(pub User);

/// The Axum middleware for checking a user's Access Token (JWT).
/// This is the "Adapter" that calls the "Application Service".
pub async fn auth_guard(
    State(state): State<AppState>, // 1. Get the unified state
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    mut request: Request,
    next: Next,
) -> impl IntoResponse {
    let token = auth_header.token();

    // 2. Call the single application service method
    match state.auth_service.validate_token_and_get_user(token).await {
        Ok(user) => {
            // 3. Attach the user to the request
            request.extensions_mut().insert(AuthUser(user));

            // 4. Continue to the next handler
            next.run(request).await
        }
        Err(e) => {
            // 5. If it fails, our web/error.rs adapter handles it
            e.into_response()
        }
    }
}
