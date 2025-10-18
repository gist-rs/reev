//! Unified Session File Logger
//!
//! Simple file-based logging system for agent execution sessions.
//! Replaces complex FlowLogger with structured JSON logging and database persistence.

use anyhow::{Context, Result};

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

// Import ExecutionTrace for ASCII tree compatibility
use crate::trace::ExecutionTrace;

/// Session event types for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEventType {
    /// LLM request event
    LlmRequest,
    /// Tool call event
    ToolCall,
    /// Tool result event
    ToolResult,
    /// Transaction execution event
    TransactionExecution,
    /// Error event
    Error,
    /// Session start
    SessionStart,
    /// Session end
    SessionEnd,
}

/// Individual session event with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    /// Unix timestamp
    pub timestamp: u64,
    /// Event type
    pub event_type: SessionEventType,
    /// Event depth for nested operations
    pub depth: u32,
    /// Event data as JSON value
    pub data: serde_json::Value,
}

/// Complete session log with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLog {
    /// Unique session identifier
    pub session_id: String,
    /// Benchmark identifier
    pub benchmark_id: String,
    /// Agent type
    pub agent_type: String,
    /// Session start time (Unix timestamp)
    pub start_time: u64,
    /// Session end time (Unix timestamp, optional)
    pub end_time: Option<u64>,
    /// All session events
    pub events: Vec<SessionEvent>,
    /// Final execution result
    pub final_result: Option<ExecutionResult>,
    /// Session metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Final execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether execution was successful
    pub success: bool,
    /// Final score (0.0 to 1.0)
    pub score: f64,
    /// Final status message
    pub status: String,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Additional result data
    pub data: serde_json::Value,
}

/// Simple file-based session logger
pub struct SessionFileLogger {
    session_id: String,
    benchmark_id: String,
    agent_type: String,
    start_time: SystemTime,
    log_file: PathBuf,
    events: Vec<SessionEvent>,
    metadata: std::collections::HashMap<String, String>,
}

impl SessionFileLogger {
    /// Create a new session file logger
    pub fn new(
        session_id: String,
        benchmark_id: String,
        agent_type: String,
        log_dir: &Path,
    ) -> Result<Self> {
        // Ensure log directory exists
        std::fs::create_dir_all(log_dir)
            .with_context(|| format!("Failed to create log directory: {log_dir:?}"))?;

        // Create log file path
        let filename = format!("session_{session_id}.json");
        let log_file = log_dir.join(filename);

        info!(
            session_id = %session_id,
            benchmark_id = %benchmark_id,
            agent_type = %agent_type,
            log_file = %log_file.display(),
            "Initializing session file logger"
        );

        Ok(Self {
            session_id,
            benchmark_id,
            agent_type,
            start_time: SystemTime::now(),
            log_file,
            events: Vec::new(),
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Add metadata to the session
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Log an event to the session
    pub fn log_event(&mut self, event_type: SessionEventType, depth: u32, data: serde_json::Value) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let event = SessionEvent {
            timestamp,
            event_type: event_type.clone(),
            depth,
            data,
        };

        self.events.push(event);
        debug!(
            session_id = %self.session_id,
            event_type = ?event_type,
            "Logged session event"
        );
    }

    /// Log LLM request
    pub fn log_llm_request(&mut self, content: serde_json::Value, depth: u32) {
        self.log_event(SessionEventType::LlmRequest, depth, content);
    }

    /// Log tool call
    pub fn log_tool_call(&mut self, content: serde_json::Value, depth: u32) {
        self.log_event(SessionEventType::ToolCall, depth, content);
    }

    /// Log tool result
    pub fn log_tool_result(&mut self, content: serde_json::Value, depth: u32) {
        self.log_event(SessionEventType::ToolResult, depth, content);
    }

    /// Log transaction execution
    pub fn log_transaction(&mut self, content: serde_json::Value, depth: u32) {
        self.log_event(SessionEventType::TransactionExecution, depth, content);
    }

    /// Log error
    pub fn log_error(&mut self, content: serde_json::Value, depth: u32) {
        self.log_event(SessionEventType::Error, depth, content);
    }

    /// Complete the session and write to file
    pub fn complete(self, result: ExecutionResult) -> Result<PathBuf> {
        let end_time = SystemTime::now();
        let end_timestamp = end_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let start_timestamp = self
            .start_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let session_log = SessionLog {
            session_id: self.session_id.clone(),
            benchmark_id: self.benchmark_id.clone(),
            agent_type: self.agent_type.clone(),
            start_time: start_timestamp,
            end_time: Some(end_timestamp),
            events: self.events.clone(),
            final_result: Some(result),
            metadata: self.metadata,
        };

        // Write session log to file
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.log_file)
            .with_context(|| format!("Failed to open log file: {:?}", self.log_file))?;

        let mut writer = BufWriter::new(file);
        let json_content = serde_json::to_string_pretty(&session_log)
            .with_context(|| "Failed to serialize session log")?;

        writer
            .write_all(json_content.as_bytes())
            .with_context(|| "Failed to write session log to file")?;
        writer
            .flush()
            .with_context(|| "Failed to flush session log file")?;

        info!(
            session_id = %self.session_id,
            log_file = %self.log_file.display(),
            events_count = self.events.len(),
            "Session log completed and written to file"
        );

        Ok(self.log_file)
    }

    /// Complete the session with ExecutionTrace for ASCII tree compatibility
    pub fn complete_with_trace(self, trace: ExecutionTrace) -> Result<PathBuf> {
        let end_time = SystemTime::now();
        let end_timestamp = end_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let start_timestamp = self
            .start_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Create session log with ExecutionTrace embedded in final_result
        let session_log = SessionLog {
            session_id: self.session_id.clone(),
            benchmark_id: self.benchmark_id.clone(),
            agent_type: self.agent_type.clone(),
            start_time: start_timestamp,
            end_time: Some(end_timestamp),
            events: self.events.clone(),
            final_result: Some(ExecutionResult {
                success: trace
                    .steps
                    .iter()
                    .any(|step| step.observation.last_transaction_status == "Success"),
                score: if trace
                    .steps
                    .iter()
                    .any(|step| step.observation.last_transaction_status == "Success")
                {
                    1.0
                } else {
                    0.0
                },
                status: if trace
                    .steps
                    .iter()
                    .any(|step| step.observation.last_transaction_status == "Success")
                {
                    "Succeeded".to_string()
                } else {
                    "Failed".to_string()
                },
                execution_time_ms: (end_timestamp - start_timestamp) * 1000,
                data: serde_json::to_value(trace).unwrap_or_default(),
            }),
            metadata: self.metadata,
        };

        // Write session log to file
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.log_file)
            .with_context(|| format!("Failed to open log file: {:?}", self.log_file))?;

        let mut writer = BufWriter::new(file);
        let json_content = serde_json::to_string_pretty(&session_log)
            .with_context(|| "Failed to serialize session log with ExecutionTrace")?;

        writer
            .write_all(json_content.as_bytes())
            .with_context(|| "Failed to write session log with ExecutionTrace to file")?;
        writer
            .flush()
            .with_context(|| "Failed to flush session log with ExecutionTrace file")?;

        info!(
            session_id = %self.session_id,
            log_file = %self.log_file.display(),
            events_count = self.events.len(),
            "Session log with ExecutionTrace completed and written to file"
        );

        Ok(self.log_file)
    }

    /// Get session statistics
    pub fn get_statistics(&self) -> SessionStatistics {
        let mut stats = SessionStatistics::default();

        for event in &self.events {
            match event.event_type {
                SessionEventType::LlmRequest => stats.llm_requests += 1,
                SessionEventType::ToolCall => stats.tool_calls += 1,
                SessionEventType::ToolResult => stats.tool_results += 1,
                SessionEventType::TransactionExecution => stats.transactions += 1,
                SessionEventType::Error => stats.errors += 1,
                _ => {}
            }
        }

        stats.max_depth = self.events.iter().map(|e| e.depth).max().unwrap_or(0);
        stats.total_events = self.events.len();

        stats
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get benchmark ID
    pub fn benchmark_id(&self) -> &str {
        &self.benchmark_id
    }

    /// Get agent type
    pub fn agent_type(&self) -> &str {
        &self.agent_type
    }
}

/// Session statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SessionStatistics {
    /// Total number of events
    pub total_events: usize,
    /// Number of LLM requests
    pub llm_requests: usize,
    /// Number of tool calls
    pub tool_calls: usize,
    /// Number of tool results
    pub tool_results: usize,
    /// Number of transactions
    pub transactions: usize,
    /// Number of errors
    pub errors: usize,
    /// Maximum depth reached
    pub max_depth: u32,
}

/// Load session log from file
pub fn load_session_log(file_path: &Path) -> Result<SessionLog> {
    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read session log file: {file_path:?}"))?;

    let session_log: SessionLog = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse session log from: {file_path:?}"))?;

    Ok(session_log)
}

/// Convert legacy FlowLogger events to SessionEvent format
pub fn convert_legacy_flow_event(legacy_event: &serde_json::Value) -> Result<SessionEvent> {
    let event_type_str = legacy_event
        .get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let event_type = match event_type_str {
        "LlmRequest" => SessionEventType::LlmRequest,
        "ToolCall" => SessionEventType::ToolCall,
        "ToolResult" => SessionEventType::ToolResult,
        "TransactionExecution" => SessionEventType::TransactionExecution,
        "Error" => SessionEventType::Error,
        _ => SessionEventType::Error, // Default unknown events to errors
    };

    let timestamp = legacy_event
        .get("timestamp")
        .and_then(|v| v.as_u64())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });

    let depth = legacy_event
        .get("depth")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let data = legacy_event
        .get("content")
        .and_then(|v| v.get("data"))
        .cloned()
        .unwrap_or_else(|| legacy_event.clone());

    Ok(SessionEvent {
        timestamp,
        event_type,
        depth,
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_session_file_logger_basic() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut logger = SessionFileLogger::new(
            "test-session".to_string(),
            "test-benchmark".to_string(),
            "test-agent".to_string(),
            temp_dir.path(),
        )?;

        logger.add_metadata("test_key".to_string(), "test_value".to_string());
        logger.log_llm_request(serde_json::json!({"prompt": "test"}), 0);
        logger.log_tool_call(serde_json::json!({"tool": "test"}), 1);

        let result = ExecutionResult {
            success: true,
            score: 0.8,
            status: "completed".to_string(),
            execution_time_ms: 1000,
            data: serde_json::json!({}),
        };

        let log_file = logger.complete(result)?;

        // Verify file was created and contains valid JSON
        assert!(log_file.exists());
        let loaded_log = load_session_log(&log_file)?;
        assert_eq!(loaded_log.session_id, "test-session");
        assert_eq!(loaded_log.events.len(), 2);
        assert_eq!(
            loaded_log.metadata.get("test_key"),
            Some(&"test_value".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_session_statistics() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut logger = SessionFileLogger::new(
            "stats-test".to_string(),
            "test-benchmark".to_string(),
            "test-agent".to_string(),
            temp_dir.path(),
        )?;

        logger.log_llm_request(serde_json::json!({}), 0);
        logger.log_tool_call(serde_json::json!({}), 1);
        logger.log_tool_result(serde_json::json!({}), 1);
        logger.log_error(serde_json::json!({}), 2);

        let stats = logger.get_statistics();
        assert_eq!(stats.total_events, 4);
        assert_eq!(stats.llm_requests, 1);
        assert_eq!(stats.tool_calls, 1);
        assert_eq!(stats.tool_results, 1);
        assert_eq!(stats.errors, 1);
        assert_eq!(stats.max_depth, 2);

        Ok(())
    }
}
