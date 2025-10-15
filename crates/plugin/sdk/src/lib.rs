//! The official SDK for building backend plugins for ReAuth.
//!
//! This crate provides the necessary traits, functions, and generated gRPC types
//! to easily create a new plugin that can communicate with the ReAuth Core.

// Declare the modules of the crate.
pub mod constants;
pub mod grpc;
pub mod plugin;
pub mod runner;

/// A "prelude" for easily importing the most common types.
pub mod prelude {
    pub use crate::grpc::plugin::v1;
    pub use crate::plugin::Plugin;
    pub use crate::runner::run;
}