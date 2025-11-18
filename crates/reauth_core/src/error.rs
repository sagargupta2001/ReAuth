//! Defines the custom `Error` and `Result` types for the reauth_core application.

use http::header::InvalidHeaderValue;

/// A specialized `Result` type for reauth_core operations.
pub type Result<T> = std::result::Result<T, Error>;

/// The primary error enum for the reauth_core application.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}.")]
    Config(#[from] config::ConfigError),

    #[error("Database initialization failed: {0}.")]
    DatabaseInit(String),

    #[error("A user with this username already exists.")]
    UserAlreadyExists,

    #[error("User not found.")]
    UserNotFound,

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),

    #[error("A role with this name already exists.")]
    RoleAlreadyExists,

    #[error("A group with this name already exists.")]
    GroupAlreadyExists,

    #[error("The credentials provided are incorrect.")]
    InvalidCredentials,

    #[error("Invalid or expired refresh token")]
    InvalidRefreshToken,

    #[error("The session has been revoked.")]
    SessionRevoked,

    #[error("A realm with this name already exists.")]
    RealmAlreadyExists,

    #[error("Realm not found: {0}")]
    RealmNotFound(String),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Flow not found: {0}")]
    FlowNotFound(String),

    #[error("Invalid login session.")]
    InvalidLoginSession,

    #[error("Invalid login step.")]
    InvalidLoginStep,

    #[error("Authenticator not found: {0}")]
    AuthenticatorNotFound(String),

    #[error(transparent)]
    Uuid(#[from] uuid::Error),

    #[error(transparent)]
    InvalidHeader(#[from] InvalidHeaderValue),
}
