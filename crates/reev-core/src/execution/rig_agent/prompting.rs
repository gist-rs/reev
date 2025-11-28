//! LLM Prompting and Response Parsing for RigAgent
//!
//! This module contains methods for prompting the LLM and parsing responses.

use anyhow::{anyhow, Result};
use reev_types::tools::ToolName;
use serde_json::json;
use std::collections::HashMap;
use tracing::{debug, info};

use super::types::{LLMMessage, LLMRequest, LLMResponse};

/// Trait for LLM prompting operations
#[allow(async_fn_in_trait)]
pub trait PromptProvider {
    /// Prompt the agent with expected tools hints
    async fn prompt_with_expected_tools(
        &self,
        prompt: &str,
        expected_tools: &[ToolName],
    ) -> Result<String>;

    /// Prompt the agent and get the response
    async fn prompt_agent(&self, prompt: &str) -> Result<String>;

    /// Extract tool calls from agent response
    fn extract_tool_calls(&self, response: &str) -> Result<HashMap<String, serde_json::Value>>;

    /// Parse tool calls from LLM response
    fn parse_tool_calls_from_response(
        &self,
        response: &str,
    ) -> Result<HashMap<String, serde_json::Value>>;

    /// Extract tool calls from text response
    fn extract_tool_calls_from_text(
        &self,
        response: &str,
    ) -> Result<HashMap<String, serde_json::Value>>;
}

/// Trait for multi-step operation handling
pub trait MultiStepHandler {
    /// Extract additional tool calls for multi-step operations
    fn extract_multi_step_tool_calls(
        &self,
        response: &str,
        existing_calls: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>>;

    /// Extract lend amount from a multi-step prompt
    fn extract_lend_amount_from_prompt(&self, prompt: &str) -> Result<u64>;

    /// Extract swap parameters from a multi-step prompt
    fn extract_swap_params_from_prompt(&self, prompt: &str) -> Result<Option<serde_json::Value>>;
}

/// Implementation for any struct with http_client, model_name, and api_key fields
impl<T> PromptProvider for T
where
    T: HttpProvider,
{
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
        let model_name = if self.model_name() == "glm-4.6-coding" {
            "glm-4.6"
        } else {
            self.model_name()
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
        info!("DEBUG: Full prompt being sent to ZAI API: {}", prompt);

        // Make the API call
        let response_body: LLMResponse = self.make_api_request(&request_payload).await?;

        // Extract the content from the response
        let content = response_body
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow!("No content in LLM response"))?;

        info!("LLM response: {}", content);
        Ok(content)
    }

    /// Extract tool calls from agent response
    fn extract_tool_calls(&self, response: &str) -> Result<HashMap<String, serde_json::Value>> {
        // This is a simplified implementation
        // In a real implementation, we would parse the JSON response to extract tool calls
        info!("DEBUG: Full response from LLM: {}", response);
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
}

/// Default implementation of MultiStepHandler for any struct
impl<T> MultiStepHandler for T {
    /// Extract additional tool calls for multi-step operations
    fn extract_multi_step_tool_calls(
        &self,
        response: &str,
        existing_calls: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>> {
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
}

/// Trait for HTTP operations
#[allow(async_fn_in_trait)]
pub trait HttpProvider {
    fn model_name(&self) -> &str;
    fn api_key(&self) -> &str;
    fn http_client(&self) -> &reqwest::Client;

    async fn make_api_request(&self, request_payload: &LLMRequest) -> Result<LLMResponse> {
        // Make the API call
        let api_base = std::env::var("ZAI_API_BASE")
            .unwrap_or_else(|_| "https://api.z.ai/api/coding/paas/v4".to_string());
        let url = format!("{api_base}/chat/completions");

        let response = self
            .http_client()
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .header("Content-Type", "application/json")
            .json(request_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "API request failed with status: {}",
                response.status()
            ));
        }

        let response_body: LLMResponse = response.json().await?;
        Ok(response_body)
    }
}
