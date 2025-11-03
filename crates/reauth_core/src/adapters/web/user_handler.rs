use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::adapters::web::server::AppState;
use crate::error::Result;

#[derive(Deserialize)]
pub struct CreateUserPayload {
    username: String,
    // TODO: Add `password: String` field here for a real implementation
    // The `role` field will be removed as roles will be assigned via the RBAC API.
}

/// The handler now returns the application's Result type.
pub async fn create_user_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<impl IntoResponse> {

    // In a real implementation, you would get a password from the payload
    // and hash it here or in the user_service.
    // For now, we'll create a user with a placeholder role.

    // The `?` operator automatically propagates any `Err` from the service,
    // which our `IntoResponse` implementation for `Error` will turn into
    // the correct HTTP status code (e.g., 409 CONFLICT).
    let user = state.user_service
        .create_user(&payload.username, "default-password-hash") // Placeholder for hashed password
        .await?;

    // If successful, just return the `Ok` response.
    Ok((StatusCode::CREATED, Json(user)))
}