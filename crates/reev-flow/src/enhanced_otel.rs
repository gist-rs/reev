//! Enhanced OpenTelemetry logging for tool call tracking
//!
//! This module provides enhanced OpenTelemetry logging capabilities
//! that automatically capture tool execution details and write them
//! to unique session files in logs/sessions/.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, EnhancedOtelError>;

#[derive(Debug, Error)]
pub enum EnhancedOtelError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Mutex error: {0}")]
    Mutex(String),
    #[error("Logger not initialized")]
    NotInitialized,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use tracing::{info, warn};
use uuid::Uuid;

/// Enhanced tool call information captured from OpenTelemetry spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedToolCall {
    /// Unique session identifier
    pub session_id: String,
    /// Tool name (e.g., "sol_transfer", "jupiter_swap")
    pub tool_name: String,
    /// Tool execution timestamp
    pub timestamp: DateTime<Utc>,
    /// Tool execution duration in milliseconds
    pub execution_time_ms: u64,
    /// Tool input parameters
    pub input_params: serde_json::Value,
    /// Tool output result
    pub output_result: serde_json::Value,
    /// Tool execution status
    pub status: ToolExecutionStatus,
    /// Error message if execution failed
    pub error_message: Option<String>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Tool execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolExecutionStatus {
    Success,
    Error,
    Timeout,
}

/// Enhanced OpenTelemetry logger for tool calls
pub struct EnhancedOtelLogger {
    /// Session ID for this logging session
    session_id: String,
    /// Log file path
    log_file: String,
    /// Mutex for thread-safe file writing
    file_writer: Mutex<File>,
    /// Tool calls collected in this session
    tool_calls: Mutex<Vec<EnhancedToolCall>>,
}

impl EnhancedOtelLogger {
    /// Create new enhanced otel logger with unique session ID
    pub fn new() -> Result<Self> {
        let session_id = Uuid::new_v4().to_string();
        let log_file = format!("logs/sessions/otel_{session_id}.jsonl");

        // Ensure logs directory exists
        if let Some(parent) = Path::new(&log_file).parent() {
            std::fs::create_dir_all(parent).map_err(EnhancedOtelError::Io)?;
        }

        // Create/open log file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .map_err(EnhancedOtelError::Io)?;

        info!("Enhanced OpenTelemetry logging initialized");
        info!("Session ID: {}", session_id);
        info!("Log file: {}", log_file);

        Ok(Self {
            session_id,
            log_file,
            file_writer: Mutex::new(file),
            tool_calls: Mutex::new(Vec::new()),
        })
    }

    /// Create new enhanced otel logger with specific session ID
    pub fn with_session_id(session_id: String) -> Result<Self> {
        let log_file = format!("logs/sessions/otel_{session_id}.jsonl");

        // Ensure logs directory exists
        if let Some(parent) = Path::new(&log_file).parent() {
            std::fs::create_dir_all(parent).map_err(EnhancedOtelError::Io)?;
        }

        // Create/open log file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .map_err(EnhancedOtelError::Io)?;

        info!(
            "Enhanced OpenTelemetry logging initialized with session ID: {}",
            session_id
        );
        info!("Log file: {}", log_file);

        Ok(Self {
            session_id,
            log_file,
            file_writer: Mutex::new(file),
            tool_calls: Mutex::new(Vec::new()),
        })
    }

    /// Log a tool call with enhanced details
    pub fn log_tool_call(&self, tool_call: EnhancedToolCall) -> Result<()> {
        // Add to memory collection
        {
            let mut calls = self
                .tool_calls
                .lock()
                .map_err(|e| EnhancedOtelError::Mutex(e.to_string()))?;
            calls.push(tool_call.clone());
        }

        // Write to file
        {
            let mut writer = self
                .file_writer
                .lock()
                .map_err(|e| EnhancedOtelError::Mutex(e.to_string()))?;

            let json_line =
                serde_json::to_string(&tool_call).map_err(EnhancedOtelError::Serialization)?;

            writeln!(writer, "{json_line}").map_err(EnhancedOtelError::Io)?;

            writer.flush().map_err(EnhancedOtelError::Io)?;
        }

        info!(
            "Logged tool call: {} ({}ms) - {}",
            tool_call.tool_name,
            tool_call.execution_time_ms,
            match tool_call.status {
                ToolExecutionStatus::Success => "SUCCESS",
                ToolExecutionStatus::Error => "ERROR",
                ToolExecutionStatus::Timeout => "TIMEOUT",
            }
        );

        Ok(())
    }

    /// Get all collected tool calls
    pub fn get_tool_calls(&self) -> Result<Vec<EnhancedToolCall>> {
        let calls = self
            .tool_calls
            .lock()
            .map_err(|e| EnhancedOtelError::Mutex(e.to_string()))?;
        Ok(calls.clone())
    }

    /// Update existing tool call with completion data
    pub fn update_tool_call(
        &self,
        tool_name: &str,
        execution_time_ms: u64,
        output_result: serde_json::Value,
        status: ToolExecutionStatus,
    ) -> Result<()> {
        // Update in memory collection
        {
            let mut calls = self
                .tool_calls
                .lock()
                .map_err(|e| EnhancedOtelError::Mutex(e.to_string()))?;

            // Find the most recent matching tool call
            if let Some(call) = calls.iter_mut().rev().find(|c| {
                c.tool_name == tool_name && matches!(c.status, ToolExecutionStatus::Success)
            }) {
                call.execution_time_ms = execution_time_ms;
                call.output_result = output_result.clone();
                call.status = status.clone();
            } else {
                // If no matching call found, create a new one
                let new_call = EnhancedToolCall {
                    session_id: self.session_id().to_string(),
                    tool_name: tool_name.to_string(),
                    timestamp: chrono::Utc::now(),
                    execution_time_ms,
                    input_params: serde_json::json!({}),
                    output_result: output_result.clone(),
                    status: status.clone(),
                    error_message: None,
                    metadata: serde_json::json!({}),
                };
                calls.push(new_call);
            }
        }

        // Append update to file as new entry (append-only pattern)
        {
            let mut writer = self
                .file_writer
                .lock()
                .map_err(|e| EnhancedOtelError::Mutex(e.to_string()))?;

            let update_call = EnhancedToolCall {
                session_id: self.session_id().to_string(),
                tool_name: tool_name.to_string(),
                timestamp: chrono::Utc::now(),
                execution_time_ms,
                input_params: serde_json::json!({}),
                output_result: output_result.clone(),
                status: status.clone(),
                error_message: None,
                metadata: serde_json::json!({}),
            };

            let json_line =
                serde_json::to_string(&update_call).map_err(EnhancedOtelError::Serialization)?;

            writeln!(writer, "{json_line}").map_err(EnhancedOtelError::Io)?;
            writer.flush().map_err(EnhancedOtelError::Io)?;
        }

        Ok(())
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get log file path
    pub fn log_file(&self) -> &str {
        &self.log_file
    }

    /// Write summary statistics to file
    pub fn write_summary(&self) -> Result<()> {
        let calls = self.get_tool_calls()?;

        let tools_used = {
            let mut tools = std::collections::HashSet::new();
            for call in &calls {
                tools.insert(&call.tool_name);
            }
            tools.into_iter().collect::<Vec<_>>()
        };

        let average_execution_time_ms = if !calls.is_empty() {
            calls.iter().map(|c| c.execution_time_ms).sum::<u64>() as f64 / calls.len() as f64
        } else {
            0.0
        };

        let summary = serde_json::json!({
            "session_id": self.session_id,
            "timestamp": Utc::now(),
            "total_tool_calls": calls.len(),
            "successful_calls": calls.iter().filter(|c| matches!(c.status, ToolExecutionStatus::Success)).count(),
            "failed_calls": calls.iter().filter(|c| matches!(c.status, ToolExecutionStatus::Error)).count(),
            "timeout_calls": calls.iter().filter(|c| matches!(c.status, ToolExecutionStatus::Timeout)).count(),
            "average_execution_time_ms": average_execution_time_ms,
            "tools_used": tools_used
        });

        {
            let mut writer = self
                .file_writer
                .lock()
                .map_err(|e| EnhancedOtelError::Mutex(e.to_string()))?;

            writeln!(
                writer,
                "\n# SESSION_SUMMARY: {}",
                serde_json::to_string(&summary)?
            )
            .map_err(EnhancedOtelError::Io)?;

            writer.flush().map_err(EnhancedOtelError::Io)?;
        }

        info!("Session summary written to: {}", self.log_file);
        Ok(())
    }
}

impl Drop for EnhancedOtelLogger {
    fn drop(&mut self) {
        if let Err(e) = self.write_summary() {
            warn!("Failed to write session summary: {}", e);
        }
    }
}

/// Global enhanced otel logger instance
static ENHANCED_OTEL_LOGGER: OnceLock<EnhancedOtelLogger> = OnceLock::new();

/// Initialize enhanced OpenTelemetry logging globally
pub fn init_enhanced_otel_logging() -> Result<String> {
    info!("=== INITIALIZING ENHANCED OPENTELEMETRY LOGGING ===");

    // Use OnceLock's set method which returns error if already set
    let logger = EnhancedOtelLogger::new()?;
    let log_file = logger.log_file().to_string();

    ENHANCED_OTEL_LOGGER
        .set(logger)
        .map_err(|_| EnhancedOtelError::Mutex("Logger already initialized".to_string()))?;

    info!("✅ Enhanced OpenTelemetry logging initialized globally");
    info!("📁 Log file: {}", log_file);

    Ok(log_file)
}

/// Initialize enhanced otel logger with specific session ID
pub fn init_enhanced_otel_logging_with_session(session_id: String) -> Result<String> {
    info!(
        "=== INITIALIZING ENHANCED OPENTELEMETRY LOGGING WITH SESSION: {} ===",
        session_id
    );

    let logger = EnhancedOtelLogger::with_session_id(session_id)?;
    let log_file = logger.log_file().to_string();

    ENHANCED_OTEL_LOGGER
        .set(logger)
        .map_err(|_| EnhancedOtelError::Mutex("Logger already initialized".to_string()))?;

    info!("✅ Enhanced OpenTelemetry logging initialized with session");

    Ok(log_file)
}

/// Get the global enhanced otel logger
pub fn get_enhanced_otel_logger() -> Result<&'static EnhancedOtelLogger> {
    ENHANCED_OTEL_LOGGER
        .get()
        .ok_or(EnhancedOtelError::NotInitialized)
}

/// Enhanced logging macro for tool calls
#[macro_export]
macro_rules! log_enhanced_tool_call {
    ($tool_name:expr, $execution_time_ms:expr, $input_params:expr, $output_result:expr, $status:expr, $error_message:expr) => {
        if let Ok(logger) = $crate::enhanced_otel::get_enhanced_otel_logger() {
            let tool_call = $crate::enhanced_otel::EnhancedToolCall {
                session_id: logger.session_id().to_string(),
                tool_name: $tool_name.to_string(),
                timestamp: chrono::Utc::now(),
                execution_time_ms: $execution_time_ms,
                input_params: $input_params.clone(),
                output_result: $output_result.clone(),
                status: $status,
                error_message: $error_message.map(|s| s.to_string()),
                metadata: serde_json::json!({
                    "logged_at": chrono::Utc::now().to_rfc3339(),
                    "tool_type": $tool_name,
                    "logger_version": "1.0.0",
                    "hostname": std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string()),
                    "pid": std::process::id(),
                    "consolidation": "enabled"
                }),
            };

            if let Err(e) = logger.log_tool_call(tool_call) {
                tracing::warn!("Failed to log enhanced tool call: {}", e);
            }
        }
    };
}

/// Success variant of the macro
#[macro_export]
macro_rules! log_enhanced_tool_success {
    ($tool_name:expr, $execution_time_ms:expr, $input_params:expr, $output_result:expr) => {
        $crate::log_enhanced_tool_call!(
            $tool_name,
            $execution_time_ms,
            $input_params,
            $output_result,
            $crate::enhanced_otel::ToolExecutionStatus::Success,
            None::<&str>
        );
    };
}

/// Error variant of the macro
#[macro_export]
macro_rules! log_enhanced_tool_error {
    ($tool_name:expr, $execution_time_ms:expr, $input_params:expr, $error_message:expr) => {
        $crate::log_enhanced_tool_call!(
            $tool_name,
            $execution_time_ms,
            $input_params,
            serde_json::json!({}),
            $crate::enhanced_otel::ToolExecutionStatus::Error,
            Some($error_message)
        );
    };
}

/// Update existing tool call with completion data
#[allow(dead_code)]
macro_rules! log_enhanced_tool_update {
    ($tool_name:expr, $execution_time_ms:expr, $output_result:expr, $status:expr) => {
        if let Ok(logger) = $crate::enhanced_otel::get_enhanced_otel_logger() {
            logger
                .update_tool_call($tool_name, $execution_time_ms, $output_result, $status)
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to update tool call {}: {}", $tool_name, e);
                });
        }
    };
}

/// Update tool call with success status
#[allow(dead_code)]
macro_rules! log_enhanced_tool_success_update {
    ($tool_name:expr, $execution_time_ms:expr, $output_result:expr) => {
        $crate::log_enhanced_tool_update!(
            $tool_name,
            $execution_time_ms,
            $output_result,
            $crate::enhanced_otel::ToolExecutionStatus::Success
        );
    };
}

/// Update tool call with error status
#[allow(dead_code)]
macro_rules! log_enhanced_tool_error_update {
    ($tool_name:expr, $execution_time_ms:expr, $error_message:expr) => {
        $crate::log_enhanced_tool_update!(
            $tool_name,
            $execution_time_ms,
            serde_json::json!({"error": $error_message}),
            $crate::enhanced_otel::ToolExecutionStatus::Error
        );
    };
}

/// Enhanced tool logging macro for consistent OpenTelemetry tracking
#[macro_export]
macro_rules! log_tool_call {
    ($tool_name:expr, $args:expr) => {
        // Enhanced otel logging is enabled by default (can be disabled with REEV_ENHANCED_OTEL=0)
        if std::env::var("REEV_ENHANCED_OTEL").unwrap_or_else(|_| "1".to_string()) != "0" {
            tracing::info!("🔧 [{}] Enhanced otel logging ENABLED", $tool_name);

            // Also log to enhanced file-based system
            let input_params = serde_json::to_value($args)
                .unwrap_or_else(|_| serde_json::Value::Object(Default::default()));
            tracing::info!(
                "📝 [{}] Attempting to log to enhanced otel system",
                $tool_name
            );

            // Check if EnhancedOtelLogger is available before trying to log
            if let Ok(logger) = $crate::enhanced_otel::get_enhanced_otel_logger() {
                tracing::info!(
                    "🔍 [{}] EnhancedOtelLogger found with session_id: {}",
                    $tool_name,
                    logger.session_id()
                );
                $crate::log_enhanced_tool_call!(
                    $tool_name,
                    0, // Will be updated on completion
                    input_params,
                    serde_json::Value::Object(Default::default()),
                    $crate::enhanced_otel::ToolExecutionStatus::Success,
                    None::<&str>
                );
                tracing::info!("✅ [{}] Enhanced otel log call completed", $tool_name);
            } else {
                tracing::warn!(
                    "❌ [{}] EnhancedOtelLogger NOT AVAILABLE - tool calls will not be captured!",
                    $tool_name
                );
            }
        } else {
            tracing::info!("🚫 [{}] Enhanced otel logging DISABLED", $tool_name);
        }
        tracing::info!("[{}] Tool execution started", $tool_name);
    };
}

/// Enhanced tool completion logging macro
#[macro_export]
macro_rules! log_tool_completion {
    ($tool_name:expr, $execution_time_ms:expr, $result:expr, $success:expr) => {
        // Enhanced otel logging is enabled by default (can be disabled with REEV_ENHANCED_OTEL=0)
        if std::env::var("REEV_ENHANCED_OTEL").unwrap_or_else(|_| "1".to_string()) != "0" {
            // Also log to enhanced file-based system
            let input_params = serde_json::json!({}); // Will be populated from earlier call
            if $success {
                // Convert result to JSON value for enhanced logging
                let result_value = serde_json::to_value(&$result).unwrap_or_default();
                $crate::log_enhanced_tool_success!(
                    $tool_name,
                    $execution_time_ms,
                    input_params,
                    result_value
                );
            } else {
                // Handle error case - convert result to string for error message
                let error_msg = $result.to_string();
                $crate::log_enhanced_tool_error!(
                    $tool_name,
                    $execution_time_ms,
                    input_params,
                    error_msg
                );
            }
        }
        if $success {
            tracing::info!(
                "[{}] Tool execution completed in {}ms",
                $tool_name,
                $execution_time_ms
            );
        } else {
            tracing::error!(
                "[{}] Tool execution failed in {}ms: {}",
                $tool_name,
                $execution_time_ms,
                serde_json::to_value($result).unwrap_or_default()
            );
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_otel_logger_creation() -> Result<()> {
        let logger = EnhancedOtelLogger::new()?;
        assert!(!logger.session_id().is_empty());
        assert!(logger.log_file().contains("otel_"));
        assert!(logger.log_file().ends_with(".json"));
        Ok(())
    }

    #[test]
    fn test_tool_call_logging() -> Result<()> {
        let logger = EnhancedOtelLogger::new()?;

        let tool_call = EnhancedToolCall {
            session_id: logger.session_id().to_string(),
            tool_name: "test_tool".to_string(),
            timestamp: Utc::now(),
            execution_time_ms: 100,
            input_params: serde_json::json!({"param1": "value1"}),
            output_result: serde_json::json!({"result": "success"}),
            status: ToolExecutionStatus::Success,
            error_message: None,
            metadata: serde_json::json!({}),
        };

        logger.log_tool_call(tool_call)?;

        let calls = logger.get_tool_calls()?;
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool_name, "test_tool");
        assert_eq!(calls[0].execution_time_ms, 100);

        Ok(())
    }

    #[test]
    fn test_session_summary() -> Result<()> {
        let logger = EnhancedOtelLogger::new()?;

        // Log some test calls
        for i in 0..3 {
            let tool_call = EnhancedToolCall {
                session_id: logger.session_id().to_string(),
                tool_name: format!("tool_{i}"),
                timestamp: Utc::now(),
                execution_time_ms: 100 + i * 50,
                input_params: serde_json::json!({"id": i}),
                output_result: serde_json::json!({"result": "ok"}),
                status: if i < 2 {
                    ToolExecutionStatus::Success
                } else {
                    ToolExecutionStatus::Error
                },
                error_message: if i < 2 {
                    None
                } else {
                    Some("test error".to_string())
                },
                metadata: serde_json::json!({}),
            };
            logger.log_tool_call(tool_call)?;
        }

        logger.write_summary()?;

        let calls = logger.get_tool_calls()?;
        assert_eq!(calls.len(), 3);

        Ok(())
    }
}
