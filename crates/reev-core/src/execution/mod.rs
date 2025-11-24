//! Tool Execution Module for reev-core
//!
//! This module provides integration with reev-tools for actual tool execution
//! in the executor module.

pub mod tool_executor;
pub mod trait_def;

// Re-export for convenience
pub use tool_executor::ToolExecutor;
pub use trait_def::{Executor as ToolExecutorTrait, SharedExecutor};
