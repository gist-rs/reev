//! Reev API Library
//!
//! This library provides the core API functionality for the Reev system.
//! It exposes handlers and services for testing and reuse.

pub mod handlers;
pub mod services;
pub mod types;

// Re-export commonly used types and handlers for external use
pub use handlers::*;
pub use services::*;
pub use types::*;
