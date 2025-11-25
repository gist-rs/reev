use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde_json::json;
// use solana_sdk::signer::Signer; // Not used here
use tracing::{debug, error, info, instrument, warn};

use crate::execution::handlers::swap::jupiter_swap::*;
use crate::execution::handlers::transfer::sol_transfer::*;
use crate::execution::types::recovery_config::RecoveryConfig;
use crate::yml_schema::YmlStep;
use crate::YmlToolCall;

// use reev_lib::agent::RawInstruction; // Not used here
// use reev_lib::utils::{execute_transaction, get_keypair}; // Not used here
use reev_types::flow::{StepResult, WalletContext};
use reev_types::tools::ToolName;

// Import context resolver and AgentTools
use reev_agent::enhanced::common::AgentTools;
use rig::tool::Tool;

// Import RigAgent for tool selection
use crate::execution::rig_agent::RigAgent;

/// Executor for AI agent tools
pub struct ToolExecutor {
    agent_tools: Option<Arc<AgentTools>>,
    rig_agent: Option<Arc<RigAgent>>,
    api_key: Option<String>,
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
            rig_agent: None,
            api_key,
            _model_name: model_name,
        })
    }

    /// Set recovery configuration
    pub fn with_recovery_config(self, _config: RecoveryConfig) -> Self {
        // Recovery config would be stored here if needed
        self
    }

    /// Enable rig agent for tool selection
    pub async fn enable_rig_agent(mut self) -> Result<Self> {
        info!("Enabling rig agent for tool selection");
        self.rig_agent = Some(self.initialize_rig_agent().await?);
        Ok(self)
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

        // Use rig agent if available and the step has expected_tools
        if self.rig_agent.is_some() && step.expected_tools.is_some() {
            info!("Using rig agent for tool selection");
            let rig_agent = self.rig_agent.as_ref().unwrap();
            return rig_agent.execute_step_with_rig(step, wallet_context).await;
        }

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
            // Check if this is a transfer operation
            let prompt_lower = step.prompt.to_lowercase();
            if prompt_lower.contains("transfer") || prompt_lower.contains("send") {
                // If this is a transfer, create a YmlToolCall for SolTransfer
                info!("No expected tool calls specified for transfer, executing SolTransferTool directly");
                vec![YmlToolCall {
                    tool_name: reev_types::tools::ToolName::SolTransfer,
                    critical: true,
                    expected_parameters: None,
                }]
            } else {
                // If no specific tool calls are specified, execute jupiter_swap tool directly
                info!("No expected tool calls specified, executing jupiter_swap directly");
                vec![YmlToolCall {
                    tool_name: reev_types::tools::ToolName::JupiterSwap,
                    critical: true,
                    expected_parameters: None,
                }]
            }
        } else {
            step.expected_tool_calls.clone().unwrap_or_default()
        };

        // Handle special case where we need to execute a swap directly without expected parameters
        if !tool_calls.is_empty()
            && tool_calls[0].tool_name == ToolName::JupiterSwap
            && tool_calls[0].expected_parameters.is_none()
        {
            return self
                .execute_direct_jupiter_swap(tools, &wallet_context.owner, &step.prompt)
                .await;
        }

        // Handle special case where we need to execute a transfer directly without expected parameters
        if !tool_calls.is_empty()
            && tool_calls[0].tool_name == ToolName::SolTransfer
            && tool_calls[0].expected_parameters.is_none()
        {
            return self
                .execute_direct_sol_transfer(tools, &step.prompt, &wallet_context.owner)
                .await;
        }

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

            // Execute the tool and process the result
            let result = self
                .execute_tool_call(tool_call.tool_name.clone(), &params, &tools)
                .await;

            match result {
                Ok(tool_result) => {
                    tool_results.push(tool_result);
                }
                Err(e) => {
                    error!("Tool execution failed: {}", e);
                    tool_results.push(json!({
                        "tool_name": tool_call.tool_name,
                        "error": format!("Tool execution failed: {e}")
                    }));
                    all_success = false;
                    if first_error.is_none() {
                        first_error = Some(e);
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

    /// Execute a direct Jupiter swap operation without expected parameters
    async fn execute_direct_jupiter_swap(
        &self,
        tools: Arc<AgentTools>,
        wallet_owner: &str,
        prompt: &str,
    ) -> Result<StepResult> {
        execute_direct_jupiter_swap(&tools, wallet_owner, prompt).await
    }

    /// Execute a direct SOL transfer operation without expected parameters
    async fn execute_direct_sol_transfer(
        &self,
        tools: Arc<AgentTools>,
        prompt: &str,
        wallet_owner: &str,
    ) -> Result<StepResult> {
        execute_direct_sol_transfer(&tools, prompt, wallet_owner).await
    }

    /// Execute a specific tool call and return the result
    async fn execute_tool_call(
        &self,
        tool_name: ToolName,
        params: &HashMap<String, serde_json::Value>,
        tools: &Arc<AgentTools>,
    ) -> Result<serde_json::Value> {
        match tool_name {
            ToolName::JupiterSwap => self.handle_jupiter_swap(params, tools).await,
            ToolName::JupiterLendEarnDeposit => {
                self.handle_jupiter_lend_earn_deposit(params, tools).await
            }
            ToolName::SolTransfer => self.handle_sol_transfer(params, tools).await,
            _ => {
                let error_msg = format!("Unsupported tool: {tool_name:?}");
                error!("{}", error_msg);
                Err(anyhow!(error_msg))
            }
        }
    }

    /// Handle Jupiter swap operation
    async fn handle_jupiter_swap(
        &self,
        params: &HashMap<String, serde_json::Value>,
        tools: &Arc<AgentTools>,
    ) -> Result<serde_json::Value> {
        handle_jupiter_swap(params, tools).await
    }

    /// Handle Jupiter Lend Earn Deposit operation
    async fn handle_jupiter_lend_earn_deposit(
        &self,
        params: &HashMap<String, serde_json::Value>,
        tools: &Arc<AgentTools>,
    ) -> Result<serde_json::Value> {
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
            Ok(json!({
                "tool_name": "JupiterLendEarnDeposit",
                "instructions": instructions
            }))
        } else {
            // If parsing fails, include the raw response
            warn!("Failed to parse JupiterLendEarnDeposit response");
            Ok(json!({
                "tool_name": "JupiterLendEarnDeposit",
                "error": "Failed to parse response",
                "raw_response": result
            }))
        }
    }

    /// Handle SOL transfer operation
    async fn handle_sol_transfer(
        &self,
        params: &HashMap<String, serde_json::Value>,
        tools: &Arc<AgentTools>,
    ) -> Result<serde_json::Value> {
        handle_sol_transfer(params, tools).await
    }

    /// Initialize tools for a wallet
    async fn initialize_tools(&self, wallet_pubkey: &str) -> Result<AgentTools> {
        info!("Initializing tools for wallet: {}", wallet_pubkey);

        // Set up key_map for authentication
        let mut key_map = HashMap::new();
        if let Some(ref api_key) = self.api_key {
            key_map.insert("ZAI_API_KEY".to_string(), api_key.clone());
        }

        // Add wallet pubkey to key_map so tools can access it
        key_map.insert("WALLET_PUBKEY".to_string(), wallet_pubkey.to_string());

        let tools = AgentTools::new(key_map);
        Ok(tools)
    }

    /// Initialize RigAgent for tool selection
    async fn initialize_rig_agent(&self) -> Result<Arc<RigAgent>> {
        info!("Initializing RigAgent for tool selection");

        let rig_agent = Arc::new(
            RigAgent::new(self.api_key.clone(), Some("glm-4.6-coding".to_string())).await?,
        );

        Ok(rig_agent)
    }
}
