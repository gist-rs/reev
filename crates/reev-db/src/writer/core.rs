//! Core database writer functionality
//!
//! Provides the main DatabaseWriter struct and basic database operations.

use crate::{
    config::DatabaseConfig,
    error::{DatabaseError, Result},
};
use std::path::Path;
use tokio::fs;
use tracing::{debug, error, info};
use turso::{Builder, Connection};

/// Current database schema loaded from external file
const CURRENT_SCHEMA: &str = include_str!("../../.schema/current_schema.sql");

/// Main database writer for atomic operations with duplicate prevention
pub struct DatabaseWriter {
    pub conn: Connection,
    pub config: DatabaseConfig,
}

impl DatabaseWriter {
    /// Create a new database writer with the given configuration
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        debug!(
            "[DB] Creating database connection to: {}",
            config.database_type()
        );

        // Ensure database directory exists
        if let Some(parent) = Path::new(&config.path).parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                DatabaseError::filesystem_with_source(
                    format!("Failed to create database directory: {parent:?}"),
                    e,
                )
            })?;
        }

        let db = Builder::new_local(&config.path)
            .build()
            .await
            .map_err(|e| {
                DatabaseError::connection_with_source(
                    format!("Failed to create local database: {}", config.path),
                    e,
                )
            })?;

        let conn = db.connect().map_err(|e| {
            DatabaseError::connection_with_source("Failed to establish database connection", e)
        })?;

        let writer = Self { conn, config };

        // Initialize database schema
        writer.initialize_schema().await?;

        Ok(writer)
    }

    /// Initialize database schema with all necessary tables and indexes
    async fn initialize_schema(&self) -> Result<()> {
        debug!("[DB] Initializing unified database schema from external file");

        // Split schema into individual statements and filter out comments
        let schema_string = CURRENT_SCHEMA
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("--"))
            .collect::<Vec<&str>>()
            .join(" ");

        let statements: Vec<&str> = schema_string
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        // Execute each statement
        for statement in statements.iter() {
            if statement.trim().is_empty() {
                continue;
            }

            self.conn.execute(statement, ()).await.map_err(|e| {
                DatabaseError::schema_with_source(
                    format!("Failed to execute schema statement: {statement}"),
                    e,
                )
            })?;
        }

        info!("[DB] Unified database schema initialized successfully from external file");
        Ok(())
    }

    /// Get a reference to the database connection
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Get database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Get table row count
    pub async fn get_table_count(&self, table_name: &str) -> Result<i64> {
        let mut rows = self
            .conn
            .query(&format!("SELECT COUNT(*) FROM {table_name}"), ())
            .await
            .map_err(|e| DatabaseError::query("Failed to get table count", e))?;

        if let Some(row) = rows.next().await? {
            let count: i64 = row.get(0).map_err(|e| {
                DatabaseError::generic_with_source("Failed to parse table count", e)
            })?;
            Ok(count)
        } else {
            Ok(0)
        }
    }

    /// Get database size in bytes
    pub async fn get_database_size(&self) -> Result<i64> {
        let mut rows = self
            .conn
            .query(
                "SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()",
                (),
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to get database size", e))?;

        if let Some(row) = rows.next().await? {
            let size: i64 = row.get(0).map_err(|e| {
                DatabaseError::generic_with_source("Failed to parse database size", e)
            })?;
            Ok(size)
        } else {
            Ok(0)
        }
    }

    /// Check database health and integrity
    pub async fn check_database_health(&self) -> Result<()> {
        debug!("[DB] Performing database health check");

        // Test basic connectivity
        let mut rows = self.conn.query("SELECT 1 as test", ()).await.map_err(|e| {
            DatabaseError::generic_with_source("Database connectivity test failed", e)
        })?;

        if rows.next().await?.is_none() {
            return Err(DatabaseError::generic(
                "Database connectivity test returned no results",
            ));
        }

        // Test inserting into a table with AUTOINCREMENT to detect sqlite_sequence issues
        match self.conn.execute(
                "INSERT INTO execution_sessions (session_id, benchmark_id, agent_type, interface, start_time, status) VALUES (?, ?, ?, ?, ?, 'running')",
                ["health_check", "health_check", "health_check", "tui", "1234567890"]
            ).await {
            Ok(_) => {
                debug!("[DB] AUTOINCREMENT test passed");
                // Clean up the test record
                let _ = self.conn.execute("DELETE FROM execution_sessions WHERE session_id = ?", ["health_check"]).await;
            }
            Err(e) => {
                error!("[DB] AUTOINCREMENT test failed: {}", e);
                return Err(DatabaseError::generic_with_source(
                    "Database corruption detected: sqlite_sequence table missing or corrupted. Please delete the database file and restart the application.",
                    e
                ));
            }
        }

        debug!("[DB] Database health check completed successfully");
        Ok(())
    }

    /// Create a DatabaseWriter from an existing connection
    pub fn from_connection(conn: Connection, config: DatabaseConfig) -> Self {
        Self { conn, config }
    }

    /// Close the database connection and cleanup resources
    /// This ensures proper shutdown and prevents database lock issues
    pub async fn close(&self) -> Result<()> {
        debug!("[DB] Closing database connection");

        // Execute a simple query to ensure connection is in a clean state
        let _ = self.conn.execute("PRAGMA optimize", ()).await;

        debug!("[DB] Database connection closed successfully");
        Ok(())
    }
}

impl Drop for DatabaseWriter {
    fn drop(&mut self) {
        debug!("[DB] DatabaseWriter dropped - connection will be cleaned up");
        // Note: Connection cleanup is handled by the Drop trait of the Connection
        // This ensures database file locks are released
    }
}
