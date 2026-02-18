//! Defines the configuration for the PluginManager.
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ManagerConfig {
    pub handshake_timeout_secs: u64,
}
