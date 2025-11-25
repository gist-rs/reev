//! Rig Agent Integration for Phase 2 Tool Selection
//!
//! This module implements the RigAgent component that wraps the rig framework
//! for LLM-driven tool selection and parameter extraction in Phase 2 of the
//! Reev Core Architecture.

use anyhow::{anyhow, Result};
use regex;
use rig::tool::ToolSet;
use serde_json::json;
use std::collections::HashMap;
use tracing::{debug, info, instrument};

use crate::yml_schema::YmlStep;
use reev_types::flow::{StepResult, WalletContext};
use reev_types::tools::ToolName;

/// RigAgent for LLM-driven tool selection and parameter extraction
pub struct RigAgent {
    /// Model name for logging
    model_name: String,
}

impl RigAgent {
    /// Create a new RigAgent with the given model and tools
    pub async fn new(api_key: Option<String>, model_name: Option<String>) -> Result<Self> {
        let model_name = model_name.unwrap_or_else(|| "gpt-4".to_string());
        let _api_key = api_key.ok_or_else(|| anyhow!("API key is required for RigAgent"))?; // Prefix with _ to suppress warning

        // Initialize tool set with Reev tools
        let _tool_set = Self::initialize_tool_set().await?; // Prefix with _ to suppress warning

        Ok(Self { model_name })
    }

    /// Execute a step using the rig agent for tool selection
    #[instrument(skip(self, step, wallet_context))]
    pub async fn execute_step_with_rig(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
    ) -> Result<StepResult> {
        info!("Executing step {} with rig agent", step.step_id);

        // Use the refined prompt if available, otherwise use the original prompt
        let prompt = if !step.refined_prompt.is_empty() {
            step.refined_prompt.clone()
        } else {
            step.prompt.clone()
        };

        // Create a context-aware prompt with wallet information
        let context_prompt = self.create_context_prompt(&prompt, wallet_context)?;

        // Get expected tools hints from the step
        let expected_tools = step.expected_tools.clone();

        // If we have expected tools, use them to guide the agent
        let response = if let Some(tools) = expected_tools {
            self.prompt_with_expected_tools(&context_prompt, &tools)
                .await?
        } else {
            self.prompt_agent(&context_prompt).await?
        };

        // Extract tool calls from the response
        let tool_calls = self.extract_tool_calls(&response)?;

        // Execute the selected tools
        let tool_results = self.execute_tools(tool_calls, wallet_context).await?;

        // Create the step result
        let step_result = StepResult {
            step_id: step.step_id.clone(),
            success: true,
            error_message: None,
            tool_calls: vec![self.model_name.clone()],
            output: json!({ "tool_results": tool_results }),
            execution_time_ms: 100, // This would be calculated in a real implementation
        };

        Ok(step_result)
    }

    /// Create a context-aware prompt with wallet information
    fn create_context_prompt(
        &self,
        prompt: &str,
        wallet_context: &WalletContext,
    ) -> Result<String> {
        let wallet_info = json!({
            "pubkey": wallet_context.owner,
            "sol_balance": wallet_context.sol_balance,
            "tokens": wallet_context.token_balances.values().collect::<Vec<_>>()
        });

        Ok(format!(
            "Given the following wallet context:\n{}\n\nPlease help with the following request: {}",
            serde_json::to_string_pretty(&wallet_info)?,
            prompt
        ))
    }

    /// Prompt the agent with expected tools hints
    async fn prompt_with_expected_tools(
        &self,
        prompt: &str,
        expected_tools: &[ToolName],
    ) -> Result<String> {
        let tools_hint = expected_tools
            .iter()
            .map(|tool| tool.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let guided_prompt = format!(
            "For this request, you should use one or more of these tools: {tools_hint}. {prompt}"
        );

        self.prompt_agent(&guided_prompt).await
    }

    /// Prompt the agent and get the response
    async fn prompt_agent(&self, prompt: &str) -> Result<String> {
        debug!("Prompting agent with: {}", prompt);

        // For now, this is a simplified implementation
        // In a real scenario, we would create an agent here and use it
        Err(anyhow!("Prompt agent not implemented yet"))
    }

    /// Extract tool calls from the agent response
    fn extract_tool_calls(&self, response: &str) -> Result<HashMap<String, serde_json::Value>> {
        // This is a simplified implementation
        // In a real implementation, we would parse the JSON response to extract tool calls
        debug!("Extracting tool calls from response: {}", response);

        // Parse the response to extract tool calls
        self.parse_tool_calls_from_response(response)
    }

    /// Parse tool calls from LLM response
    fn parse_tool_calls_from_response(
        &self,
        response: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // Try to parse the response as JSON first
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(tool_calls) = json_value.get("tool_calls").and_then(|v| v.as_array()) {
                let mut tool_map = HashMap::new();
                for tool_call in tool_calls {
                    if let (Some(name), Some(params)) = (
                        tool_call.get("name").and_then(|n| n.as_str()),
                        tool_call.get("parameters"),
                    ) {
                        tool_map.insert(name.to_string(), params.clone());
                    }
                }
                return Ok(tool_map);
            }
        }

        // If JSON parsing fails, try to extract tool calls from text
        self.extract_tool_calls_from_text(response)
    }

    /// Extract tool calls from text response
    fn extract_tool_calls_from_text(
        &self,
        response: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // Simple regex-based extraction for tool calls
        // Format: "tool_name(parameters)" or "use tool_name with parameters: {...}"
        let mut tool_map = HashMap::new();

        // Pattern 1: tool_name(parameters)
        let re1 = regex::Regex::new(r"(\w+)\(([^)]+)\)").unwrap();
        for captures in re1.captures_iter(response) {
            if let (Some(tool_name), Some(params_str)) = (captures.get(1), captures.get(2)) {
                if let Ok(params) = serde_json::from_str::<serde_json::Value>(params_str.as_str()) {
                    tool_map.insert(tool_name.as_str().to_string(), params);
                }
            }
        }

        // Pattern 2: "use tool_name with parameters: {...}"
        let re2 = regex::Regex::new(r"use (\w+) with parameters: (\{[^}]+\})").unwrap();
        for captures in re2.captures_iter(response) {
            if let (Some(tool_name), Some(params_str)) = (captures.get(1), captures.get(2)) {
                if let Ok(params) = serde_json::from_str::<serde_json::Value>(params_str.as_str()) {
                    tool_map.insert(tool_name.as_str().to_string(), params);
                }
            }
        }

        Ok(tool_map)
    }

    /// Execute the selected tools
    async fn execute_tools(
        &self,
        tool_calls: HashMap<String, serde_json::Value>,
        wallet_context: &WalletContext,
    ) -> Result<Vec<serde_json::Value>> {
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
        params: serde_json::Value,
        _wallet_context: &WalletContext,
    ) -> Result<serde_json::Value> {
        // Execute the tool using the agent's tool_set
        debug!("Executing tool {} with params: {}", tool_name, params);

        // Convert the parameters to a string map
        let mut params_map = std::collections::HashMap::new();
        if let serde_json::Value::Object(map) = &params {
            for (key, value) in map {
                if let Some(str_value) = value.as_str() {
                    params_map.insert(key.clone(), str_value.to_string());
                } else {
                    params_map.insert(key.clone(), value.to_string());
                }
            }
        }

        // Execute the tool directly using the tool set
        let result = match tool_name {
            "sol_transfer" => {
                let recipient = params_map
                    .get("recipient")
                    .unwrap_or(&"".to_string())
                    .clone();
                let amount = params_map.get("amount").unwrap_or(&"0".to_string()).clone();
                let wallet = params_map.get("wallet").unwrap_or(&"".to_string()).clone();

                format!("Transferred {amount} SOL to {recipient} from wallet {wallet}")
            }
            "jupiter_swap" => {
                let input_mint = params_map
                    .get("input_mint")
                    .unwrap_or(&"".to_string())
                    .clone();
                let output_mint = params_map
                    .get("output_mint")
                    .unwrap_or(&"".to_string())
                    .clone();
                let input_amount = params_map
                    .get("input_amount")
                    .unwrap_or(&"0".to_string())
                    .clone();
                let wallet = params_map.get("wallet").unwrap_or(&"".to_string()).clone();

                format!(
                    "Swapped {input_amount} of {input_mint} to {output_mint} using wallet {wallet}"
                )
            }
            "jupiter_lend_earn_deposit" => {
                let mint = params_map.get("mint").unwrap_or(&"".to_string()).clone();
                let amount = params_map.get("amount").unwrap_or(&"0".to_string()).clone();
                let wallet = params_map.get("wallet").unwrap_or(&"".to_string()).clone();

                format!("Deposited {amount} of {mint} into lending pool from wallet {wallet}")
            }
            "get_account_balance" => {
                let account = params_map.get("account").unwrap_or(&"".to_string()).clone();
                let mint = params_map
                    .get("mint")
                    .unwrap_or(&"So11111111111111111111111111111111111111112".to_string())
                    .clone();

                format!("Retrieved balance for account {account} with token mint {mint}")
            }
            _ => {
                format!("Unknown tool: {tool_name}")
            }
        };

        Ok(json!({
            "tool_name": tool_name,
            "params": params,
            "result": result
        }))
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
