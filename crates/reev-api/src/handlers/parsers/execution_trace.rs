//! Execution trace parser for converting execution data to ASCII format
//!
//! This module provides functionality to parse execution data from various sources
//! and convert them into human-readable ASCII trace formats. It supports:
//! - Session data from completed executions with step-by-step actions
//! - Session logs from database
//! - Various execution result formats
//!
//! The parser integrates with FlowLog system to generate structured
//! ASCII trees showing the execution flow with proper hierarchy and visual indicators.

use reev_lib::flow::{EventContent, FlowEventType, FlowLog, FlowLogRenderer};
use reev_lib::flow::{ExecutionResult, ExecutionStatistics};
use reev_types::ExecutionState;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Trace format options
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TraceFormat {
    /// ASCII tree format with Unicode characters
    AsciiTree,
    /// Plain text format
    #[allow(dead_code)]
    PlainText,
}

/// Execution trace parser
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExecutionTraceParser {
    /// Default format for trace generation
    default_format: TraceFormat,
}

impl ExecutionTraceParser {
    /// Create a new execution trace parser
    pub fn new() -> Self {
        Self {
            default_format: TraceFormat::AsciiTree,
        }
    }

    /// Create parser with specific default format
    #[allow(dead_code)]
    pub fn with_format(format: TraceFormat) -> Self {
        Self {
            default_format: format,
        }
    }

    /// Generate trace from execution state
    pub async fn generate_trace_from_state(
        &self,
        state: &ExecutionState,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        debug!(
            "Generating trace for execution_id: {}, status: {:?}",
            state.execution_id, state.status
        );

        // If we have result data, parse it as session data
        if let Some(ref result_data) = state.result_data {
            self.generate_trace_from_session_data_with_state(result_data, state)
                .await
        } else {
            // Try to generate a trace from the status and metadata
            self.generate_trace_from_metadata(state).await
        }
    }

    /// Generate trace from session data format
    pub async fn generate_trace_from_session_data(
        &self,
        result_data: &Value,
        execution_id: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Generating trace from session data for: {}", execution_id);

        // Create a dummy ExecutionState for this data
        let dummy_state = ExecutionState::new(
            execution_id.to_string(),
            "unknown".to_string(),
            "unknown".to_string(),
        );
        self.generate_trace_from_session_data_with_state(result_data, &dummy_state)
            .await
    }

    /// Generate trace from session data with execution state context
    pub async fn generate_trace_from_session_data_with_state(
        &self,
        result_data: &Value,
        state: &ExecutionState,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        debug!(
            "Generating trace from session data with state context for: {}",
            state.execution_id
        );

        // Extract session information from result data
        // Handle both direct data and nested result.final_result.data structures
        let data_source = if result_data.get("final_result").is_some() {
            result_data
                .get("final_result")
                .and_then(|fr| fr.get("data"))
                .unwrap_or(result_data)
        } else {
            result_data
        };

        let session_id = data_source
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| &state.execution_id);

        let benchmark_id = data_source
            .get("benchmark_id")
            .or_else(|| result_data.get("benchmark_id"))
            .and_then(|v| v.as_str())
            .unwrap_or(state.benchmark_id.as_str());

        let agent_type = data_source
            .get("agent_type")
            .or_else(|| result_data.get("agent_type"))
            .and_then(|v| v.as_str())
            .unwrap_or(state.agent.as_str());

        let start_time = std::time::SystemTime::now();
        let end_time = start_time + std::time::Duration::from_secs(60); // Default 1 minute

        // Create FlowLog from session data
        let mut flow_log = FlowLog {
            session_id: session_id.to_string(),
            benchmark_id: benchmark_id.to_string(),
            agent_type: agent_type.to_string(),
            start_time,
            end_time: Some(end_time),
            events: Vec::new(),
            final_result: None, // Will be set below if data available
        };

        // Extract events from session data - convert steps to events
        let steps_source = if result_data.get("final_result").is_some() {
            result_data
                .get("final_result")
                .and_then(|fr| fr.get("data"))
                .and_then(|d| d.get("steps"))
                .unwrap_or(&serde_json::Value::Null)
        } else {
            result_data.get("steps").unwrap_or(&serde_json::Value::Null)
        };

        if let Some(steps) = steps_source.as_array() {
            for (i, step_data) in steps.iter().enumerate() {
                let timestamp = (i as u64) * 1000; // Simple timestamp based on step order

                // Create LLM Request event for each step
                let flow_event = reev_lib::flow::FlowEvent {
                    timestamp: std::time::SystemTime::UNIX_EPOCH
                        + std::time::Duration::from_millis(timestamp),
                    event_type: FlowEventType::LlmRequest,
                    depth: i as u32,
                    content: EventContent {
                        data: serde_json::json!({
                            "model": agent_type,
                            "context_tokens": 1000,
                            "step_index": i
                        }),
                    },
                };

                flow_log.events.push(flow_event);

                // Create Tool Call event for each action
                if let Some(action) = step_data.get("action").and_then(|v| v.as_array()) {
                    if !action.is_empty() {
                        let action_item = &action[0]; // Use first action item
                        let (action_details, data_value) = Self::parse_action_details(action_item);

                        let tool_event = reev_lib::flow::FlowEvent {
                            timestamp: std::time::SystemTime::UNIX_EPOCH
                                + std::time::Duration::from_millis(timestamp + 500),
                            event_type: FlowEventType::ToolCall,
                            depth: i as u32 + 1,
                            content: EventContent {
                                data: serde_json::json!({
                                    "tool_name": reev_constants::EXECUTE_TRANSACTION,
                                    "tool_args": format!("Step {} action", i + 1),
                                    "action_details": action_details,
                                    "data": data_value
                                }),
                            },
                        };

                        flow_log.events.push(tool_event);

                        // Create Tool Result event
                        let result_event = reev_lib::flow::FlowEvent {
                            timestamp: std::time::SystemTime::UNIX_EPOCH
                                + std::time::Duration::from_millis(timestamp + 1000),
                            event_type: FlowEventType::ToolResult,
                            depth: i as u32 + 1,
                            content: EventContent {
                                data: serde_json::json!({
                                    "tool_name": reev_constants::EXECUTE_TRANSACTION,
                                    "result_status": "success",
                                    "result_data": action
                                }),
                            },
                        };

                        flow_log.events.push(result_event);
                    }
                }
            }
        }

        // Extract final result - handle nested structure
        let (success, score) = if let Some(final_result) = result_data.get("final_result") {
            (
                final_result
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                final_result
                    .get("score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0),
            )
        } else {
            (
                result_data
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                result_data
                    .get("score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0),
            )
        };

        let execution_result = ExecutionResult {
            success,
            score,
            total_time_ms: 60000, // Default 1 minute
            statistics: ExecutionStatistics {
                total_llm_calls: flow_log
                    .events
                    .iter()
                    .filter(|e| matches!(e.event_type, FlowEventType::LlmRequest))
                    .count() as u32,
                total_tool_calls: flow_log
                    .events
                    .iter()
                    .filter(|e| matches!(e.event_type, FlowEventType::ToolCall))
                    .count() as u32,
                total_tokens: 0,
                tool_usage: HashMap::new(),
                max_depth: 0,
            },
            scoring_breakdown: None,
        };

        flow_log.final_result = Some(execution_result.clone());

        flow_log.final_result = Some(execution_result);

        // Render as ASCII tree using the existing renderer
        Ok(flow_log.render_as_ascii_tree())
    }

    /// Generate trace from session log content
    pub async fn generate_trace_from_session_log(
        &self,
        log_content: &str,
        execution_id: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Generating trace from session log for: {}", execution_id);

        // Parse the log content as JSON to get session data
        let session_data: serde_json::Value = serde_json::from_str(log_content)
            .map_err(|e| format!("Failed to parse session log JSON: {e}"))?;

        // Use the same conversion logic as session data
        self.generate_trace_from_session_data(&session_data, execution_id)
            .await
    }

    /// Generate trace from execution metadata when no result data is available
    pub async fn generate_trace_from_metadata(
        &self,
        state: &ExecutionState,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Generating trace from metadata for: {}", state.execution_id);

        let mut trace = String::new();

        // Basic execution information
        trace.push_str(&format!(
            "📊 Execution: {} [{}]\n",
            state.execution_id, state.agent
        ));
        trace.push_str(&format!("   Benchmark: {}\n", state.benchmark_id));
        trace.push_str(&format!("   Status: {:?}\n", state.status));

        if let Some(ref error) = state.error_message {
            trace.push_str(&format!("   ❌ Error: {error}\n"));
        }

        if let Some(progress) = state.progress {
            trace.push_str(&format!("   Progress: {:.1}%\n", progress * 100.0));
        }

        // Add metadata if available
        if !state.metadata.is_empty() {
            trace.push_str("   📋 Metadata:\n");
            for (key, value) in &state.metadata {
                trace.push_str(&format!("      {key}: {value}\n"));
            }
        }

        if trace.is_empty() {
            trace = "📝 No execution trace available".to_string();
        }

        Ok(trace)
    }

    /// Generate error trace when parsing fails
    pub fn generate_error_trace(&self, error: &str, execution_id: &str) -> String {
        warn!("Generating error trace for {}: {}", execution_id, error);

        format!(
            "⚠️  Failed to generate execution trace\n   Execution ID: {execution_id}\n   Error: {error}"
        )
    }

    /// Generate trace from JSON value with automatic format detection
    #[allow(dead_code)]
    pub async fn generate_trace_from_json(
        &self,
        json_data: &Value,
        execution_id: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Generating trace from JSON for: {}", execution_id);

        // Check if this looks like session data
        if json_data.get("steps").is_some() || json_data.get("session_id").is_some() {
            self.generate_trace_from_session_data(json_data, execution_id)
                .await
        } else if json_data.is_string() {
            // Assume it's a session log content
            let log_content = json_data.as_str().unwrap_or("");
            self.generate_trace_from_session_log(log_content, execution_id)
                .await
        } else {
            // Fallback to basic formatting
            Err("Unknown JSON format for trace generation".into())
        }
    }

    /// Parse action details from step action for formatting
    /// This extracts program_id, accounts, and data from action arrays
    /// and formats them for display in various trace formats
    /// Returns: (action_details, data_value)
    pub fn parse_action_details(action: &serde_json::Value) -> (Vec<String>, Option<String>) {
        let mut action_details = Vec::new();
        let mut data_value = None;

        if let Some(action_array) = action.as_array() {
            for action_item in action_array {
                let mut item_details = Vec::new();

                // Extract program ID
                if let Some(program_id) = action_item.get("program_id") {
                    item_details.push(format!("   🔹 Program: {program_id}"));
                }

                // Extract and format accounts
                if let Some(accounts) = action_item.get("accounts") {
                    if let Some(accounts_array) = accounts.as_array() {
                        item_details.push("   📋 Accounts:".to_string());
                        for (idx, account) in accounts_array.iter().enumerate() {
                            if let Some(pubkey) = account.get("pubkey") {
                                let is_signer = account
                                    .get("is_signer")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                let is_writable = account
                                    .get("is_writable")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                let icon = if is_signer { "🖋️" } else { "🖍️" };
                                let arrow = if is_writable { "➕" } else { "➖" };
                                item_details.push(format!("     [{idx}] {icon} {arrow} {pubkey}"));
                            }
                        }
                    }
                }

                // Extract data
                if let Some(data) = action_item.get("data") {
                    data_value = Some(format!("   📝 Data: {data}"));
                }

                action_details.push(item_details.join("\n"));
            }
        }

        (action_details, data_value)
    }
}

impl Default for ExecutionTraceParser {
    fn default() -> Self {
        Self::new()
    }
}
