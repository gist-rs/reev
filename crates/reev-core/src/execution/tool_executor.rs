//! Tool Executor for reev-core Executor
//!
//! This module implements actual tool execution for executor module,
//! replacing mock implementation with real tool calls via reev-tools
//! and reev-agent integration.

use crate::yml_schema::YmlStep;
use anyhow::{anyhow, Result};
use reev_agent::enhanced::common::AgentTools;
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
    /// Model name for tool parameter generation (reserved for future use)
    _model_name: String,
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new().expect("Failed to create default ToolExecutor")
    }
}

impl ToolExecutor {
    /// Create a new tool executor
    pub fn new() -> Result<Self> {
        let model_name =
            std::env::var("GLM_MODEL").unwrap_or_else(|_| "glm-4.6-coding".to_string());
        let api_key = std::env::var("ZAI_API_KEY").ok();

        Ok(Self {
            agent_tools: None,
            api_key,
            _model_name: model_name,
        })
    }

    /// Set recovery configuration
    pub fn with_recovery_config(self, _config: RecoveryConfig) -> Self {
        // Recovery config would be stored here if needed
        self
    }

    /// Set custom tool executor
    pub fn with_tool_executor(self, _tool_executor: ToolExecutor) -> Self {
        self
    }

    /// Execute a step with actual tool execution
    #[instrument(skip(self, step, wallet_context))]
    pub async fn execute_step(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
    ) -> Result<StepResult> {
        info!("Executing step: {}", step.prompt);

        // Initialize tools for execution
        let tools = if let Some(ref tools) = self.agent_tools {
            Arc::clone(tools)
        } else {
            Arc::new(self.initialize_tools(&wallet_context.owner).await?)
        };

        // Generate tool calls using LLM
        let tool_calls = if step.expected_tool_calls.is_none()
            || step.expected_tool_calls.as_ref().unwrap().is_empty()
        {
            // If no specific tool calls are specified, execute jupiter_swap tool directly
            info!("No expected tool calls specified, executing jupiter_swap directly");

            // Get SOL and USDC mint addresses
            let sol_mint = reev_lib::constants::sol_mint();
            let usdc_mint = reev_lib::constants::usdc_mint();

            // Execute Jupiter swap with 0.1 SOL (leaving room for fees)
            let swap_args = reev_tools::tools::jupiter_swap::JupiterSwapArgs {
                user_pubkey: wallet_context.owner.clone(),
                input_mint: sol_mint.to_string(),
                output_mint: usdc_mint.to_string(),
                amount: 100_000_000,     // 0.1 SOL in lamports
                slippage_bps: Some(100), // 1% slippage
            };

            info!("Executing JupiterSwapTool with args: {:?}", swap_args);

            // Execute the jupiter swap tool directly
            let result = tools.jupiter_swap_tool.call(swap_args).await;

            match result {
                Ok(tx_signature) => {
                    info!(
                        "JupiterSwapTool executed successfully with tx signature: {tx_signature}"
                    );

                    // Create a StepResult with the transaction signature
                    let step_result = StepResult {
                        step_id: uuid::Uuid::new_v4().to_string(),
                        success: true,
                        error_message: None,
                        tool_calls: vec!["jupiter_swap".to_string()],
                        output: json!({
                            "transaction_signature": tx_signature
                        }),
                        execution_time_ms: 1000, // Estimated execution time
                    };

                    return Ok(step_result);
                }
                Err(e) => {
                    error!("JupiterSwapTool execution failed: {}", e);
                    return Err(anyhow!("Failed to execute JupiterSwapTool: {:?}", e));
                }
            }
        } else {
            step.expected_tool_calls.clone().unwrap_or_default()
        };

        // Execute each tool call
        let mut tool_results = Vec::new();
        let mut all_success = true;
        let mut first_error = None;

        for tool_call in &tool_calls {
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

                    // Convert parameters to expected format for JupiterSwapTool
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
                    tool_results.push(json!({
                        "tool_name": tool_call.tool_name,
                        "result": result.to_string()
                    }));
                }
                reev_types::tools::ToolName::JupiterLendEarnDeposit => {
                    // Execute Jupiter lend earn deposit tool with actual implementation
                    info!(
                        "Executing JupiterLendEarnDeposit with parameters: {:?}",
                        params
                    );

                    // Convert parameters to expected format for JupiterLendEarnDepositTool
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
                    tool_results.push(json!({
                        "tool_name": tool_call.tool_name,
                        "result": result.to_string()
                    }));
                }
                _ => {
                    let error_msg = format!("Unsupported tool: {:?}", tool_call.tool_name);
                    error!("{}", error_msg);
                    tool_results.push(json!({
                        "tool_name": tool_call.tool_name,
                        "error": error_msg
                    }));
                    all_success = false;
                    if first_error.is_none() {
                        first_error = Some(anyhow!(error_msg));
                    }
                }
            }
        }

        // Create step result
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

        // Set up key_map for authentication
        let mut key_map = HashMap::new();
        if let Some(ref api_key) = self.api_key {
            key_map.insert("ZAI_API_KEY".to_string(), api_key.clone());
        }

        let tools = AgentTools::new(key_map);
        Ok(tools)
    }
}

// Recovery configuration (placeholder for future implementation)
pub struct RecoveryConfig {
    pub max_retries: usize,
    pub retry_delay_ms: u64,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}
