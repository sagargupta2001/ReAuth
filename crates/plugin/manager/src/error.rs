//! Defines the custom `Error` and `Result` types for the plugin manager crate.

use std::path::PathBuf;

/// A specialized `Result` type for plugin manager operations.
pub type Result<T> = std::result::Result<T, Error>;

/// The primary error type for the plugin manager.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Plugins directory not found: {0:?}")]
    PluginsDirNotFound(PathBuf),

    #[error("Failed to read plugin directory entry")]
    ReadDirEntry(#[from] std::io::Error),

    #[error("Failed to parse manifest for plugin at {path:?}: {source}")]
    ManifestParse {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("Unsupported OS for plugin '{0}'")]
    UnsupportedOS(String),

    #[error("Failed to spawn plugin '{name}': {source}")]
    PluginSpawn {
        name: String,
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Plugin '{0}' did not send a valid handshake within the time limit")]
    HandshakeTimeout(String),

    #[error("Handshake with plugin '{name}' was invalid: {reason}")]
    HandshakeInvalid { name: String, reason: String },

    #[error("Failed to connect to gRPC for plugin '{name}': {source}")]
    GrpcConnection {
        name: String,
        source: tonic::transport::Error,
    },

    #[error("gRPC handshake verification failed for plugin '{name}': {source}")]
    GrpcVerification { name: String, source: tonic::Status },

    #[error("Plugin not found on disk: {0}")]
    PluginNotFound(String),

    #[error("Plugin is not active (running): {0}")]
    PluginNotActive(String),
}
