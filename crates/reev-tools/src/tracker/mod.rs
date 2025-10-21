//! Flow tracking module for capturing tool calls and execution order
//!
//! This module provides functionality to track tool calls during agent execution,
//! capturing the sequence, timing, and results of each tool invocation.

use std::collections::HashMap;
use std::time::{Instant, SystemTime};
use tracing::{debug, info, warn};

use reev_lib::agent::{FlowData, ToolCallInfo, ToolResultStatus};

/// Flow tracker for capturing tool calls during agent execution
#[derive(Debug, Clone)]
pub struct FlowTracker {
    /// List of tool calls made during execution
    tool_calls: Vec<ToolCallInfo>,
    /// Currently executing tool calls (for timing)
    active_calls: HashMap<String, Instant>,
    /// Tool usage statistics
    tool_usage: HashMap<String, u32>,
}

impl FlowTracker {
    /// Create a new flow tracker
    pub fn new() -> Self {
        debug!("Flow tracker initialized");

        Self {
            tool_calls: Vec::new(),
            active_calls: HashMap::new(),
            tool_usage: HashMap::new(),
        }
    }

    /// Start tracking a tool call
    pub fn start_tool_call(&mut self, tool_name: &str, _tool_args: &str, depth: u32) -> String {
        let call_id = format!("{}_{}", tool_name, uuid::Uuid::new_v4());
        let start_time = Instant::now();

        self.active_calls.insert(call_id.clone(), start_time);

        info!(
            "[FlowTracker] Starting tool call: {} at depth {}",
            tool_name, depth
        );

        call_id
    }

    /// Complete a tool call with success result
    pub fn complete_tool_call(
        &mut self,
        call_id: &str,
        tool_name: &str,
        tool_args: &str,
        result_data: Option<serde_json::Value>,
        depth: u32,
    ) {
        if call_id.is_empty() {
            return;
        }

        let start_time = match self.active_calls.remove(call_id) {
            Some(time) => time,
            None => {
                warn!(
                    "[FlowTracker] Tool call {} not found in active calls",
                    call_id
                );
                return;
            }
        };

        let execution_time_ms = start_time.elapsed().as_millis() as u32;

        let tool_call_info = ToolCallInfo {
            tool_name: tool_name.to_string(),
            tool_args: tool_args.to_string(),
            execution_time_ms,
            result_status: ToolResultStatus::Success,
            result_data,
            error_message: None,
            timestamp: SystemTime::now(),
            depth,
        };

        self.tool_calls.push(tool_call_info);
        *self.tool_usage.entry(tool_name.to_string()).or_insert(0) += 1;

        info!(
            "[FlowTracker] Completed tool call: {} in {}ms",
            tool_name, execution_time_ms
        );
    }

    /// Complete a tool call with error result
    pub fn fail_tool_call(
        &mut self,
        call_id: &str,
        tool_name: &str,
        tool_args: &str,
        error_message: &str,
        depth: u32,
    ) {
        if call_id.is_empty() {
            return;
        }

        let start_time = match self.active_calls.remove(call_id) {
            Some(time) => time,
            None => {
                warn!(
                    "[FlowTracker] Tool call {} not found in active calls",
                    call_id
                );
                return;
            }
        };

        let execution_time_ms = start_time.elapsed().as_millis() as u32;

        let tool_call_info = ToolCallInfo {
            tool_name: tool_name.to_string(),
            tool_args: tool_args.to_string(),
            execution_time_ms,
            result_status: ToolResultStatus::Error,
            result_data: None,
            error_message: Some(error_message.to_string()),
            timestamp: SystemTime::now(),
            depth,
        };

        self.tool_calls.push(tool_call_info);
        *self.tool_usage.entry(tool_name.to_string()).or_insert(0) += 1;

        warn!(
            "[FlowTracker] Failed tool call: {} in {}ms - {}",
            tool_name, execution_time_ms, error_message
        );
    }

    /// Get flow data for inclusion in LlmResponse
    pub fn get_flow_data(&self) -> Option<FlowData> {
        if self.tool_calls.is_empty() {
            return None;
        }

        Some(FlowData {
            tool_calls: self.tool_calls.clone(),
            total_tool_calls: self.tool_calls.len() as u32,
            tool_usage: self.tool_usage.clone(),
        })
    }

    /// Reset the tracker (for new sessions)
    pub fn reset(&mut self) {
        self.tool_calls.clear();
        self.active_calls.clear();
        self.tool_usage.clear();
        debug!("[FlowTracker] Reset flow tracker");
    }

    /// Get current statistics
    pub fn get_statistics(&self) -> (u32, HashMap<String, u32>) {
        (self.tool_calls.len() as u32, self.tool_usage.clone())
    }
}

impl Default for FlowTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Flow-aware tool wrapper for capturing tool calls
#[derive(Debug)]
pub struct FlowAwareTool<T> {
    inner: T,
    #[allow(dead_code)]
    flow_tracker: std::sync::Arc<std::sync::Mutex<FlowTracker>>,
}

impl<T> FlowAwareTool<T> {
    /// Create a new flow-aware tool wrapper
    pub fn new(tool: T, flow_tracker: std::sync::Arc<std::sync::Mutex<FlowTracker>>) -> Self {
        Self {
            inner: tool,
            flow_tracker,
        }
    }

    /// Get reference to inner tool
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

pub mod tool_wrapper;
