//! Execution module for reev-core
//!
//! This module provides execution capabilities for agent flows, including tool execution
//! and transaction processing.

pub mod context_builder;
pub mod handlers;
pub mod rig_agent;
pub mod tool_executor;
pub mod trait_def;
pub mod types;

// Re-export for convenience
pub use tool_executor::ToolExecutor;
pub use trait_def::{Executor as ToolExecutorTrait, SharedExecutor};
