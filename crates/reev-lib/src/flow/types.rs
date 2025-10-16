//! Flow types re-exported from reev-flow
//!
//! This module re-exports all flow types from the reev-flow crate
//! to maintain backward compatibility while centralizing type definitions.

// Re-export all types from reev-flow
pub use reev_flow::{
    AgentBehaviorAnalysis, ErrorContent, EventContent, ExecutionResult, ExecutionStatistics,
    FlowEdge, FlowEvent, FlowEventType, FlowGraph, FlowLog, FlowNode, LlmRequestContent,
    PerformanceMetrics, ScoringBreakdown, ToolCallContent, ToolResultStatus, ToolUsageStats,
    TransactionExecutionContent, WebsiteData,
};

// Re-export utilities
pub use reev_flow::{FlowSummary, FlowUtils};

// Re-export database types when feature is enabled
#[cfg(feature = "database")]
pub use reev_flow::database::{
    DBFlowLog, DBFlowLogConverter, DBStorageFormat, FlowLogDB, FlowLogQuery,
};
