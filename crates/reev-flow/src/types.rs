//! Core flow types for reev ecosystem
//!
//! This module contains the fundamental types for tracking agent execution flows.
//! These types are designed to be:
//! 1. Serializable for storage and API communication
//! 2. Generic enough for different use cases
//! 3. Easily convertible to/from domain-specific types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

#[cfg(feature = "database")]
use crate::database::DBFlowLog;

/// Main flow log structure for complete benchmark execution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowLog {
    /// Unique session identifier
    pub session_id: String,
    /// Benchmark identifier
    pub benchmark_id: String,
    /// Agent type (deterministic, local, gemini, etc.)
    pub agent_type: String,
    /// Start timestamp
    pub start_time: SystemTime,
    /// End timestamp
    pub end_time: Option<SystemTime>,
    /// All events in chronological order
    pub events: Vec<FlowEvent>,
    /// Final execution result
    pub final_result: Option<ExecutionResult>,
}

/// Individual event within a flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEvent {
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Type of event
    pub event_type: FlowEventType,
    /// Conversation depth when event occurred
    pub depth: u32,
    /// Event-specific content
    pub content: EventContent,
}

/// Types of events that can occur during flow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum FlowEventType {
    /// LLM request/response cycle
    LlmRequest,
    /// Tool invocation
    ToolCall,
    /// Tool result/response
    ToolResult,
    /// Transaction execution
    TransactionExecution,
    /// Error occurred
    Error,
    /// Benchmark state change
    BenchmarkStateChange,
}

/// Event content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContent {
    /// Event-specific data
    pub data: serde_json::Value,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// LLM request content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequestContent {
    /// The prompt sent to LLM
    pub prompt: String,
    /// Number of context tokens
    pub context_tokens: u32,
    /// Model name
    pub model: String,
    /// Request ID for tracking
    pub request_id: String,
}

/// Tool call content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallContent {
    /// Tool name
    pub tool_name: String,
    /// Arguments passed to tool
    pub tool_args: String,
    /// Execution time in milliseconds
    pub execution_time_ms: u32,
    /// Tool result status
    pub result_status: ToolResultStatus,
    /// Result data if successful
    pub result_data: Option<serde_json::Value>,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Tool execution result status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolResultStatus {
    Success,
    Error,
    Timeout,
}

/// Transaction execution content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionExecutionContent {
    /// Transaction signature
    pub signature: String,
    /// Number of instructions
    pub instruction_count: u32,
    /// Execution time
    pub execution_time_ms: u32,
    /// Success status
    pub success: bool,
    /// Error if any
    pub error: Option<String>,
}

/// Error event content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContent {
    /// Error type
    pub error_type: String,
    /// Error message
    pub message: String,
    /// Stack trace if available
    pub stack_trace: Option<String>,
    /// Context when error occurred
    pub context: HashMap<String, String>,
}

/// Final execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Overall success status
    pub success: bool,
    /// Final score
    pub score: f64,
    /// Total execution time
    pub total_time_ms: u64,
    /// Summary statistics
    pub statistics: ExecutionStatistics,
    /// Detailed scoring breakdown
    pub scoring_breakdown: Option<ScoringBreakdown>,
}

/// Execution statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionStatistics {
    /// Total LLM calls
    pub total_llm_calls: u32,
    /// Total tool calls
    pub total_tool_calls: u32,
    /// Total tokens used
    pub total_tokens: u64,
    /// Tool usage breakdown
    pub tool_usage: HashMap<String, u32>,
    /// Conversation depth reached
    pub max_depth: u32,
}

/// Detailed scoring breakdown for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringBreakdown {
    /// Instruction matching score (0-1)
    pub instruction_score: f64,
    /// On-chain execution score (0-1)
    pub onchain_score: f64,
    /// Weighted final score (0-1)
    pub final_score: f64,
    /// Issues that affected the score
    pub issues: Vec<String>,
    /// Specific mismatches found
    pub mismatches: Vec<String>,
}

/// Website data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteData {
    pub flows: Vec<FlowLog>,
    pub flow_visualization: FlowGraph,
    pub tool_usage_stats: ToolUsageStats,
    pub performance_metrics: PerformanceMetrics,
    pub agent_behavior_analysis: AgentBehaviorAnalysis,
}

/// Flow graph for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGraph {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
}

/// Flow node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub position: (f64, f64),
}

/// Flow edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    pub label: String,
}

/// Tool usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageStats {
    pub total_usage: HashMap<String, u32>,
    pub success_rates: HashMap<String, f64>,
    pub average_execution_times: HashMap<String, f64>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_execution_time_ms: u64,
    pub average_execution_time_ms: u64,
    pub total_llm_calls: u32,
    pub total_tool_calls: u32,
    pub total_tokens: u64,
    pub success_rate: f64,
}

/// Agent behavior analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBehaviorAnalysis {
    pub depth_patterns: HashMap<u32, u32>,
    pub common_tool_sequences: Vec<Vec<String>>,
    pub average_decision_time_ms: u64,
    pub error_recovery_rate: f64,
}

/// Simple conversion methods for FlowLog
#[cfg(feature = "database")]
pub trait FlowLogDbExt {
    /// Convert to database-friendly format
    fn to_db_flow_log(self) -> DBFlowLog;

    /// Convert from database-friendly format
    fn from_db_flow_log(db_flow_log: DBFlowLog) -> Self;
}

#[cfg(feature = "database")]
impl FlowLogDbExt for FlowLog {
    fn to_db_flow_log(self) -> DBFlowLog {
        DBFlowLog::new(self)
    }

    fn from_db_flow_log(db_flow_log: DBFlowLog) -> Self {
        db_flow_log.flow
    }
}
