//! # reev-orchestrator
//!
//! Dynamic flow orchestration for reev agents.
//!
//! This crate provides the ability to generate context-aware flows from natural language prompts,
//! replacing static YML files with dynamic, adaptable flow execution.

pub mod context_resolver;
pub mod execution;
pub mod gateway;
pub mod generators;
pub mod recovery;
pub mod templates;

pub use context_resolver::ContextResolver;
pub use execution::{ExecutionContext, PingPongExecutor, StepResultExt};
pub use gateway::OrchestratorGateway;
pub use generators::YmlGenerator;
pub use recovery::{RecoveryConfig, RecoveryEngine, RecoveryOutcome, RecoveryResult};
pub use templates::{TemplateEngine, TemplateRenderer, TemplateType};

/// Result type for orchestrator operations
pub type Result<T> = anyhow::Result<T>;

/// Re-export common types for convenience
pub use reev_types::flow::{
    BenchmarkSource, DynamicFlowPlan, DynamicStep, FlowMetadata, FlowMetrics, FlowResult,
    PromptContext, WalletContext,
};
