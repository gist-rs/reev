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
use tracing::info;
use turso::{Builder, Connection};

/// Shared database writer for all database operations
pub struct DatabaseWriter {
    pub conn: Connection,
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
            "CREATE INDEX IF NOT EXISTS idx_results_prompt_md5 ON results(prompt_md5)"];

        for index in indexes.iter() {
            conn.execute(index, ())
                .await
                .context("Failed to create index")?;
        }

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

    /// Upsert a benchmark into the database
    pub async fn upsert_benchmark(&self, prompt: &str, content: &str) -> Result<String> {
        let prompt_md5 = format!("{:x}", md5::compute(prompt.as_bytes()));
        let timestamp = chrono::Utc::now().to_rfc3339();

        let query = "
            INSERT INTO benchmarks (id, prompt, content, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
                prompt = excluded.prompt,
                content = excluded.content,
                updated_at = excluded.updated_at;
        ";

        self.conn
            .execute(
                query,
                [&prompt_md5, prompt, content, &timestamp, &timestamp],
            )
            .await
            .context("Failed to upsert benchmark into database")?;

        info!(
            "[DB] Upserted benchmark with MD5 '{}' (prompt: {:.50}...)",
            prompt_md5, prompt
        );
        Ok(prompt_md5)
    }

    /// Sync all benchmark files from the benchmarks directory to the database
    pub async fn sync_benchmarks_to_db(&self, benchmarks_dir: &str) -> Result<usize> {
        let mut synced_count = 0;

        // Read all YAML files from benchmarks directory
        let mut entries = tokio::fs::read_dir(benchmarks_dir)
            .await
            .with_context(|| format!("Failed to read benchmarks directory: {benchmarks_dir}"))?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yml") {
                match self.sync_single_benchmark(&path).await {
                    Ok(_) => {
                        synced_count += 1;
                        info!("[DB] Synced benchmark: {:?}", path.file_name());
                    }
                    Err(e) => {
                        tracing::error!("[DB] Failed to sync benchmark {:?}: {}", path, e);
                    }
                }
            }
        }

        info!("[DB] Synced {} benchmarks to database", synced_count);
        Ok(synced_count)
    }

    /// Sync a single benchmark file to the database
    async fn sync_single_benchmark(&self, path: &std::path::Path) -> Result<()> {
        let content = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read benchmark file")?;

        // Parse YAML to extract prompt
        let benchmark_data: BenchmarkYml =
            serde_yaml::from_str(&content).context("Failed to parse benchmark YAML")?;

        // Upsert to database
        self.upsert_benchmark(&benchmark_data.prompt, &content)
            .await?;
        Ok(())
    }

    /// Get benchmark content by MD5 ID
    pub async fn get_benchmark_by_id(&self, prompt_md5: &str) -> Result<Option<BenchmarkData>> {
        let query = "
            SELECT id, prompt, content, created_at, updated_at
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
                prompt: row.get(1).context("Failed to get benchmark prompt")?,
                content: row.get(2).context("Failed to get benchmark content")?,
                created_at: row.get(3).context("Failed to get benchmark created_at")?,
                updated_at: row.get(4).context("Failed to get benchmark updated_at")?,
            };
            Ok(Some(benchmark))
        } else {
            Ok(None)
        }
    }

    /// Get all benchmarks from database
    pub async fn get_all_benchmarks(&self) -> Result<Vec<BenchmarkData>> {
        let query = "
            SELECT id, prompt, content, created_at, updated_at
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
            let benchmark = BenchmarkData {
                id: row.get(0).context("Failed to get benchmark id")?,
                prompt: row.get(1).context("Failed to get benchmark prompt")?,
                content: row.get(2).context("Failed to get benchmark content")?,
                created_at: row.get(3).context("Failed to get benchmark created_at")?,
                updated_at: row.get(4).context("Failed to get benchmark updated_at")?,
            };
            benchmarks.push(benchmark);
        }

        Ok(benchmarks)
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
    pub prompt: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}
