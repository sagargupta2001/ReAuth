use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

// Import the application's crate error type
use crate::error::Error;

/// This is the adapter's translation layer.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message, details) = match self {
            // 401 Unauthorized
            Error::InvalidCredentials
            | Error::SessionRevoked
            | Error::ReauthRequired
            | Error::InvalidRefreshToken
            | Error::OidcInvalidCode => (StatusCode::UNAUTHORIZED, self.to_string(), None),

            // 403 Forbidden
            Error::SecurityViolation(_) => (StatusCode::FORBIDDEN, self.to_string(), None),

            // 409 Conflict
            Error::UserAlreadyExists
            | Error::UsernameAlreadyExists
            | Error::EmailAlreadyExists
            | Error::PhoneNumberAlreadyExists
            | Error::RoleAlreadyExists
            | Error::GroupAlreadyExists
            | Error::RealmAlreadyExists
            | Error::Conflict(_) => (StatusCode::CONFLICT, self.to_string(), None),

            // 429 Too Many Requests
            Error::RateLimited(_) => (StatusCode::TOO_MANY_REQUESTS, self.to_string(), None),

            // 404 Not Found
            Error::UserNotFound
            | Error::UserEmailNotFound
            | Error::RealmNotFound(_)
            | Error::FlowNotFound(_)
            | Error::AuthenticatorNotFound(_)
            | Error::InvalidLoginStep
            | Error::NotFound(_)
            | Error::InvalidLoginSession => (StatusCode::NOT_FOUND, self.to_string(), None),

            // 400 Bad Request
            Error::InvalidActionToken => (StatusCode::BAD_REQUEST, self.to_string(), None),

            // 422 Unprocessable Entity
            Error::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string(), None),

            Error::FieldsValidation {
                ref message,
                ref fields,
            } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                message.clone(),
                Some(json!({ "fields": fields })),
            ),

            Error::FlowPublishValidation(ref details) => (
                StatusCode::BAD_REQUEST,
                details.message.clone(),
                Some(serde_json::to_value(details).unwrap_or_default()),
            ),

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
                    None,
                )
            }

            Error::OidcClientNotFound(_)
            | Error::OidcInvalidRedirect(_)
            | Error::OidcInvalidRequest(_) => (StatusCode::BAD_REQUEST, self.to_string(), None),

            Error::Jwt(_) => (
                StatusCode::UNAUTHORIZED,
                "Invalid or expired token.".to_string(),
                None,
            ),
        };

        let code = error_code(&self);
        let mut body = json!({ "error": message, "code": code });

        if let Some(details) = details {
            if let Some(details_obj) = details.as_object() {
                for (k, v) in details_obj {
                    body[k] = v.clone();
                }
            } else {
                body["details"] = details;
            }
        }

        (status, Json(body)).into_response()
    }
}

fn error_code(error: &Error) -> &'static str {
    match error {
        Error::InvalidCredentials => "auth.invalid_credentials",
        Error::SessionRevoked => "auth.session_revoked",
        Error::ReauthRequired => "auth.reauth_required",
        Error::InvalidRefreshToken => "auth.invalid_refresh_token",
        Error::InvalidActionToken => "auth.invalid_action_token",
        Error::OidcInvalidCode => "oidc.invalid_code",
        Error::SecurityViolation(_) => "security.violation",
        Error::UserAlreadyExists => "user.already_exists",
        Error::UsernameAlreadyExists => "user.username_already_exists",
        Error::EmailAlreadyExists => "user.email_already_exists",
        Error::PhoneNumberAlreadyExists => "user.phone_number_already_exists",
        Error::RoleAlreadyExists => "rbac.role.already_exists",
        Error::GroupAlreadyExists => "rbac.group.already_exists",
        Error::RealmAlreadyExists => "realm.already_exists",
        Error::RateLimited(_) => "request.rate_limited",
        Error::UserNotFound => "user.not_found",
        Error::UserEmailNotFound => "user.email_not_found",
        Error::RealmNotFound(_) => "realm.not_found",
        Error::FlowNotFound(_) => "flow.not_found",
        Error::AuthenticatorNotFound(_) => "authenticator.not_found",
        Error::InvalidLoginStep => "auth.invalid_login_step",
        Error::InvalidLoginSession => "auth.invalid_login_session",
        Error::Validation(_) => "validation.failed",
        Error::Conflict(_) => "request.conflict",
        Error::FieldsValidation { .. } => "validation.failed",
        Error::FlowPublishValidation(_) => "validation.failed",
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
