use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

// Import the application's reauth_core error type
use crate::error::Error;

/// A simple struct to serialize error messages as JSON.
#[derive(serde::Serialize)]
struct ErrorResponse {
    message: String,
}

/// This is the adapter's translation layer.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            // 401 Unauthorized
            Error::InvalidCredentials | Error::SessionRevoked => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }

            // 409 Conflict
            Error::UserAlreadyExists
            | Error::RoleAlreadyExists
            | Error::GroupAlreadyExists
            | Error::RealmAlreadyExists => (StatusCode::CONFLICT, self.to_string()),

            // 404 Not Found
            Error::UserNotFound => (StatusCode::NOT_FOUND, self.to_string()),

            // 500 Internal Server Error (for things the user can't fix)
            Error::Config(_) | Error::DatabaseInit(_) | Error::Unexpected(_) => {
                // Log the detailed, internal error for developers
                tracing::error!("Internal server error: {:?}", self);
                // Return a generic, safe message to the client
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An unexpected error occurred.".to_string(),
                )
            }

            Error::Jwt(_) => (
                StatusCode::UNAUTHORIZED,
                "Invalid or expired token.".to_string(),
            ),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
