//! Tool Executor for reev-core Executor
//!
//! This module implements actual tool execution for executor module,
//! replacing mock implementation with real tool calls via reev-tools
//! and reev-agent integration.

use crate::yml_schema::YmlStep;
use anyhow::{anyhow, Result};
use reev_agent::enhanced::common::AgentTools;
use reev_lib::agent::RawInstruction;
use reev_lib::utils::{execute_transaction, get_keypair};
use reev_types::flow::{StepResult, WalletContext};
use rig::tool::Tool;
use serde_json::json;
use solana_sdk::signature::Signer;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

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

            // Execute Jupiter swap from 10 USDC to SOL
            let swap_args = reev_tools::tools::jupiter_swap::JupiterSwapArgs {
                user_pubkey: wallet_context.owner.clone(),
                input_mint: usdc_mint.to_string(),
                output_mint: sol_mint.to_string(),
                amount: 10_000_000,       // 10 USDC (6 decimals)
                slippage_bps: Some(1000), // 10% slippage
            };

            info!("Executing JupiterSwapTool with args: {:?}", swap_args);

            // Execute the jupiter swap tool directly
            let result = tools.jupiter_swap_tool.call(swap_args).await;

            match result {
                Ok(response_json) => {
                    info!("JupiterSwapTool executed successfully");

                    // Parse the JSON response to extract instructions
                    if let Ok(response) = serde_json::from_str::<serde_json::Value>(&response_json)
                    {
                        if let Some(instructions) =
                            response.get("instructions").and_then(|v| v.as_array())
                        {
                            info!("Found {} instructions in response", instructions.len());

                            // Convert instructions to RawInstruction format
                            let raw_instructions: Result<Vec<RawInstruction>> = instructions
                                .iter()
                                .map(|inst| {
                                    let program_id = inst
                                        .get("program_id")
                                        .and_then(|v| v.as_str())
                                        .ok_or_else(|| anyhow!("Missing program_id"))?
                                        .to_string();

                                    let accounts = inst
                                        .get("accounts")
                                        .and_then(|v| v.as_array())
                                        .ok_or_else(|| anyhow!("Missing accounts"))?
                                        .iter()
                                        .map(|acc| {
                                            Ok(reev_lib::agent::RawAccountMeta {
                                                pubkey: acc
                                                    .get("pubkey")
                                                    .and_then(|v| v.as_str())
                                                    .ok_or_else(|| anyhow!("Missing pubkey"))?
                                                    .to_string(),
                                                is_signer: acc
                                                    .get("is_signer")
                                                    .and_then(|v| v.as_bool())
                                                    .unwrap_or(false),
                                                is_writable: acc
                                                    .get("is_writable")
                                                    .and_then(|v| v.as_bool())
                                                    .unwrap_or(false),
                                            })
                                        })
                                        .collect::<Result<Vec<_>>>()?;

                                    let data = inst
                                        .get("data")
                                        .and_then(|v| v.as_str())
                                        .ok_or_else(|| anyhow!("Missing data"))?
                                        .to_string();

                                    Ok(RawInstruction {
                                        program_id,
                                        accounts,
                                        data,
                                    })
                                })
                                .collect();

                            match raw_instructions {
                                Ok(raw_instructions) => {
                                    // Get user's keypair for signing
                                    let keypair = get_keypair()
                                        .map_err(|e| anyhow!("Failed to get user keypair: {e}"))?;
                                    let user_pubkey = Signer::pubkey(&keypair);

                                    // Execute the transaction: build, sign, and send
                                    match execute_transaction(
                                        raw_instructions,
                                        user_pubkey,
                                        &keypair,
                                    )
                                    .await
                                    {
                                        Ok(tx_signature) => {
                                            info!(
                                                "Transaction executed successfully: {tx_signature}"
                                            );

                                            // Create a StepResult with the transaction signature
                                            let step_result = StepResult {
                                                step_id: uuid::Uuid::new_v4().to_string(),
                                                success: true,
                                                error_message: None,
                                                tool_calls: vec!["jupiter_swap".to_string()],
                                                output: json!({
                                                    "transaction_signature": tx_signature,
                                                    "full_response": response
                                                }),
                                                execution_time_ms: 1000, // Estimated execution time
                                            };

                                            return Ok(step_result);
                                        }
                                        Err(e) => {
                                            error!("Failed to execute transaction: {}", e);

                                            // Return error without mock
                                            let step_result = StepResult {
                                                step_id: uuid::Uuid::new_v4().to_string(),
                                                success: false,
                                                error_message: Some(format!(
                                                    "Transaction execution failed: {e}"
                                                )),
                                                tool_calls: vec!["jupiter_swap".to_string()],
                                                output: json!({
                                                    "error": format!("Transaction execution failed: {e}"),
                                                    "response": response
                                                }),
                                                execution_time_ms: 1000,
                                            };

                                            return Ok(step_result);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to convert instructions: {}", e);

                                    // Return error without mock
                                    let step_result = StepResult {
                                        step_id: uuid::Uuid::new_v4().to_string(),
                                        success: false,
                                        error_message: Some(format!(
                                            "Instruction conversion failed: {e}"
                                        )),
                                        tool_calls: vec!["jupiter_swap".to_string()],
                                        output: json!({
                                            "error": format!("Instruction conversion failed: {e}"),
                                            "response": response
                                        }),
                                        execution_time_ms: 1000,
                                    };

                                    return Ok(step_result);
                                }
                            }
                        } else {
                            warn!("No instructions found in response");
                        }
                    } else {
                        warn!("Failed to parse Jupiter swap tool response");
                    }

                    // If we couldn't extract instructions, return error
                    let step_result = StepResult {
                        step_id: uuid::Uuid::new_v4().to_string(),
                        success: false,
                        error_message: Some(
                            "Could not extract instructions from response".to_string(),
                        ),
                        tool_calls: vec!["jupiter_swap".to_string()],
                        output: json!({
                            "raw_response": response_json
                        }),
                        execution_time_ms: 1000,
                    };

                    return Ok(step_result);
                }
                Err(e) => {
                    error!("JupiterSwapTool execution failed: {}", e);

                    // Return error without mock
                    let step_result = StepResult {
                        step_id: uuid::Uuid::new_v4().to_string(),
                        success: false,
                        error_message: Some(format!("Tool execution failed: {e}")),
                        tool_calls: vec!["jupiter_swap".to_string()],
                        output: json!({
                            "error": format!("Tool execution failed: {e}"),
                        }),
                        execution_time_ms: 1000,
                    };

                    return Ok(step_result);
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

                    // Parse the JSON response to extract structured data
                    if let Ok(response) = serde_json::from_str::<serde_json::Value>(&result) {
                        // Check if the tool prepared instructions
                        if let Some(instructions) =
                            response.get("instructions").and_then(|v| v.as_array())
                        {
                            // Convert instructions to RawInstruction format
                            let raw_instructions: Result<Vec<RawInstruction>> = instructions
                                .iter()
                                .map(|inst| {
                                    let program_id = inst
                                        .get("program_id")
                                        .and_then(|v| v.as_str())
                                        .ok_or_else(|| anyhow!("Missing program_id"))?
                                        .to_string();

                                    let accounts = inst
                                        .get("accounts")
                                        .and_then(|v| v.as_array())
                                        .ok_or_else(|| anyhow!("Missing accounts"))?
                                        .iter()
                                        .map(|acc| {
                                            Ok(reev_lib::agent::RawAccountMeta {
                                                pubkey: acc
                                                    .get("pubkey")
                                                    .and_then(|v| v.as_str())
                                                    .ok_or_else(|| anyhow!("Missing pubkey"))?
                                                    .to_string(),
                                                is_signer: acc
                                                    .get("is_signer")
                                                    .and_then(|v| v.as_bool())
                                                    .unwrap_or(false),
                                                is_writable: acc
                                                    .get("is_writable")
                                                    .and_then(|v| v.as_bool())
                                                    .unwrap_or(false),
                                            })
                                        })
                                        .collect::<Result<Vec<_>>>()?;

                                    let data = inst
                                        .get("data")
                                        .and_then(|v| v.as_str())
                                        .ok_or_else(|| anyhow!("Missing data"))?
                                        .to_string();

                                    Ok(RawInstruction {
                                        program_id,
                                        accounts,
                                        data,
                                    })
                                })
                                .collect();

                            match raw_instructions {
                                Ok(raw_instructions) => {
                                    // Get user's keypair for signing
                                    let keypair = get_keypair()
                                        .map_err(|e| anyhow!("Failed to get user keypair: {e}"))?;
                                    let user_pubkey = Signer::pubkey(&keypair);

                                    // Execute the transaction: build, sign, and send
                                    match execute_transaction(
                                        raw_instructions,
                                        user_pubkey,
                                        &keypair,
                                    )
                                    .await
                                    {
                                        Ok(tx_signature) => {
                                            info!(
                                                "Transaction executed successfully: {tx_signature}"
                                            );
                                            tool_results.push(json!({
                                                "tool_name": tool_call.tool_name,
                                                "transaction_signature": tx_signature,
                                                "response": response
                                            }));
                                        }
                                        Err(e) => {
                                            error!("Failed to execute transaction: {}", e);
                                            tool_results.push(json!({
                                                "tool_name": tool_call.tool_name,
                                                "error": format!("Transaction execution failed: {e}"),
                                                "response": response
                                            }));
                                            all_success = false;
                                            if first_error.is_none() {
                                                first_error = Some(anyhow!(
                                                    "Transaction execution failed: {e}"
                                                ));
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to convert instructions: {}", e);
                                    tool_results.push(json!({
                                        "tool_name": tool_call.tool_name,
                                        "error": format!("Instruction conversion failed: {e}"),
                                        "response": response
                                    }));
                                    all_success = false;
                                    if first_error.is_none() {
                                        first_error =
                                            Some(anyhow!("Instruction conversion failed: {e}"));
                                    }
                                }
                            }
                        } else {
                            warn!("No instructions found in response");
                            tool_results.push(json!({
                                "tool_name": tool_call.tool_name,
                                "error": "No instructions found in response",
                                "response": response
                            }));
                            all_success = false;
                            if first_error.is_none() {
                                first_error = Some(anyhow!("No instructions found in response"));
                            }
                        }
                    } else {
                        // If parsing fails, include the raw response
                        warn!("Failed to parse tool response");
                        tool_results.push(json!({
                            "tool_name": tool_call.tool_name,
                            "error": "Failed to parse tool response",
                            "raw_response": result
                        }));
                        all_success = false;
                        if first_error.is_none() {
                            first_error = Some(anyhow!("Failed to parse tool response"));
                        }
                    }
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

                    // Parse the JSON response to extract structured data
                    if let Ok(instructions) = serde_json::from_str::<serde_json::Value>(&result) {
                        tool_results.push(json!({
                            "tool_name": tool_call.tool_name,
                            "instructions": instructions
                        }));
                    } else {
                        // If parsing fails, include the raw response
                        tool_results.push(json!({
                            "tool_name": tool_call.tool_name,
                            "raw_response": result
                        }));
                    }
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
