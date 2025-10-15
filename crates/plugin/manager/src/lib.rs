//! The ReAuth Plugin Manager crate.
//!
//! This crate is responsible for discovering, spawning, and managing the lifecycle
//! of backend plugins for the ReAuth application.

// Declare the public modules of the crate.
pub mod error;
pub mod grpc;
pub mod manager;
pub mod plugin;

// Re-export the most important types for consumers of this crate.
// This is often called a "prelude."
pub use error::{Error, Result};
pub use manager::PluginManager;
pub use plugin::{Manifest, PluginInstance};