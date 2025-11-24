//! Tool Executor for reev-core Executor
//!
//! This module implements actual tool execution for executor module,
//! replacing mock implementation with real tool calls via reev-tools
//! and reev-agent integration.

use crate::yml_schema::YmlStep;
use anyhow::{anyhow, Result};
use reev_agent::enhanced::{
    common::{AgentTools, UnifiedGLMAgent},
    UnifiedGLMData,
};
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
        let model_name =
            std::env::var("GLM_MODEL").unwrap_or_else(|_| "glm-4.6-coding".to_string());
        let api_key = std::env::var("ZAI_API_KEY").ok();

        Ok(Self {
            agent_tools: None,
            api_key,
            model_name,
        })
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
            // If no specific tool calls are specified, generate them from the prompt
            let unified_data = self
                .get_unified_data(&step.prompt, wallet_context, &tools)
                .await?;

            // Extract transaction signature from execution results
            let has_signature = !unified_data.execution_result.signatures.is_empty();
            let signatures = unified_data.execution_result.signatures;

            // Create the step result with the execution results
            let step_result = StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                success: has_signature,
                error_message: if !has_signature {
                    Some("No transaction signature found".to_string())
                } else {
                    None
                },
                tool_calls: vec!["jupiter_swap".to_string()],
                output: json!({
                    "tool_results": unified_data.execution_result.transactions,
                    "signatures": signatures
                }),
                execution_time_ms: 100, // Default execution time
            };

            return Ok(step_result);
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
                        "result": result
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
                        "result": result
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

    /// Get UnifiedGLMData to access execution results
    async fn get_unified_data(
        &self,
        prompt: &str,
        wallet_context: &WalletContext,
        _tools: &AgentTools,
    ) -> Result<UnifiedGLMData> {
        debug!("Getting unified data from LLM for prompt: {}", prompt);

        // Create request for LLM with proper wallet context
        let payload = reev_agent::LlmRequest {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            prompt: format!(
                "Generate tool calls for the following request: {}\n\nWallet Context:\nOwner: {}\n{} SOL\n{} USD total value",
                prompt,
                wallet_context.owner,
                wallet_context.sol_balance as f64 / 1_000_000_000.0,
                wallet_context.total_value_usd
            ),
            context_prompt: "Generate a list of tool calls to fulfill user request. For each tool, specify: tool name and parameters."
                .to_string(),
            model_name: self.model_name.clone(),
            mock: false,
            initial_state: None,
            allowed_tools: Some(vec![
                reev_types::ToolName::JupiterSwap.to_string(),
                reev_types::ToolName::JupiterLendEarnDeposit.to_string(),
                reev_types::ToolName::SolTransfer.to_string(),
                reev_types::ToolName::GetAccountBalance.to_string(),
            ]),
            account_states: None,
            key_map: Some(HashMap::new()),
        };

        // Set up key_map for authentication
        let mut key_map = HashMap::new();
        if let Some(ref api_key) = self.api_key {
            key_map.insert("ZAI_API_KEY".to_string(), api_key.clone());
        }

        // Call() unified GLM agent
        UnifiedGLMAgent::run(&self.model_name, payload, key_map)
            .await
            .map_err(|e| {
                error!("Failed to generate tool calls with GLM: {}", e);
                anyhow!("LLM tool generation failed: {e}")
            })
    }
}

pub type SharedExecutor = Arc<ToolExecutor>;
