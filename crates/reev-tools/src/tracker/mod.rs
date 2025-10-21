//! Flow tracking module for capturing tool calls and execution order
//!
//! This module provides functionality to track tool calls during agent execution,
//! capturing the sequence, timing, and results of each tool invocation using
//! OpenTelemetry integration with the rig framework.

use std::collections::HashMap;
use std::time::Instant;
use tracing::debug;

use reev_lib::agent::{FlowData, ToolCallInfo};

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

// Export the OpenTelemetry wrapper module
pub mod otel_wrapper;

// Re-export commonly used types
pub use otel_wrapper::{
    init_simple_tool_tracing, OtelMetricsCollector, SimpleToolWrapper, ToolExecutionMetrics,
};
