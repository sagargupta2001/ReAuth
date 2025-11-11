//! The ReAuth Plugin Manager crate.
//!
//! This crate is responsible for discovering, spawning, and managing the lifecycle
//! of backend plugins for the ReAuth application.

// Declare the public modules of the crate.
pub mod config;
pub mod constants;
pub mod error;
pub mod grpc;
pub mod log_bus;
pub mod manager;
pub mod plugin;

// Re-export the most important types for consumers of this crate.
pub use config::ManagerConfig;
pub use error::{Error, Result};
pub use log_bus::{LogPublisher, LogSubscriber};
pub use manager::{LogEntry, PluginManager};
pub use plugin::{Manifest, PluginInstance};
