//! Enhanced OpenTelemetry logging for structured tool call tracking
//!
//! This module provides enhanced logging capabilities for tool calls, prompts,
//! and flow execution with JSONL output format for analysis and visualization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, EnhancedOtelError>;

#[derive(Debug, thiserror::Error)]
pub enum EnhancedOtelError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Mutex error: {0}")]
    Mutex(String),

    #[error("EnhancedOtelLogger not initialized")]
    NotInitialized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Prompt,
    ToolInput,
    ToolOutput,
    StepComplete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptInfo {
    pub tool_name_list: Vec<String>,
    pub user_prompt: String,
    pub final_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputInfo {
    pub tool_name: String,
    pub tool_args: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutputInfo {
    pub success: bool,
    pub results: serde_json::Value,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingInfo {
    pub flow_timeuse_ms: u64,
    pub step_timeuse_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedToolCall {
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub reev_runner_version: String,
    pub reev_agent_version: String,
    pub event_type: EventType,
    pub prompt: Option<PromptInfo>,
    pub tool_input: Option<ToolInputInfo>,
    pub tool_output: Option<ToolOutputInfo>,
    pub timing: TimingInfo,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolExecutionStatus {
    Success,
    Error,
    Timeout,
}

/// Enhanced OpenTelemetry logger for structured JSONL output
pub struct EnhancedOtelLogger {
    session_id: String,
    log_file: String,
    file_writer: Arc<Mutex<std::fs::File>>,
    tool_calls: Arc<Mutex<Vec<EnhancedToolCall>>>,
}

impl EnhancedOtelLogger {
    /// Create a new enhanced otel logger
    pub fn new() -> Result<Self> {
        let session_id = Uuid::new_v4().to_string();
        let default_log_file = format!("logs/sessions/enhanced_otel_{session_id}.jsonl");
        let log_file = std::env::var("REEV_ENHANCED_OTEL_FILE").unwrap_or(default_log_file);

        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(&log_file).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;

        Ok(Self {
            session_id,
            log_file,
            file_writer: Arc::new(Mutex::new(file)),
            tool_calls: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Create logger with specific session ID
    pub fn with_session_id(session_id: String) -> Result<Self> {
        let default_log_file = format!("logs/sessions/enhanced_otel_{session_id}.jsonl");
        let log_file = std::env::var("REEV_ENHANCED_OTEL_FILE").unwrap_or(default_log_file);

        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(&log_file).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;

        Ok(Self {
            session_id,
            log_file,
            file_writer: Arc::new(Mutex::new(file)),
            tool_calls: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Log a tool call event
    pub fn log_tool_call(&self, tool_call: EnhancedToolCall) -> Result<()> {
        // Store in memory
        {
            let mut calls = self
                .tool_calls
                .lock()
                .map_err(|e| EnhancedOtelError::Mutex(e.to_string()))?;
            calls.push(tool_call.clone());
        }

        // Write to file as JSONL
        let json_line = serde_json::to_string(&tool_call)?;
        let mut writer = self
            .file_writer
            .lock()
            .map_err(|e| EnhancedOtelError::Mutex(e.to_string()))?;
        writeln!(writer, "{json_line}")?;
        writer.flush()?;

        Ok(())
    }

    /// Get all tool calls for this session
    pub fn get_tool_calls(&self) -> Result<Vec<EnhancedToolCall>> {
        let calls = self
            .tool_calls
            .lock()
            .map_err(|e| EnhancedOtelError::Mutex(e.to_string()))?;
        Ok(calls.clone())
    }

    /// Update an existing tool call (for completion)
    pub fn update_tool_call(
        &self,
        tool_name: &str,
        execution_time_ms: u64,
        output_result: serde_json::Value,
        status: ToolExecutionStatus,
    ) -> Result<()> {
        let mut calls = self
            .tool_calls
            .lock()
            .map_err(|e| EnhancedOtelError::Mutex(e.to_string()))?;

        // Find the most recent matching tool call
        if let Some(call) = calls.iter_mut().rev().find(|c| {
            c.tool_input
                .as_ref()
                .is_some_and(|input| input.tool_name == tool_name)
        }) {
            let success = matches!(status, ToolExecutionStatus::Success);
            call.tool_output = Some(ToolOutputInfo {
                success,
                results: output_result,
                error_message: None,
            });
            call.timing.step_timeuse_ms = execution_time_ms;

            // Write updated version
            let json_line = serde_json::to_string(call)?;
            let mut writer = self
                .file_writer
                .lock()
                .map_err(|e| EnhancedOtelError::Mutex(e.to_string()))?;
            writeln!(writer, "{json_line}")?;
            writer.flush()?;
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

    /// Write session summary
    pub fn write_summary(&self) -> Result<()> {
        let calls = self.get_tool_calls()?;
        let summary = serde_json::json!({
            "session_id": self.session_id,
            "total_events": calls.len(),
            "tool_calls": calls.iter().filter(|c| c.tool_input.is_some()).count(),
            "successful_tools": calls.iter().filter(|c| {
                c.tool_output.as_ref().is_some_and(|o| o.success)
            }).count(),
            "failed_tools": calls.iter().filter(|c| {
                c.tool_output.as_ref().is_some_and(|o| !o.success)
            }).count(),
            "logged_at": Utc::now(),
        });

        let mut writer = self
            .file_writer
            .lock()
            .map_err(|e| EnhancedOtelError::Mutex(e.to_string()))?;
        writeln!(writer, "{}", serde_json::to_string(&summary)?)?;
        writer.flush()?;

        Ok(())
    }
}

impl Drop for EnhancedOtelLogger {
    fn drop(&mut self) {
        let _ = self.write_summary();
    }
}

// Global logger instance
static ENHANCED_OTEL_LOGGER: std::sync::OnceLock<Arc<EnhancedOtelLogger>> =
    std::sync::OnceLock::new();

/// Initialize enhanced otel logging
pub fn init_enhanced_otel_logging() -> Result<String> {
    let logger = Arc::new(EnhancedOtelLogger::new()?);
    let session_id = logger.session_id().to_string();

    ENHANCED_OTEL_LOGGER
        .set(logger)
        .map_err(|_| EnhancedOtelError::Mutex("Logger already initialized".to_string()))?;

    tracing::info!(
        "✅ Enhanced OpenTelemetry logging initialized with session: {}",
        session_id
    );
    Ok(session_id)
}

/// Initialize enhanced otel logging with specific session ID
pub fn init_enhanced_otel_logging_with_session(session_id: String) -> Result<String> {
    let logger = Arc::new(EnhancedOtelLogger::with_session_id(session_id.clone())?);

    ENHANCED_OTEL_LOGGER
        .set(logger)
        .map_err(|_| EnhancedOtelError::Mutex("Logger already initialized".to_string()))?;

    tracing::info!(
        "✅ Enhanced OpenTelemetry logging initialized with session: {}",
        session_id
    );
    Ok(session_id)
}

/// Get the global enhanced otel logger
pub fn get_enhanced_otel_logger() -> Result<Arc<EnhancedOtelLogger>> {
    ENHANCED_OTEL_LOGGER
        .get()
        .cloned()
        .ok_or(EnhancedOtelError::NotInitialized)
}

// Simple working macros

/// Simple tool call logging macro
#[macro_export]
macro_rules! log_tool_call {
    ($tool_name:expr, $args:expr) => {{
        // Enhanced otel logging is enabled by default
        tracing::info!("🔍 [{}] log_tool_call macro called", $tool_name);
        tracing::info!(
            "🔍 [{}] REEV_ENHANCED_OTEL env: {:?}",
            $tool_name,
            std::env::var("REEV_ENHANCED_OTEL")
        );

        if std::env::var("REEV_ENHANCED_OTEL").unwrap_or_else(|_| "1".to_string()) != "0" {
            tracing::info!(
                "🔍 [{}] Enhanced logging enabled, getting logger",
                $tool_name
            );
            match $crate::enhanced_otel::get_enhanced_otel_logger() {
                Ok(logger) => {
                    tracing::info!("🔍 [{}] Logger obtained successfully", $tool_name);
                    let session_id = logger.session_id().to_string();
                    tracing::info!("🔍 [{}] Session ID: {}", $tool_name, session_id);

                    let tool_input = $crate::enhanced_otel::ToolInputInfo {
                        tool_name: $tool_name.to_string(),
                        tool_args: serde_json::to_value($args).unwrap_or_default(),
                    };

                    let tool_call = $crate::enhanced_otel::EnhancedToolCall {
                        timestamp: chrono::Utc::now(),
                        session_id,
                        reev_runner_version: env!("CARGO_PKG_VERSION").to_string(),
                        reev_agent_version: env!("CARGO_PKG_VERSION").to_string(),
                        event_type: $crate::enhanced_otel::EventType::ToolInput,
                        prompt: None,
                        tool_input: Some(tool_input),
                        tool_output: None,
                        timing: $crate::enhanced_otel::TimingInfo {
                            flow_timeuse_ms: 0,
                            step_timeuse_ms: 0,
                        },
                        metadata: serde_json::json!({}),
                    };

                    tracing::info!("🔍 [{}] Calling logger.log_tool_call", $tool_name);
                    match logger.log_tool_call(tool_call) {
                        Ok(()) => {
                            tracing::info!("✅ [{}] Tool call logged successfully", $tool_name);
                        }
                        Err(e) => {
                            tracing::warn!("❌ [{}] Failed to log tool call: {}", $tool_name, e);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("❌ [{}] Failed to get logger: {:?}", $tool_name, e);
                }
            }
        } else {
            tracing::info!("🔍 [{}] Enhanced logging disabled", $tool_name);
        }

        tracing::info!("[{}] Tool execution started", $tool_name);
    }};
}

/// Simple tool completion logging macro
#[macro_export]
macro_rules! log_tool_completion {
    ($tool_name:expr, $execution_time_ms:expr, $result:expr, $success:expr) => {{
        // Enhanced otel logging is enabled by default
        if std::env::var("REEV_ENHANCED_OTEL").unwrap_or_else(|_| "1".to_string()) != "0" {
            if let Ok(logger) = $crate::enhanced_otel::get_enhanced_otel_logger() {
                let session_id = logger.session_id().to_string();
                // Merge execution time into results
                let mut results = serde_json::to_value($result).unwrap_or_default();
                if let Some(obj) = results.as_object_mut() {
                    obj.insert(
                        "execution_time_ms".to_string(),
                        serde_json::Value::Number(serde_json::Number::from($execution_time_ms)),
                    );
                }

                let tool_output = $crate::enhanced_otel::ToolOutputInfo {
                    success: $success,
                    results,
                    error_message: if $success {
                        None
                    } else {
                        Some(format!("Tool failed: {:?}", $result))
                    },
                };

                let tool_call = $crate::enhanced_otel::EnhancedToolCall {
                    timestamp: chrono::Utc::now(),
                    session_id,
                    reev_runner_version: env!("CARGO_PKG_VERSION").to_string(),
                    reev_agent_version: env!("CARGO_PKG_VERSION").to_string(),
                    event_type: $crate::enhanced_otel::EventType::ToolOutput,
                    prompt: None,
                    tool_input: None,
                    tool_output: Some(tool_output),
                    timing: $crate::enhanced_otel::TimingInfo {
                        flow_timeuse_ms: 0,
                        step_timeuse_ms: $execution_time_ms,
                    },
                    metadata: serde_json::json!({}),
                };

                if let Err(e) = logger.log_tool_call(tool_call) {
                    tracing::warn!("❌ [{}] Failed to log tool completion: {}", $tool_name, e);
                } else {
                    tracing::debug!("✅ [{}] Tool completion logged", $tool_name);
                }
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
                "[{}] Tool execution failed in {}ms: {:?}",
                $tool_name,
                $execution_time_ms,
                $result
            );
        }
    }};
}

/// Simple prompt logging macro
#[macro_export]
macro_rules! log_prompt_event {
    ($tool_name_list:expr, $user_prompt:expr, $final_prompt:expr) => {
        // Enhanced otel logging is enabled by default
        if std::env::var("REEV_ENHANCED_OTEL").unwrap_or_else(|_| "1".to_string()) != "0" {
            if let Ok(logger) = $crate::enhanced_otel::get_enhanced_otel_logger() {
                let session_id = logger.session_id().to_string();
                let prompt = $crate::enhanced_otel::PromptInfo {
                    tool_name_list: $tool_name_list.to_vec(),
                    user_prompt: $user_prompt.to_string(),
                    final_prompt: if $final_prompt.len() > 500 {
                        format!(
                            "{}...[truncated, original size: {} bytes]",
                            &$final_prompt[..500.min($final_prompt.len())],
                            $final_prompt.len()
                        )
                    } else {
                        $final_prompt.to_string()
                    },
                };

                let tool_call = $crate::enhanced_otel::EnhancedToolCall {
                    timestamp: chrono::Utc::now(),
                    session_id,
                    reev_runner_version: env!("CARGO_PKG_VERSION").to_string(),
                    reev_agent_version: env!("CARGO_PKG_VERSION").to_string(),
                    event_type: $crate::enhanced_otel::EventType::Prompt,
                    prompt: Some(prompt),
                    tool_input: None,
                    tool_output: None,
                    timing: $crate::enhanced_otel::TimingInfo {
                        flow_timeuse_ms: 0,
                        step_timeuse_ms: 0,
                    },
                    metadata: serde_json::json!({}),
                };

                if let Err(e) = logger.log_tool_call(tool_call) {
                    tracing::warn!("❌ Failed to log prompt event: {}", e);
                }
            }
        }
    };
}

/// Simple step completion logging macro
#[macro_export]
macro_rules! log_step_complete {
    ($step_name:expr, $flow_time_ms:expr, $step_time_ms:expr) => {
        // Enhanced otel logging is enabled by default
        if std::env::var("REEV_ENHANCED_OTEL").unwrap_or_else(|_| "1".to_string()) != "0" {
            if let Ok(logger) = $crate::enhanced_otel::get_enhanced_otel_logger() {
                let session_id = logger.session_id().to_string();

                let tool_call = $crate::enhanced_otel::EnhancedToolCall {
                    timestamp: chrono::Utc::now(),
                    session_id,
                    reev_runner_version: env!("CARGO_PKG_VERSION").to_string(),
                    reev_agent_version: env!("CARGO_PKG_VERSION").to_string(),
                    event_type: $crate::enhanced_otel::EventType::StepComplete,
                    prompt: None,
                    tool_input: None,
                    tool_output: None,
                    timing: $crate::enhanced_otel::TimingInfo {
                        flow_timeuse_ms: $flow_time_ms,
                        step_timeuse_ms: $step_time_ms,
                    },
                    metadata: serde_json::json!({
                        "step_name": $step_name
                    }),
                };

                if let Err(e) = logger.log_tool_call(tool_call) {
                    tracing::warn!("❌ Failed to log step complete: {}", e);
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_otel_logger_creation() {
        let logger = EnhancedOtelLogger::new();
        assert!(logger.is_ok());
    }

    #[test]
    fn test_tool_call_logging() {
        let logger = EnhancedOtelLogger::new().unwrap();
        let tool_call = EnhancedToolCall {
            timestamp: Utc::now(),
            session_id: logger.session_id().to_string(),
            reev_runner_version: "0.1.0".to_string(),
            reev_agent_version: "0.1.0".to_string(),
            event_type: EventType::ToolInput,
            prompt: None,
            tool_input: Some(ToolInputInfo {
                tool_name: "test_tool".to_string(),
                tool_args: serde_json::json!({"param": "value"}),
            }),
            tool_output: None,
            timing: TimingInfo {
                flow_timeuse_ms: 100,
                step_timeuse_ms: 50,
            },
            metadata: serde_json::json!({}),
        };

        let result = logger.log_tool_call(tool_call);
        assert!(result.is_ok());
    }

    #[test]
    fn test_session_summary() {
        let logger = EnhancedOtelLogger::new().unwrap();
        let result = logger.write_summary();
        assert!(result.is_ok());
    }
}
