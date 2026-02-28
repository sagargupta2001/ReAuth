use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

// Import the application's reauth_core error type
use crate::error::Error;

/// This is the adapter's translation layer.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            // 401 Unauthorized
            Error::InvalidCredentials
            | Error::SessionRevoked
            | Error::InvalidRefreshToken
            | Error::InvalidActionToken
            | Error::OidcInvalidCode => (StatusCode::UNAUTHORIZED, self.to_string()),

            // 403 Forbidden
            Error::SecurityViolation(_) => (StatusCode::FORBIDDEN, self.to_string()),

            // 409 Conflict
            Error::UserAlreadyExists
            | Error::RoleAlreadyExists
            | Error::GroupAlreadyExists
            | Error::RealmAlreadyExists => (StatusCode::CONFLICT, self.to_string()),

            // 404 Not Found
            Error::UserNotFound
            | Error::RealmNotFound(_)
            | Error::FlowNotFound(_)
            | Error::AuthenticatorNotFound(_)
            | Error::InvalidLoginStep
            | Error::NotFound(_)
            | Error::InvalidLoginSession => (StatusCode::NOT_FOUND, self.to_string()),

            // 422 Unprocessable Entity
            Error::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),

            // 500 Internal Server Error (for things the user can't fix)
            Error::Config(_)
            | Error::DatabaseInit(_)
            | Error::Unexpected(_)
            | Error::Uuid(_)
            | Error::System(_)
            | Error::InvalidHeader(_) => {
                // Log the detailed, internal error for developers
                tracing::error!("Internal server error: {:?}", self);
                // Return a generic, safe message to the client
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An unexpected error occurred.".to_string(),
                )
            }

            Error::OidcClientNotFound(_)
            | Error::OidcInvalidRedirect(_)
            | Error::OidcInvalidRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),

            Error::Jwt(_) => (
                StatusCode::UNAUTHORIZED,
                "Invalid or expired token.".to_string(),
            ),
        };

        let code = error_code(&self);
        (status, Json(json!({ "error": message, "code": code }))).into_response()
    }
}

fn error_code(error: &Error) -> &'static str {
    match error {
        Error::InvalidCredentials => "auth.invalid_credentials",
        Error::SessionRevoked => "auth.session_revoked",
        Error::InvalidRefreshToken => "auth.invalid_refresh_token",
        Error::InvalidActionToken => "auth.invalid_action_token",
        Error::OidcInvalidCode => "oidc.invalid_code",
        Error::SecurityViolation(_) => "security.violation",
        Error::UserAlreadyExists => "user.already_exists",
        Error::RoleAlreadyExists => "rbac.role.already_exists",
        Error::GroupAlreadyExists => "rbac.group.already_exists",
        Error::RealmAlreadyExists => "realm.already_exists",
        Error::UserNotFound => "user.not_found",
        Error::RealmNotFound(_) => "realm.not_found",
        Error::FlowNotFound(_) => "flow.not_found",
        Error::AuthenticatorNotFound(_) => "authenticator.not_found",
        Error::InvalidLoginStep => "auth.invalid_login_step",
        Error::InvalidLoginSession => "auth.invalid_login_session",
        Error::Validation(_) => "validation.failed",
        Error::NotFound(_) => "resource.not_found",
        Error::OidcClientNotFound(_) => "oidc.client_not_found",
        Error::OidcInvalidRedirect(_) => "oidc.invalid_redirect",
        Error::OidcInvalidRequest(_) => "oidc.invalid_request",
        Error::Jwt(_) => "auth.invalid_token",
        Error::InvalidHeader(_) => "request.invalid_header",
        Error::Config(_) => "config.error",
        Error::DatabaseInit(_) => "database.init_failed",
        Error::System(_) => "system.error",
        Error::Uuid(_) => "system.uuid_error",
        Error::Unexpected(_) => "internal_error",
    }
}
