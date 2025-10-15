//! Shared database module for reev
//!
//! This module provides a unified interface for database operations
//! used by both web and TUI interfaces.

pub mod reader;
pub mod types;
pub mod writer;

// Re-export commonly used types
pub use types::*;
pub use writer::DatabaseWriter;
