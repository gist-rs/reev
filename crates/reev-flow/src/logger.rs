//! Flow logging functionality
//!
//! This module provides the main FlowLogger interface for tracking agent execution flows.
//! It supports both file-based logging and database integration through the reev-db crate.

use super::error::{FlowError, FlowResult};
use super::types::*;
use super::utils::calculate_execution_statistics;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

/// Database writer trait for integration with reev-db
#[async_trait::async_trait]
pub trait DatabaseWriter: Send + Sync {
    /// Insert a flow log into the database
    async fn insert_flow_log(&self, flow_log: &super::database::DBFlowLog) -> FlowResult<i64>;

    /// Insert agent performance data
    async fn insert_agent_performance(&self, performance: &AgentPerformanceData)
        -> FlowResult<i64>;

    /// Get prompt MD5 by benchmark name
    async fn get_prompt_md5_by_benchmark_name(
        &self,
        benchmark_id: &str,
    ) -> FlowResult<Option<String>>;
}

/// Agent performance data for database storage
#[derive(Debug, Clone)]
pub struct AgentPerformanceData {
    pub benchmark_id: String,
    pub agent_type: String,
    pub score: f64,
    pub final_status: String,
    pub execution_time_ms: u64,
    pub timestamp: String,
    pub flow_log_id: Option<i64>,
    pub prompt_md5: Option<String>,
}

/// Main flow logger interface
pub struct FlowLogger {
    session_id: String,
    benchmark_id: String,
    agent_type: String,
    start_time: SystemTime,
    events: Vec<FlowEvent>,
    output_path: PathBuf,
    database: Option<Arc<dyn DatabaseWriter>>,
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
        }
    }

    /// Create a new flow logger instance with specific session ID
    pub fn new_with_session(
        session_id: String,
        benchmark_id: String,
        agent_type: String,
        output_path: PathBuf,
    ) -> Self {
        let start_time = SystemTime::now();

        info!(
            session_id = %session_id,
            benchmark_id = %benchmark_id,
            agent_type = %agent_type,
            "Initializing flow logger with session"
        );

        Self {
            session_id,
            benchmark_id,
            agent_type,
            start_time,
            events: Vec::new(),
            output_path,
            database: None,
        }
    }

    /// Create a new flow logger with database support
    pub fn new_with_database(
        benchmark_id: String,
        agent_type: String,
        output_path: PathBuf,
        database: Arc<dyn DatabaseWriter>,
    ) -> Self {
        let session_id = uuid::Uuid::new_v4().to_string();
        let start_time = SystemTime::now();

        info!(
            session_id = %session_id,
            benchmark_id = %benchmark_id,
            agent_type = %agent_type,
            "Initializing flow logger with database support"
        );

        Self {
            session_id,
            benchmark_id,
            agent_type,
            start_time,
            events: Vec::new(),
            output_path,
            database: Some(database),
        }
    }

    /// Create a new flow logger with database support, preserving existing session_id
    pub fn new_with_database_preserve_session(
        benchmark_id: String,
        agent_type: String,
        output_path: PathBuf,
        database: Arc<dyn DatabaseWriter>,
        existing_session_id: Option<String>,
    ) -> Self {
        let session_id = existing_session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let start_time = SystemTime::now();

        info!(
            session_id = %session_id,
            benchmark_id = %benchmark_id,
            agent_type = %agent_type,
            "Initializing flow logger with database support, preserving session_id: {:?}",
            existing_session_id
        );

        Self {
            session_id,
            benchmark_id,
            agent_type,
            start_time,
            events: Vec::new(),
            output_path,
            database: Some(database),
        }
    }

    /// Set database on existing logger instance
    pub fn with_database(mut self, database: Arc<dyn DatabaseWriter>) -> Self {
        self.database = Some(database);
        self
    }

    /// Log an LLM request event
    pub fn log_llm_request(&mut self, content: LlmRequestContent, depth: u32) {
        let event = FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::LlmRequest,
            depth,
            content: EventContent {
                data: serde_json::to_value(content).unwrap_or_default(),
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
            },
        };
        self.events.push(event);
        debug!("Logged transaction execution");
    }

    /// Log an error event
    pub fn log_error(&mut self, content: ErrorContent, depth: u32) {
        let message = content.message.clone();
        let event = FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::Error,
            depth,
            content: EventContent {
                data: serde_json::to_value(content).unwrap_or_default(),
            },
        };
        self.events.push(event);
        debug!("Logged error: {}", message);
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

        // Save to database if available
        if let Some(database) = &self.database {
            info!("[FLOW] 🎯 Using database for session: {}", self.session_id);

            let db_flow_log = super::database::DBFlowLog::new(flow_log.clone());

            match database.insert_flow_log(&db_flow_log).await {
                Ok(flow_log_id) => {
                    // Insert agent performance data
                    let timestamp = chrono::Utc::now().to_rfc3339();
                    let execution_time_ms = super::utils::FlowUtils::calculate_duration(&flow_log)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(5000u64); // 5 seconds default
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
                    info!(
                        "[FLOW] 🔍 Looking up prompt MD5 for benchmark_id: {}",
                        flow_log.benchmark_id
                    );

                    let prompt_md5 = match database
                        .get_prompt_md5_by_benchmark_name(&flow_log.benchmark_id)
                        .await
                    {
                        Ok(Some(md5)) => {
                            info!(
                                "[FLOW] ✅ Found prompt MD5 for {}: {}",
                                flow_log.benchmark_id, md5
                            );
                            Some(md5)
                        }
                        Ok(None) => {
                            warn!(
                                "[FLOW] ❌ No prompt MD5 found for benchmark: {}",
                                flow_log.benchmark_id
                            );
                            None
                        }
                        Err(e) => {
                            error!(
                                "[FLOW] 💥 Error looking up prompt MD5 for {}: {}",
                                flow_log.benchmark_id, e
                            );
                            None
                        }
                    };

                    info!(
                        "[FLOW] 📝 Storing agent performance with prompt_md5: {:?}",
                        prompt_md5
                    );

                    let performance_data = AgentPerformanceData {
                        benchmark_id: flow_log.benchmark_id.clone(),
                        agent_type: flow_log.agent_type.clone(),
                        score,
                        final_status: final_status.to_string(),
                        execution_time_ms,
                        timestamp,
                        flow_log_id: Some(flow_log_id),
                        prompt_md5: prompt_md5.clone(),
                    };

                    if let Err(e) = database.insert_agent_performance(&performance_data).await {
                        error!(
                            "💥 Failed to insert agent performance for session {}: {}",
                            self.session_id, e
                        );
                    } else {
                        info!(
                            "✅ Successfully inserted agent performance for session {} with prompt_md5: {:?}",
                            self.session_id, prompt_md5
                        );
                    }
                }
                Err(e) => {
                    error!(
                        "💥 Failed to save flow log to database for session {}: {}",
                        self.session_id, e
                    );
                }
            }
        } else {
            warn!(
                "[FLOW] ⚠️ No database available for session: {} - logging to file only",
                self.session_id
            );
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

/// Initialize flow tracing with OpenTelemetry
pub fn init_flow_tracing(service_name: &str) -> FlowResult<()> {
    // Note: This is a simplified initialization
    // Full OpenTelemetry integration can be added later
    info!("Flow tracing initialized for service: {}", service_name);
    Ok(())
}
