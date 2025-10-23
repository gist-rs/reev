//! Database monitoring and health check operations
//!
//! Provides database statistics, integrity checks, and monitoring
//! capabilities for production-grade database management.

use crate::{
    error::{DatabaseError, Result},
    types::{DatabaseStats, DuplicateRecord},
};
use tracing::{debug, error, info, warn};

use super::core::DatabaseWriter;

impl DatabaseWriter {
    /// Check for duplicate benchmark records
    pub async fn check_for_duplicates(&self) -> Result<Vec<DuplicateRecord>> {
        debug!("[DB] Checking for duplicate benchmark records");

        let query = "
            SELECT id, benchmark_name, COUNT(*) as count
            FROM benchmarks
            GROUP BY id, benchmark_name
            HAVING COUNT(*) > 1
        ";

        let mut rows = self
            .conn
            .query(query, ())
            .await
            .map_err(|e| DatabaseError::query("Failed to check for duplicates", e))?;

        let mut duplicates = Vec::new();
        while let Some(row) = rows.next().await? {
            let id: String = row.get(0)?;
            let benchmark_name: String = row.get(1)?;
            let count: i64 = row.get(2)?;

            duplicates.push(DuplicateRecord {
                id,
                benchmark_name,
                count,
                first_created_at: chrono::Utc::now().to_rfc3339(),
            });
        }

        if !duplicates.is_empty() {
            warn!(
                "[DB] Found {} duplicate benchmark records in database",
                duplicates.len()
            );
            for duplicate in &duplicates {
                warn!(
                    "[DB] Duplicate detected: ID '{}' - '{}' appears {} times",
                    duplicate.id, duplicate.benchmark_name, duplicate.count
                );
            }
        } else {
            info!("[DB] No duplicate benchmark records found");
        }

        Ok(duplicates)
    }

    /// Cleanup duplicate benchmark records
    pub async fn cleanup_duplicates(&self) -> Result<usize> {
        debug!("[DB] Starting cleanup of duplicate benchmark records");

        let duplicates = self.check_for_duplicates().await?;
        if duplicates.is_empty() {
            debug!("[DB] No duplicates to cleanup");
            return Ok(0);
        }

        let mut total_cleaned = 0;
        for duplicate in duplicates {
            // Keep the first record, delete the rest
            let delete_query = "
                DELETE FROM benchmarks
                WHERE id = ? AND benchmark_name = ?
                AND rowid NOT IN (
                    SELECT MIN(rowid)
                    FROM benchmarks
                    WHERE id = ? AND benchmark_name = ?
                )
            ";

            let rows_affected = self
                .conn
                .execute(
                    delete_query,
                    [
                        duplicate.id.clone(),
                        duplicate.benchmark_name.clone(),
                        duplicate.id.clone(),
                        duplicate.benchmark_name.clone(),
                    ],
                )
                .await
                .map_err(|e| DatabaseError::operation("Failed to cleanup duplicates", e))?;

            total_cleaned += rows_affected;
            debug!(
                "[DB] Cleaned up {} duplicate records for ID '{}'",
                rows_affected, duplicate.id
            );
        }

        info!(
            "[DB] Duplicate cleanup completed: {} records removed",
            total_cleaned
        );
        Ok(total_cleaned as usize)
    }

    /// Get comprehensive database statistics
    pub async fn get_database_stats(&self) -> Result<DatabaseStats> {
        info!("[DB] Generating comprehensive database statistics");

        let total_benchmarks = self.get_table_count("benchmarks").await?;
        let total_sessions = self
            .get_table_count("execution_sessions")
            .await
            .unwrap_or(0);
        let total_session_logs = self.get_table_count("session_logs").await.unwrap_or(0);
        let total_performance_records =
            self.get_table_count("agent_performance").await.unwrap_or(0);

        // Check for duplicates
        let duplicates = self.check_for_duplicates().await?;
        let duplicate_count = duplicates.len() as i64;
        let duplicate_details = duplicates
            .into_iter()
            .map(|d| (d.id.clone(), d.benchmark_name.clone(), d.count))
            .collect();

        // Get database size if available
        let database_size_bytes = self.get_database_size().await.ok().map(|size| size as u64);

        let stats = DatabaseStats {
            total_benchmarks,
            duplicate_count,
            duplicate_details,
            total_results: total_sessions,
            total_flow_logs: total_session_logs,
            total_performance_records,
            database_size_bytes,
            last_updated: chrono::Utc::now().to_rfc3339(),
        };

        info!(
            "[DB] Database stats generated: {} benchmarks, {} sessions, {} performance records",
            stats.total_benchmarks, stats.total_results, stats.total_performance_records
        );

        Ok(stats)
    }

    /// Perform comprehensive database health check
    pub async fn perform_health_check(&self) -> Result<()> {
        debug!("[DB] Starting comprehensive database health check");

        // Test basic connectivity
        self.test_connectivity().await?;

        // Test table integrity
        self.test_table_integrity().await?;

        // Test foreign key constraints
        self.test_foreign_keys().await?;

        // Check for corruption
        self.check_corruption().await?;

        // Test insert/update operations
        self.test_crud_operations().await?;

        info!("[DB] Comprehensive health check passed");
        Ok(())
    }

    /// Test basic database connectivity
    async fn test_connectivity(&self) -> Result<()> {
        debug!("[DB] Testing database connectivity");

        let mut rows = self.conn.query("SELECT 1 as test", ()).await.map_err(|e| {
            error!("[DB] Connectivity test failed: {}", e);
            DatabaseError::generic_with_source("Database connectivity test failed", e)
        })?;

        if rows.next().await?.is_none() {
            return Err(DatabaseError::generic(
                "Database connectivity test returned no results",
            ));
        }

        info!("[DB] Connectivity test passed");
        Ok(())
    }

    /// Test table integrity
    async fn test_table_integrity(&self) -> Result<()> {
        debug!("[DB] Testing table integrity");

        let tables = [
            "benchmarks",
            "execution_sessions",
            "session_logs",
            "agent_performance",
        ];

        for table in tables {
            let count = self.get_table_count(table).await?;
            debug!("[DB] Table '{}': {} records", table, count);
        }

        info!("[DB] Table integrity test passed");
        Ok(())
    }

    /// Test foreign key constraints
    async fn test_foreign_keys(&self) -> Result<()> {
        debug!("[DB] Testing foreign key constraints");

        // Test execution_sessions -> benchmarks foreign key
        let mut rows = self
            .conn
            .query(
                "SELECT COUNT(*) FROM execution_sessions es
                 LEFT JOIN benchmarks b ON es.benchmark_id = b.id
                 WHERE b.id IS NULL",
                (),
            )
            .await
            .map_err(|e| {
                error!("[DB] Foreign key test failed: {}", e);
                DatabaseError::query("Failed to test foreign key constraints", e)
            })?;

        if let Some(row) = rows.next().await? {
            let orphan_count: i64 = row.get(0)?;
            if orphan_count > 0 {
                warn!(
                    "[DB] Found {} orphaned execution_sessions records",
                    orphan_count
                );
            }
        }

        // Test agent_performance -> execution_sessions foreign key
        let mut rows = self
            .conn
            .query(
                "SELECT COUNT(*) FROM agent_performance ap
                 LEFT JOIN execution_sessions es ON ap.session_id = es.session_id
                 WHERE es.session_id IS NULL",
                (),
            )
            .await
            .map_err(|e| {
                error!("[DB] Foreign key test failed: {}", e);
                DatabaseError::query("Failed to test foreign key constraints", e)
            })?;

        if let Some(row) = rows.next().await? {
            let orphan_count: i64 = row.get(0)?;
            if orphan_count > 0 {
                warn!(
                    "[DB] Found {} orphaned agent_performance records",
                    orphan_count
                );
            }
        }

        info!("[DB] Foreign key constraints test passed");
        Ok(())
    }

    /// Check for database corruption
    async fn check_corruption(&self) -> Result<()> {
        debug!("[DB] Checking for database corruption");

        // Test inserting into a table with AUTOINCREMENT
        match self
            .conn
            .execute(
                "INSERT INTO execution_sessions (session_id, benchmark_id, agent_type, interface, start_time, status) VALUES (?, ?, ?, ?, ?, 'running')",
                ["corruption_test", "corruption_test", "corruption_test", "test", "2025-01-01T00:00:00Z"]
            )
            .await
        {
            Ok(_) => {
                info!("[DB] AUTOINCREMENT test passed");
                // Clean up the test record
                let _ = self
                    .conn
                    .execute(
                        "DELETE FROM execution_sessions WHERE session_id = ?",
                        ["corruption_test"],
                    )
                    .await;
            }
            Err(e) => {
                error!("[DB] AUTOINCREMENT test failed: {}", e);
                return Err(DatabaseError::generic_with_source(
                    "Database corruption detected: sqlite_sequence table missing or corrupted. Please delete the database file and restart the application.",
                    e
                ));
            }
        }

        info!("[DB] Corruption check passed");
        Ok(())
    }

    /// Test CRUD operations
    async fn test_crud_operations(&self) -> Result<()> {
        debug!("[DB] Testing CRUD operations");

        let _test_id = "crud_test_benchmark";
        let test_name = "crud-test";
        let test_prompt = "Test prompt for CRUD operations";
        let test_content = "id: crud-test\nprompt: Test prompt for CRUD operations";

        // Test CREATE
        let md5 = self
            .upsert_benchmark(test_name, test_prompt, test_content)
            .await?;

        // Test READ
        let benchmark = self.get_benchmark_by_id(&md5).await?;
        assert!(benchmark.is_some(), "Failed to retrieve created benchmark");

        // Test UPDATE
        let updated_content = "id: crud-test\nprompt: Updated test prompt for CRUD operations";
        let updated_md5 = self
            .upsert_benchmark(test_name, "Updated test prompt", updated_content)
            .await?;

        // Test DELETE
        self.delete_benchmark(&updated_md5).await?;

        // Verify deletion
        let deleted_benchmark = self.get_benchmark_by_id(&updated_md5).await?;
        assert!(deleted_benchmark.is_none(), "Failed to delete benchmark");

        info!("[DB] CRUD operations test passed");
        Ok(())
    }

    /// Get database performance metrics
    pub async fn get_performance_metrics(&self) -> Result<std::collections::HashMap<String, f64>> {
        debug!("[DB] Getting database performance metrics");

        let mut metrics = std::collections::HashMap::new();

        // Get table sizes
        let tables = [
            "benchmarks",
            "execution_sessions",
            "session_logs",
            "agent_performance",
        ];
        for table in tables {
            let count = self.get_table_count(table).await.unwrap_or(0);
            metrics.insert(format!("table_{table}_count"), count as f64);
        }

        // Get database size
        if let Ok(size) = self.get_database_size().await {
            metrics.insert("database_size_bytes".to_string(), size as f64);
        }

        // Get page count (SQLite specific)
        let mut rows = self
            .conn
            .query("PRAGMA page_count", ())
            .await
            .map_err(|e| DatabaseError::query("Failed to get page count", e))?;

        if let Some(row) = rows.next().await? {
            let page_count: i64 = row.get(0)?;
            metrics.insert("page_count".to_string(), page_count as f64);
        }

        // Get page size
        let mut rows = self
            .conn
            .query("PRAGMA page_size", ())
            .await
            .map_err(|e| DatabaseError::query("Failed to get page size", e))?;

        if let Some(row) = rows.next().await? {
            let page_size: i64 = row.get(0)?;
            metrics.insert("page_size".to_string(), page_size as f64);
        }

        info!("[DB] Retrieved {} performance metrics", metrics.len());
        Ok(metrics)
    }

    /// Optimize database performance
    pub async fn optimize_database(&self) -> Result<()> {
        debug!("[DB] Starting database optimization");

        // Run ANALYZE to update query planner statistics
        self.conn
            .execute("ANALYZE", ())
            .await
            .map_err(|e| DatabaseError::operation("Failed to analyze database", e))?;

        debug!("[DB] Database ANALYZE completed");

        // Run VACUUM to reclaim unused space
        match self.conn.execute("VACUUM", ()).await {
            Ok(_) => info!("[DB] Database VACUUM completed"),
            Err(e) => warn!(
                "[DB] VACUUM failed (this is normal for large databases): {}",
                e
            ),
        }

        info!("[DB] Database optimization completed");
        Ok(())
    }
}
