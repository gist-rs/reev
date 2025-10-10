//! Tool wrapper for flow tracking
//!
//! This module provides simplified flow tracking for tool calls during agent execution.

use reev_lib::agent::{FlowData, ToolCallInfo, ToolResultStatus};
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// Simple flow tracker for capturing tool call information
#[derive(Debug, Clone)]
pub struct SimpleFlowTracker {
    /// List of tool calls made during execution
    tool_calls: Vec<ToolCallInfo>,
    /// Tool usage statistics
    tool_usage: std::collections::HashMap<String, u32>,
}

/// Parameters for recording a tool call
#[derive(Debug)]
pub struct ToolCallParams {
    pub tool_name: String,
    pub tool_args: String,
    pub execution_time_ms: u32,
    pub result_status: ToolResultStatus,
    pub result_data: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub depth: u32,
}

impl SimpleFlowTracker {
    /// Create a new simple flow tracker
    pub fn new() -> Self {
        debug!("Simple flow tracker initialized");
        Self {
            tool_calls: Vec::new(),
            tool_usage: std::collections::HashMap::new(),
        }
    }

    /// Check if flow tracking is enabled
    pub fn is_enabled(&self) -> bool {
        std::env::var("REEV_ENABLE_FLOW_LOGGING").is_ok()
    }

    /// Record a tool call
    pub fn record_tool_call(&mut self, params: ToolCallParams) {
        if !self.is_enabled() {
            return;
        }

        let tool_call_info = ToolCallInfo {
            tool_name: params.tool_name.clone(),
            tool_args: params.tool_args,
            execution_time_ms: params.execution_time_ms,
            result_status: params.result_status,
            result_data: params.result_data,
            error_message: params.error_message,
            timestamp: std::time::SystemTime::now(),
            depth: params.depth,
        };

        self.tool_calls.push(tool_call_info);
        *self.tool_usage.entry(params.tool_name.clone()).or_insert(0) += 1;

        info!(
            "[SimpleFlowTracker] Recorded tool call: {} in {}ms",
            params.tool_name, params.execution_time_ms
        );
    }

    /// Get flow data for inclusion in LlmResponse
    pub fn get_flow_data(&self) -> Option<FlowData> {
        if !self.is_enabled() || self.tool_calls.is_empty() {
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
        self.tool_usage.clear();
        debug!("[SimpleFlowTracker] Reset flow tracker");
    }
}

impl Default for SimpleFlowTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Global flow tracker for tool calls
pub struct GlobalFlowTracker;

impl GlobalFlowTracker {
    /// Record a tool call globally
    pub fn record_tool_call(params: ToolCallParams) {
        if let Ok(mut tracker) = GLOBAL_TRACKER.lock() {
            tracker.record_tool_call(params);
        }
    }

    /// Get the current flow data
    pub fn get_flow_data() -> Option<FlowData> {
        if let Ok(tracker) = GLOBAL_TRACKER.lock() {
            tracker.get_flow_data()
        } else {
            None
        }
    }

    /// Reset the global tracker
    pub fn reset() {
        if let Ok(mut tracker) = GLOBAL_TRACKER.lock() {
            tracker.reset();
        }
    }
}

/// Global flow tracker instance
static GLOBAL_TRACKER: std::sync::LazyLock<Arc<Mutex<SimpleFlowTracker>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(SimpleFlowTracker::new())));

/// Helper function to create flow infrastructure for enhanced agents
pub fn create_flow_infrastructure() -> Arc<Mutex<SimpleFlowTracker>> {
    let tracker = Arc::clone(&*GLOBAL_TRACKER);
    tracker.lock().unwrap().reset(); // Reset for new session
    tracker
}

/// Helper function to extract flow data from tracker
pub fn extract_flow_data(_flow_tracker: &Arc<Mutex<SimpleFlowTracker>>) -> Option<FlowData> {
    GlobalFlowTracker::get_flow_data()
}
