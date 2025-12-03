//! Tool Execution Implementation
//!
//! This module contains the implementation of the ToolExecutor trait that
//! delegates to the specific tool implementations.

use anyhow::{anyhow, Result};
use reev_agent::enhanced::common::AgentTools;
use reev_types::flow::WalletContext;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

use super::traits::{AgentProvider, AgentToolHelper, ToolExecutor};
use super::{
    account_balance::execute_get_account_balance, jupiter_lend::execute_jupiter_lend_deposit,
    jupiter_swap::execute_jupiter_swap, sol_transfer::execute_sol_transfer,
    spl_transfer::execute_spl_transfer,
};

/// Implementation for any struct with agent_tools field
impl<T> ToolExecutor for T
where
    T: AgentProvider + crate::execution::rig_agent::prompting::HttpProvider,
{
    /// Execute the selected tools
    async fn execute_tools(
        &self,
        tool_calls: HashMap<String, Value>,
        wallet_context: &WalletContext,
    ) -> Result<Vec<Value>> {
        let mut results = Vec::new();

        for (tool_name, params) in tool_calls {
            let result = self
                .execute_single_tool(&tool_name, params, wallet_context)
                .await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Execute a single tool
    async fn execute_single_tool(
        &self,
        tool_name: &str,
        params: Value,
        wallet_context: &WalletContext,
    ) -> Result<Value> {
        // Execute the tool using the agent's tool_set
        debug!("Executing tool {} with params: {}", tool_name, params);

        // Convert the parameters to a string map
        let mut params_map = HashMap::new();
        if let Value::Object(map) = &params {
            for (key, value) in map {
                if let Some(str_value) = value.as_str() {
                    params_map.insert(key.clone(), str_value.to_string());
                } else {
                    // Handle numeric values more carefully to avoid scientific notation issues
                    match value {
                        Value::Number(n) => {
                            if let Some(u) = n.as_u64() {
                                // For u64 values, use directly to avoid scientific notation
                                params_map.insert(key.clone(), u.to_string());
                            } else if let Some(i) = n.as_i64() {
                                // For i64 values, use directly to avoid scientific notation
                                params_map.insert(key.clone(), i.to_string());
                            } else if let Some(f) = n.as_f64() {
                                // For floating point values, format without scientific notation
                                // Check if it's an integer value first to preserve precision
                                if f.fract() == 0.0 && f.abs() < (i64::MAX as f64) {
                                    params_map.insert(key.clone(), (f as i64).to_string());
                                } else {
                                    params_map.insert(key.clone(), f.to_string());
                                }
                            } else {
                                params_map.insert(key.clone(), value.to_string());
                            }
                        }
                        _ => {
                            params_map.insert(key.clone(), value.to_string());
                        }
                    }
                }
            }
        }

        // Execute the tool based on its name
        match tool_name {
            "sol_transfer" => execute_sol_transfer(&params_map, wallet_context).await,
            "spl_transfer" => execute_spl_transfer(&params_map, wallet_context).await,
            "jupiter_swap" => execute_jupiter_swap(&params_map, wallet_context).await,
            "jupiter_lend_earn_deposit" => {
                let agent_tools = self.get_or_create_agent_tools(wallet_context)?;
                execute_jupiter_lend_deposit(&params_map, wallet_context, agent_tools).await
            }
            "get_account_balance" => execute_get_account_balance(&params_map, wallet_context).await,
            _ => Ok(json!({
                "tool_name": tool_name,
                "params": params,
                "error": format!("Unknown tool: {tool_name}")
            })),
        }
    }

    /// Execute SOL transfer
    async fn execute_sol_transfer(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<Value> {
        execute_sol_transfer(params, wallet_context).await
    }

    /// Execute Jupiter swap
    async fn execute_jupiter_swap(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<Value> {
        execute_jupiter_swap(params, wallet_context).await
    }

    /// Execute Jupiter lend/earn deposit
    async fn execute_jupiter_lend_deposit(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<Value> {
        let agent_tools = self.get_or_create_agent_tools(wallet_context)?;
        execute_jupiter_lend_deposit(params, wallet_context, agent_tools).await
    }

    /// Execute get account balance
    async fn execute_get_account_balance(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<Value> {
        execute_get_account_balance(params, wallet_context).await
    }

    /// Execute SPL transfer
    async fn execute_spl_transfer(
        &self,
        params: &HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<Value> {
        execute_spl_transfer(params, wallet_context).await
    }
}

/// Implementation for any struct with agent_tools and HttpProvider capabilities
impl<T> AgentToolHelper for T
where
    T: AgentProvider + crate::execution::rig_agent::prompting::HttpProvider,
{
    fn get_or_create_agent_tools(&self, wallet_context: &WalletContext) -> Result<Arc<AgentTools>> {
        // Use the existing AgentTools if available
        if let Some(ref tools) = self.agent_tools() {
            return Ok(Arc::clone(tools));
        }

        // Create new AgentTools using the wallet context
        let keypair = reev_lib::get_keypair().map_err(|e| {
            anyhow!(
                "Failed to get keypair for wallet {}: {}",
                wallet_context.owner,
                e
            )
        })?;

        // Include both public key and private key base58 in key_map
        let mut key_map = std::collections::HashMap::new();
        key_map.insert("WALLET_PUBKEY".to_string(), wallet_context.owner.clone());
        key_map.insert("WALLET_KEYPAIR".to_string(), keypair.to_base58_string());
        Ok(Arc::new(AgentTools::new(key_map)))
    }
}
