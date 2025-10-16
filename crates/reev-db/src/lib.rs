//! # Reev Database Library
//!
//! A reusable database utility crate for SQLite/Turso with robust sync operations,
//! duplicate prevention, and comprehensive monitoring capabilities.
//!
//! ## Features
//!
//! - **Atomic Operations**: Built-in duplicate prevention with ON CONFLICT
//! - **Sequential Processing**: Race-condition free sync operations
//! - **Comprehensive Monitoring**: Duplicate detection and database statistics
//! - **Error Handling**: Detailed error context with thiserror
//! - **Testing Support**: Built-in testing utilities and inspection tools
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use reev_db::{DatabaseConfig, DatabaseWriter};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = DatabaseConfig::new("path/to/database.db");
//!     let db = DatabaseWriter::new(config).await?;
//!
//!     // Sync benchmarks from directory
//!     let count = db.sync_benchmarks_from_dir("benchmarks").await?;
//!     println!("Synced {} benchmarks", count);
//!
//!     // Check for duplicates
//!     let duplicates = db.check_for_duplicates().await?;
//!     if !duplicates.is_empty() {
//!         println!("Found {} duplicate records", duplicates.len());
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod error;
pub mod reader;
pub mod types;
pub mod writer;

// Re-export commonly used types
pub use config::DatabaseConfig;
pub use error::{DatabaseError, Result};
pub use reader::DatabaseReader;
pub use types::*;
pub use writer::DatabaseWriter;

/// Current library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library metadata
pub struct LibraryInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
}

impl LibraryInfo {
    /// Get library information
    pub fn new() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME"),
            version: VERSION,
            description: env!("CARGO_PKG_DESCRIPTION"),
        }
    }
}

impl Default for LibraryInfo {
    fn default() -> Self {
        Self::new()
    }
}
