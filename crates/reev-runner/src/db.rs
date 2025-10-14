use anyhow::{Context, Result};
use reev_lib::{agent::AgentObservation, results::FinalStatus};
use serde::Serialize;
use tracing::info;
use turso::{Builder, Connection};

/// Manages the connection and operations for the results database.
pub struct Db {
    conn: Connection,
}

impl Db {
    /// Creates a new database manager, connecting to a local file and initializing the schema.
    pub async fn new(path: &str) -> Result<Self> {
        let db = Builder::new_local(path)
            .build()
            .await
            .context("Failed to build local database")?;
        let conn = db.connect().context("Failed to connect to database")?;

        info!("[DB] Connected to database at: {path}");

        // Define and execute the schema creation query.
        let schema_query = "
            CREATE TABLE IF NOT EXISTS results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                benchmark_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                prompt TEXT NOT NULL,
                generated_instruction TEXT NOT NULL,
                final_on_chain_state TEXT NOT NULL,
                final_status TEXT NOT NULL,
                score REAL NOT NULL
            );

            CREATE TABLE IF NOT EXISTS flow_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                benchmark_id TEXT NOT NULL,
                agent_type TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                final_result TEXT, -- JSON
                flow_data TEXT NOT NULL, -- Complete FlowLog as JSON
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS agent_performance (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                benchmark_id TEXT NOT NULL,
                agent_type TEXT NOT NULL,
                score REAL NOT NULL,
                final_status TEXT NOT NULL,
                execution_time_ms INTEGER,
                timestamp TEXT NOT NULL,
                flow_log_id INTEGER,
                FOREIGN KEY (flow_log_id) REFERENCES flow_logs (id)
            );

            CREATE TABLE IF NOT EXISTS yml_testresults (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                benchmark_id TEXT NOT NULL,
                agent_type TEXT NOT NULL,
                yml_content TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_flow_logs_benchmark_agent ON flow_logs(benchmark_id, agent_type);
            CREATE INDEX IF NOT EXISTS idx_agent_performance_score ON agent_performance(score);
            CREATE INDEX IF NOT EXISTS idx_agent_performance_timestamp ON agent_performance(timestamp);
            CREATE INDEX IF NOT EXISTS idx_yml_testresults_benchmark_agent ON yml_testresults(benchmark_id, agent_type);
        ";
        conn.execute(schema_query, ())
            .await
            .context("Failed to create database schema")?;

        info!("[DB] Database schema initialized with flow logs support.");

        Ok(Self { conn })
    }

    /// Inserts the complete result of a benchmark evaluation into the database.
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

        let insert_query = "
            INSERT INTO results (
                benchmark_id,
                timestamp,
                prompt,
                generated_instruction,
                final_on_chain_state,
                final_status,
                score
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);
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
                ],
            )
            .await
            .context("Failed to insert result into database")?;

        info!("[DB] Saved result for benchmark '{benchmark_id}' to database.");
        Ok(())
    }

    /// Inserts a complete flow log into the database
    pub async fn insert_flow_log(&self, flow_log: &reev_lib::flow::types::FlowLog) -> Result<i64> {
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
        let insert_query = "
                    INSERT INTO agent_performance (
                        benchmark_id,
                        agent_type,
                        score,
                        final_status,
                        execution_time_ms,
                        timestamp,
                        flow_log_id
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);
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
                    &performance.flow_log_id.unwrap_or_default().to_string(),
                ],
            )
            .await
            .context("Failed to insert agent performance into database")?;

        info!(
            "[DB] Saved agent performance for '{}' agent on benchmark '{}'.",
            performance.agent_type, performance.benchmark_id
        );
        Ok(())
    }

    /// Retrieves all flow logs for a specific benchmark
    pub async fn get_flow_logs(
        &self,
        benchmark_id: &str,
    ) -> Result<Vec<reev_lib::flow::types::FlowLog>> {
        let query =
            "SELECT flow_data FROM flow_logs WHERE benchmark_id = ?1 ORDER BY start_time DESC";
        let mut stmt = self
            .conn
            .prepare(query)
            .await
            .context("Failed to prepare flow logs query")?;

        let mut rows = stmt
            .query([benchmark_id])
            .await
            .context("Failed to execute flow logs query")?;

        let mut flow_logs = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| anyhow::anyhow!("Row iteration error: {e}"))?
        {
            let flow_data: String = row
                .get(0)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let flow_log: reev_lib::flow::types::FlowLog =
                serde_json::from_str(&flow_data).context("Failed to deserialize flow log")?;
            flow_logs.push(flow_log);
        }

        Ok(flow_logs)
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
                    "{}", // final_result as JSON placeholder
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

    /// Retrieves YML flow logs for a specific benchmark
    pub async fn get_yml_flow_logs(&self, benchmark_id: &str) -> Result<Vec<String>> {
        let query = "
            SELECT flow_data FROM flow_logs
            WHERE benchmark_id = ?1
            ORDER BY created_at DESC
        ";

        let mut stmt = self
            .conn
            .prepare(query)
            .await
            .context("Failed to prepare YML flow logs query")?;

        let mut rows = stmt
            .query([benchmark_id])
            .await
            .context("Failed to execute YML flow logs query")?;

        let mut yml_logs = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| anyhow::anyhow!("Row iteration error: {e}"))?
        {
            let yml_content: String = row
                .get(0)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            yml_logs.push(yml_content);
        }

        Ok(yml_logs)
    }

    /// Retrieves YML TestResult for a specific benchmark and agent
    pub async fn get_yml_testresult(
        &self,
        benchmark_id: &str,
        agent_type: &str,
    ) -> Result<Option<String>> {
        let query = "
            SELECT yml_content FROM yml_testresults
            WHERE benchmark_id = ?1 AND agent_type = ?2
            ORDER BY created_at DESC
            LIMIT 1
        ";

        let mut stmt = self
            .conn
            .prepare(query)
            .await
            .context("Failed to prepare YML TestResult query")?;

        let mut rows = stmt
            .query([benchmark_id, agent_type])
            .await
            .context("Failed to execute YML TestResult query")?;

        match rows
            .next()
            .await
            .map_err(|e| anyhow::anyhow!("Row iteration error: {e}"))?
        {
            Some(row) => {
                let yml_content: String = row
                    .get(0)
                    .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
                Ok(Some(yml_content))
            }
            None => Ok(None),
        }
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

    /// Retrieves agent performance summary by agent type
    pub async fn get_agent_performance(&self) -> Result<Vec<AgentPerformanceSummary>> {
        let query = "
            SELECT
                agent_type,
                COUNT(*) as total_benchmarks,
                AVG(score) as average_score,
                SUM(CASE WHEN final_status = 'Succeeded' THEN 1 ELSE 0 END) * 1.0 / COUNT(*) as success_rate
            FROM agent_performance
            GROUP BY agent_type
            ORDER BY agent_type;
        ";

        let mut stmt = self
            .conn
            .prepare(query)
            .await
            .context("Failed to prepare agent performance query")?;

        let mut rows = stmt
            .query(())
            .await
            .context("Failed to execute agent performance query")?;

        let mut summaries = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| anyhow::anyhow!("Row iteration error: {e}"))?
        {
            let agent_type: String = row
                .get(0)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let total_benchmarks: i64 = row
                .get(1)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let average_score: f64 = row
                .get(2)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let success_rate: f64 = row
                .get(3)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;

            // Get detailed results for this agent
            let detailed_results = self.get_agent_results(&agent_type).await?;

            summaries.push(AgentPerformanceSummary {
                agent_type: agent_type.clone(),
                total_benchmarks: total_benchmarks as u32,
                average_score,
                success_rate,
                best_benchmarks: self.get_best_benchmarks(&agent_type, 5).await?,
                worst_benchmarks: self.get_worst_benchmarks(&agent_type, 5).await?,
                results: detailed_results,
            });
        }

        Ok(summaries)
    }

    /// Gets detailed results for a specific agent
    async fn get_agent_results(&self, agent_type: &str) -> Result<Vec<BenchmarkResult>> {
        let query = "
            SELECT benchmark_id, agent_type, score, final_status, execution_time_ms, timestamp
            FROM agent_performance
            WHERE agent_type = ?1
            ORDER BY timestamp DESC;
        ";

        let mut stmt = self
            .conn
            .prepare(query)
            .await
            .context("Failed to prepare agent results query")?;

        let mut rows = stmt
            .query([agent_type])
            .await
            .context("Failed to execute agent results query")?;

        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| anyhow::anyhow!("Row iteration error: {e}"))?
        {
            let benchmark_id: String = row
                .get(0)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let agent_type: String = row
                .get(1)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let score: f64 = row
                .get(2)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let final_status: String = row
                .get(3)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let execution_time_ms: i64 = row
                .get(4)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let timestamp: String = row
                .get(5)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;

            results.push(BenchmarkResult {
                id: format!("{agent_type}-{benchmark_id}"),
                benchmark_id,
                agent_type,
                score,
                final_status,
                execution_time_ms: execution_time_ms as u64,
                timestamp,
                color_class: get_color_class(score),
            });
        }

        Ok(results)
    }

    /// Gets best performing benchmarks for an agent
    async fn get_best_benchmarks(&self, agent_type: &str, limit: i32) -> Result<Vec<String>> {
        let query = "
            SELECT benchmark_id
            FROM agent_performance
            WHERE agent_type = ?1
            ORDER BY score DESC
            LIMIT ?2;
        ";

        let mut stmt = self
            .conn
            .prepare(query)
            .await
            .context("Failed to prepare best benchmarks query")?;

        let mut rows = stmt
            .query([agent_type, &limit.to_string()])
            .await
            .context("Failed to execute best benchmarks query")?;

        let mut benchmarks = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| anyhow::anyhow!("Row iteration error: {e}"))?
        {
            let benchmark_id: String = row
                .get(0)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            benchmarks.push(benchmark_id);
        }

        Ok(benchmarks)
    }

    /// Gets worst performing benchmarks for an agent
    async fn get_worst_benchmarks(&self, agent_type: &str, limit: i32) -> Result<Vec<String>> {
        let query = "
            SELECT benchmark_id
            FROM agent_performance
            WHERE agent_type = ?1
            ORDER BY score ASC
            LIMIT ?2;
        ";

        let mut stmt = self
            .conn
            .prepare(query)
            .await
            .context("Failed to prepare worst benchmarks query")?;

        let mut rows = stmt
            .query([agent_type, &limit.to_string()])
            .await
            .context("Failed to execute worst benchmarks query")?;

        let mut benchmarks = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| anyhow::anyhow!("Row iteration error: {e}"))?
        {
            let benchmark_id: String = row
                .get(0)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            benchmarks.push(benchmark_id);
        }

        Ok(benchmarks)
    }
}

/// Data structure for agent performance data
pub struct AgentPerformanceData {
    pub benchmark_id: String,
    pub agent_type: String,
    pub score: f64,
    pub final_status: String,
    pub execution_time_ms: u64,
    pub timestamp: String,
    pub flow_log_id: Option<i64>,
}

/// Helper function to determine color class based on score
fn get_color_class(score: f64) -> String {
    if score >= 1.0 {
        "green".to_string()
    } else if score >= 0.25 {
        "yellow".to_string()
    } else {
        "red".to_string()
    }
}

/// Data structures for API responses
#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkResult {
    pub id: String,
    pub benchmark_id: String,
    pub agent_type: String,
    pub score: f64,
    pub final_status: String,
    pub execution_time_ms: u64,
    pub timestamp: String,
    pub color_class: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentPerformanceSummary {
    pub agent_type: String,
    pub total_benchmarks: u32,
    pub average_score: f64,
    pub success_rate: f64,
    pub best_benchmarks: Vec<String>,
    pub worst_benchmarks: Vec<String>,
    pub results: Vec<BenchmarkResult>,
}
