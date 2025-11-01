//! Contains the reauth_core data structures for representing a plugin.

use tokio::process::Child;
use tonic::transport::Channel;

/// Represents a single, live, running plugin instance.
#[derive(Debug)]
pub struct PluginInstance {
    /// The handle to the running OS process for the plugin.
    pub process: Child,
    /// The deserialized manifest data for the plugin.
    pub manifest: Manifest,
    /// The active gRPC channel for communicating with the plugin.
    pub grpc_channel: Channel,
}

/// Defines the structure of the executable paths in `plugin.json`.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ExecutableConfig {
    /// The path to the executable for Linux (x86_64).
    pub linux_amd64: String,
    /// The path to the executable for Windows (x86_64).
    pub windows_amd64: String,
}

/// Defines the structure of the frontend configuration in `plugin.json`.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct FrontendConfig {
    /// The entry point for the plugin's UI module.
    pub entry: String,
    /// The route where the plugin's main page should be rendered.
    pub route: String,
    /// The label to display in the UI sidebar.
    #[serde(rename = "sidebarLabel")]
    pub sidebar_label: String,
}

/// Represents the `plugin.json` manifest file.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Manifest {
    /// A unique identifier for the plugin.
    pub id: String,
    /// The human-readable name of the plugin.
    pub name: String,
    /// The version of the plugin.
    pub version: String,
    /// Configuration for the backend executable.
    pub executable: ExecutableConfig,
    /// Configuration for the frontend UI.
    pub frontend: FrontendConfig,
}