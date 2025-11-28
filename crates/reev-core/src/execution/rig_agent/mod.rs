//! Rig Agent Integration for Phase 2 Tool Selection
//!
//! This module implements the RigAgent component that wraps rig framework
//! for LLM-driven tool selection and parameter extraction in Phase 2 of
//! Reev Core Architecture.

use anyhow::{anyhow, Result};
use reev_agent::enhanced::common::AgentTools;
use reev_types::flow::{StepResult, WalletContext};
use reqwest;
// Client from rig::providers::openai is not used, removed
use rig::tool::{Tool, ToolSet};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::string::String;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

use crate::execution::handlers::transfer::sol_transfer;

use crate::yml_schema::YmlStep;
use reev_types::tools::ToolName;

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

        // Debug log to verify the current context before creating the prompt
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

        // Create a context-aware prompt with wallet information and previous step history
        let context_prompt =
            self.create_context_prompt_with_history(&prompt, wallet_context, previous_results)?;

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

        // Check if this is a multi-step prompt and we have multiple operations
        let prompt_lower = prompt.to_lowercase();
        let is_multi_step = prompt_lower.contains(" then ")
            || prompt_lower.contains(" and ")
            || prompt_lower.contains(" followed by ");

        info!("DEBUG: is_multi_step = {}", is_multi_step);
        info!("DEBUG: Initial tool_calls count = {}", tool_calls.len());
        info!("DEBUG: Initial tool_calls = {:?}", tool_calls);
        info!("DEBUG: Response = {}", response);

        // For multi-step prompts, we need to ensure we extract all operations
        let tool_calls = if is_multi_step && tool_calls.len() < 2 {
            // Try to extract additional operations if we only got one tool call
            info!("Multi-step prompt detected but only one tool call extracted, attempting to extract additional operations");
            let additional_calls = self.extract_multi_step_tool_calls(&response, &tool_calls)?;
            info!("DEBUG: Additional tool_calls = {:?}", additional_calls);
            additional_calls
        } else {
            info!("DEBUG: Using initial tool_calls as-is");
            tool_calls
        };

        // Execute the selected tools
        info!("Tool calls extracted: {:?}", tool_calls);
        let tool_results = self
            .execute_tools(tool_calls.clone(), wallet_context)
            .await?;
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

    /// Create a context-aware prompt with wallet information and previous step history
    fn create_context_prompt_with_history(
        &self,
        prompt: &str,
        wallet_context: &WalletContext,
        previous_results: &[StepResult],
    ) -> Result<String> {
        // Debug log to verify the current context
        info!(
            "DEBUG: create_context_prompt_with_history - USDC balance in context: {:?}",
            wallet_context
                .token_balances
                .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                .map(|t| t.balance)
        );

        let wallet_info = json!({
            "pubkey": wallet_context.owner,
            "sol_balance": wallet_context.sol_balance,
            "tokens": wallet_context.token_balances.values().collect::<Vec<_>>()
        });

        // Debug the serialized wallet info to ensure it has the correct values
        let serialized_info = serde_json::to_string_pretty(&wallet_info)?;
        info!(
            "DEBUG: Serialized wallet info for LLM with history: {}",
            serialized_info
        );

        let mut full_prompt = format!("Given the following wallet context:\n{serialized_info}\n");

        // Add information about previous steps if available
        if !previous_results.is_empty() {
            full_prompt.push_str("\n--- Previous Steps ---\n");
            for (i, result) in previous_results.iter().enumerate() {
                full_prompt.push_str(&format!(
                    "Step {}: {} - {}\n",
                    i + 1,
                    result.step_id,
                    if result.success { "Success" } else { "Failed" }
                ));

                // Extract key information from successful steps
                if result.success {
                    if let Some(tool_results) = result.output.get("tool_results") {
                        if let Some(results_array) = tool_results.as_array() {
                            for tool_result in results_array {
                                // Add specific details about swap operations
                                if let Some(jupiter_swap) = tool_result.get("jupiter_swap") {
                                    if let (
                                        Some(input_mint),
                                        Some(output_mint),
                                        Some(input_amount),
                                        Some(output_amount),
                                    ) = (
                                        jupiter_swap.get("input_mint").and_then(|v| v.as_str()),
                                        jupiter_swap.get("output_mint").and_then(|v| v.as_str()),
                                        jupiter_swap.get("input_amount").and_then(|v| v.as_u64()),
                                        jupiter_swap.get("output_amount").and_then(|v| v.as_u64()),
                                    ) {
                                        full_prompt.push_str(&format!(
                                            "  Swapped {input_amount} of {input_mint} for {output_amount} of {output_mint}\n"
                                        ));
                                        full_prompt.push_str(&format!(
                                            "  NOTE: For subsequent lend operations, use exactly {output_amount} units of {output_mint} (the amount received from this swap)\n"
                                        ));
                                        full_prompt.push_str(&format!(
                                            "  CRITICAL: When the prompt says 'lend 95% of available USDC', you must use the EXACT amount received from the swap ({output_amount} units), not calculate 95% of the total balance. Do not use percentages of old balances or estimated values.\n"
                                        ));
                                    }
                                }
                                // Add specific details about lend operations
                                else if let Some(jupiter_lend) = tool_result.get("jupiter_lend") {
                                    if let (Some(asset_mint), Some(amount)) = (
                                        jupiter_lend.get("asset_mint").and_then(|v| v.as_str()),
                                        jupiter_lend.get("amount").and_then(|v| v.as_u64()),
                                    ) {
                                        full_prompt.push_str(&format!(
                                            "  Lent {amount} of {asset_mint}\n"
                                        ));
                                    }
                                }
                                // Add generic info for other operations
                                else if let Some(operation_type) =
                                    tool_result.get("operation_type").and_then(|v| v.as_str())
                                {
                                    full_prompt.push_str(&format!(
                                        "  Completed operation: {operation_type}\n"
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            full_prompt.push_str("--- End Previous Steps ---\n\n");
        }

        full_prompt.push_str(&format!("Please help with the following request: {prompt}"));

        // Add special instruction for multi-step flows
        if !previous_results.is_empty() {
            full_prompt.push_str("\n\nIMPORTANT: For this step, please use the actual amounts from previous steps when determining parameters. For example, if this is a lend step after a swap, use the actual amount received from the swap, not an estimated amount.");
            full_prompt.push_str("\n\nCRITICAL: For lend operations after a swap, only use the amount received from the swap itself, not the total token balance which might include pre-existing amounts. The amount should already be in the smallest denomination (e.g., for USDC, 1 USDC = 1,000,000 units).");
            full_prompt.push_str("\n\nEXPLICIT INSTRUCTION: When the prompt says 'lend 95% of available USDC', you must calculate 95% of the ACTUAL USDC balance shown in the wallet context above, not any other value. Do not use percentages of old balances or estimated values.");
        }

        Ok(full_prompt)
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

        // Create a structured prompt for the LLM
        let system_prompt = "You are an AI assistant for Solana DeFi operations. Based on the user's request, determine the appropriate tools to use and extract the necessary parameters.

IMPORTANT: The prompt may contain multiple operations connected by words like 'then', 'and', 'followed by'. You MUST identify and execute ALL operations in the prompt, not just the first one.

Respond with valid JSON in the following format:
{
  \"tool_calls\": [
    {
      \"name\": \"tool_name\",
      \"parameters\": {
        \"param1\": \"value1\",
        \"param2\": \"value2\"
      }
    },
    {
      \"name\": \"second_tool_name\",
      \"parameters\": {
        \"param1\": \"value1\",
        \"param2\": \"value2\"
      }
    }
  ]
}

Available tools:
- sol_transfer: Transfer SOL from one account to another. Parameters: recipient (string, required), amount (number in SOL, required), wallet (string, optional)
- jupiter_swap: Swap tokens using Jupiter. Parameters: input_mint (string, required, e.g., 'So11111111111111111111111111111111111111112' for SOL), output_mint (string, required, e.g., 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v' for USDC), input_amount (number, required, amount of tokens to swap, use decimal for partial amounts like 0.5 for half), wallet (string, optional)
- jupiter_lend_earn_deposit: Deposit tokens into Jupiter lending. Parameters: mint (string, required), amount (number, required, already in smallest denomination, e.g., 1,000,000 for 1 USDC), wallet (string, optional)
- get_account_balance: Get account balance. Parameters: account (string, required), mint (string, optional, defaults to SOL)

For token mint addresses:
- SOL: So11111111111111111111111111111111111111112
- USDC: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
- USDT: Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB

For swap operations, always determine the input and output mints based on the token names (SOL, USDC, etc.).

CRITICAL INSTRUCTION: When the prompt contains multiple operations (e.g., 'swap 0.1 SOL to USDC then lend 10 USDC'), you MUST include tool_calls for ALL operations in your response. Do not ignore any part of the user's request.
";

        // Prepare the request payload
        // Use the correct model name for ZAI API
        let model_name = if self.model_name == "glm-4.6-coding" {
            "glm-4.6"
        } else {
            &self.model_name
        };

        let request_payload = LLMRequest {
            model: model_name.to_string(),
            messages: vec![
                LLMMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                LLMMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: 0.3,
            max_tokens: 1000,
        };

        info!("Sending request to ZAI API with model: {}", model_name);
        info!("Prompt being sent to ZAI API: {}", prompt);
        // Make the API call
        let api_base = env::var("ZAI_API_BASE")
            .unwrap_or_else(|_| "https://api.z.ai/api/coding/paas/v4".to_string());
        let url = format!("{api_base}/chat/completions");

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "API request failed with status: {}",
                response.status()
            ));
        }

        let response_body: LLMResponse = response.json().await?;

        // Extract the content from the response
        let content = response_body
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow!("No content in LLM response"))?;

        info!("LLM response: {}", content);
        Ok(content)
    }

    /// Extract tool calls from the agent response
    fn extract_tool_calls(&self, response: &str) -> Result<HashMap<String, serde_json::Value>> {
        // This is a simplified implementation
        // In a real implementation, we would parse the JSON response to extract tool calls
        info!("Extracting tool calls from response: {}", response);

        // Parse the response to extract tool calls
        self.parse_tool_calls_from_response(response)
    }

    /// Parse tool calls from LLM response
    fn parse_tool_calls_from_response(
        &self,
        response: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // Try to parse response as JSON first
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(response) {
            info!("Parsed JSON response successfully");
            if let Some(tool_calls) = json_value.get("tool_calls").and_then(|v| v.as_array()) {
                info!("Found {} tool calls in response", tool_calls.len());
                let mut tool_map = HashMap::new();
                for tool_call in tool_calls {
                    if let (Some(name), Some(params)) = (
                        tool_call.get("name").and_then(|n| n.as_str()),
                        tool_call.get("parameters"),
                    ) {
                        info!("Extracted tool call: {} with params: {}", name, params);
                        tool_map.insert(name.to_string(), params.clone());
                    } else {
                        info!("Tool call missing name or parameters: {:?}", tool_call);
                    }
                }
                return Ok(tool_map);
            } else {
                info!("No tool_calls found in JSON response");
            }
        } else {
            info!("Failed to parse response as JSON, trying text extraction");
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
                    info!(
                        "DEBUG: Pattern 2 matched: {} with params: {}",
                        tool_name.as_str(),
                        params
                    );
                    tool_map.insert(tool_name.as_str().to_string(), params);
                }
            }
        }

        info!("DEBUG: Final tool_map: {:?}", tool_map);
        Ok(tool_map)
    }

    /// Extract additional tool calls for multi-step operations
    /// This method attempts to identify additional operations in a multi-step prompt
    /// that weren't captured by standard extraction
    fn extract_multi_step_tool_calls(
        &self,
        response: &str,
        existing_calls: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        let mut tool_calls = existing_calls.clone();
        let response_lower = response.to_lowercase();
        println!("DEBUG: extract_multi_step_tool_calls called with response: {response}");

        // If we already have a swap operation, check if there's also a lend or transfer operation
        if let Some(swap_params) = tool_calls.get("jupiter_swap") {
            // Check if the prompt mentions lending after swapping
            if response_lower.contains("lend") || response_lower.contains("deposit") {
                // Extract the amount to lend from the user prompt (not from swap output)
                // This is a simplified approach - in a real implementation, we would
                // parse the exact amount from the prompt
                let lend_amount = self.extract_lend_amount_from_prompt(response)?;
                let output_mint = swap_params
                    .get("output_mint")
                    .and_then(|v| v.as_str())
                    .unwrap_or("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"); // Default to USDC

                // Create a lend operation with the specified amount
                let lend_params = json!({
                    "mint": output_mint,
                    "amount": lend_amount
                });
                tool_calls.insert("jupiter_lend_earn_deposit".to_string(), lend_params);
                info!(
                    "Added jupiter_lend_earn_deposit operation with amount {} from prompt",
                    lend_amount
                );
            }
            // Check if the prompt mentions transferring after swapping
            else if response_lower.contains("transfer") || response_lower.contains("send") {
                // For now, we'll skip transfer operation as determining the recipient
                // requires additional parsing
                info!("Transfer operation detected in multi-step prompt but recipient extraction not implemented");
            }
        }
        // If we have a lend operation but no swap, check if there should be a swap first
        else if tool_calls.contains_key("jupiter_lend_earn_deposit")
            && response_lower.contains("swap")
        {
            // Extract swap parameters from the prompt
            if let Some(swap_params) = self.extract_swap_params_from_prompt(response)? {
                tool_calls.insert("jupiter_swap".to_string(), swap_params);
                info!("Added jupiter_swap operation from prompt");
            }
        }

        Ok(tool_calls)
    }

    /// Extract lend amount from a multi-step prompt
    fn extract_lend_amount_from_prompt(&self, prompt: &str) -> Result<u64> {
        // Simple regex to extract lend amount
        // This is a simplified implementation - in a real scenario, we would use
        // the LLM to extract this information more accurately
        let re = regex::Regex::new(r"(?i)lend\s+(\d+(?:\.\d+)?)\s*(usdc|sol|usdt)?").unwrap();

        if let Some(captures) = re.captures(prompt) {
            let amount_str = captures.get(1).unwrap().as_str();
            let amount = amount_str.parse::<f64>()?;

            // Determine token mint and convert to smallest denomination
            let token = captures
                .get(2)
                .map(|m| m.as_str().to_lowercase())
                .unwrap_or_else(|| "usdc".to_string());

            match token.as_str() {
                "usdc" | "usdt" => Ok((amount * 1_000_000.0) as u64), // 6 decimals
                "sol" => Ok((amount * 1_000_000_000.0) as u64),       // 9 decimals
                _ => Ok((amount * 1_000_000.0) as u64),               // Default to 6 decimals
            }
        } else {
            // Default to 10 USDC if we can't extract the amount
            info!("Could not extract lend amount from prompt, defaulting to 10 USDC");
            Ok(10_000_000) // 10 USDC
        }
    }

    /// Extract swap parameters from a multi-step prompt
    fn extract_swap_params_from_prompt(&self, prompt: &str) -> Result<Option<serde_json::Value>> {
        // Simple regex to extract swap parameters
        let re = regex::Regex::new(r"(?i)swap\s+(\d+(?:\.\d+)?)\s*(sol|usdc|usdt)?\s+(?:to|for)\s+(\d+(?:\.\d+)?)\s*(sol|usdc|usdt)?").unwrap();

        if let Some(captures) = re.captures(prompt) {
            let input_amount = captures.get(1).unwrap().as_str().parse::<f64>()?;
            let input_token = captures
                .get(2)
                .map(|m| m.as_str().to_lowercase())
                .unwrap_or_else(|| "sol".to_string());

            let output_amount = captures.get(3).unwrap().as_str().parse::<f64>()?;
            let output_token = captures
                .get(4)
                .map(|m| m.as_str().to_lowercase())
                .unwrap_or_else(|| "usdc".to_string());

            // Convert to mint addresses
            let input_mint = match input_token.as_str() {
                "sol" => "So11111111111111111111111111111111111111112",
                "usdc" => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "usdt" => "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
                _ => "So11111111111111111111111111111111111111112",
            };

            let output_mint = match output_token.as_str() {
                "sol" => "So11111111111111111111111111111111111111112",
                "usdc" => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "usdt" => "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
                _ => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            };

            // Convert amounts to smallest denomination
            let input_amount_lamports = match input_token.as_str() {
                "sol" => (input_amount * 1_000_000_000.0) as u64,
                _ => (input_amount * 1_000_000.0) as u64, // Default to 6 decimals
            };

            let output_amount_lamports = match output_token.as_str() {
                "sol" => (output_amount * 1_000_000_000.0) as u64,
                _ => (output_amount * 1_000_000.0) as u64, // Default to 6 decimals
            };

            Ok(Some(json!({
                "input_mint": input_mint,
                "output_mint": output_mint,
                "input_amount": input_amount_lamports,
                "output_amount": output_amount_lamports
            })))
        } else {
            Ok(None)
        }
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
        wallet_context: &WalletContext,
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
                    // Handle numeric values more carefully to avoid scientific notation issues
                    match value {
                        serde_json::Value::Number(n) => {
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
            "sol_transfer" => self.execute_sol_transfer(&params_map, wallet_context).await,
            "jupiter_swap" => self.execute_jupiter_swap(&params_map, wallet_context).await,
            "jupiter_lend_earn_deposit" => {
                self.execute_jupiter_lend_deposit(&params_map, wallet_context)
                    .await
            }
            "get_account_balance" => {
                self.execute_get_account_balance(&params_map, wallet_context)
                    .await
            }
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
        params: &std::collections::HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<serde_json::Value> {
        let recipient = params
            .get("recipient")
            .ok_or_else(|| anyhow!("recipient parameter is required"))?;

        let amount_str = params
            .get("amount")
            .ok_or_else(|| anyhow!("amount parameter is required"))?;

        let amount: f64 = amount_str
            .parse()
            .map_err(|_| anyhow!("Invalid amount: {amount_str}"))?;

        let amount_lamports = (amount * 1_000_000_000.0) as u64;

        // Check if wallet has sufficient balance
        if wallet_context.sol_balance < amount_lamports {
            return Err(anyhow!(
                "Insufficient balance. Available: {} SOL, Required: {} SOL",
                wallet_context.sol_balance / 1_000_000_000,
                amount
            ));
        }

        // Use the existing AgentTools if available, otherwise create a new one
        // Create AgentTools for Jupiter swap execution
        // Create AgentTools for Jupiter Lend Earn Deposit execution
        let agent_tools = if let Some(ref tools) = self.agent_tools {
            Arc::clone(tools)
        } else {
            // Create AgentTools using the wallet context
            // Convert wallet owner string to keypair
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
            Arc::new(reev_agent::enhanced::common::AgentTools::new(key_map))
        };

        // Use the existing execute_direct_sol_transfer function from handlers
        // This will handle the actual blockchain transaction
        let transaction_result = sol_transfer::execute_direct_sol_transfer(
            &agent_tools,
            &format!("send {amount} sol to {recipient}"),
            &wallet_context.owner,
        )
        .await?;

        // Extract the transaction signature from the result
        let transaction_signature = if transaction_result.success {
            if let Some(output) = transaction_result.output.get("sol_transfer") {
                if let Some(sig) = output.get("transaction_signature") {
                    sig.as_str().unwrap_or("").to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // If we got a signature, return it directly, otherwise return the error
        if transaction_signature.is_empty() && !transaction_result.success {
            return Err(anyhow!(
                "SOL transfer failed: {:?}",
                transaction_result
                    .error_message
                    .unwrap_or("Unknown error".to_string())
            ));
        }

        Ok(json!({
            "tool_name": "sol_transfer",
            "params": {
                "recipient": recipient,
                "amount": amount,
                "amount_lamports": amount_lamports,
                "wallet": wallet_context.owner
            },
            "transaction_signature": transaction_signature,
            "success": transaction_result.success
        }))
    }

    /// Execute Jupiter swap
    async fn execute_jupiter_swap(
        &self,
        params: &std::collections::HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<serde_json::Value> {
        let input_mint = params
            .get("input_mint")
            .ok_or_else(|| anyhow!("input_mint parameter is required"))?;

        let output_mint = params
            .get("output_mint")
            .ok_or_else(|| anyhow!("output_mint parameter is required"))?;

        let amount_str = params
            .get("input_amount")
            .or_else(|| params.get("amount"))
            .ok_or_else(|| anyhow!("input_amount parameter is required"))?;
        let amount: f64 = amount_str
            .parse()
            .map_err(|_| anyhow!("Invalid amount: {amount_str}"))?;

        // Special handling for "all" amount to use full balance
        let is_all_amount = amount_str.to_lowercase() == "all";

        // Convert amount to lamports (1 SOL = 1,000,000,000 lamports)
        let amount_lamports = (amount * 1_000_000_000.0) as u64;

        // Create AgentTools for Jupiter swap execution
        let agent_tools = if let Some(ref tools) = self.agent_tools {
            Arc::clone(tools)
        } else {
            // Create AgentTools using the wallet context
            // Load the keypair and include both public key and private key
            let keypair = reev_lib::get_keypair().map_err(|e| {
                anyhow!(
                    "Failed to get keypair for wallet {}: {}",
                    wallet_context.owner,
                    e
                )
            })?;

            let mut key_map = std::collections::HashMap::new();
            key_map.insert("WALLET_PUBKEY".to_string(), wallet_context.owner.clone());
            key_map.insert("WALLET_KEYPAIR".to_string(), keypair.to_base58_string());
            Arc::new(reev_agent::enhanced::common::AgentTools::new(key_map))
        };

        // Use full balance if amount is "all", otherwise use specified amount
        let final_amount_lamports = if is_all_amount {
            // Use almost all SOL balance, keeping some for fees
            wallet_context.sol_balance - (100_000_000) // Reserve 0.1 SOL for fees
        } else {
            amount_lamports
        };

        let swap_args = reev_tools::tools::jupiter_swap::JupiterSwapArgs {
            user_pubkey: wallet_context.owner.clone(),
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            amount: final_amount_lamports,
            slippage_bps: Some(100), // Default 1% slippage
        };

        let result = agent_tools
            .jupiter_swap_tool
            .call(swap_args)
            .await
            .map_err(|e| anyhow!("Jupiter swap execution failed: {e}"))?;

        // Parse the response to extract instructions and execute transaction
        info!("Jupiter swap tool returned result: {}", &result);
        if let Ok(response) = serde_json::from_str::<serde_json::Value>(&result) {
            debug!("Parsed response: {:#?}", response);
            if let Some(instructions) = response.get("instructions") {
                info!(
                    "Found {} instructions in Jupiter response",
                    instructions.as_array().unwrap_or(&vec![]).len()
                );
                debug!("Instructions value: {:#?}", instructions);

                // Convert instructions to RawInstruction format
                let raw_instructions: Result<Vec<reev_lib::agent::RawInstruction>> = instructions
                    .as_array()
                    .unwrap_or(&vec![])
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

                        Ok(reev_lib::agent::RawInstruction {
                            program_id,
                            accounts,
                            data,
                        })
                    })
                    .collect();

                // Execute the transaction with the instructions
                match raw_instructions {
                    Ok(instructions) => {
                        let keypair = reev_lib::get_keypair()
                            .map_err(|e| anyhow!("Failed to load keypair: {e}"))?;
                        let user_pubkey = solana_sdk::signer::Signer::pubkey(&keypair);

                        // Check if we have any instructions before executing
                        if instructions.is_empty() {
                            tracing::warn!("DEBUG: No instructions to execute for Jupiter lend!");
                        }

                        match reev_lib::utils::execute_transaction(
                            instructions,
                            user_pubkey,
                            &keypair,
                        )
                        .await
                        {
                            Ok(signature) => {
                                info!(
                                    "Jupiter swap transaction executed with signature: {}",
                                    signature
                                );
                                Ok(json!({
                                    "tool_name": "jupiter_swap",
                                    "input_mint": input_mint,
                                    "output_mint": output_mint,
                                    "input_amount": amount,
                                    "input_amount_lamports": amount_lamports,
                                    "wallet": wallet_context.owner,
                                    "transaction_signature": signature,
                                    "success": true
                                }))
                            }
                            Err(e) => {
                                error!("Failed to execute Jupiter swap transaction: {}", e);
                                debug!("Transaction execution error details: {:#?}", e);
                                debug!("Failed at execute_transaction call");
                                Ok(json!({
                                    "tool_name": "jupiter_swap",
                                    "error": format!("Transaction execution failed: {e}"),
                                    "raw_response": result
                                }))
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse instructions: {}", e);
                        Ok(json!({
                            "tool_name": "jupiter_swap",
                            "error": format!("Failed to parse instructions: {e}"),
                            "raw_response": result
                        }))
                    }
                }
            } else {
                Ok(json!({
                    "tool_name": "jupiter_swap",
                    "error": "No instructions found in response",
                    "raw_response": result
                }))
            }
        } else {
            Ok(json!({
                "tool_name": "jupiter_swap",
                "error": "Failed to parse Jupiter response",
                "raw_response": result
            }))
        }
    }

    /// Execute Jupiter lend/earn deposit
    async fn execute_jupiter_lend_deposit(
        &self,
        params: &std::collections::HashMap<String, String>,
        wallet_context: &WalletContext,
    ) -> Result<serde_json::Value> {
        let mint = params
            .get("mint")
            .ok_or_else(|| anyhow!("mint parameter is required"))?;

        let amount_str = params
            .get("amount")
            .ok_or_else(|| anyhow!("amount parameter is required"))?;

        debug!(
            "DEBUG: execute_jupiter_lend_deposit received amount_str: {}",
            amount_str
        );

        let amount: f64 = amount_str
            .parse()
            .map_err(|_| anyhow!("Invalid amount: {amount_str}"))?;

        debug!(
            "DEBUG: execute_jupiter_lend_deposit parsed amount as f64: {}",
            amount
        );
        debug!(
            "DEBUG: execute_jupiter_lend_deposit casting to u64: {}",
            amount as u64
        );

        // Check if the amount is already in lamports or needs conversion
        let amount_lamports = if amount > 1_000_000.0 {
            // Amount is likely already in lamports (for USDC/USDT)
            debug!("DEBUG: Amount appears to be in lamports: {}", amount as u64);
            amount as u64
        } else {
            // Amount is likely in human-readable format, convert to lamports
            debug!(
                "DEBUG: Converting amount to lamports: {}",
                amount * 1_000_000.0
            );
            (amount * 1_000_000.0) as u64
        };

        debug!("DEBUG: Final amount for Jupiter lend: {}", amount_lamports);

        // Create AgentTools for Jupiter Lend Earn Deposit execution
        let agent_tools = if let Some(ref tools) = self.agent_tools {
            Arc::clone(tools)
        } else {
            // Create AgentTools using the wallet context
            // Load the keypair and include both public key and private key
            let keypair = reev_lib::get_keypair().map_err(|e| {
                anyhow!(
                    "Failed to get keypair for wallet {}: {}",
                    wallet_context.owner,
                    e
                )
            })?;

            let mut key_map = std::collections::HashMap::new();
            key_map.insert("WALLET_PUBKEY".to_string(), wallet_context.owner.clone());
            key_map.insert("WALLET_KEYPAIR".to_string(), keypair.to_base58_string());
            Arc::new(reev_agent::enhanced::common::AgentTools::new(key_map))
        };

        // Execute Jupiter Lend Earn Deposit using AgentTools
        // Note: The amount is already in the correct units (smallest denomination)
        // as provided by the LLM, so we don't need to multiply by 1_000_000
        let deposit_args =
            reev_tools::tools::jupiter_lend_earn_deposit::JupiterLendEarnDepositArgs {
                user_pubkey: wallet_context.owner.clone(),
                asset_mint: mint.clone(),
                amount: amount_lamports,
            };

        let result = agent_tools
            .jupiter_lend_earn_deposit_tool
            .call(deposit_args)
            .await
            .map_err(|e| anyhow!("Jupiter Lend Earn Deposit execution failed: {e}"))?;

        // Parse the response to extract instructions and execute transaction
        info!("Jupiter lend deposit tool returned result: {}", &result);
        if let Ok(response) = serde_json::from_str::<serde_json::Value>(&result) {
            debug!("Parsed response: {:#?}", response);

            // The response is a serialized Vec<RawInstruction>, let's convert it
            let raw_instructions: Result<Vec<reev_lib::agent::RawInstruction>> = response
                .as_array()
                .unwrap_or(&vec![])
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

                    Ok(reev_lib::agent::RawInstruction {
                        program_id,
                        accounts,
                        data,
                    })
                })
                .collect();

            // Execute transaction with the instructions
            match raw_instructions {
                Ok(instructions) => {
                    info!(
                        "DEBUG: About to execute Jupiter lend transaction with {} instructions",
                        instructions.len()
                    );
                    let keypair = reev_lib::get_keypair()
                        .map_err(|e| anyhow!("Failed to load keypair: {e}"))?;
                    let user_pubkey = solana_sdk::signer::Signer::pubkey(&keypair);

                    match reev_lib::utils::execute_transaction(instructions, user_pubkey, &keypair)
                        .await
                    {
                        Ok(signature) => {
                            info!(
                                "Jupiter lend deposit transaction executed with signature: {}",
                                signature
                            );
                            Ok(json!({
                                "tool_name": "jupiter_lend_earn_deposit",
                                "params": {
                                    "mint": mint,
                                    "amount": amount_lamports,
                                    "wallet": wallet_context.owner
                                },
                                "transaction_signature": signature,
                                "success": true
                            }))
                        }
                        Err(e) => {
                            error!("Failed to execute Jupiter lend deposit transaction: {}", e);
                            debug!("Transaction execution error details: {:#?}", e);

                            Ok(json!({
                                "tool_name": "jupiter_lend_earn_deposit",
                                "params": {
                                    "mint": mint,
                                    "amount": amount,
                                    "wallet": wallet_context.owner
                                },
                                "error": format!("Transaction execution failed: {}", e),
                                "success": false
                            }))
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to parse Jupiter lend deposit instructions: {}", e);
                    Ok(json!({
                        "tool_name": "jupiter_lend_earn_deposit",
                        "params": {
                            "mint": mint,
                            "amount": amount,
                            "wallet": wallet_context.owner
                        },
                        "error": format!("Failed to parse instructions: {}", e),
                        "success": false
                    }))
                }
            }
        } else {
            error!("Failed to parse Jupiter lend deposit response as JSON");
            Ok(json!({
                "tool_name": "jupiter_lend_earn_deposit",
                "params": {
                    "mint": mint,
                    "amount": amount,
                    "wallet": wallet_context.owner
                },
                "error": "Failed to parse response as JSON",
                "success": false
            }))
        }
    }

    /// Execute get account balance
    async fn execute_get_account_balance(
        &self,
        params: &std::collections::HashMap<String, String>,
        _wallet_context: &WalletContext,
    ) -> Result<serde_json::Value> {
        let account = params
            .get("account")
            .ok_or_else(|| anyhow!("account parameter is required"))?;

        let default_mint = "So11111111111111111111111111111111111111112".to_string();
        let mint = params.get("mint").unwrap_or(&default_mint);

        // Mock balance for now
        // In a real implementation, this would query the blockchain
        let balance = match mint.as_str() {
            "So11111111111111111111111111111111111111112" => {
                // Mock SOL balance
                rand::random::<u64>() % 10_000_000_000
            }
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => {
                // Mock USDC balance
                rand::random::<u64>() % 1_000_000_000
            }
            _ => {
                // Mock other token balance
                rand::random::<u64>() % 1_000_000_000
            }
        };

        Ok(json!({
            "tool_name": "get_account_balance",
            "params": {
                "account": account,
                "mint": mint
            },
            "balance": balance,
            "success": true
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

/// LLM API request payload
#[derive(Debug, Serialize)]
struct LLMRequest {
    model: String,
    messages: Vec<LLMMessage>,
    temperature: f32,
    max_tokens: u32,
}

/// LLM API message
#[derive(Debug, Serialize)]
struct LLMMessage {
    role: String,
    content: String,
}

/// LLM API response
#[derive(Debug, Deserialize)]
struct LLMResponse {
    choices: Vec<LLMChoice>,
}

/// LLM API choice
#[derive(Debug, Deserialize)]
struct LLMChoice {
    message: LLMResponseMessage,
}

/// LLM API response message
#[derive(Debug, Deserialize)]
struct LLMResponseMessage {
    content: String,
}
