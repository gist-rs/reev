use anyhow::{Context, Result};
use reev_lib::{
    agent::{AgentAction, AgentObservation},
    results::FinalStatus,
};
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

        // println!("[DB] Connected to database at: {path}");

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
        ";
        conn.execute(schema_query, ())
            .await
            .context("Failed to create results table")?;

        // println!("[DB] Database schema initialized.");

        Ok(Self { conn })
    }

    /// Inserts the complete result of a benchmark evaluation into the database.
    pub async fn insert_result(
        &self,
        benchmark_id: &str,
        prompt: &str,
        action: &AgentAction,
        final_observation: &AgentObservation,
        final_status: FinalStatus,
        score: f64,
    ) -> Result<()> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let generated_instruction =
            serde_json::to_string(&action.0).context("Failed to serialize instruction to JSON")?;
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
                    &generated_instruction,
                    &final_on_chain_state,
                    &final_status_str,
                    &final_status_str,
                    &score.to_string(),
                ],
            )
            .await
            .context("Failed to insert result into database")?;

        println!("[DB] Saved result for benchmark '{benchmark_id}' to database.");
        Ok(())
    }
}
