use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::adapters::web::server::AppState;
use crate::error::Result;

#[derive(Deserialize)]
pub struct CreateUserPayload {
    username: String,
    password: String,
}

/// The handler now returns the application's Result type.
pub async fn create_user_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<impl IntoResponse> {
    // 2. Pass the real username and password to the service
    let user = state
        .user_service
        .create_user(&payload.username, &payload.password)
        .await?;

    // If successful, just return the `Ok` response.
    Ok((StatusCode::CREATED, Json(user)))
}
