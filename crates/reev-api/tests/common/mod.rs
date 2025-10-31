//! Common utilities for test database management
//!
//! This module provides utilities to centralize test database creation
//! in the /reev-api/db directory to avoid cluttering the project root.

use std::path::PathBuf;

/// Get the centralized test database directory path
pub fn test_db_dir() -> PathBuf {
    // Navigate from crates/reev-api/tests to reev/reev-api/db within project
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Remove tests
    path.pop(); // Remove reev-api
    path.pop(); // Remove crates
    path.push("reev");
    path.push("crates");
    path.push("reev-api");
    path.push("db");
    path
}

/// Generate a test database path with the given name
#[allow(dead_code)]
pub fn test_db_path(db_name: &str) -> String {
    let mut path = test_db_dir();
    path.push(format!("test_{db_name}.db"));
    path.to_string_lossy().to_string()
}

/// Generate a unique test database path with timestamp
#[allow(dead_code)]
pub fn unique_test_db_path(prefix: &str) -> String {
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let mut path = test_db_dir();
    path.push(format!("test_{prefix}_{timestamp}.db"));
    path.to_string_lossy().to_string()
}
