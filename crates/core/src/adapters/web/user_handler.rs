use crate::application::user_service::UserService;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CreateUserPayload {
    username: String,
    role: String,
}

pub async fn create_user_handler(
    State(user_service): State<Arc<UserService>>,
    Json(payload): Json<CreateUserPayload>,
) -> impl IntoResponse {
    match user_service.create_user(&payload.username, &payload.role).await {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(e) => {
            // In a real app, map errors to specific HTTP status codes
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}