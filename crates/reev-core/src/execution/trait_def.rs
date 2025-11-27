//! Trait for Tool Execution
//!
//! This module defines a trait for tool execution implementations.

use crate::yml_schema::YmlStep;
use anyhow::Result;
use reev_types::flow::{StepResult, WalletContext};
use std::sync::Arc;

/// Trait for executing tools in a flow step
#[async_trait::async_trait]
pub trait Executor: Send + Sync {
    /// Execute a step with given wallet context
    async fn execute_step(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
    ) -> Result<StepResult>;

    /// Execute a step with wallet context and previous step history
    async fn execute_step_with_history(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
        previous_results: &[StepResult],
    ) -> Result<StepResult>;
}

/// Type alias for a shared tool executor
pub type SharedExecutor = Arc<dyn Executor>;

#[async_trait::async_trait]
impl Executor for crate::execution::ToolExecutor {
    async fn execute_step(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
    ) -> Result<StepResult> {
        self.execute_step(step, wallet_context).await
    }

    async fn execute_step_with_history(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
        previous_results: &[StepResult],
    ) -> Result<StepResult> {
        self.execute_step_with_history(step, wallet_context, previous_results)
            .await
    }
}
