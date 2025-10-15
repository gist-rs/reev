use super::error::{FlowError, FlowResult};
use super::types::*;
use super::utils::calculate_execution_statistics;
use crate::db::{AgentPerformanceData as DbAgentPerformanceData, DatabaseWriter};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

// Re-export for backward compatibility
pub use crate::db::AgentPerformanceData;

// Database trait for backward compatibility - deprecated, use DatabaseWriter directly
#[async_trait::async_trait]
pub trait FlowLogDatabase: Send + Sync {
    async fn insert_flow_log(&self, flow_log: &FlowLog) -> Result<i64, FlowError>;
    async fn insert_agent_performance(&self, data: &AgentPerformanceData) -> Result<(), FlowError>;
}

/// Main flow logger interface
pub struct FlowLogger {
    session_id: String,
    benchmark_id: String,
    agent_type: String,
    start_time: SystemTime,
    events: Vec<FlowEvent>,
    output_path: PathBuf,
    database: Option<Arc<DatabaseWriter>>,
    legacy_database: Option<Arc<dyn FlowLogDatabase>>,
}

impl FlowLogger {
    /// Create a new flow logger instance
    pub fn new(benchmark_id: String, agent_type: String, output_path: PathBuf) -> Self {
        let session_id = uuid::Uuid::new_v4().to_string();
        let start_time = SystemTime::now();

        info!(
            session_id = %session_id,
            benchmark_id = %benchmark_id,
            agent_type = %agent_type,
            "Initializing flow logger"
        );

        Self {
            session_id,
            benchmark_id,
            agent_type,
            start_time,
            events: Vec::new(),
            output_path,
            database: None,
            legacy_database: None,
        }
    }

    /// Create a new flow logger with shared database support
    pub fn new_with_database(
        benchmark_id: String,
        agent_type: String,
        output_path: PathBuf,
        database: Arc<DatabaseWriter>,
    ) -> Self {
        let session_id = uuid::Uuid::new_v4().to_string();
        let start_time = SystemTime::now();

        info!(
            session_id = %session_id,
            benchmark_id = %benchmark_id,
            agent_type = %agent_type,
            "Initializing flow logger with shared database support"
        );

        Self {
            session_id,
            benchmark_id,
            agent_type,
            start_time,
            events: Vec::new(),
            output_path,
            database: Some(database),
            legacy_database: None,
        }
    }

    /// Create a new flow logger with legacy database support (deprecated)
    pub fn new_with_legacy_database(
        benchmark_id: String,
        agent_type: String,
        output_path: PathBuf,
        database: Arc<dyn FlowLogDatabase>,
    ) -> Self {
        let session_id = uuid::Uuid::new_v4().to_string();
        let start_time = SystemTime::now();

        info!(
            session_id = %session_id,
            benchmark_id = %benchmark_id,
            agent_type = %agent_type,
            "Initializing flow logger with legacy database support (deprecated)"
        );

        Self {
            session_id,
            benchmark_id,
            agent_type,
            start_time,
            events: Vec::new(),
            output_path,
            database: None,
            legacy_database: Some(database),
        }
    }

    /// Log an LLM request event
    pub fn log_llm_request(&mut self, content: LlmRequestContent, depth: u32) {
        let event = FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::LlmRequest,
            depth,
            content: EventContent {
                data: serde_json::to_value(content).unwrap_or_default(),
                metadata: std::collections::HashMap::new(),
            },
        };
        self.events.push(event);
        debug!("Logged LLM request");
    }

    /// Log a tool call event
    pub fn log_tool_call(&mut self, content: ToolCallContent, depth: u32) {
        let event = FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::ToolCall,
            depth,
            content: EventContent {
                data: serde_json::to_value(content).unwrap_or_default(),
                metadata: std::collections::HashMap::new(),
            },
        };
        self.events.push(event);
        debug!("Logged tool call");
    }

    /// Log a tool result event
    pub fn log_tool_result(&mut self, content: ToolCallContent, depth: u32) {
        let event = FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::ToolResult,
            depth,
            content: EventContent {
                data: serde_json::to_value(content).unwrap_or_default(),
                metadata: std::collections::HashMap::new(),
            },
        };
        self.events.push(event);
        debug!("Logged tool result");
    }

    /// Log a transaction execution event
    pub fn log_transaction(&mut self, content: TransactionExecutionContent, depth: u32) {
        let event = FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::TransactionExecution,
            depth,
            content: EventContent {
                data: serde_json::to_value(content).unwrap_or_default(),
                metadata: std::collections::HashMap::new(),
            },
        };
        self.events.push(event);
        debug!("Logged transaction execution");
    }

    /// Log an error event
    pub fn log_error(&mut self, content: ErrorContent, depth: u32) {
        let event = FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::Error,
            depth,
            content: EventContent {
                data: serde_json::to_value(&content).unwrap_or_default(),
                metadata: std::collections::HashMap::new(),
            },
        };
        self.events.push(event);
        debug!("Logged error: {}", content.message);
    }

    /// Complete the flow log with final results
    pub async fn complete(&mut self, result: ExecutionResult) -> FlowResult<PathBuf> {
        let end_time = SystemTime::now();

        let flow_log = FlowLog {
            session_id: self.session_id.clone(),
            benchmark_id: self.benchmark_id.clone(),
            agent_type: self.agent_type.clone(),
            start_time: self.start_time,
            end_time: Some(end_time),
            events: self.events.clone(),
            final_result: Some(result),
        };

        // Save to shared database if available
        if let Some(database) = &self.database {
            match database.insert_flow_log(&flow_log).await {
                Ok(flow_log_id) => {
                    // Insert agent performance data
                    let timestamp = chrono::Utc::now().to_rfc3339();
                    // Use a reasonable default execution time since total_time_ms doesn't exist in TestResult
                    let execution_time_ms = 5000u64; // 5 seconds default execution time
                    let score = flow_log
                        .final_result
                        .as_ref()
                        .map(|r| r.score)
                        .unwrap_or(0.0);
                    let final_status = if flow_log
                        .final_result
                        .as_ref()
                        .map(|r| r.success)
                        .unwrap_or(false)
                    {
                        "Succeeded"
                    } else {
                        "Failed"
                    };

                    // Look up prompt MD5 by benchmark name
                    let prompt_md5 = database
                        .get_prompt_md5_by_benchmark_name(&flow_log.benchmark_id)
                        .await
                        .ok()
                        .flatten();

                    let performance_data = DbAgentPerformanceData {
                        benchmark_id: flow_log.benchmark_id.clone(),
                        agent_type: flow_log.agent_type.clone(),
                        score,
                        final_status: final_status.to_string(),
                        execution_time_ms,
                        timestamp,
                        flow_log_id: Some(flow_log_id),
                        prompt_md5,
                    };

                    if let Err(e) = database.insert_agent_performance(&performance_data).await {
                        warn!(
                            "Failed to insert agent performance for session {}: {}",
                            self.session_id, e
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to save flow log to database for session {}: {}",
                        self.session_id, e
                    );
                }
            }
        }
        // Fallback to legacy database if available
        else if let Some(legacy_database) = &self.legacy_database {
            match legacy_database.insert_flow_log(&flow_log).await {
                Ok(flow_log_id) => {
                    // Insert agent performance data
                    let timestamp = chrono::Utc::now().to_rfc3339();
                    // Use a reasonable default execution time since total_time_ms doesn't exist in TestResult
                    let execution_time_ms = 5000u64; // 5 seconds default execution time
                    let score = flow_log
                        .final_result
                        .as_ref()
                        .map(|r| r.score)
                        .unwrap_or(0.0);
                    let final_status = if flow_log
                        .final_result
                        .as_ref()
                        .map(|r| r.success)
                        .unwrap_or(false)
                    {
                        "Succeeded"
                    } else {
                        "Failed"
                    };

                    // Note: Legacy database doesn't have prompt MD5 lookup
                    // This could be enhanced if needed for legacy support
                    let performance_data = AgentPerformanceData {
                        benchmark_id: flow_log.benchmark_id.clone(),
                        agent_type: flow_log.agent_type.clone(),
                        score,
                        final_status: final_status.to_string(),
                        execution_time_ms,
                        timestamp,
                        flow_log_id: Some(flow_log_id),
                        prompt_md5: None,
                    };

                    if let Err(e) = legacy_database
                        .insert_agent_performance(&performance_data)
                        .await
                    {
                        warn!(
                            "Failed to insert agent performance for session {}: {}",
                            self.session_id, e
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to save flow log to legacy database for session {}: {}",
                        self.session_id, e
                    );
                }
            }
        }

        // Still save YML file for debugging if enabled
        if std::env::var("REEV_ENABLE_YML_EXPORT").unwrap_or_default() == "true" {
            let timestamp = end_time
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let filename = format!(
                "flow_{}_{}_{}.yml",
                self.benchmark_id, self.agent_type, timestamp
            );
            let file_path = self.output_path.join(filename);

            let yml_content = serde_yaml::to_string(&flow_log)
                .map_err(|e| FlowError::serialization(e.to_string()))?;

            std::fs::write(&file_path, yml_content).map_err(|e| FlowError::file(e.to_string()))?;

            info!(
                session_id = %self.session_id,
                file_path = %file_path.display(),
                "Flow log YML export completed"
            );
        }

        info!(
            session_id = %self.session_id,
            "Flow log completed"
        );

        // Return output path for backward compatibility
        Ok(self.output_path.clone())
    }

    /// Get current statistics
    pub fn get_current_statistics(&self) -> ExecutionStatistics {
        calculate_execution_statistics(&self.events)
    }
}
