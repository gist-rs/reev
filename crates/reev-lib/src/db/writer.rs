//! Shared database writer for unified database operations
//!
//! This module provides a single source of truth for all database write operations
//! used by both web and TUI interfaces.

use super::types::*;
use crate::agent::AgentObservation;
use crate::flow::types::FlowLog;
use crate::results::FinalStatus;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use turso::{Builder, Connection};

/// Shared database writer for all database operations
pub struct DatabaseWriter {
    pub conn: Connection,
}

/// Database statistics for monitoring
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub total_benchmarks: i64,
    pub duplicate_count: i64,
    pub duplicate_details: Vec<(String, String, i64)>,
}

impl DatabaseWriter {
    /// Creates a new database writer with the given configuration
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let db = Builder::new_local(&config.path)
            .build()
            .await
            .context("Failed to build local database")?;
        let conn = db.connect().context("Failed to connect to database")?;

        info!("[DB] Connected to database at: {}", config.path);

        // Initialize database schema
        Self::initialize_schema(&conn).await?;

        Ok(Self { conn })
    }

    /// Initialize database schema with all required tables
    async fn initialize_schema(conn: &Connection) -> Result<()> {
        // Create tables one by one for better error handling
        let tables = [
            "CREATE TABLE IF NOT EXISTS benchmarks (
                id TEXT PRIMARY KEY,  -- MD5 of prompt
                benchmark_name TEXT NOT NULL,  -- e.g., 001-sol-transfer
                prompt TEXT NOT NULL,
                content TEXT NOT NULL, -- Full YML content
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
                .context("Failed to create table")?;
        }

        // Create indexes
        let indexes = ["CREATE INDEX IF NOT EXISTS idx_flow_logs_benchmark_agent ON flow_logs(benchmark_id, agent_type)",
            "CREATE INDEX IF NOT EXISTS idx_agent_performance_score ON agent_performance(score)",
            "CREATE INDEX IF NOT EXISTS idx_agent_performance_timestamp ON agent_performance(timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_yml_testresults_benchmark_agent ON yml_testresults(benchmark_id, agent_type)",
            "CREATE INDEX IF NOT EXISTS idx_agent_performance_prompt_md5 ON agent_performance(prompt_md5)",
            "CREATE INDEX IF NOT EXISTS idx_results_prompt_md5 ON results(prompt_md5)",
            "CREATE INDEX IF NOT EXISTS idx_benchmarks_name ON benchmarks(benchmark_name)"];

        for index in indexes.iter() {
            conn.execute(index, ())
                .await
                .context("Failed to create index")?;
        }

        // No migration needed - we always start with fresh database
        info!("[DB] Database initialized with benchmark_name column");

        info!("[DB] Database schema initialized with flow logs support.");
        Ok(())
    }

    /// Inserts the complete result of a benchmark evaluation into the database
    pub async fn insert_result(
        &self,
        benchmark_id: &str,
        prompt: &str,
        generated_instruction: &str,
        final_observation: &AgentObservation,
        final_status: FinalStatus,
        score: f64,
    ) -> Result<()> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let final_on_chain_state = serde_json::to_string(&final_observation.account_states)
            .context("Failed to serialize final state to JSON")?;
        let final_status_str = format!("{final_status:?}");
        let prompt_md5 = format!("{:x}", md5::compute(prompt.as_bytes()));

        let insert_query = "
            INSERT INTO results (
                benchmark_id,
                timestamp,
                prompt,
                generated_instruction,
                final_on_chain_state,
                final_status,
                score,
                prompt_md5
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);
        ";

        self.conn
            .execute(
                insert_query,
                [
                    benchmark_id,
                    &timestamp,
                    prompt,
                    generated_instruction,
                    &final_on_chain_state,
                    &final_status_str,
                    &score.to_string(),
                    &prompt_md5,
                ],
            )
            .await
            .context("Failed to insert result into database")?;

        info!("[DB] Saved result for benchmark '{benchmark_id}' to database.");
        Ok(())
    }

    /// Inserts a complete flow log into the database
    pub async fn insert_flow_log(&self, flow_log: &FlowLog) -> Result<i64> {
        let start_time = flow_log
            .start_time
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();
        let end_time = flow_log.end_time.map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string()
        });
        let final_result = flow_log
            .final_result
            .as_ref()
            .map(|r| serde_json::to_string(r).unwrap_or_default());
        let flow_data =
            serde_json::to_string(flow_log).context("Failed to serialize flow log to JSON")?;

        let insert_query = "
            INSERT INTO flow_logs (
                session_id,
                benchmark_id,
                agent_type,
                start_time,
                end_time,
                final_result,
                flow_data
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);
        ";

        self.conn
            .execute(
                insert_query,
                [
                    flow_log.session_id.as_str(),
                    flow_log.benchmark_id.as_str(),
                    flow_log.agent_type.as_str(),
                    start_time.as_str(),
                    end_time.as_deref().unwrap_or(""),
                    final_result.as_deref().unwrap_or(""),
                    flow_data.as_str(),
                ],
            )
            .await
            .context("Failed to insert flow log into database")?;

        let flow_log_id = self.conn.last_insert_rowid();
        info!(
            "[DB] Saved flow log '{}' for benchmark '{}' to database.",
            flow_log.session_id, flow_log.benchmark_id
        );
        Ok(flow_log_id)
    }

    /// Inserts agent performance data into the database
    pub async fn insert_agent_performance(&self, performance: &AgentPerformanceData) -> Result<()> {
        match (&performance.flow_log_id, &performance.prompt_md5) {
            (Some(flow_log_id), Some(prompt_md5)) => {
                let insert_query = "
                    INSERT INTO agent_performance (
                        benchmark_id,
                        agent_type,
                        score,
                        final_status,
                        execution_time_ms,
                        timestamp,
                        flow_log_id,
                        prompt_md5
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);
                ";

                self.conn
                    .execute(
                        insert_query,
                        [
                            performance.benchmark_id.as_str(),
                            performance.agent_type.as_str(),
                            &performance.score.to_string(),
                            performance.final_status.as_str(),
                            &performance.execution_time_ms.to_string(),
                            performance.timestamp.as_str(),
                            &flow_log_id.to_string(),
                            prompt_md5.as_str(),
                        ],
                    )
                    .await
                    .context("Failed to insert agent performance into database")?;
            }
            (Some(flow_log_id), None) => {
                let insert_query = "
                    INSERT INTO agent_performance (
                        benchmark_id,
                        agent_type,
                        score,
                        final_status,
                        execution_time_ms,
                        timestamp,
                        flow_log_id,
                        prompt_md5
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL);
                ";

                self.conn
                    .execute(
                        insert_query,
                        [
                            performance.benchmark_id.as_str(),
                            performance.agent_type.as_str(),
                            &performance.score.to_string(),
                            performance.final_status.as_str(),
                            &performance.execution_time_ms.to_string(),
                            performance.timestamp.as_str(),
                            &flow_log_id.to_string(),
                        ],
                    )
                    .await
                    .context("Failed to insert agent performance into database")?;
            }
            (None, Some(prompt_md5)) => {
                let insert_query = "
                    INSERT INTO agent_performance (
                        benchmark_id,
                        agent_type,
                        score,
                        final_status,
                        execution_time_ms,
                        timestamp,
                        flow_log_id,
                        prompt_md5
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7);
                ";

                self.conn
                    .execute(
                        insert_query,
                        [
                            performance.benchmark_id.as_str(),
                            performance.agent_type.as_str(),
                            &performance.score.to_string(),
                            performance.final_status.as_str(),
                            &performance.execution_time_ms.to_string(),
                            performance.timestamp.as_str(),
                            prompt_md5.as_str(),
                        ],
                    )
                    .await
                    .context("Failed to insert agent performance into database")?;
            }
            (None, None) => {
                let insert_query = "
                    INSERT INTO agent_performance (
                        benchmark_id,
                        agent_type,
                        score,
                        final_status,
                        execution_time_ms,
                        timestamp,
                        flow_log_id,
                        prompt_md5
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL);
                ";

                self.conn
                    .execute(
                        insert_query,
                        [
                            performance.benchmark_id.as_str(),
                            performance.agent_type.as_str(),
                            &performance.score.to_string(),
                            performance.final_status.as_str(),
                            &performance.execution_time_ms.to_string(),
                            performance.timestamp.as_str(),
                        ],
                    )
                    .await
                    .context("Failed to insert agent performance into database")?;
            }
        }

        info!(
            "[DB] Saved agent performance for '{}' agent on benchmark '{}' with score {}.",
            performance.agent_type, performance.benchmark_id, performance.score
        );
        Ok(())
    }

    /// Store YML flow log directly in database
    pub async fn insert_yml_flow_log(&self, benchmark_id: &str, yml_content: &str) -> Result<i64> {
        let insert_query = "
            INSERT INTO flow_logs (
                session_id, benchmark_id, agent_type, start_time, end_time,
                final_result, flow_data
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);
        ";

        let session_id = uuid::Uuid::new_v4().to_string();
        let start_time = chrono::Utc::now().to_rfc3339();
        let end_time = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                insert_query,
                [
                    session_id.as_str(),
                    benchmark_id,
                    "deterministic",
                    start_time.as_str(),
                    &end_time,
                    "{}",
                    yml_content,
                ],
            )
            .await
            .context("Failed to insert YML flow log into database")?;

        let flow_log_id = self.conn.last_insert_rowid();
        info!(
            "[DB] Saved YML flow log '{}' for benchmark '{}' to database.",
            session_id, benchmark_id
        );
        Ok(flow_log_id)
    }

    /// Inserts YML TestResult into database
    pub async fn insert_yml_testresult(
        &self,
        benchmark_id: &str,
        agent_type: &str,
        yml_content: &str,
    ) -> Result<()> {
        let query = "
            INSERT INTO yml_testresults (
                benchmark_id,
                agent_type,
                yml_content,
                created_at
            ) VALUES (?1, ?2, ?3, ?4);
        ";

        let timestamp = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(query, [benchmark_id, agent_type, yml_content, &timestamp])
            .await
            .context("Failed to insert YML TestResult into database")?;

        info!(
            "YML TestResult stored for benchmark: {} by agent: {}",
            benchmark_id, agent_type
        );
        Ok(())
    }

    /// Upsert a benchmark into the database using proper INSERT ON CONFLICT pattern
    pub async fn upsert_benchmark(
        &self,
        benchmark_name: &str,
        prompt: &str,
        content: &str,
    ) -> Result<String> {
        let prompt_md5 = format!(
            "{:x}",
            md5::compute(format!("{}:{}", benchmark_name, prompt).as_bytes())
        );
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Use INSERT ... ON CONFLICT DO UPDATE pattern from Firebase examples
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
                turso::params![
                    prompt_md5.clone(),
                    benchmark_name,
                    prompt,
                    content,
                    timestamp.clone(),
                    timestamp.clone()
                ],
            )
            .await
            .context("Failed to upsert benchmark into database")?;

        tracing::info!(
            "[DB] Upserted benchmark '{}' with MD5 '{}' (prompt: {:.50}...)",
            benchmark_name,
            prompt_md5,
            prompt
        );
        Ok(prompt_md5)
    }

    /// Sync all benchmark files from the benchmarks directory to the database
    pub async fn sync_benchmarks_to_db(&self, benchmarks_dir: &str) -> Result<usize> {
        let mut synced_count = 0;

        tracing::info!(
            "[DB] Starting benchmark sync from directory: {}",
            benchmarks_dir
        );

        // Check database state before sync
        let initial_count = self.get_all_benchmark_count().await.unwrap_or(0);
        tracing::info!(
            "[DB] Initial benchmark count in database: {}",
            initial_count
        );

        // Read all YAML files from benchmarks directory first
        let mut yaml_files = Vec::new();
        let mut entries = tokio::fs::read_dir(benchmarks_dir)
            .await
            .with_context(|| format!("Failed to read benchmarks directory: {benchmarks_dir}"))?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yml") {
                yaml_files.push(path);
            }
        }

        // Sort files for consistent processing order
        yaml_files.sort();
        tracing::info!("[DB] Found {} benchmark files to process", yaml_files.len());

        // Process benchmarks one by one (sequentially) without explicit transaction
        // Let each upsert handle its own atomicity
        for (index, path) in yaml_files.iter().enumerate() {
            match self.sync_single_benchmark(path).await {
                Ok(md5) => {
                    synced_count += 1;
                    tracing::info!(
                        "[DB] [{}/{}] Synced benchmark: {:?} -> MD5: {}",
                        index + 1,
                        yaml_files.len(),
                        path.file_name(),
                        md5
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "[DB] [{}/{}] Failed to sync benchmark {:?}: {}",
                        index + 1,
                        yaml_files.len(),
                        path,
                        e
                    );
                    // Continue with other benchmarks even if one fails
                }
            }
        }

        // Check database state after sync
        let final_count = self.get_all_benchmark_count().await.unwrap_or(0);
        tracing::info!("[DB] Final benchmark count in database: {}", final_count);

        // Log summary
        if final_count > initial_count {
            tracing::info!(
                "[DB] Sync completed: {} new benchmarks added ({} -> {})",
                final_count - initial_count,
                initial_count,
                final_count
            );
        } else if final_count == initial_count {
            tracing::info!(
                "[DB] Sync completed: {} benchmarks updated (no new records)",
                synced_count
            );
        } else {
            tracing::warn!("[DB] Sync completed with unexpected count change: {} -> {} (some records may have been removed)",
                   initial_count, final_count);
        }

        // Check for potential duplicates (should not happen with ON CONFLICT)
        if final_count != initial_count && (final_count - initial_count) != synced_count as i64 {
            tracing::warn!("[DB] Potential duplicate detection: Expected {} new records, but count changed by {}",
                   synced_count, final_count - initial_count);
        }

        tracing::info!("[DB] Synced {} benchmarks to database", synced_count);
        Ok(synced_count)
    }

    /// Sync a single benchmark file to the database
    async fn sync_single_benchmark(&self, path: &std::path::Path) -> Result<String> {
        let content = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read benchmark file")?;

        // Parse YAML to extract prompt
        let benchmark_data: BenchmarkYml =
            serde_yaml::from_str(&content).context("Failed to parse benchmark YAML")?;

        // Upsert to database using the upsert_benchmark function
        let prompt_md5 = self
            .upsert_benchmark(&benchmark_data.id, &benchmark_data.prompt, &content)
            .await?;

        tracing::info!(
            "[DB] Synced benchmark '{}' with prompt MD5: {}",
            benchmark_data.id,
            prompt_md5
        );

        Ok(prompt_md5)
    }

    /// Get total count of benchmarks in database
    pub async fn get_all_benchmark_count(&self) -> Result<i64> {
        let query = "SELECT COUNT(*) FROM benchmarks;";

        let mut rows = self
            .conn
            .query(query, ())
            .await
            .context("Failed to query benchmark count")?;

        if let Some(row) = rows.next().await? {
            let count: i64 = row.get(0).context("Failed to get benchmark count")?;
            Ok(count)
        } else {
            Ok(0)
        }
    }

    /// Get benchmark content by MD5 ID
    pub async fn get_benchmark_by_id(&self, prompt_md5: &str) -> Result<Option<BenchmarkData>> {
        let query = "
            SELECT id, benchmark_name, prompt, content, created_at, updated_at
            FROM benchmarks
            WHERE id = ?1;
        ";

        let mut rows = self
            .conn
            .query(query, [prompt_md5])
            .await
            .context("Failed to query benchmark by ID")?;

        if let Some(row) = rows.next().await? {
            let benchmark = BenchmarkData {
                id: row.get(0).context("Failed to get benchmark id")?,
                benchmark_name: row.get(1).context("Failed to get benchmark name")?,
                prompt: row.get(2).context("Failed to get benchmark prompt")?,
                content: row.get(3).context("Failed to get benchmark content")?,
                created_at: row.get(4).context("Failed to get benchmark created_at")?,
                updated_at: row.get(5).context("Failed to get benchmark updated_at")?,
            };
            Ok(Some(benchmark))
        } else {
            Ok(None)
        }
    }

    /// Get prompt MD5 by benchmark name (e.g., "001-sol-transfer")
    pub async fn get_prompt_md5_by_benchmark_name(
        &self,
        benchmark_name: &str,
    ) -> Result<Option<String>> {
        info!(
            "[DB] Looking up prompt MD5 for benchmark_name: '{}'",
            benchmark_name
        );

        let query = "
            SELECT id, benchmark_name
            FROM benchmarks
            WHERE benchmark_name = ?1;
        ";

        let mut rows = self
            .conn
            .query(query, [benchmark_name])
            .await
            .context("Failed to query prompt MD5 by benchmark name")?;

        if let Some(row) = rows.next().await? {
            let prompt_md5: String = row.get(0).context("Failed to get prompt MD5")?;
            let stored_name: String = row.get(1).context("Failed to get benchmark name")?;

            tracing::info!(
                "[DB] Found prompt MD5 '{}' for benchmark_name '{}' (stored as '{}')",
                prompt_md5,
                benchmark_name,
                stored_name
            );
            Ok(Some(prompt_md5))
        } else {
            tracing::warn!(
                "[DB] No prompt MD5 found for benchmark_name: '{}'",
                benchmark_name
            );

            // Debug: List all available benchmark names
            let list_query = "SELECT benchmark_name, id FROM benchmarks LIMIT 10;";
            let mut list_rows = self
                .conn
                .query(list_query, ())
                .await
                .context("Failed to list benchmarks")?;

            tracing::info!("[DB] Available benchmarks in database:");
            while let Some(list_row) = list_rows.next().await? {
                let name: String = list_row.get(0).context("Failed to get name")?;
                let id: String = list_row.get(1).context("Failed to get id")?;
                tracing::info!("[DB]   - '{}': {}", name, id);
            }

            Ok(None)
        }
    }

    /// Test prompt MD5 lookup by benchmark name (for debugging)
    pub async fn test_prompt_md5_lookup(&self, benchmark_name: &str) -> Result<()> {
        match self
            .get_prompt_md5_by_benchmark_name(benchmark_name)
            .await?
        {
            Some(prompt_md5) => {
                info!(
                    "[DB] Found prompt MD5 for '{}': {}",
                    benchmark_name, prompt_md5
                );

                // Also get the benchmark details
                if let Some(benchmark) = self.get_benchmark_by_id(&prompt_md5).await? {
                    info!("[DB] Benchmark prompt: {:.50}...", benchmark.prompt);
                }
            }
            None => {
                warn!("[DB] No prompt MD5 found for benchmark: {}", benchmark_name);
            }
        }
        Ok(())
    }

    /// Get all benchmarks from database
    pub async fn get_all_benchmarks(&self) -> Result<Vec<BenchmarkData>> {
        let query = "
            SELECT id, benchmark_name, prompt, content, created_at, updated_at
            FROM benchmarks
            ORDER BY created_at;
        ";

        let mut rows = self
            .conn
            .query(query, ())
            .await
            .context("Failed to query all benchmarks")?;

        let mut benchmarks = Vec::new();
        while let Some(row) = rows.next().await? {
            benchmarks.push(BenchmarkData {
                id: row.get(0).context("Failed to get benchmark id")?,
                benchmark_name: row.get(1).context("Failed to get benchmark name")?,
                prompt: row.get(2).context("Failed to get benchmark prompt")?,
                content: row.get(3).context("Failed to get benchmark content")?,
                created_at: row.get(4).context("Failed to get benchmark created_at")?,
                updated_at: row.get(5).context("Failed to get benchmark updated_at")?,
            });
        }

        Ok(benchmarks)
    }

    /// Check for duplicate benchmark records (for monitoring/debugging)
    pub async fn check_for_duplicates(&self) -> Result<Vec<(String, String, i64)>> {
        let query = "
                SELECT id, benchmark_name, COUNT(*) as count
                FROM benchmarks
                GROUP BY id, benchmark_name
                HAVING COUNT(*) > 1
                ORDER BY id
            ";

        let mut rows = self
            .conn
            .query(query, ())
            .await
            .context("Failed to check for duplicates")?;

        let mut duplicates = Vec::new();
        while let Some(row) = rows.next().await? {
            let id: String = row.get(0).context("Failed to get id")?;
            let benchmark_name: String = row.get(1).context("Failed to get benchmark name")?;
            let count: i64 = row.get(2).context("Failed to get count")?;
            duplicates.push((id, benchmark_name, count));
        }

        if !duplicates.is_empty() {
            tracing::warn!(
                "[DB] Found {} duplicate benchmark records in database",
                duplicates.len()
            );
            for (id, name, count) in &duplicates {
                tracing::warn!(
                    "[DB] Duplicate detected: ID '{}' - '{}' appears {} times",
                    id,
                    name,
                    count
                );
            }
        } else {
            tracing::info!("[DB] No duplicate benchmark records found");
        }

        Ok(duplicates)
    }

    /// Get database statistics for monitoring
    pub async fn get_database_stats(&self) -> Result<DatabaseStats> {
        let total_count = self.get_all_benchmark_count().await?;
        let duplicates = self.check_for_duplicates().await?;

        Ok(DatabaseStats {
            total_benchmarks: total_count,
            duplicate_count: duplicates.len() as i64,
            duplicate_details: duplicates,
        })
    }
}

/// Benchmark data structure for YAML parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkYml {
    pub id: String,
    pub description: String,
    pub tags: Vec<String>,
    pub prompt: String,
    pub initial_state: Vec<serde_yaml::Value>,
    pub ground_truth: serde_yaml::Value,
}

/// Benchmark data structure from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkData {
    pub id: String,
    pub benchmark_name: String,
    pub prompt: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}
