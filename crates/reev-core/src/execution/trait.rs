//! Trait for Tool Execution
//!
//! This module defines a trait for tool execution implementations,
//! allowing for both real and mock implementations.

use crate::yml_schema::YmlStep;
use anyhow::Result;
use reev_types::flow::{StepResult, WalletContext};
use std::sync::Arc;

/// Trait for executing tools in a flow step
pub trait Executor: Send + Sync {
    /// Execute a step with the given wallet context
    async fn execute_step(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
    ) -> Result<StepResult>;
}

/// Type alias for a shared tool executor
pub type SharedExecutor = Arc<dyn Executor>;

impl Executor for crate::execution::ToolExecutor {
    async fn execute_step(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
    ) -> Result<StepResult> {
        self.execute_step(step, wallet_context).await
    }
}

impl Executor for crate::execution::MockToolExecutor {
    async fn execute_step(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
    ) -> Result<StepResult> {
        self.execute_step(step, wallet_context).await
    }
}
