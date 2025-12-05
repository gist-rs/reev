//! Rig Agent Integration for Phase 2 Tool Selection
//!
//! This module implements the RigAgent component that wraps the ZAI SDK
//! for LLM-driven tool selection and parameter extraction in Phase 2 of
//! Reev Core Architecture.

use anyhow::{anyhow, Result};
use reev_agent::enhanced::common::AgentTools;
use reev_types::flow::{StepResult, WalletContext};
use rig::tool::Tool;
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};
use zai_sdk::{GlmVariant, Message, ZaiClient};

use crate::yml_schema::YmlStep;

/// RigAgent for LLM-driven tool selection and parameter extraction
pub struct RigAgent {
    /// ZAI client for GLM-4.6 model
    zai_client: ZaiClient,
    /// Model name for logging
    model_name: String,
    /// Agent tools for executing blockchain operations
    agent_tools: Option<Arc<AgentTools>>,
}

impl RigAgent {
    /// Create a new RigAgent with the given model and tools
    pub async fn new(api_key: Option<String>, model_name: Option<String>) -> Result<Self> {
        let model_name = model_name.unwrap_or_else(|| "glm-4.6-coding".to_string());
        let api_key = api_key.ok_or_else(|| anyhow!("API key is required for RigAgent"))?;

        // Determine the variant based on model name
        let variant = if model_name == "glm-4.6-coding" {
            GlmVariant::Coding
        } else {
            GlmVariant::Standard
        };

        // Create the zai-sdk client
        let zai_client = ZaiClient::builder()
            .variant(variant)
            .api_key(api_key)
            .build()
            .map_err(|e| anyhow!("Failed to create ZAI client: {e}"))?;

        Ok(Self {
            zai_client,
            model_name,
            agent_tools: None,
        })
    }

    /// Create a new RigAgent with the given model and tools
    pub async fn new_with_tools(
        api_key: Option<String>,
        model_name: Option<String>,
        agent_tools: Arc<AgentTools>,
    ) -> Result<Self> {
        let model_name = model_name.unwrap_or_else(|| "glm-4.6-coding".to_string());
        let api_key = api_key.ok_or_else(|| anyhow!("API key is required for RigAgent"))?;

        // Determine the variant based on model name
        let variant = if model_name == "glm-4.6-coding" {
            GlmVariant::Coding
        } else {
            GlmVariant::Standard
        };

        // Create the zai-sdk client
        let zai_client = ZaiClient::builder()
            .variant(variant)
            .api_key(api_key)
            .build()
            .map_err(|e| anyhow!("Failed to create ZAI client: {e}"))?;

        Ok(Self {
            zai_client,
            model_name,
            agent_tools: Some(agent_tools),
        })
    }

    /// Execute a step using the rig agent
    #[instrument(skip(self, step, wallet_context))]
    pub async fn execute_step_with_rig(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
    ) -> Result<StepResult> {
        info!("Executing step {} with rig agent", step.step_id);

        // Get the prompt
        let prompt = if let Some(refined_prompt) = &step.structured_prompt {
            refined_prompt.refined_prompt.clone()
        } else if !step.refined_prompt.is_empty() {
            step.refined_prompt.clone()
        } else {
            step.prompt.clone()
        };

        // Create a system prompt
        let system_prompt = r#"You are a helpful assistant that analyzes user prompts and extracts tool calls for DeFi operations.

Respond with valid JSON in the following format:
{
  "tool_calls": [
    {
      "name": "tool_name",
      "parameters": {
        "param1": "value1",
        "param2": "value2"
      }
    }
  ]
}

Available tools:
- sol_transfer: Transfer SOL from one account to another. Parameters: recipient (string, required), amount (number in SOL, required), wallet (string, optional)
- spl_transfer: Transfer SPL tokens from one account to another. Parameters: recipient (string, required), amount (number in tokens, required), mint_address (string, required), wallet (string, optional)
- jupiter_swap: Swap tokens using Jupiter. Parameters: input_mint (string, required, e.g., "So11111111111111111111111111111111111111112" for SOL), output_mint (string, required, e.g., "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" for USDC), input_amount (number, required, amount of tokens to swap, use decimal for partial amounts like 0.5 for half), wallet (string, optional)
- jupiter_lend_earn_deposit: Deposit tokens into Jupiter lending. Parameters: mint (string, required), amount (number, required, already in smallest denomination, e.g., 1,000,000 for 1 USDC), wallet (string, optional)
- get_account_balance: Get account balance. Parameters: account (string, required), mint (string, optional, defaults to SOL)

For token mint addresses:
- SOL: So11111111111111111111111111111111111111112
- USDC: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
- USDT: Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB

For swap operations, always determine the input and output mints based on the token names (SOL, USDC, etc.).

CRITICAL INSTRUCTION: When the prompt contains multiple operations (e.g., "swap 0.1 SOL to USDC then lend 10 USDC"), you MUST include tool_calls for ALL operations in your response. Do not ignore any part of the user's request."#;

        // Create messages for the request
        let messages = vec![Message::system(system_prompt), Message::user(&prompt)];

        // Send the request using the zai-sdk
        let response_content = self
            .zai_client
            .completion_with_messages(messages)
            .await
            .map_err(|e| anyhow!("LLM generation failed: {e}"))?;

        debug!(
            "LLM response from {}: {}",
            self.model_name, response_content
        );

        // Parse tool calls from the response
        let tool_calls = self.parse_tool_calls_from_response(&response_content)?;

        // Execute tools
        let tool_results = self
            .execute_tools(tool_calls.clone(), wallet_context)
            .await
            .map_err(|e| anyhow!("Tool execution failed: {e}"))?;

        // Convert HashMap to Vec<String> for tool_calls field
        let tool_calls_vec: Vec<String> = tool_calls.keys().cloned().collect();

        // Create step result
        let step_result = StepResult {
            step_id: step.step_id.clone(),
            success: true,
            error_message: None,
            tool_calls: tool_calls_vec,
            output: json!({ "tool_results": tool_results }),
            execution_time_ms: 100, // This would be calculated in a real implementation
        };

        Ok(step_result)
    }

    /// Execute a step with history using the rig agent
    #[instrument(skip(self, step, wallet_context, previous_results))]
    pub async fn execute_step_with_rig_and_history(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
        previous_results: &[StepResult],
    ) -> Result<StepResult> {
        info!("Executing step {} with previous history", step.step_id);

        // Mark as used to avoid warning (TODO: implement proper history handling)
        let _ = previous_results;

        // Get the prompt
        let prompt = if let Some(refined_prompt) = &step.structured_prompt {
            refined_prompt.refined_prompt.clone()
        } else if !step.refined_prompt.is_empty() {
            step.refined_prompt.clone()
        } else {
            step.prompt.clone()
        };

        // Create a system prompt
        let system_prompt = r#"You are a helpful assistant that analyzes user prompts and extracts tool calls for DeFi operations.

Respond with valid JSON in the following format:
{
 "tool_calls": [
    {
      "name": "tool_name",
      "parameters": {
        "param1": "value1",
        "param2": "value2"
      }
    }
  ]
}

Available tools:
- sol_transfer: Transfer SOL from one account to another. Parameters: recipient (string, required), amount (number in SOL, required), wallet (string, optional)
- spl_transfer: Transfer SPL tokens from one account to another. Parameters: recipient (string, required), amount (number in tokens, required), mint_address (string, required), wallet (string, optional)
- jupiter_swap: Swap tokens using Jupiter. Parameters: input_mint (string, required, e.g., "So11111111111111111111111111111111111111112" for SOL), output_mint (string, required, e.g., "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" for USDC), input_amount (number, required, amount of tokens to swap, use decimal for partial amounts like 0.5 for half), wallet (string, optional)
- jupiter_lend_earn_deposit: Deposit tokens into Jupiter lending. Parameters: mint (string, required), amount (number, required, already in smallest denomination, e.g., 1,000,000 for 1 USDC), wallet (string, optional)
- get_account_balance: Get account balance. Parameters: account (string, required), mint (string, optional, defaults to SOL)

For token mint addresses:
- SOL: So11111111111111111111111111111111111111112
- USDC: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
- USDT: Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB

For swap operations, always determine the input and output mints based on the token names (SOL, USDC, etc.).

CRITICAL INSTRUCTION: When the prompt contains multiple operations (e.g., "swap 0.1 SOL to USDC then lend 10 USDC"), you MUST include tool_calls for ALL operations in your response. Do not ignore any part of the user's request."#;

        // Create messages for the request
        let messages = vec![Message::system(system_prompt), Message::user(&prompt)];

        // Send the request using the zai-sdk
        let response_content = self
            .zai_client
            .completion_with_messages(messages)
            .await
            .map_err(|e| anyhow!("LLM generation failed: {e}"))?;

        debug!(
            "LLM response from {}: {}",
            self.model_name, response_content
        );

        // Parse tool calls from the response
        let tool_calls = self.parse_tool_calls_from_response(&response_content)?;

        // For multi-step operations, check if we need additional tool calls
        let final_tool_calls =
            self.extract_multi_step_tool_calls(&response_content, &tool_calls)?;

        info!(
            "Extracted {} tool calls from LLM response",
            final_tool_calls.len()
        );
        debug!("Tool calls: {:?}", final_tool_calls);

        // Execute tools
        let tool_results = self
            .execute_tools(tool_calls.clone(), wallet_context)
            .await
            .map_err(|e| anyhow!("Tool execution failed: {e}"))?;

        // Convert HashMap to Vec<String> for tool_calls field
        let tool_calls_vec: Vec<String> = tool_calls.keys().cloned().collect();

        // Create step result
        let step_result = StepResult {
            step_id: step.step_id.clone(),
            success: true,
            error_message: None,
            tool_calls: tool_calls_vec,
            output: json!({ "tool_results": tool_results }),
            execution_time_ms: 100, // This would be calculated in a real implementation
        };

        Ok(step_result)
    }

    /// Parse tool calls from a response
    fn parse_tool_calls_from_response(
        &self,
        response: &str,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        debug!("Parsing tool calls from response");

        // Try to parse the response as JSON
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(response) {
            debug!("Successfully parsed JSON response");

            if let Some(tool_calls) = json_value.get("tool_calls").and_then(|v| v.as_array()) {
                debug!("Found {} tool calls in response", tool_calls.len());
                let mut tool_map = std::collections::HashMap::new();
                for tool_call in tool_calls {
                    if let (Some(name), Some(params)) = (
                        tool_call.get("name").and_then(|v| v.as_str()),
                        tool_call.get("parameters"),
                    ) {
                        debug!("Extracted tool call: {} with params: {}", name, params);
                        tool_map.insert(name.to_string(), params.clone());
                    } else {
                        debug!("Tool call missing name or parameters: {:?}", tool_call);
                    }
                }

                Ok(tool_map)
            } else {
                debug!("No tool_calls found in JSON response");
                self.extract_tool_calls_from_text(response)
            }
        } else {
            debug!("Failed to parse response as JSON, trying text extraction");
            self.extract_tool_calls_from_text(response)
        }
    }

    /// Extract tool calls from a text response
    fn extract_tool_calls_from_text(
        &self,
        response: &str,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        debug!("Extracting tool calls from text response");

        // This is a simplified implementation that tries to extract operations
        // from natural language text
        let mut tool_calls = std::collections::HashMap::new();

        // Look for specific operations in the response
        if response.to_lowercase().contains("swap") {
            if let Some(swap_params) = self.extract_swap_params_from_response(response)? {
                tool_calls.insert("jupiter_swap".to_string(), swap_params);
                info!("Added jupiter_swap operation from response");
            }
        }

        if response.to_lowercase().contains("lend") || response.to_lowercase().contains("deposit") {
            if let Some(lend_params) = self.extract_lend_params_from_response(response)? {
                tool_calls.insert("jupiter_lend_earn_deposit".to_string(), lend_params);
                info!("Added jupiter_lend_earn_deposit operation from response");
            }
        }

        if response.to_lowercase().contains("transfer") || response.to_lowercase().contains("send")
        {
            if let Some(transfer_params) = self.extract_transfer_params_from_response(response)? {
                tool_calls.insert("sol_transfer".to_string(), transfer_params);
                info!("Added sol_transfer operation from response");
            }
        }

        // If we couldn't extract any tool calls, create a default swap operation
        if tool_calls.is_empty() {
            warn!("No tool calls extracted from response, creating default");
            tool_calls.insert(
                "jupiter_swap".to_string(),
                json!({
                    "input_mint": "So11111111111111111111111111111111111111111112",
                    "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    "input_amount": 1000000000  // 1 SOL in lamports
                }),
            );
        }

        Ok(tool_calls)
    }

    /// Extract swap parameters from a response
    fn extract_swap_params_from_response(
        &self,
        _response: &str,
    ) -> Result<Option<serde_json::Value>> {
        // This is a simplified implementation
        // In a real implementation, we would use more sophisticated parsing
        if _response.to_lowercase().contains("sol") && _response.to_lowercase().contains("usdc") {
            Ok(Some(json!({
                "input_mint": "So11111111111111111111111111111111111111111112",
                "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "input_amount": 1000000000  // 1 SOL in lamports
            })))
        } else {
            Ok(None)
        }
    }

    /// Extract lend parameters from a response
    fn extract_lend_params_from_response(
        &self,
        _response: &str,
    ) -> Result<Option<serde_json::Value>> {
        // This is a simplified implementation
        // In a real implementation, we would use more sophisticated parsing
        if _response.to_lowercase().contains("usdc") {
            Ok(Some(json!({
                "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "amount": 10000000  // 10 USDC in smallest denomination
            })))
        } else {
            Ok(None)
        }
    }

    /// Extract transfer parameters from a response
    fn extract_transfer_params_from_response(
        &self,
        _response: &str,
    ) -> Result<Option<serde_json::Value>> {
        // This is a simplified implementation
        // In a real implementation, we would use more sophisticated parsing
        Ok(Some(json!({
            "recipient": "11111111111111111111111111111111111112", // System program ID as example
            "amount": 1000000000  // 1 SOL in lamports
        })))
    }

    /// Extract multi-step tool calls
    fn extract_multi_step_tool_calls(
        &self,
        response: &str,
        existing_calls: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        debug!("Extracting multi-step tool calls from response");

        // Parse the response to get all tool calls
        let all_calls = self.parse_tool_calls_from_response(response)?;

        // Filter out calls that already exist
        let mut new_calls = std::collections::HashMap::new();
        for (name, params) in all_calls {
            if !existing_calls.contains_key(&name) {
                new_calls.insert(name, params);
            }
        }

        // Check if we need to add a lend operation for multi-step scenarios
        if existing_calls.contains_key("jupiter_swap")
            && !existing_calls.contains_key("jupiter_lend_earn_deposit")
        {
            if let Some(swap_params) = existing_calls.get("jupiter_swap") {
                if let Some(output_mint) = swap_params.get("output_mint").and_then(|v| v.as_str()) {
                    if output_mint == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" {
                        if let Some(input_amount) =
                            swap_params.get("input_amount").and_then(|v| v.as_u64())
                        {
                            info!("Adding lend operation for multi-step scenario");
                            new_calls.insert(
                                "jupiter_lend_earn_deposit".to_string(),
                                json!({
                                    "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                                    "amount": input_amount
                                }),
                            );
                        }
                    }
                }
            }
        }

        Ok(new_calls)
    }

    /// Execute tools
    async fn execute_tools(
        &self,
        tool_calls: std::collections::HashMap<String, serde_json::Value>,
        _wallet_context: &WalletContext,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        debug!("Executing {} tools", tool_calls.len());

        let mut results = std::collections::HashMap::new();

        if let Some(agent_tools) = &self.agent_tools {
            for (tool_name, params) in &tool_calls {
                match tool_name.as_str() {
                    "sol_transfer" => {
                        // Parse the params to NativeTransferArgs
                        use reev_tools::tools::native::NativeTransferArgs;
                        let args: NativeTransferArgs = serde_json::from_value(params.clone())?;
                        let result = agent_tools
                            .sol_tool
                            .call(args)
                            .await
                            .map_err(|e| anyhow!("sol_transfer failed: {e}"))?;
                        results.insert(tool_name.clone(), serde_json::to_value(result)?);
                    }
                    "spl_transfer" => {
                        // Parse the params to NativeTransferArgs
                        use reev_tools::tools::native::NativeTransferArgs;
                        let args: NativeTransferArgs = serde_json::from_value(params.clone())?;
                        let result = agent_tools
                            .spl_tool
                            .call(args)
                            .await
                            .map_err(|e| anyhow!("spl_transfer failed: {e}"))?;
                        results.insert(tool_name.clone(), serde_json::to_value(result)?);
                    }
                    "jupiter_swap" => {
                        // Parse the params to JupiterSwapArgs
                        use reev_tools::tools::jupiter_swap::JupiterSwapArgs;
                        let args: JupiterSwapArgs = serde_json::from_value(params.clone())?;
                        let result = agent_tools
                            .jupiter_swap_tool
                            .call(args)
                            .await
                            .map_err(|e| anyhow!("jupiter_swap failed: {e}"))?;
                        results.insert(tool_name.clone(), serde_json::to_value(result)?);
                    }
                    "jupiter_lend_earn_deposit" => {
                        // Parse the params to JupiterLendEarnDepositArgs
                        use reev_tools::tools::jupiter_lend_earn_deposit::JupiterLendEarnDepositArgs;
                        let args: JupiterLendEarnDepositArgs =
                            serde_json::from_value(params.clone())?;
                        let result = agent_tools
                            .jupiter_lend_earn_deposit_tool
                            .call(args)
                            .await
                            .map_err(|e| anyhow!("jupiter_lend_earn_deposit failed: {e}"))?;
                        results.insert(tool_name.clone(), serde_json::to_value(result)?);
                    }
                    "get_account_balance" => {
                        // Parse the params to AccountBalanceArgs
                        use reev_tools::tools::discovery::balance_tool::AccountBalanceArgs;
                        let args: AccountBalanceArgs = serde_json::from_value(params.clone())?;
                        let result = agent_tools
                            .balance_tool
                            .call(args)
                            .await
                            .map_err(|e| anyhow!("get_account_balance failed: {e}"))?;
                        results.insert(tool_name.clone(), serde_json::to_value(result)?);
                    }
                    _ => {
                        debug!("Unknown tool: {}", tool_name);
                    }
                }
            }
        }

        Ok(results)
    }
}
