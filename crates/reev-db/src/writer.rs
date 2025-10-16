//! Database writer module for reev-db
//!
//! Provides robust database write operations with atomic upserts,
//! duplicate prevention, and comprehensive monitoring capabilities.

use crate::{
    config::DatabaseConfig,
    error::{DatabaseError, Result},
    shared::performance::AgentPerformance,
    types::{
        BenchmarkData, BenchmarkYml, DBFlowLog, DatabaseStats, DuplicateRecord, SyncError,
        SyncResult, SyncedBenchmark,
    },
    AgentPerformanceSummary,
};
use chrono::Utc;
use reev_flow::database::DBFlowLogConverter;
use std::error::Error;
use std::path::Path;
use tokio::fs;
use tracing::{error, info, warn};
use turso::{Builder, Connection};

/// Database writer for atomic operations with duplicate prevention
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

        info!("[DB] Database connection established");

        // Initialize database schema first
        Self::initialize_schema(&conn).await?;

        // Check database health for corruption issues
        Self::check_database_health(&conn).await?;

        Ok(Self { conn, config })
    }

    /// Initialize database schema with all necessary tables and indexes
    async fn initialize_schema(conn: &Connection) -> Result<()> {
        info!("[DB] Initializing database schema");

        // Create tables
        let tables = [
            "CREATE TABLE IF NOT EXISTS benchmarks (
                id TEXT PRIMARY KEY,
                benchmark_name TEXT NOT NULL,
                prompt TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            "CREATE TABLE IF NOT EXISTS results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                benchmark_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                prompt TEXT NOT NULL,
                generated_instruction TEXT NOT NULL,
                final_on_chain_state TEXT NOT NULL,
                final_status TEXT NOT NULL,
                score REAL NOT NULL,
                prompt_md5 TEXT,
                FOREIGN KEY (prompt_md5) REFERENCES benchmarks (id)
            )",
            "CREATE TABLE IF NOT EXISTS flow_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                benchmark_id TEXT NOT NULL,
                agent_type TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                final_result TEXT,
                flow_data TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            "CREATE TABLE IF NOT EXISTS agent_performance (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                benchmark_id TEXT NOT NULL,
                agent_type TEXT NOT NULL,
                score REAL NOT NULL,
                final_status TEXT NOT NULL,
                execution_time_ms INTEGER,
                timestamp TEXT NOT NULL,
                flow_log_id INTEGER,
                prompt_md5 TEXT,
                FOREIGN KEY (flow_log_id) REFERENCES flow_logs (id),
                FOREIGN KEY (prompt_md5) REFERENCES benchmarks (id)
            )",
            "CREATE TABLE IF NOT EXISTS yml_testresults (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                benchmark_id TEXT NOT NULL,
                agent_type TEXT NOT NULL,
                yml_content TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
        ];

        for table in tables.iter() {
            conn.execute(table, ())
                .await
                .map_err(|_e| DatabaseError::schema("Failed to create table"))?;
        }

        // Create indexes
        let indexes = [
            "CREATE INDEX IF NOT EXISTS idx_benchmarks_name ON benchmarks(benchmark_name)",
            "CREATE INDEX IF NOT EXISTS idx_results_prompt_md5 ON results(prompt_md5)",
            "CREATE INDEX IF NOT EXISTS idx_results_timestamp ON results(timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_flow_logs_benchmark_agent ON flow_logs(benchmark_id, agent_type)",
            "CREATE INDEX IF NOT EXISTS idx_agent_performance_prompt_md5 ON agent_performance(prompt_md5)",
            "CREATE INDEX IF NOT EXISTS idx_agent_performance_score ON agent_performance(score)",
            "CREATE INDEX IF NOT EXISTS idx_agent_performance_timestamp ON agent_performance(timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_yml_testresults_benchmark_agent ON yml_testresults(benchmark_id, agent_type)",
        ];

        for index in indexes.iter() {
            conn.execute(index, ())
                .await
                .map_err(|_e| DatabaseError::schema("Failed to create index"))?;
        }

        info!("[DB] Database schema initialized successfully");
        Ok(())
    }

    /// Upsert a benchmark with atomic conflict resolution
    pub async fn upsert_benchmark(
        &self,
        benchmark_name: &str,
        prompt: &str,
        content: &str,
    ) -> Result<String> {
        // Generate MD5 using the utility function for consistency
        let prompt_md5 =
            crate::shared::benchmark::BenchmarkUtils::generate_md5(benchmark_name, prompt);

        // Use fixed timestamp for consistent content
        let timestamp = Utc::now().to_rfc3339();

        info!(
            "[DB] Upserting benchmark '{}' with MD5 '{}'",
            benchmark_name, prompt_md5
        );

        // Atomic upsert with proper ON CONFLICT handling
        let query = "
            INSERT INTO benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                benchmark_name = excluded.benchmark_name,
                prompt = excluded.prompt,
                content = excluded.content,
                updated_at = excluded.updated_at;
        ";

        self.conn
            .execute(
                query,
                [
                    prompt_md5.clone(),
                    benchmark_name.to_string(),
                    prompt.to_string(),
                    content.to_string(),
                    timestamp.clone(),
                    timestamp.clone(),
                ],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to upsert benchmark with ON CONFLICT", e))?;

        info!(
            "[DB] Upserted benchmark '{}' with MD5 '{}' (prompt: {:.50}...)",
            benchmark_name, prompt_md5, prompt
        );

        Ok(prompt_md5)
    }

    /// Sync all benchmark files from a directory to the database
    pub async fn sync_benchmarks_from_dir<P: AsRef<Path>>(
        &self,
        benchmarks_dir: P,
    ) -> Result<SyncResult> {
        let start_time = std::time::Instant::now();
        let benchmarks_dir = benchmarks_dir.as_ref();

        info!(
            "[DB] Starting benchmark sync from directory: {}",
            benchmarks_dir.display()
        );

        // Check database state before sync
        let initial_count = self.get_all_benchmark_count().await.unwrap_or(0);
        info!(
            "[DB] Initial benchmark count in database: {}",
            initial_count
        );

        // Read all YAML files from benchmarks directory
        let mut yaml_files = Vec::new();
        let mut entries = fs::read_dir(benchmarks_dir)
            .await
            .map_err(|e| DatabaseError::filesystem(benchmarks_dir.display().to_string(), e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| DatabaseError::filesystem(benchmarks_dir.display().to_string(), e))?
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yml") {
                yaml_files.push(path);
            }
        }

        // Sort files for consistent processing order
        yaml_files.sort();
        info!("[DB] Found {} benchmark files to process", yaml_files.len());

        // Process benchmarks sequentially (avoid race conditions)
        let mut sync_result = SyncResult {
            processed_count: 0,
            new_count: 0,
            updated_count: 0,
            error_count: 0,
            duration_ms: 0,
            processed_benchmarks: Vec::new(),
            errors: Vec::new(),
        };

        // Get initial benchmark mapping for comparison
        let initial_benchmarks = self
            .get_all_benchmarks()
            .await
            .map(|benchmarks| {
                benchmarks
                    .into_iter()
                    .map(|b| (b.benchmark_name.clone(), b.id.clone()))
                    .collect::<std::collections::HashMap<String, String>>()
            })
            .unwrap_or_default();

        for (index, path) in yaml_files.iter().enumerate() {
            match self.sync_single_benchmark(path).await {
                Ok((benchmark_name, md5, operation, processing_time)) => {
                    sync_result.processed_count += 1;
                    info!(
                        "[DB] Synced benchmark '{}' with MD5 '{}' ({})",
                        benchmark_name, md5, operation
                    );

                    // Determine if this is new or updated
                    if let Some(existing_md5) = initial_benchmarks.get(&benchmark_name) {
                        if existing_md5 != &md5 {
                            sync_result.updated_count += 1;
                            info!(
                                "[DB] Updated benchmark '{}' MD5: {} -> {}",
                                benchmark_name, existing_md5, md5
                            );
                        } else {
                            info!("[DB] Benchmark '{}' MD5 unchanged: {}", benchmark_name, md5);
                        }
                    } else {
                        sync_result.new_count += 1;
                        info!("[DB] New benchmark '{}' with MD5: {}", benchmark_name, md5);
                    }

                    sync_result.processed_benchmarks.push(SyncedBenchmark {
                        name: benchmark_name.clone(),
                        md5: md5.clone(),
                        operation: operation.clone(),
                        processing_time_ms: processing_time,
                    });

                    info!(
                        "[DB] [{}/{}] Synced benchmark: {:?} -> {} ({})",
                        index + 1,
                        yaml_files.len(),
                        path.file_name(),
                        md5,
                        operation
                    );
                }
                Err(e) => {
                    sync_result.error_count += 1;
                    let error_msg = format!("Failed to sync benchmark: {e}");
                    error!("[DB] [{}/{}] {}", index + 1, yaml_files.len(), error_msg);

                    sync_result.errors.push(SyncError {
                        file_path: path.display().to_string(),
                        error_message: error_msg.clone(),
                        error_type: "sync_error".to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                    });
                }
            }
        }

        sync_result.duration_ms = start_time.elapsed().as_millis() as u64;

        // Check database state after sync
        let final_count = self.get_all_benchmark_count().await.unwrap_or(0);
        info!("[DB] Final benchmark count in database: {}", final_count);

        // Log summary
        if final_count > initial_count {
            info!(
                "[DB] Sync completed: {} new benchmarks added ({} -> {})",
                final_count - initial_count,
                initial_count,
                final_count
            );
        } else if final_count == initial_count {
            info!(
                "[DB] Sync completed: {} benchmarks updated (no new records)",
                sync_result.processed_count
            );
        } else {
            warn!(
                "[DB] Sync completed with unexpected count change: {} -> {} (some records may have been removed)",
                initial_count,
                final_count
            );
        }

        // Check for potential duplicates
        if final_count != initial_count
            && (final_count - initial_count) != sync_result.new_count as i64
        {
            warn!(
                "[DB] Potential duplicate detection: Expected {} new records, but count changed by {}",
                sync_result.new_count,
                final_count - initial_count
            );
        }

        info!(
            "[DB] Sync completed in {}ms: {} processed, {} new, {} updated, {} errors",
            sync_result.duration_ms,
            sync_result.processed_count,
            sync_result.new_count,
            sync_result.updated_count,
            sync_result.error_count
        );

        Ok(sync_result)
    }

    /// Sync a single benchmark file to the database
    async fn sync_single_benchmark(&self, path: &Path) -> Result<(String, String, String, u64)> {
        let start_time = std::time::Instant::now();

        let content = fs::read_to_string(path)
            .await
            .map_err(|e| DatabaseError::filesystem(path.display().to_string(), e))?;

        // Parse YAML to extract benchmark data
        let benchmark_data: BenchmarkYml = serde_yaml::from_str(&content)
            .map_err(|e| DatabaseError::yaml("Failed to parse benchmark YAML", e))?;

        // Check if benchmark already exists
        let existing_benchmark = self.get_benchmark_by_name(&benchmark_data.id).await?;

        // Upsert to database
        let prompt_md5 = self
            .upsert_benchmark(&benchmark_data.id, &benchmark_data.prompt, &content)
            .await?;

        let operation = if existing_benchmark.is_some() {
            "updated"
        } else {
            "created"
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        info!(
            "[DB] Synced benchmark '{}' with prompt MD5: {} ({})",
            benchmark_data.id, prompt_md5, operation
        );

        Ok((
            benchmark_data.id,
            prompt_md5,
            operation.to_string(),
            processing_time,
        ))
    }

    /// Get total count of benchmarks in database
    pub async fn get_all_benchmark_count(&self) -> Result<i64> {
        let mut rows = self
            .conn
            .query("SELECT COUNT(*) FROM benchmarks", ())
            .await
            .map_err(|e| DatabaseError::query("Failed to count benchmarks", e))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::query("Failed to get count result", e))?
        {
            let count: i64 = row.get(0)?;
            Ok(count)
        } else {
            Ok(0)
        }
    }

    /// Get a benchmark by its name
    pub async fn get_benchmark_by_name(&self, name: &str) -> Result<Option<BenchmarkData>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, benchmark_name, prompt, content, created_at, updated_at
                 FROM benchmarks WHERE benchmark_name = ?",
                [name],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to query benchmark by name", e))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::query("Failed to get benchmark result", e))?
        {
            let benchmark = BenchmarkData {
                id: row.get(0)?,
                benchmark_name: row.get(1)?,
                prompt: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            };
            Ok(Some(benchmark))
        } else {
            Ok(None)
        }
    }

    /// Get a benchmark by its ID (MD5)
    pub async fn get_benchmark_by_id(&self, id: &str) -> Result<Option<BenchmarkData>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, benchmark_name, prompt, content, created_at, updated_at
                 FROM benchmarks WHERE id = ?",
                [id],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to query benchmark by ID", e))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::query("Failed to get benchmark result", e))?
        {
            let benchmark = BenchmarkData {
                id: row.get(0)?,
                benchmark_name: row.get(1)?,
                prompt: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            };
            Ok(Some(benchmark))
        } else {
            Ok(None)
        }
    }

    /// Get all benchmarks from database
    pub async fn get_all_benchmarks(&self) -> Result<Vec<BenchmarkData>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, benchmark_name, prompt, content, created_at, updated_at
                 FROM benchmarks ORDER BY created_at DESC",
                (),
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to query all benchmarks", e))?;

        let mut benchmarks = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::query("Failed to iterate benchmarks", e))?
        {
            benchmarks.push(BenchmarkData {
                id: row.get(0)?,
                benchmark_name: row.get(1)?,
                prompt: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            });
        }

        Ok(benchmarks)
    }

    /// Check for duplicate benchmark records
    pub async fn check_for_duplicates(&self) -> Result<Vec<DuplicateRecord>> {
        let query = "
            SELECT id, benchmark_name, COUNT(*) as count,
                   MIN(created_at) as first_created_at,
                   MAX(updated_at) as last_updated_at
            FROM benchmarks
            GROUP BY id, benchmark_name
            HAVING COUNT(*) > 1
            ORDER BY id
        ";

        let mut rows = self
            .conn
            .query(query, ())
            .await
            .map_err(|e| DatabaseError::query("Failed to check for duplicates", e))?;

        let mut duplicates = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::query("Failed to iterate duplicate results", e))?
        {
            let duplicate = DuplicateRecord {
                id: row.get(0)?,
                benchmark_name: row.get(1)?,
                count: row.get(2)?,
                first_created_at: row.get(3)?,
                last_updated_at: row.get(4)?,
            };
            duplicates.push(duplicate);
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

    /// Get comprehensive database statistics
    pub async fn get_database_stats(&self) -> Result<DatabaseStats> {
        let total_benchmarks = self.get_all_benchmark_count().await?;
        let duplicates = self.check_for_duplicates().await?;

        // Get table counts
        let total_results = self.get_table_count("results").await.unwrap_or(0);
        let total_flow_logs = self.get_table_count("flow_logs").await.unwrap_or(0);
        let total_performance_records =
            self.get_table_count("agent_performance").await.unwrap_or(0);

        // Get database size if available
        let database_size_bytes = self.get_database_size().await.ok();

        let stats = DatabaseStats {
            total_benchmarks,
            duplicate_count: duplicates.len() as i64,
            duplicate_details: duplicates
                .into_iter()
                .map(|d| (d.id.clone(), d.benchmark_name.clone(), d.count))
                .collect(),
            total_results,
            total_flow_logs,
            total_performance_records,
            database_size_bytes,
            last_updated: Utc::now().to_rfc3339(),
        };

        Ok(stats)
    }

    /// Get count of records in a specific table
    async fn get_table_count(&self, table_name: &str) -> Result<i64> {
        let mut rows = self
            .conn
            .query(&format!("SELECT COUNT(*) FROM {table_name}"), ())
            .await
            .map_err(|e| {
                DatabaseError::query(format!("Failed to count records in table {table_name}"), e)
            })?;

        if let Some(row) = rows.next().await.map_err(|e| {
            DatabaseError::query(
                format!("Failed to get count result from table {table_name}"),
                e,
            )
        })? {
            let count: i64 = row.get(0)?;
            Ok(count)
        } else {
            Ok(0)
        }
    }

    /// Get database size in bytes
    async fn get_database_size(&self) -> Result<i64> {
        let mut rows = self
            .conn
            .query(
                "SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()",
                (),
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to get database size", e))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::query("Failed to get size result", e))?
        {
            let size: i64 = row.get(0)?;
            Ok(size)
        } else {
            Err(DatabaseError::generic("Could not determine database size"))
        }
    }

    /// Clean up duplicate records (keep the most recent one)
    pub async fn cleanup_duplicates(&self) -> Result<usize> {
        let duplicates = self.check_for_duplicates().await?;
        let mut cleaned_count = 0;

        for duplicate in duplicates {
            info!(
                "[DB] Cleaning up duplicates for ID '{}': {} records",
                duplicate.id, duplicate.count
            );

            // Keep the most recent record, delete older ones
            let delete_query = "
                DELETE FROM benchmarks
                WHERE id = ? AND rowid NOT IN (
                    SELECT rowid FROM benchmarks
                    WHERE id = ?
                    ORDER BY updated_at DESC
                    LIMIT 1
                )
            ";

            let result = self
                .conn
                .execute(delete_query, [duplicate.id.clone(), duplicate.id.clone()])
                .await
                .map_err(|e| DatabaseError::query("Failed to cleanup duplicates", e))?;

            cleaned_count += result as usize;
        }

        info!("[DB] Cleaned up {} duplicate records", cleaned_count);
        Ok(cleaned_count)
    }

    /// Insert agent performance data into the database
    pub async fn insert_agent_performance(&self, data: &AgentPerformance) -> Result<()> {
        let query = "
            INSERT INTO agent_performance (
                benchmark_id, agent_type, score, final_status,
                execution_time_ms, timestamp, flow_log_id, prompt_md5
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ";

        self.conn
            .execute(
                query,
                [
                    data.benchmark_id.clone(),
                    data.agent_type.clone(),
                    data.score.to_string(),
                    data.final_status.clone(),
                    data.execution_time_ms.unwrap_or(0).to_string(),
                    data.timestamp.clone(),
                    data.flow_log_id.unwrap_or(0).to_string(),
                    data.prompt_md5.clone().unwrap_or_default(),
                ],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to insert agent performance", e))?;

        Ok(())
    }

    /// Insert flow log data into the database
    pub async fn insert_flow_log(&self, data: &DBFlowLog) -> Result<i64> {
        let query = "
            INSERT INTO flow_logs (
                session_id, benchmark_id, agent_type, start_time, end_time,
                final_result, flow_data, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ";

        let _result = self
            .conn
            .execute(
                query,
                [
                    data.session_id().to_string(),
                    data.benchmark_id().to_string(),
                    data.agent_type().to_string(),
                    data.start_time().unwrap_or_default(),
                    data.end_time().unwrap_or_default().unwrap_or_default(),
                    data.final_result_json()
                        .unwrap_or_default()
                        .unwrap_or_default(),
                    data.events_json().unwrap_or_default(),
                    data.created_at.clone().unwrap_or_default(),
                ],
            )
            .await
            .map_err(|e| {
                let error_msg = format!(
                    "Failed to insert flow log. SQLite error: {}. Source: {:?}",
                    e,
                    e.source()
                );
                DatabaseError::generic_with_source(error_msg, e)
            })?;

        // Get the ID of the inserted row
        let mut rows = self.conn.query("SELECT last_insert_rowid()", ()).await?;
        if let Some(row) = rows.next().await? {
            let id: i64 = row
                .get(0)
                .map_err(|_| DatabaseError::generic("Failed to get flow log ID"))?;
            Ok(id)
        } else {
            Err(DatabaseError::generic("Failed to get flow log ID"))
        }
    }

    /// Insert test result data into the database
    pub async fn insert_result(
        &self,
        benchmark_id: &str,
        prompt: &str,
        generated_instruction: &str,
        final_on_chain_state: &str,
        final_status: &str,
        score: f64,
    ) -> Result<()> {
        let timestamp = Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();

        let query = "
            INSERT INTO results (
                id, benchmark_id, timestamp, prompt, generated_instruction,
                final_on_chain_state, final_status, score
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ";

        self.conn
            .execute(
                query,
                [
                    id,
                    benchmark_id.to_string(),
                    timestamp,
                    prompt.to_string(),
                    generated_instruction.to_string(),
                    final_on_chain_state.to_string(),
                    final_status.to_string(),
                    score.to_string(),
                ],
            )
            .await
            .map_err(|e| DatabaseError::generic_with_source("Failed to insert test result", e))?;

        Ok(())
    }

    /// Get prompt MD5 by benchmark name
    pub async fn get_prompt_md5_by_benchmark_name(
        &self,
        benchmark_name: &str,
    ) -> Result<Option<String>> {
        info!(
            "[DB] Looking up prompt MD5 for benchmark_name: '{}'",
            benchmark_name
        );

        let query = "
            SELECT id
            FROM benchmarks
            WHERE benchmark_name = ?
            ORDER BY created_at DESC
            LIMIT 1
        ";
        let mut rows = self
            .conn
            .query(query, [benchmark_name])
            .await
            .map_err(|e| {
                error!("[DB] Query failed: {} - Full error: {:?}", e, e.source());
                DatabaseError::query("Failed to get prompt MD5 by benchmark name", e)
            })?;

        if let Some(row) = rows.next().await? {
            let prompt_md5: Option<String> = row
                .get(0)
                .map_err(|e| DatabaseError::generic_with_source("Failed to parse prompt MD5", e))?;

            Ok(prompt_md5)
        } else {
            Ok(None)
        }
    }

    /// Get agent performance summaries
    pub async fn get_agent_performance(&self) -> Result<Vec<AgentPerformanceSummary>> {
        let query = "
            SELECT
                agent_type,
                COUNT(*) as execution_count,
                AVG(score) as avg_score,
                MAX(timestamp) as latest_timestamp
            FROM agent_performance
            GROUP BY agent_type
            ORDER BY agent_type
        ";

        let mut rows =
            self.conn.query(query, ()).await.map_err(|e| {
                DatabaseError::query("Failed to get agent performance summaries", e)
            })?;

        let mut summaries = Vec::new();
        while let Some(row) = rows.next().await? {
            let agent_type: String = row.get(0)?;
            let execution_count: i64 = row.get(1)?;
            let avg_score: f64 = row.get(2)?;
            let latest_timestamp: String = row.get(3)?;

            summaries.push(AgentPerformanceSummary {
                agent_type: agent_type.clone(),
                execution_count,
                avg_score,
                latest_timestamp,
                results: Vec::new(), // TODO: Populate with actual results if needed
            });
        }

        Ok(summaries)
    }

    /// Get YML flow logs for a benchmark
    pub async fn get_yml_flow_logs(&self, benchmark_id: &str) -> Result<Option<String>> {
        let query = "
            SELECT flow_data
            FROM flow_logs
            WHERE benchmark_id = ?
            ORDER BY created_at DESC
            LIMIT 1
        ";

        let mut rows = self
            .conn
            .query(query, [benchmark_id])
            .await
            .map_err(|e| DatabaseError::query("Failed to get YML flow logs", e))?;

        if let Some(row) = rows.next().await? {
            let flow_data: String = row.get(0)?;
            Ok(Some(flow_data))
        } else {
            Ok(None)
        }
    }

    /// Get YML test result for a benchmark and agent
    pub async fn get_yml_testresult(
        &self,
        benchmark_id: &str,
        agent_type: &str,
    ) -> Result<Option<String>> {
        let query = "
            SELECT generated_instruction, final_on_chain_state, final_status, score
            FROM results
            WHERE benchmark_id = ? AND prompt LIKE ?
            ORDER BY timestamp DESC
            LIMIT 1
        ";

        let mut rows = self
            .conn
            .query(query, (benchmark_id, format!("%{agent_type}%")))
            .await
            .map_err(|e| DatabaseError::query("Failed to get YML test result", e))?;

        if let Some(row) = rows.next().await? {
            let instruction: String = row.get(0)?;
            let state: String = row.get(1)?;
            let status: String = row.get(2)?;
            let score: f64 = row.get(3)?;

            let yml_content = format!(
                "generated_instruction: {instruction}\nfinal_on_chain_state: {state}\nfinal_status: {status}\nscore: {score}"
            );
            Ok(Some(yml_content))
        } else {
            Ok(None)
        }
    }

    /// Check database health for corruption issues
    async fn check_database_health(conn: &Connection) -> Result<()> {
        info!("[DB] Checking database health...");

        // Test inserting into a table with AUTOINCREMENT to detect sqlite_sequence issues
        match conn.execute(
            "INSERT INTO flow_logs (session_id, benchmark_id, agent_type, start_time, flow_data) VALUES (?, ?, ?, ?, ?)",
            ["health_check", "health_check", "health_check", "2025-01-01T00:00:00Z", "[]"]
        ).await {
            Ok(_) => {
                info!("[DB] AUTOINCREMENT test passed");
                // Clean up the test record
                let _ = conn.execute("DELETE FROM flow_logs WHERE session_id = ?", ["health_check"]).await;
            }
            Err(e) => {
                error!("[DB] AUTOINCREMENT test failed: {}", e);
                error!("[DB] Database has sqlite_sequence corruption");
                error!("[DB] SOLUTION: Remove the database file and let it recreate");
                error!("[DB] Command: rm db/reev_results.db");

                return Err(DatabaseError::generic_with_source(
                    "Database corruption detected: sqlite_sequence table missing or corrupted. \
                    Please delete the database file and restart the application.",
                    e
                ));
            }
        }

        Ok(())
    }

    /// Test prompt MD5 lookup
    pub async fn test_prompt_md5_lookup(&self, benchmark_name: &str) -> Result<Option<String>> {
        self.get_prompt_md5_by_benchmark_name(benchmark_name).await
    }

    /// Insert YML flow log
    pub async fn insert_yml_flow_log(&self, benchmark_id: &str, _yml_content: &str) -> Result<i64> {
        let flow_log = reev_flow::database::FlowLogDB::create(
            format!("yml-import-{}", uuid::Uuid::new_v4()),
            benchmark_id.to_string(),
            "yml-import".to_string(),
        );

        let storage_format = flow_log
            .to_db_storage()
            .map_err(|e| DatabaseError::generic_with_source("Failed to convert flow log", e))?;

        self.insert_flow_log_storage(&storage_format).await
    }

    /// Insert flow log from storage format
    async fn insert_flow_log_storage(
        &self,
        data: &reev_flow::database::DBStorageFormat,
    ) -> Result<i64> {
        let query = "
            INSERT INTO flow_logs (
                session_id, benchmark_id, agent_type, start_time, end_time,
                final_result, flow_data, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ";

        let _result = self
            .conn
            .execute(
                query,
                [
                    data.session_id.clone(),
                    data.benchmark_id.clone(),
                    data.agent_type.clone(),
                    data.start_time.clone(),
                    data.end_time.clone().unwrap_or_default(),
                    data.final_result.clone().unwrap_or_default(),
                    data.flow_data.clone(),
                    data.created_at.clone().unwrap_or_default(),
                ],
            )
            .await
            .map_err(|e| DatabaseError::generic_with_source("Failed to insert flow log", e))?;

        // Get the ID of the inserted row
        let mut rows = self.conn.query("SELECT last_insert_rowid()", ()).await?;
        if let Some(row) = rows.next().await? {
            let id: i64 = row.get(0).map_err(|e| {
                DatabaseError::generic_with_source("Failed to get inserted flow log ID", e)
            })?;
            Ok(id)
        } else {
            Err(DatabaseError::generic(
                "Failed to retrieve inserted flow log ID",
            ))
        }
    }

    /// Insert YML test result
    pub async fn insert_yml_testresult(
        &self,
        benchmark_id: &str,
        agent_type: &str,
        yml_content: &str,
    ) -> Result<()> {
        self.insert_result(
            benchmark_id,
            &format!("YML import for {agent_type}"),
            yml_content,
            "YML imported",
            "Completed",
            1.0,
        )
        .await
    }

    /// Get the underlying connection (for advanced operations)
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}
