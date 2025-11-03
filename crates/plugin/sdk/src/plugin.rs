//! Defines the reauth_core `Plugin` trait.

use crate::grpc::plugin::v1::PluginInfo;

/// The main trait a plugin's metadata struct must implement.
///
/// This provides the ReAuth Core with essential information about the plugin
/// during the handshake and discovery process.
pub trait Plugin {
    /// Returns the basic information about the plugin.
    fn info(&self) -> PluginInfo;
}