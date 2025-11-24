//! Tool Executor for reev-core Executor
//!
//! This module implements actual tool execution for the executor module,
//! replacing the mock implementation with real tool calls via reev-tools
//! and reev-agent integration.

use crate::yml_schema::YmlStep;
use anyhow::{anyhow, Result};
use reev_agent::enhanced::common::{AgentTools, UnifiedGLMAgent};
use reev_types::flow::{StepResult, WalletContext};
use rig::tool::Tool;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

/// Tool Executor for executing actual tools
pub struct ToolExecutor {
    /// Agent tools for execution
    agent_tools: Option<Arc<AgentTools>>,
    /// API key for LLM calls
    api_key: Option<String>,
    /// Model name for tool parameter generation
    model_name: String,
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new().expect("Failed to create default ToolExecutor")
    }
}

impl ToolExecutor {
    /// Create a new tool executor
    pub fn new() -> Result<Self> {
        // Use mock executor in test mode
        if std::env::var("REEV_TEST_MODE").is_ok() {
            return Err(anyhow!(
                "ToolExecutor cannot be created in test mode. Use MockToolExecutor instead."
            ));
        }

        let model_name =
            std::env::var("GLM_MODEL").unwrap_or_else(|_| "glm-4.6-coding".to_string());
        let api_key = std::env::var("ZAI_API_KEY").ok();

        Ok(Self {
            agent_tools: None,
            api_key,
            model_name,
        })
    }

    /// Set the model name
    pub fn with_model_name(mut self, model_name: &str) -> Self {
        self.model_name = model_name.to_string();
        self
    }

    /// Execute a single step with actual tools
    #[instrument(skip(self, step, wallet_context))]
    pub async fn execute_step(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
    ) -> Result<StepResult> {
        info!("Executing step: {}", step.prompt);

        // Initialize tools if not already done
        let tools = if let Some(tools) = &self.agent_tools {
            tools.clone()
        } else {
            Arc::new(self.initialize_tools(&wallet_context.owner).await?)
        };

        // Generate tool parameters using LLM if needed
        let tool_calls = if step.expected_tool_calls.is_none()
            || step.expected_tool_calls.as_ref().unwrap().is_empty()
        {
            // If no specific tool calls are specified, generate them from the prompt
            self.generate_tool_calls(&step.prompt, wallet_context)
                .await?
        } else {
            step.expected_tool_calls.clone().unwrap_or_default()
        };

        // Execute each tool call
        let mut tool_results = Vec::new();
        let mut all_success = true;
        let mut first_error = None;

        for tool_call in &tool_calls {
            match self.execute_single_tool(&tools, tool_call).await {
                Ok(result) => {
                    tool_results.push(json!({
                        "tool_name": tool_call.tool_name,
                        "result": result
                    }));
                }
                Err(e) => {
                    error!("Failed to execute tool {}: {}", tool_call.tool_name, e);
                    tool_results.push(json!({
                        "tool_name": tool_call.tool_name,
                        "error": e.to_string()
                    }));

                    if tool_call.critical {
                        all_success = false;
                        if first_error.is_none() {
                            first_error = Some(e);
                        }
                    }
                }
            }
        }

        let step_result = StepResult {
            step_id: uuid::Uuid::new_v4().to_string(),
            success: all_success,
            error_message: first_error.map(|e| e.to_string()),
            tool_calls: tool_calls
                .iter()
                .map(|t| format!("{:?}", t.tool_name))
                .collect(),
            output: json!({ "tool_results": tool_results }),
            execution_time_ms: 100, // Default execution time
        };

        Ok(step_result)
    }

    /// Initialize tools for a wallet
    async fn initialize_tools(&self, wallet_pubkey: &str) -> Result<AgentTools> {
        info!("Initializing tools for wallet: {}", wallet_pubkey);

        let mut key_map = HashMap::new();
        if let Some(ref api_key) = self.api_key {
            key_map.insert("ZAI_API_KEY".to_string(), api_key.clone());
        }

        let tools = AgentTools::new(key_map);
        Ok(tools)
    }

    /// Generate tool calls from a prompt using LLM
    async fn generate_tool_calls(
        &self,
        prompt: &str,
        wallet_context: &WalletContext,
    ) -> Result<Vec<crate::yml_schema::YmlToolCall>> {
        debug!("Generating tool calls from prompt: {}", prompt);

        // Create request for LLM
        let payload = reev_agent::LlmRequest {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            prompt: format!(
                "Generate tool calls for the following request: {}\n\nWallet Context:\n{} SOL\n{} USD total value",
                prompt,
                wallet_context.sol_balance as f64 / 1_000_000_000.0,
                wallet_context.total_value_usd
            ),
            context_prompt: "Generate a list of tool calls to fulfill the user request. For each tool, specify the tool name and parameters."
                .to_string(),
            model_name: self.model_name.clone(),
            mock: false,
            initial_state: None,
            allowed_tools: None,
            account_states: None,
            key_map: Some(HashMap::new()),
        };

        // Set up key_map for authentication
        let mut key_map = HashMap::new();
        if let Some(ref api_key) = self.api_key {
            key_map.insert("ZAI_API_KEY".to_string(), api_key.clone());
        }

        // Call the unified GLM agent
        let result = UnifiedGLMAgent::run(&self.model_name, payload, key_map)
            .await
            .map_err(|e| {
                error!("Failed to generate tool calls with GLM: {}", e);
                anyhow!("LLM tool generation failed: {e}")
            })?;

        // Extract tool calls from the response
        let response = result.execution_result.summary;

        // Parse the response to extract tool calls
        // This is a simplified implementation - in a production system,
        // we would need more robust parsing of the LLM response
        let tool_calls = self.parse_tool_calls_from_response(&response)?;

        Ok(tool_calls)
    }

    /// Parse tool calls from LLM response
    fn parse_tool_calls_from_response(
        &self,
        response: &str,
    ) -> Result<Vec<crate::yml_schema::YmlToolCall>> {
        // This is a simplified implementation
        // In a production system, we would use structured output or better parsing

        let tool_name = if response.to_lowercase().contains("swap") {
            "jupiter_swap"
        } else if response.to_lowercase().contains("lend") {
            "jupiter_lend_earn_deposit"
        } else if response.to_lowercase().contains("transfer") {
            "sol_transfer"
        } else {
            return Ok(Vec::new()); // No tool calls detected
        };

        let tool_name_enum = if tool_name == "jupiter_swap" {
            reev_types::tools::ToolName::JupiterSwap
        } else if tool_name == "jupiter_lend_earn_deposit" {
            reev_types::tools::ToolName::JupiterLendEarnDeposit
        } else if tool_name == "sol_transfer" {
            reev_types::tools::ToolName::SolTransfer
        } else {
            return Err(anyhow!("Unknown tool: {tool_name}"));
        };

        let tool_call = crate::yml_schema::YmlToolCall {
            tool_name: tool_name_enum,
            critical: true,
            expected_parameters: Some(HashMap::new()), // Empty parameters for simplicity
        };

        Ok(vec![tool_call])
    }

    /// Execute a single tool call
    async fn execute_single_tool(
        &self,
        tools: &AgentTools,
        tool_call: &crate::yml_schema::YmlToolCall,
    ) -> Result<serde_json::Value> {
        debug!("Executing tool: {}", tool_call.tool_name);

        // Extract parameters from expected_parameters
        let params = if let Some(ref params) = tool_call.expected_parameters {
            params.clone()
        } else {
            HashMap::new()
        };

        // Execute the actual tool through the Tool trait
        match tool_call.tool_name {
            reev_types::tools::ToolName::JupiterSwap => {
                // Execute Jupiter swap tool with actual implementation
                info!("Executing JupiterSwap with parameters: {:?}", params);

                // Convert parameters to the expected format for JupiterSwapTool
                let swap_args = reev_tools::tools::jupiter_swap::JupiterSwapArgs {
                    user_pubkey: params
                        .get("user_pubkey")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    input_mint: params
                        .get("input_mint")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    output_mint: params
                        .get("output_mint")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    amount: params.get("amount").and_then(|v| v.as_u64()).unwrap_or(0),
                    slippage_bps: params
                        .get("slippage_bps")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u16),
                };

                let result = tools
                    .jupiter_swap_tool
                    .call(swap_args)
                    .await
                    .map_err(|e| anyhow!("JupiterSwap execution failed: {e}"))?;
                Ok(serde_json::to_value(result)?)
            }
            reev_types::tools::ToolName::JupiterLendEarnDeposit => {
                // Execute Jupiter lend earn deposit tool with actual implementation
                info!(
                    "Executing JupiterLendEarnDeposit with parameters: {:?}",
                    params
                );

                // Convert parameters to the expected format for JupiterLendEarnDepositTool
                let deposit_args =
                    reev_tools::tools::jupiter_lend_earn_deposit::JupiterLendEarnDepositArgs {
                        user_pubkey: params
                            .get("user_pubkey")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        asset_mint: params
                            .get("asset_mint")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        amount: params.get("amount").and_then(|v| v.as_u64()).unwrap_or(0),
                    };

                let result = tools
                    .jupiter_lend_earn_deposit_tool
                    .call(deposit_args)
                    .await
                    .map_err(|e| anyhow!("JupiterLendEarnDeposit execution failed: {e}"))?;
                Ok(serde_json::to_value(result)?)
            }
            reev_types::tools::ToolName::SolTransfer => {
                // Execute SOL transfer tool with actual implementation
                info!("Executing SolTransfer with parameters: {:?}", params);

                // Convert parameters to the expected format for SolTransferTool
                let transfer_args = reev_tools::tools::native::NativeTransferArgs {
                    user_pubkey: params
                        .get("user_pubkey")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    recipient_pubkey: params
                        .get("recipient_pubkey")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    amount: params.get("amount").and_then(|v| v.as_u64()).unwrap_or(0),
                    operation: reev_tools::tools::native::NativeTransferOperation::Sol,
                    mint_address: params
                        .get("mint_address")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                };

                let result = tools
                    .sol_tool
                    .call(transfer_args)
                    .await
                    .map_err(|e| anyhow!("SolTransfer execution failed: {e}"))?;
                Ok(serde_json::to_value(result)?)
            }
            _ => {
                error!("Unknown tool: {:?}", tool_call.tool_name);
                Err(anyhow!("Unknown tool: {:?}", tool_call.tool_name))
            }
        }
    }
}
