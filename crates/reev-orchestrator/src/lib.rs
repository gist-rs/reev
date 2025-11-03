//! # reev-orchestrator
//!
//! Dynamic flow orchestration for reev agents.
//!
//! This crate provides the ability to generate context-aware flows from natural language prompts,
//! replacing static YML files with dynamic, adaptable flow execution.

pub mod context_resolver;
pub mod gateway;
pub mod generators;

pub use context_resolver::ContextResolver;
pub use gateway::OrchestratorGateway;
pub use generators::YmlGenerator;

/// Result type for orchestrator operations
pub type Result<T> = anyhow::Result<T>;

/// Re-export common types for convenience
pub use reev_types::flow::{
    BenchmarkSource, DynamicFlowPlan, DynamicStep, FlowMetadata, FlowMetrics, FlowResult,
    PromptContext, WalletContext,
};
