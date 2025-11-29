//! Rig Agent Integration for Phase 2 Tool Selection
//!
//! This module implements the RigAgent component that wraps rig framework
//! for LLM-driven tool selection and parameter extraction in Phase 2 of
//! Reev Core Architecture.

use anyhow::{anyhow, Result};
use reev_agent::enhanced::common::AgentTools;
use reev_types::flow::{StepResult, WalletContext};
use rig::tool::ToolSet;
use serde_json::json;

use std::string::String;
use std::sync::Arc;
use tracing::{info, instrument};

use crate::yml_schema::YmlStep;

// Import modules
mod context;
mod enhancement;
mod prompting;
mod tool_execution;
mod types;

// Re-export types and traits
pub use context::ContextProvider;
pub use enhancement::{
    BalanceCalculator, ConstraintBuilder, ConstraintType, ContextPromptBuilder,
    ContextUpdateResult, DynamicContextUpdater, OperationHistory, OperationHistoryBuilder,
    ParameterValidator, StepConstraint, ValidationReport,
};
pub use prompting::{HttpProvider, MultiStepHandler, PromptProvider};
pub use tool_execution::{AgentProvider, AgentToolHelper, ToolExecutor};
pub use types::*;

/// RigAgent for LLM-driven tool selection and parameter extraction
pub struct RigAgent {
    /// Model name for logging
    model_name: String,
    /// API key for the LLM service
    api_key: String,
    /// HTTP client for direct API calls
    http_client: reqwest::Client,
    /// Agent tools for executing blockchain operations
    agent_tools: Option<Arc<AgentTools>>,
    // Dynamic context updater for enhanced context passing
    // Removed for now to avoid mutability issues
    // context_updater: Option<DynamicContextUpdater>,
    // Parameter validator for tool execution
    // Removed for now to avoid mutability issues
    // parameter_validator: Option<ParameterValidator>,
}

impl RigAgent {
    /// Create a new RigAgent with the given model and tools
    pub async fn new(api_key: Option<String>, model_name: Option<String>) -> Result<Self> {
        let model_name = model_name.unwrap_or_else(|| "gpt-4".to_string());
        let api_key = api_key.ok_or_else(|| anyhow!("API key is required for RigAgent"))?;

        // Initialize tool set with Reev tools
        let _tool_set = Self::initialize_tool_set().await?; // Prefix with _ to suppress warning

        Ok(Self {
            model_name,
            api_key,
            http_client: reqwest::Client::new(),
            agent_tools: None,
            // Removed for now to avoid mutability issues
            // context_updater: None,
            // parameter_validator: None,
        })
    }

    /// Create a new RigAgent with the given model and tools
    pub async fn new_with_tools(
        api_key: Option<String>,
        model_name: Option<String>,
        agent_tools: Arc<AgentTools>,
    ) -> Result<Self> {
        let model_name = model_name.unwrap_or_else(|| "gpt-4".to_string());
        let api_key = api_key.ok_or_else(|| anyhow!("API key is required for RigAgent"))?;

        // Initialize tool set with Reev tools
        let _tool_set = Self::initialize_tool_set().await?; // Prefix with _ to suppress warning

        Ok(Self {
            model_name,
            api_key,
            http_client: reqwest::Client::new(),
            agent_tools: Some(agent_tools),
            // Removed for now to avoid mutability issues
            // context_updater: None,
            // parameter_validator: None,
        })
    }

    /// Execute a step using the rig agent for tool selection
    #[instrument(skip(self, step, wallet_context))]
    pub async fn execute_step_with_rig(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
    ) -> Result<StepResult> {
        self.execute_step_with_rig_and_history(step, wallet_context, &[])
            .await
    }

    /// Execute a step with rig agent and previous step history
    pub async fn execute_step_with_rig_and_history(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
        previous_results: &[StepResult],
    ) -> Result<StepResult> {
        info!("Executing step {} with rig agent", step.step_id);

        // Debug log to verify current context before creating the prompt
        info!(
            "DEBUG: execute_step_with_rig_and_history - USDC balance in context: {:?}",
            wallet_context
                .token_balances
                .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                .map(|t| t.balance)
        );

        // Use the refined prompt if available, otherwise use the original prompt
        let prompt = if !step.refined_prompt.is_empty() {
            step.refined_prompt.clone()
        } else {
            step.prompt.clone()
        };

        // Create YML context and convert to prompt
        let yml_context = self.create_yml_context(step, wallet_context, previous_results)?;
        let context_prompt = self.yml_context_to_prompt(&yml_context, &prompt)?;

        // Log the YML context for debugging
        info!(
            "Generated YML context for step {}: {:?}",
            step.step_id, yml_context
        );

        // Get expected tools hints from the step
        let expected_tools = step.expected_tools.clone();

        // If we have expected tools, use them to guide the agent
        let response = if let Some(tools) = expected_tools {
            info!("Using expected tools to guide agent: {:?}", tools);
            self.prompt_with_expected_tools(&context_prompt, &tools)
                .await?
        } else {
            info!("No expected tools provided, using general agent prompt");
            self.prompt_agent(&context_prompt).await?
        };

        info!("Got response from agent: {}", response);

        info!("DEBUG: Parsing tool calls from LLM response: {}", response);

        // Extract tool calls from the response
        let tool_calls = self.extract_tool_calls(&response)?;

        // Let's LLM handle multi-step detection implicitly in the response
        // We don't use rule-based detection to determine if this is multi-step
        info!("DEBUG: Initial tool_calls count = {}", tool_calls.len());
        info!("DEBUG: Initial tool_calls = {:?}", tool_calls);
        info!("DEBUG: Response = {}", response);

        // For multi-step operations, execute tools sequentially with context updates
        let tool_results = if tool_calls.len() > 1 {
            info!("Executing multiple operations sequentially with context updates");
            self.execute_multi_step_operations(tool_calls.clone(), wallet_context, &prompt)
                .await?
        } else {
            // Single operation, execute normally
            info!("Executing single operation");
            let results = self
                .execute_tools(tool_calls.clone(), wallet_context)
                .await?;

            // Convert Vec<serde_json::Value> to a single Value with tool name keys
            let mut result_map = serde_json::Map::new();
            // Get tool names from the keys of tool_calls
            let tool_names: Vec<String> = tool_calls.keys().cloned().collect();
            for (tool_name, result) in tool_names.iter().zip(results) {
                result_map.insert(tool_name.clone(), result);
            }
            serde_json::Value::Object(result_map)
        };

        info!("Tool execution results: {:?}", tool_results);

        // Create list of tool names that were executed
        let executed_tool_names: Vec<String> = tool_calls.keys().cloned().collect();

        // Create the step result
        let step_result = StepResult {
            step_id: step.step_id.clone(),
            success: true,
            error_message: None,
            tool_calls: executed_tool_names,
            output: json!({ "tool_results": tool_results }),
            execution_time_ms: 100, // This would be calculated in a real implementation
        };

        Ok(step_result)
    }

    /// Execute multiple operations sequentially with context updates between operations
    /// This is crucial for proper multi-step execution where later operations depend on results of earlier ones
    async fn execute_multi_step_operations(
        &self,
        tool_calls: std::collections::HashMap<String, serde_json::Value>,
        initial_wallet_context: &WalletContext,
        _original_prompt: &str,
    ) -> Result<serde_json::Value> {
        info!("Executing multi-step operations with context updates");

        let mut all_results = serde_json::Map::new();
        let mut current_wallet_context = initial_wallet_context.clone();
        let mut operation_history = Vec::new();

        // Sort operations to ensure deterministic execution
        let mut sorted_operations: Vec<_> = tool_calls.into_iter().collect();
        // For now, we'll use a simple heuristic to order operations
        // In a full implementation, we would use LLM to determine the correct order
        sorted_operations.sort_by(|a, b| {
            // Jupiter swaps should typically happen before Jupiter lends
            match (a.0.as_str(), b.0.as_str()) {
                ("jupiter_swap", "jupiter_lend_earn_deposit") => std::cmp::Ordering::Less,
                ("jupiter_lend_earn_deposit", "jupiter_swap") => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            }
        });

        for (tool_name, params) in sorted_operations {
            info!(
                "Executing operation: {} with params: {:?}",
                tool_name, params
            );

            // Create a map with just this tool call
            let single_tool_call =
                std::collections::HashMap::from([(tool_name.clone(), params.clone())]);

            // Execute this operation
            let tool_results = self
                .execute_tools(single_tool_call, &current_wallet_context)
                .await?;
            info!("Operation results: {:?}", tool_results);

            // Store the results
            all_results.insert(
                tool_name.clone(),
                serde_json::Value::Array(tool_results.clone()),
            );

            // Create a history entry for this operation
            let history_entry = serde_json::json!({
                "tool_name": tool_name,
                "params": params,
                "result": tool_results
            });
            operation_history.push(history_entry);

            // Update wallet context for the next operation
            // This is crucial for multi-step operations where state changes
            if let Some(updated_context) = self.update_wallet_context_from_operation_result(
                &current_wallet_context,
                &tool_name,
                Some(&serde_json::Value::Array(tool_results.clone())),
            )? {
                current_wallet_context = updated_context;
                info!("Updated wallet context for next operation");
            }
        }

        // Add operation history to the results
        all_results.insert(
            "operation_history".to_string(),
            serde_json::Value::Array(operation_history),
        );

        Ok(serde_json::Value::Object(all_results))
    }

    /// Update wallet context based on the result of an operation
    /// This is essential for proper multi-step execution where later operations depend on earlier results
    fn update_wallet_context_from_operation_result(
        &self,
        current_context: &WalletContext,
        tool_name: &str,
        operation_result: Option<&serde_json::Value>,
    ) -> Result<Option<WalletContext>> {
        let operation_result = match operation_result {
            Some(result) => result,
            None => return Ok(None),
        };

        // Extract the actual transaction result from the operation result
        let transaction_result = operation_result
            .get("result")
            .or_else(|| operation_result.get("transaction"))
            .or(Some(operation_result));

        if let Some(result) = transaction_result {
            // For Jupiter swap operations, update token balances
            if tool_name == "jupiter_swap" {
                // Check if we have balance information in the result
                if let (Some(before), Some(after)) =
                    (result.get("before_balance"), result.get("after_balance"))
                {
                    info!("Updating token balances from swap result");
                    // Create a new wallet context with updated balances
                    let mut new_context = current_context.clone();

                    // Update token balances based on the swap result
                    // This is a simplified implementation - in a full implementation,
                    // we would parse the actual transaction result and update all affected tokens
                    if let Some(_before_bal) = before.get("USDC").and_then(|v| v.as_f64()) {
                        if let Some(after_bal) = after.get("USDC").and_then(|v| v.as_f64()) {
                            if let Some(usdc_token) = new_context
                                .token_balances
                                .get_mut("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                            {
                                usdc_token.balance = after_bal as u64;
                            }
                        }
                    }

                    return Ok(Some(new_context));
                }
            }

            // For Jupiter lend operations, update relevant balances
            if tool_name == "jupiter_lend_earn_deposit" {
                info!("Updating token balances from lend result");
                // Create a new wallet context with updated balances
                let mut new_context = current_context.clone();

                // Update token balances based on the lend result
                if let Some(amount) = operation_result
                    .get("params")
                    .and_then(|p| p.get("amount"))
                    .and_then(|a| a.as_u64())
                {
                    if let Some(usdc_token) = new_context
                        .token_balances
                        .get_mut("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                    {
                        usdc_token.balance = usdc_token.balance.saturating_sub(amount);
                    }

                    // Add jUSDC token to the wallet if it doesn't exist
                    if !new_context
                        .token_balances
                        .contains_key("jupsoL7By9suyDaGK735BLahFzhWd8vFjYUjdnFnJsw")
                    {
                        use reev_types::flow::TokenBalance;
                        new_context.token_balances.insert(
                            "jupsoL7By9suyDaGK735BLahFzhWd8vFjYUjdnFnJsw".to_string(),
                            TokenBalance {
                                mint: "jupsoL7By9suyDaGK735BLahFzhWd8vFjYUjdnFnJsw".to_string(),
                                balance: amount,
                                decimals: Some(6),
                                symbol: Some("jUSDC".to_string()),
                                formatted_amount: None,
                                owner: Some(current_context.owner.clone()),
                            },
                        );
                    }
                }

                return Ok(Some(new_context));
            }
        }

        Ok(None)
    }

    /// Validate parameters before tool execution
    pub fn validate_parameters(
        &self,
        _tool_name: &str,
        _parameters: &serde_json::Value,
        _step_number: usize,
    ) -> Result<ValidationReport> {
        // Create default validation report for now
        Ok(ValidationReport::new())
    }

    /// Initialize the tool set with Reev tools
    async fn initialize_tool_set() -> Result<ToolSet> {
        // Create a tool set with all Reev tools
        // For now, we'll create a minimal tool set as a placeholder
        // In a full implementation, we would add all Reev tools (SolTransfer, JupiterSwap, etc.)

        // Use the agent builder to create tools directly
        let tool_set = ToolSet::default();

        Ok(tool_set)
    }
}

// Implement required traits for RigAgent
impl HttpProvider for RigAgent {
    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn api_key(&self) -> &str {
        &self.api_key
    }

    fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }
}

impl AgentProvider for RigAgent {
    fn agent_tools(&self) -> Option<Arc<AgentTools>> {
        self.agent_tools.clone()
    }
}
