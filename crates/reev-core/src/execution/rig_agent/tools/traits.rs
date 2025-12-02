//! Core traits for tool execution
//!
//! This module contains the core traits and interfaces for tool execution.

use anyhow::Result;
use reev_agent::enhanced::common::AgentTools;
use reev_types::flow::WalletContext;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Trait for tool execution operations
#[allow(async_fn_in_trait)]
pub trait ToolExecutor {
    /// Execute the selected tools
    async fn execute_tools(
        &self,
        tool_calls: HashMap<String, Value>,
        wallet_context: &WalletContext,
    ) -> Result<Vec<Value>>;

    /// Execute a single tool
    async fn execute_single_tool(
        &self,
        tool_name: &str,
        params: Value,
        wallet_context: &WalletContext,
    ) -> Result<Value>;

    /// Execute SOL transfer
    async fn execute_sol_transfer(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<Value>;

    /// Execute Jupiter swap
    async fn execute_jupiter_swap(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<Value>;

    /// Execute Jupiter lend/earn deposit
    async fn execute_jupiter_lend_deposit(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<Value>;

    /// Execute get account balance
    async fn execute_get_account_balance(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<Value>;
}

/// Trait for accessing agent tools and HTTP client
pub trait AgentProvider {
    fn agent_tools(&self) -> Option<Arc<AgentTools>>;
}

/// Helper trait for getting or creating agent tools
pub trait AgentToolHelper {
    fn get_or_create_agent_tools(&self, wallet_context: &WalletContext) -> Result<Arc<AgentTools>>;
}
