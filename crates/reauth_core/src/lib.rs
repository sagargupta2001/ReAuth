pub mod adapters;
pub mod application;
pub mod bootstrap;
pub mod config;
pub mod constants;
pub mod domain;
pub mod error;
pub mod ports;

// Re-export the main API structs/functions:
pub use bootstrap::{initialize, run, AppState};
