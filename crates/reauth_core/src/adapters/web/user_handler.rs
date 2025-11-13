use crate::adapters::web::server::AppState;
use crate::adapters::web::validation::ValidatedJson;
use crate::error::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateUserPayload {
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    username: String,
    #[validate(length(
        min = 8,
        max = 100,
        message = "Password must be between 8 and 100 characters"
    ))]
    password: String,
}

pub async fn create_user_handler(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateUserPayload>,
) -> Result<impl IntoResponse> {
    let user = state
        .user_service
        .create_user(&payload.username, &payload.password)
        .await?;

    Ok((StatusCode::CREATED, Json(user)))
}
