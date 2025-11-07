//! JSONL to YML Converter Module
//!
//! This module provides utilities to convert structured JSONL logs from the enhanced
//! OpenTelemetry system into readable YML format for easier analysis and
//! ASCII tree generation.

use crate::enhanced_otel::{EnhancedToolCall, EventType, ToolInputInfo};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, JsonlConverterError>;

#[derive(Debug, Error)]
pub enum JsonlConverterError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("Invalid log format: {0}")]
    InvalidFormat(String),
    #[error("No session data found")]
    NoSessionData,
}

/// Aggregated session data for YML conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Session identifier
    pub session_id: String,
    /// Reev runner version
    pub reev_runner_version: String,
    /// Reev agent version
    pub reev_agent_version: String,
    /// Session start time
    pub start_time: String,
    /// Session end time (if completed)
    pub end_time: Option<String>,
    /// Total session duration in milliseconds
    pub total_duration_ms: u64,
    /// Prompt information
    pub prompt: Option<PromptData>,
    /// Tool calls in chronological order
    pub tool_calls: Vec<ToolCallData>,
    /// Steps completed in the flow
    pub steps: Vec<StepData>,
    /// Summary statistics
    pub summary: SessionSummary,
}

/// Prompt data from the original request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptData {
    /// List of available tools to the LLM
    pub tool_name_list: Vec<String>,
    /// Original user prompt
    pub user_prompt: String,
    /// Final enriched prompt sent to LLM
    pub final_prompt: String,
}

/// Tool call data with input and output information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallData {
    /// Tool name
    pub tool_name: String,
    /// Tool start time
    pub start_time: String,
    /// Tool end time
    pub end_time: String,
    /// Tool execution duration in milliseconds
    pub duration_ms: u64,
    /// Tool input parameters
    pub input: serde_json::Value,
    /// Tool output results
    pub output: serde_json::Value,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Step completion data for multi-step flows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepData {
    /// Step name or identifier
    pub step_name: String,
    /// Step completion time
    pub completion_time: String,
    /// Flow time at step completion
    pub flow_time_ms: u64,
    /// Step time for this step
    pub step_time_ms: u64,
}

/// Session summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Total number of tool calls
    pub total_tool_calls: usize,
    /// Number of successful tool calls
    pub successful_tool_calls: usize,
    /// Number of failed tool calls
    pub failed_tool_calls: usize,
    /// Total number of steps
    pub total_steps: usize,
    /// Average tool execution time in milliseconds
    pub avg_tool_time_ms: f64,
    /// Success rate as percentage
    pub success_rate: f64,
}

/// JSONL to YML converter
pub struct JsonlToYmlConverter;

impl JsonlToYmlConverter {
    /// Convert JSONL file to YML format
    pub fn convert_file(jsonl_path: &Path, yml_path: &Path) -> Result<SessionData> {
        let session_data = Self::parse_jsonl_file(jsonl_path)?;
        Self::write_yml_file(&session_data, yml_path)?;
        Ok(session_data)
    }

    /// Parse JSONL file and aggregate session data
    pub fn parse_jsonl_file(jsonl_path: &Path) -> Result<SessionData> {
        let file = File::open(jsonl_path)?;
        let reader = BufReader::new(file);

        let mut events = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            // Skip summary lines that don't match EnhancedToolCall format
            // Summary lines have fields like "failed_tools", "logged_at", etc. but no "timestamp"
            if line.contains("\"failed_tools\":")
                || line.contains("\"successful_tools\":")
                || line.contains("\"total_events\":")
            {
                continue;
            }

            let event: EnhancedToolCall = serde_json::from_str(&line).map_err(|e| {
                JsonlConverterError::InvalidFormat(format!("Failed to parse line: {e}"))
            })?;
            events.push(event);
        }

        Self::aggregate_session_data(events)
    }

    /// Aggregate raw events into structured session data
    fn aggregate_session_data(events: Vec<EnhancedToolCall>) -> Result<SessionData> {
        if events.is_empty() {
            return Err(JsonlConverterError::NoSessionData);
        }

        // Sort events by timestamp
        let mut events = events;
        events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let first_event = &events[0];
        let session_id = first_event.session_id.clone();
        let reev_runner_version = first_event.reev_runner_version.clone();
        let reev_agent_version = first_event.reev_agent_version.clone();

        let start_time = first_event.timestamp.to_rfc3339();
        let end_time = events.last().map(|e| e.timestamp.to_rfc3339());
        let total_duration_ms = if let (Some(first), Some(last)) = (events.first(), events.last()) {
            (last.timestamp - first.timestamp).num_milliseconds() as u64
        } else {
            0
        };

        // Extract prompt data
        let prompt = events
            .iter()
            .find(|e| matches!(e.event_type, EventType::Prompt))
            .and_then(|e| e.prompt.clone())
            .map(|p| PromptData {
                tool_name_list: p.tool_name_list,
                user_prompt: p.user_prompt,
                final_prompt: p.final_prompt,
            });

        // Extract tool calls
        let mut tool_calls = Vec::new();
        let mut pending_tool_inputs: Vec<(DateTime<Utc>, ToolInputInfo)> = Vec::new();

        // Collect all tool inputs and outputs in order
        for event in &events {
            if matches!(event.event_type, EventType::ToolInput) {
                if let Some(ref tool_input) = event.tool_input {
                    pending_tool_inputs.push((event.timestamp, tool_input.clone()));
                }
            } else if matches!(event.event_type, EventType::ToolOutput) {
                if let Some(ref tool_output) = event.tool_output {
                    // Match with the most recent pending tool input
                    if let Some((start_time, tool_input)) = pending_tool_inputs.pop() {
                        let duration_ms = event
                            .timestamp
                            .signed_duration_since(start_time)
                            .num_milliseconds() as u64;

                        tool_calls.push(ToolCallData {
                            tool_name: tool_input.tool_name.clone(),
                            start_time: start_time.to_rfc3339(),
                            end_time: event.timestamp.to_rfc3339(),
                            duration_ms,
                            input: tool_input.tool_args.clone(),
                            output: tool_output.results.clone(),
                            success: tool_output.success,
                            error_message: tool_output.error_message.clone(),
                        });
                    }
                }
            }
        }

        // Extract step completions
        let steps: Vec<StepData> = events
            .iter()
            .filter(|e| matches!(e.event_type, EventType::StepComplete))
            .map(|e| {
                let step_name = e
                    .metadata
                    .get("step_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown_step")
                    .to_string();

                StepData {
                    step_name,
                    completion_time: e.timestamp.to_rfc3339(),
                    flow_time_ms: e.timing.flow_timeuse_ms,
                    step_time_ms: e.timing.step_timeuse_ms,
                }
            })
            .collect();

        // Calculate summary
        let successful_tool_calls = tool_calls.iter().filter(|t| t.success).count();
        let total_tool_calls = tool_calls.len();
        let failed_tool_calls = total_tool_calls - successful_tool_calls;

        let avg_tool_time_ms = if total_tool_calls > 0 {
            tool_calls.iter().map(|t| t.duration_ms as f64).sum::<f64>() / total_tool_calls as f64
        } else {
            0.0
        };

        let success_rate = if total_tool_calls > 0 {
            (successful_tool_calls as f64 / total_tool_calls as f64) * 100.0
        } else {
            0.0
        };

        let summary = SessionSummary {
            total_tool_calls,
            successful_tool_calls,
            failed_tool_calls,
            total_steps: steps.len(),
            avg_tool_time_ms,
            success_rate,
        };

        Ok(SessionData {
            session_id,
            reev_runner_version,
            reev_agent_version,
            start_time,
            end_time,
            total_duration_ms,
            prompt,
            tool_calls,
            steps,
            summary,
        })
    }

    /// Write session data to YML file
    fn write_yml_file(session_data: &SessionData, yml_path: &Path) -> Result<()> {
        let yml_content = Self::format_as_yml(session_data)?;

        // Ensure parent directory exists
        if let Some(parent) = yml_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = File::create(yml_path)?;
        file.write_all(yml_content.as_bytes())?;
        Ok(())
    }

    /// Format session data as readable YML
    fn format_as_yml(session_data: &SessionData) -> Result<String> {
        let mut yml = String::new();

        // Session header
        yml.push_str("# Reev Session Log Analysis\n\n");
        yml.push_str(&format!("session_id: {}\n", session_data.session_id));
        yml.push_str(&format!(
            "reev_runner_version: {}\n",
            session_data.reev_runner_version
        ));
        yml.push_str(&format!(
            "reev_agent_version: {}\n",
            session_data.reev_agent_version
        ));
        yml.push_str(&format!("start_time: {}\n", session_data.start_time));
        if let Some(ref end_time) = session_data.end_time {
            yml.push_str(&format!("end_time: {end_time}\n"));
        }
        yml.push_str(&format!(
            "total_duration_ms: {}\n\n",
            session_data.total_duration_ms
        ));

        // Prompt information
        if let Some(ref prompt) = session_data.prompt {
            yml.push_str("prompt:\n");
            yml.push_str(&format!(
                "  tool_name_list: [{}]\n",
                prompt
                    .tool_name_list
                    .iter()
                    .map(|t| format!("\"{t}\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            yml.push_str(&format!(
                "  user_prompt: |\n    {}\n",
                prompt.user_prompt.replace('\n', "\n    ")
            ));
            yml.push_str(&format!(
                "  final_prompt: |\n    {}\n\n",
                prompt.final_prompt.replace('\n', "\n    ")
            ));
        }

        // Tool calls
        yml.push_str("tool_calls:\n");
        for (i, tool_call) in session_data.tool_calls.iter().enumerate() {
            yml.push_str(&format!("  - # Tool Call {}\n", i + 1));
            yml.push_str(&format!("    tool_name: {}\n", tool_call.tool_name));
            yml.push_str(&format!("    start_time: {}\n", tool_call.start_time));
            yml.push_str(&format!("    end_time: {}\n", tool_call.end_time));
            yml.push_str(&format!("    duration_ms: {}\n", tool_call.duration_ms));
            yml.push_str(&format!("    success: {}\n", tool_call.success));
            if let Some(ref error) = tool_call.error_message {
                yml.push_str(&format!("    error_message: \"{error}\"\n"));
            }

            // Format input as indented YAML
            yml.push_str("    input:\n");
            Self::format_json_as_yml(&tool_call.input, "      ", &mut yml);

            // Format output as indented YAML
            yml.push_str("    output:\n");
            Self::format_json_as_yml(&tool_call.output, "      ", &mut yml);

            yml.push('\n');
        }

        // Steps
        if !session_data.steps.is_empty() {
            yml.push_str("steps:\n");
            for (i, step) in session_data.steps.iter().enumerate() {
                yml.push_str(&format!("  - # Step {}\n", i + 1));
                yml.push_str(&format!("    step_name: {}\n", step.step_name));
                yml.push_str(&format!("    completion_time: {}\n", step.completion_time));
                yml.push_str(&format!("    flow_time_ms: {}\n", step.flow_time_ms));
                yml.push_str(&format!("    step_time_ms: {}\n", step.step_time_ms));
            }
            yml.push('\n');
        }

        // Summary
        yml.push_str("summary:\n");
        yml.push_str(&format!(
            "  total_tool_calls: {}\n",
            session_data.summary.total_tool_calls
        ));
        yml.push_str(&format!(
            "  successful_tool_calls: {}\n",
            session_data.summary.successful_tool_calls
        ));
        yml.push_str(&format!(
            "  failed_tool_calls: {}\n",
            session_data.summary.failed_tool_calls
        ));
        yml.push_str(&format!(
            "  total_steps: {}\n",
            session_data.summary.total_steps
        ));
        yml.push_str(&format!(
            "  avg_tool_time_ms: {:.2}\n",
            session_data.summary.avg_tool_time_ms
        ));
        yml.push_str(&format!(
            "  success_rate: {:.2}%\n",
            session_data.summary.success_rate
        ));

        Ok(yml)
    }

    /// Format JSON value as indented YAML
    fn format_json_as_yml(value: &serde_json::Value, indent: &str, yml: &mut String) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, val) in map {
                    match val {
                        serde_json::Value::String(s) => {
                            yml.push_str(&format!("{indent}{key}: \"{s}\"\n"));
                        }
                        serde_json::Value::Number(n) => {
                            yml.push_str(&format!("{indent}{key}: {n}\n"));
                        }
                        serde_json::Value::Bool(b) => {
                            yml.push_str(&format!("{indent}{key}: {b}\n"));
                        }
                        serde_json::Value::Null => {
                            yml.push_str(&format!("{indent}{key}: null\n"));
                        }
                        serde_json::Value::Array(arr) => {
                            yml.push_str(&format!("{indent}{key}:\n"));
                            Self::format_json_as_yml(
                                &serde_json::Value::Array(arr.clone()),
                                &format!("{indent}  "),
                                yml,
                            );
                        }
                        serde_json::Value::Object(_) => {
                            yml.push_str(&format!("{indent}{key}:\n"));
                            Self::format_json_as_yml(val, &format!("{indent}  "), yml);
                        }
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    match item {
                        serde_json::Value::String(s) => {
                            yml.push_str(&format!("{indent}- \"{s}\"\n"));
                        }
                        serde_json::Value::Number(n) => {
                            yml.push_str(&format!("{indent}- {n}\n"));
                        }
                        serde_json::Value::Bool(b) => {
                            yml.push_str(&format!("{indent}- {b}\n"));
                        }
                        serde_json::Value::Null => {
                            yml.push_str(&format!("{indent}- null\n"));
                        }
                        _ => {
                            yml.push_str(&format!("{indent}-\n"));
                            Self::format_json_as_yml(item, &format!("{indent}  "), yml);
                        }
                    }
                }
            }
            _ => {
                yml.push_str(&format!("{indent}{value}\n"));
            }
        }
    }
}
