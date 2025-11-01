//! Defines the custom `Error` and `Result` types for the core application.

/// A specialized `Result` type for core operations.
pub type Result<T> = std::result::Result<T, Error>;

/// The primary error enum for the core application.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Database initialization failed: {0}")]
    DatabaseInit(String),

    #[error("A user with this username already exists")]
    UserAlreadyExists,

    #[error("User not found")]
    UserNotFound,

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),

    #[error("A role with this name already exists")]
    RoleAlreadyExists,

    #[error("A group with this name already exists")]
    GroupAlreadyExists,
}