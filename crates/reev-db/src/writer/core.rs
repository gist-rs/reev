//! Core database writer functionality
//!
//! Provides the main DatabaseWriter struct and basic database operations.

use crate::{
    config::DatabaseConfig,
    error::{DatabaseError, Result},
    types::DatabaseStats,
};
use std::path::Path;
use tokio::fs;
use tracing::{error, info};
use turso::{Builder, Connection};

/// Main database writer for atomic operations with duplicate prevention
pub struct DatabaseWriter {
    pub conn: Connection,
    pub config: DatabaseConfig,
}

impl DatabaseWriter {
    /// Create a new database writer with the given configuration
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        info!(
            "[DB] Creating database connection to: {}",
            config.database_type()
        );

        // Ensure database directory exists
        if let Some(parent) = Path::new(&config.path).parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                DatabaseError::filesystem_with_source(
                    format!("Failed to create database directory: {:?}", parent),
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
        info!("[DB] Initializing unified database schema");

        // Create tables - simplified architecture
        let tables = [
            "CREATE TABLE IF NOT EXISTS benchmarks (
                id TEXT PRIMARY KEY,
                benchmark_name TEXT NOT NULL,
                prompt TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s', 'now')),
                updated_at INTEGER DEFAULT (strftime('%s', 'now'))
            )",
            "CREATE TABLE IF NOT EXISTS execution_sessions (
                session_id TEXT PRIMARY KEY,
                benchmark_id TEXT NOT NULL,
                agent_type TEXT NOT NULL,
                interface TEXT NOT NULL,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                status TEXT NOT NULL DEFAULT 'running',
                score REAL,
                final_status TEXT,
                log_file_path TEXT,
                created_at INTEGER DEFAULT (strftime('%s', 'now')),
                FOREIGN KEY (benchmark_id) REFERENCES benchmarks (id)
            )",
            "CREATE TABLE IF NOT EXISTS session_logs (
                session_id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                file_size INTEGER,
                created_at INTEGER DEFAULT (strftime('%s', 'now')),
                FOREIGN KEY (session_id) REFERENCES execution_sessions (session_id)
            )",
            "CREATE TABLE IF NOT EXISTS agent_performance (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                benchmark_id TEXT NOT NULL,
                agent_type TEXT NOT NULL,
                score REAL NOT NULL,
                final_status TEXT NOT NULL,
                execution_time_ms INTEGER,
                timestamp INTEGER NOT NULL,
                prompt_md5 TEXT,
                FOREIGN KEY (session_id) REFERENCES execution_sessions (session_id),
                FOREIGN KEY (benchmark_id) REFERENCES benchmarks (id)
            )",
        ];

        for table in tables.iter() {
            self.conn
                .execute(table, ())
                .await
                .map_err(|_e| DatabaseError::schema("Failed to create table"))?;
        }

        // Create indexes for unified schema
        let indexes = [
            "CREATE INDEX IF NOT EXISTS idx_benchmarks_name ON benchmarks(benchmark_name)",
            "CREATE INDEX IF NOT EXISTS idx_execution_sessions_benchmark_agent ON execution_sessions(benchmark_id, agent_type)",
            "CREATE INDEX IF NOT EXISTS idx_execution_sessions_interface ON execution_sessions(interface)",
            "CREATE INDEX IF NOT EXISTS idx_execution_sessions_status ON execution_sessions(status)",
            "CREATE INDEX IF NOT EXISTS idx_execution_sessions_start_time ON execution_sessions(start_time)",
            "CREATE INDEX IF NOT EXISTS idx_session_logs_created_at ON session_logs(created_at)",
            "CREATE INDEX IF NOT EXISTS idx_agent_performance_session_id ON agent_performance(session_id)",
            "CREATE INDEX IF NOT EXISTS idx_agent_performance_prompt_md5 ON agent_performance(prompt_md5)",
            "CREATE INDEX IF NOT EXISTS idx_agent_performance_score ON agent_performance(score)",
            "CREATE INDEX IF NOT EXISTS idx_agent_performance_timestamp ON agent_performance(timestamp)",
        ];

        for index in indexes.iter() {
            self.conn
                .execute(index, ())
                .await
                .map_err(|_e| DatabaseError::schema("Failed to create index"))?;
        }

        info!("[DB] Unified database schema initialized successfully");
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

    /// Get comprehensive database statistics
    pub async fn get_database_stats(&self) -> Result<DatabaseStats> {
        let total_benchmarks = self.get_table_count("benchmarks").await?;
        let total_sessions = self
            .get_table_count("execution_sessions")
            .await
            .unwrap_or(0);
        let total_session_logs = self.get_table_count("session_logs").await.unwrap_or(0);
        let total_performance_records =
            self.get_table_count("agent_performance").await.unwrap_or(0);

        // Get database size if available
        let database_size_bytes = self.get_database_size().await.ok().map(|size| size as u64);

        let stats = DatabaseStats {
            total_benchmarks,
            duplicate_count: 0, // TODO: Implement duplicate detection for new schema
            duplicate_details: Vec::new(),
            total_results: total_sessions,
            total_flow_logs: total_session_logs,
            total_performance_records,
            database_size_bytes,
            last_updated: chrono::Utc::now().to_rfc3339(),
        };

        Ok(stats)
    }

    /// Check database health and integrity
    pub async fn check_database_health(&self) -> Result<()> {
        info!("[DB] Performing database health check");

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
                "INSERT INTO execution_sessions (session_id, benchmark_id, agent_type, start_time, status) VALUES (?, ?, ?, ?, 'running')",
                ["health_check", "health_check", "health_check", "2025-01-01T00:00:00Z"]
            ).await {
            Ok(_) => {
                info!("[DB] AUTOINCREMENT test passed");
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

        info!("[DB] Database health check completed successfully");
        Ok(())
    }
}
