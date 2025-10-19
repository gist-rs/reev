//! # Flow Log Parser
//!
//! This module parses the structured flow logs to extract tool execution sequences,
//! agent states, and decision points for visualization.

use chrono;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single flow step extracted from logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    /// Unique identifier for this step
    pub id: String,
    /// Type of step (agent_start, tool_call, tool_complete, agent_end, error)
    pub step_type: StepType,
    /// Tool name if this is a tool-related step
    pub tool_name: Option<String>,
    /// Agent/model name
    pub agent_name: Option<String>,
    /// Timestamp when this step occurred
    pub timestamp: String,
    /// Duration in milliseconds (if available)
    pub duration_ms: Option<u32>,
    /// Parameters/arguments for this step
    pub parameters: HashMap<String, String>,
    /// Result/status of this step
    pub result: Option<String>,
    /// Any metadata or context
    pub metadata: HashMap<String, String>,
}

/// Types of flow steps we can identify
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepType {
    /// Agent execution started
    AgentStart,
    /// Tool was called
    ToolCall,
    /// Tool completed successfully
    ToolComplete,
    /// Tool failed with error
    ToolError,
    /// Agent execution completed
    AgentEnd,
    /// Decision point or branching
    Decision,
    /// Informational log entry
    Info,
}

/// Represents a complete flow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowExecution {
    /// Unique identifier for this flow execution
    pub execution_id: String,
    /// All steps in chronological order
    pub steps: Vec<FlowStep>,
    /// Agent/model used
    pub agent: String,
    /// Start timestamp
    pub start_time: String,
    /// End timestamp (if available)
    pub end_time: Option<String>,
    /// Total duration in milliseconds
    pub total_duration_ms: Option<u32>,
    /// Tools used in this flow
    pub tools_used: Vec<String>,
    /// Success/failure status
    pub status: ExecutionStatus,
}

/// Overall execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Success,
    Error,
    Incomplete,
}

/// Parser for extracting flow data from structured logs
pub struct FlowLogParser {
    /// Regex patterns for different log types
    agent_start_pattern: Regex,
    agent_end_pattern: Regex,
    tool_call_pattern: Regex,
    tool_complete_pattern: Regex,
    error_pattern: Regex,
    span_pattern: Regex,
}

impl FlowLogParser {
    /// Create a new flow log parser
    pub fn new() -> Self {
        Self {
            // Agent execution start: "Starting agent execution with OpenTelemetry tracing"
            agent_start_pattern: Regex::new(
                r"(?P<timestamp>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)\s+INFO.*\[OpenAIAgent\].*Starting agent execution with OpenTelemetry tracing"
            ).unwrap(),

            // Tool call: "[ToolName] Starting tool execution with OpenTelemetry tracing"
            tool_call_pattern: Regex::new(
                r"(?P<timestamp>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)\s+INFO.*\[(?P<tool_name>[^\]]+)\].*Starting tool execution with OpenTelemetry tracing"
            ).unwrap(),

            // Tool completion: "Tool execution completed - total_time: Xms"
            tool_complete_pattern: Regex::new(
                r"(?P<timestamp>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)\s+INFO.*\[(?P<tool_name>[^\]]+)\].*Tool execution completed - total_time: (?P<duration>\d+)ms"
            ).unwrap(),

            // Agent completion: "Agent execution completed"
            agent_end_pattern: Regex::new(
                r"(?P<timestamp>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)\s+INFO.*\[OpenAIAgent\].*Agent execution completed"
            ).unwrap(),

            // Error patterns
            error_pattern: Regex::new(
                r"(?P<timestamp>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)\s+(ERROR|WARN).*"
            ).unwrap(),

            // Span/context pattern: "in agent_execution with model: ..."
            span_pattern: Regex::new(
                r"in agent_execution with model: (?P<model>[^,]+), conversation_depth: (?P<depth>\d+), benchmark_id: (?P<benchmark_id>[^,]+), tools_count: (?P<tools_count>\d+)"
            ).unwrap(),
        }
    }

    /// Parse log content and extract flow executions
    pub fn parse_log(
        &self,
        log_content: &str,
    ) -> Result<Vec<FlowExecution>, Box<dyn std::error::Error>> {
        let lines: Vec<&str> = log_content.lines().collect();
        let mut executions = Vec::new();
        let mut current_execution: Option<FlowExecution> = None;
        let mut step_counter = 0;

        for line in lines {
            // Check for agent start
            if let Some(captures) = self.agent_start_pattern.captures(line) {
                // Complete previous execution if exists
                if let Some(execution) = current_execution.take() {
                    executions.push(execution);
                }

                // Start new execution
                let execution_id = format!("exec_{}", executions.len() + 1);
                let timestamp = captures.name("timestamp").unwrap().as_str().to_string();
                let agent = captures
                    .name("model")
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| "unknown_agent".to_string());

                current_execution = Some(FlowExecution {
                    execution_id: execution_id.clone(),
                    steps: Vec::new(),
                    agent,
                    start_time: timestamp.clone(),
                    end_time: None,
                    total_duration_ms: None,
                    tools_used: Vec::new(),
                    status: ExecutionStatus::Incomplete,
                });

                // Add agent start step
                if let Some(ref mut exec) = current_execution {
                    step_counter += 1;
                    exec.steps.push(FlowStep {
                        id: format!("step_{step_counter}"),
                        step_type: StepType::AgentStart,
                        tool_name: None,
                        agent_name: Some(exec.agent.clone()),
                        timestamp: timestamp.clone(),
                        duration_ms: None,
                        parameters: self.extract_span_parameters(line),
                        result: None,
                        metadata: HashMap::new(),
                    });
                }
            }
            // Check for tool calls
            else if let Some(captures) = self.tool_call_pattern.captures(line) {
                if let Some(ref mut exec) = current_execution {
                    step_counter += 1;
                    let tool_name = captures.name("tool_name").unwrap().as_str().to_string();

                    // Extract tool parameters from the line
                    let parameters = self.extract_tool_parameters(line, &tool_name);

                    exec.steps.push(FlowStep {
                        id: format!("step_{step_counter}"),
                        step_type: StepType::ToolCall,
                        tool_name: Some(tool_name.clone()),
                        agent_name: Some(exec.agent.clone()),
                        timestamp: captures.name("timestamp").unwrap().as_str().to_string(),
                        duration_ms: None,
                        parameters,
                        result: None,
                        metadata: HashMap::new(),
                    });

                    // Track tools used
                    if !exec.tools_used.contains(&tool_name) {
                        exec.tools_used.push(tool_name);
                    }
                }
            }
            // Check for tool completion
            else if let Some(captures) = self.tool_complete_pattern.captures(line) {
                if let Some(ref mut exec) = current_execution {
                    step_counter += 1;
                    let tool_name = captures.name("tool_name").unwrap().as_str().to_string();
                    let duration: u32 = captures.name("duration").unwrap().as_str().parse()?;

                    let timestamp = captures.name("timestamp").unwrap().as_str().to_string();
                    exec.steps.push(FlowStep {
                        id: format!("step_{step_counter}"),
                        step_type: StepType::ToolComplete,
                        tool_name: Some(tool_name),
                        agent_name: Some(exec.agent.clone()),
                        timestamp: timestamp.clone(),
                        duration_ms: Some(duration),
                        parameters: HashMap::new(),
                        result: Some("success".to_string()),
                        metadata: HashMap::new(),
                    });
                }
            }
            // Check for agent end
            else if let Some(captures) = self.agent_end_pattern.captures(line) {
                if let Some(ref mut exec) = current_execution {
                    step_counter += 1;
                    let timestamp = captures.name("timestamp").unwrap().as_str().to_string();

                    exec.steps.push(FlowStep {
                        id: format!("step_{step_counter}"),
                        step_type: StepType::AgentEnd,
                        tool_name: None,
                        agent_name: Some(exec.agent.clone()),
                        timestamp: timestamp.clone(),
                        duration_ms: None,
                        parameters: HashMap::new(),
                        result: Some("completed".to_string()),
                        metadata: HashMap::new(),
                    });

                    exec.end_time = Some(timestamp);
                    exec.status = ExecutionStatus::Success;
                }
            }
            // Check for errors
            else if self.error_pattern.is_match(line) {
                if let Some(ref mut exec) = current_execution {
                    step_counter += 1;

                    exec.steps.push(FlowStep {
                        id: format!("step_{step_counter}"),
                        step_type: StepType::ToolError,
                        tool_name: self.extract_tool_name_from_error(line),
                        agent_name: Some(exec.agent.clone()),
                        timestamp: self.extract_timestamp_from_line(line),
                        duration_ms: None,
                        parameters: HashMap::new(),
                        result: Some("error".to_string()),
                        metadata: HashMap::new(),
                    });

                    exec.status = ExecutionStatus::Error;
                }
            }
        }

        // Add the last execution if it exists
        if let Some(execution) = current_execution {
            executions.push(execution);
        }

        // Calculate durations and cleanup
        for execution in &mut executions {
            self.calculate_execution_durations(execution);
        }

        Ok(executions)
    }

    /// Extract parameters from span context
    fn extract_span_parameters(&self, line: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();

        if let Some(captures) = self.span_pattern.captures(line) {
            if let Some(model) = captures.name("model") {
                params.insert("model".to_string(), model.as_str().to_string());
            }
            if let Some(depth) = captures.name("depth") {
                params.insert("conversation_depth".to_string(), depth.as_str().to_string());
            }
            if let Some(benchmark_id) = captures.name("benchmark_id") {
                params.insert(
                    "benchmark_id".to_string(),
                    benchmark_id.as_str().to_string(),
                );
            }
            if let Some(tools_count) = captures.name("tools_count") {
                params.insert("tools_count".to_string(), tools_count.as_str().to_string());
            }
        } else {
            // Fallback for when span pattern doesn't match
            params.insert("model".to_string(), "unknown_agent".to_string());
            params.insert("conversation_depth".to_string(), "unknown".to_string());
        }

        params
    }

    /// Extract tool-specific parameters from log line
    fn extract_tool_parameters(&self, line: &str, _tool_name: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();

        // Extract parameters from instrument spans
        if line.contains("with args:") {
            // Try to extract JSON args
            if let Some(start) = line.find("with args:") {
                let args_part = &line[start + 10..];
                if let Some(end) = args_part.find('}') {
                    let json_str = &args_part[..=end];
                    if let Ok(serde_json::Value::Object(obj)) =
                        serde_json::from_str::<serde_json::Value>(json_str)
                    {
                        for (key, value) in obj {
                            params.insert(key, value.to_string());
                        }
                    }
                }
            }
        }

        // Extract common parameters from span fields
        if line.contains("user_pubkey:") {
            if let Some(start) = line.find("user_pubkey=") {
                let pubkey_part = &line[start..];
                if let Some(end) = pubkey_part.find(',') {
                    let pubkey = &pubkey_part[12..end];
                    params.insert("user_pubkey".to_string(), pubkey.to_string());
                }
            }
        }

        if line.contains("amount:") {
            if let Some(start) = line.find("amount=") {
                let amount_part = &line[start..];
                if let Some(end) = amount_part.find(',') {
                    let amount = &amount_part[7..end];
                    params.insert("amount".to_string(), amount.to_string());
                }
            }
        }

        params
    }

    /// Extract tool name from error line
    fn extract_tool_name_from_error(&self, line: &str) -> Option<String> {
        // Look for tool names in error patterns
        let tools = [
            "jupiter_swap",
            "sol_transfer",
            "spl_transfer",
            "get_account_balance",
        ];
        for tool in &tools {
            if line.contains(tool) {
                return Some(tool.to_string());
            }
        }
        None
    }

    /// Extract timestamp from any log line
    fn extract_timestamp_from_line(&self, line: &str) -> String {
        // Extract timestamp from beginning of line
        if let Some(space_pos) = line.find(' ') {
            line[..space_pos].to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Calculate execution durations
    fn calculate_execution_durations(&self, execution: &mut FlowExecution) {
        if let Some(ref end_time) = execution.end_time {
            if let (Ok(start), Ok(end)) = (
                execution
                    .start_time
                    .parse::<chrono::DateTime<chrono::Utc>>(),
                end_time.parse::<chrono::DateTime<chrono::Utc>>(),
            ) {
                let duration = end.signed_duration_since(start);
                execution.total_duration_ms = Some(duration.num_milliseconds() as u32);
            }
        }
    }
}

impl Default for FlowLogParser {
    fn default() -> Self {
        Self::new()
    }
}
