//! Database writer module
//!
//! Provides modular database write operations organized by functionality:
//! - Core database operations
//! - Session management
//! - Benchmark synchronization
//! - Performance tracking
//! - Database monitoring

pub mod benchmarks;
pub mod core;
pub mod monitoring;
pub mod performance;
pub mod sessions;

// Re-export main DatabaseWriter for backward compatibility
pub use core::DatabaseWriter;

/// Database writer trait for common operations
pub trait DatabaseWriterTrait {
    /// Get the underlying connection
    fn connection(&self) -> &turso::Connection;
}
