//! # reev-orchestrator
//!
//! Dynamic flow orchestration for reev agents.
//!
//! This crate provides the ability to generate context-aware flows from natural language prompts,
//! replacing static YML files with dynamic, adaptable flow execution.

pub mod benchmark_mode;
pub mod context_resolver;
pub mod dynamic_mode;
pub mod execution;
pub mod gateway;
pub mod generators;
pub mod recovery;
pub mod templates;

pub use benchmark_mode::{execute_static_benchmark, list_static_benchmarks, BenchmarkMetadata};
pub use context_resolver::ContextResolver;
pub use dynamic_mode::{analyze_simple_intent, execute_user_request, validate_user_request};
pub use execution::{ExecutionContext, PingPongExecutor, StepResultExt};
pub use gateway::OrchestratorGateway;
pub use gateway::UserIntent;
pub use generators::YmlGenerator;
pub use recovery::{RecoveryConfig, RecoveryEngine, RecoveryOutcome, RecoveryResult};
pub use templates::{TemplateEngine, TemplateRenderer, TemplateType};

/// Result type for orchestrator operations
pub type Result<T> = anyhow::Result<T>;

/// Execution mode for routing requests
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Benchmark mode - execute static YML files
    Benchmark { id: String, agent: Option<String> },
    /// Dynamic mode - execute user requests
    Dynamic {
        prompt: String,
        wallet: String,
        agent: Option<String>,
    },
}

impl ExecutionMode {
    /// Check if this is benchmark mode
    pub fn is_benchmark(&self) -> bool {
        matches!(self, ExecutionMode::Benchmark { .. })
    }

    /// Check if this is dynamic mode
    pub fn is_dynamic(&self) -> bool {
        matches!(self, ExecutionMode::Dynamic { .. })
    }

    /// Get the agent type if specified
    pub fn agent(&self) -> Option<&str> {
        match self {
            ExecutionMode::Benchmark { agent, .. } => agent.as_deref(),
            ExecutionMode::Dynamic { agent, .. } => agent.as_deref(),
        }
    }
}

/// Route execution based on mode
///
/// This function provides clean top-level separation between benchmark
/// and dynamic execution modes, using the same core logic beneath.
///
/// # Arguments
/// * `mode` - Execution mode (benchmark or dynamic)
/// * `context` - Optional wallet context (required for dynamic mode)
/// * `executor` - Function that actually executes the YML file
///
/// # Returns
/// * `Result<reev_types::execution::ExecutionResponse>` - Execution result
///
/// # Errors
/// * If mode execution fails
/// * If required context is missing
pub async fn route_execution<F, Fut>(
    mode: ExecutionMode,
    context: Option<&WalletContext>,
    executor: F,
) -> Result<reev_types::execution::ExecutionResponse>
where
    F: FnOnce(PathBuf, Option<String>) -> Fut,
    Fut: std::future::Future<Output = Result<reev_types::execution::ExecutionResponse>>,
{
    match mode {
        ExecutionMode::Benchmark { id, agent } => {
            #[cfg(feature = "benchmark")]
            {
                execute_static_benchmark(&id, agent.as_deref(), executor).await
            }
            #[cfg(not(feature = "benchmark"))]
            {
                Err(anyhow::anyhow!("Benchmark mode not enabled"))
            }
        }
        ExecutionMode::Dynamic {
            prompt,
            wallet: _,
            agent,
        } => {
            #[cfg(feature = "production")]
            {
                let context = context.ok_or_else(|| {
                    anyhow::anyhow!("Wallet context required for dynamic execution")
                })?;

                // Validate user request before execution
                validate_user_request(&prompt, context)?;

                execute_user_request(&prompt, context, agent.as_deref(), executor).await
            }
            #[cfg(not(feature = "production"))]
            {
                Err(anyhow::anyhow!("Dynamic mode not enabled"))
            }
        }
    }
}

pub use benchmark_mode::ExecutionMetadata;
/// Re-export common types for convenience
pub use reev_types::flow::{
    BenchmarkSource, DynamicFlowPlan, DynamicStep, FlowMetadata, FlowMetrics, FlowResult,
    PromptContext, WalletContext,
};
pub use reev_types::tools::ToolName;
use std::path::PathBuf;
