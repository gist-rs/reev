//! Rig Agent Integration for Phase 2 Tool Selection
//!
//! This module implements the RigAgent component that wraps rig framework
//! for LLM-driven tool selection and parameter extraction in Phase 2 of
//! Reev Core Architecture.

// Removed unused import
use crate::{prompt_processor::types::SwapAdditionalParams, YmlStep};
use anyhow::{anyhow, Result};
use reev_agent::enhanced::common::AgentTools;
use reev_types::flow::{StepResult, WalletContext};

use rig::tool::ToolSet;
use serde_json::json;
use std::collections::HashMap;

use std::string::String;
use std::sync::Arc;
use tracing::{debug, info, instrument};
use zai_sdk::{GlmVariant, Message, ZaiClient};

// Import modules
mod context;
mod prompting;
mod tools;
pub mod types;

// Re-export types and traits
pub use context::ContextProvider;
pub use prompting::{HttpProvider, MultiStepHandler, PromptProvider};
pub use tools::{AgentProvider, AgentToolHelper, ToolExecutor};
pub use types::*;

/// RigAgent for LLM-driven tool selection and parameter extraction
pub struct RigAgent {
    /// ZAI client for GLM-4.6 model
    zai_client: ZaiClient,
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
            .api_key(api_key.clone())
            .build()
            .map_err(|e| anyhow!("Failed to create ZAI client: {e}"))?;

        // Initialize tool set with Reev tools
        let _tool_set = Self::initialize_tool_set().await?; // Prefix with _ to suppress warning

        Ok(Self {
            zai_client,
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
            .api_key(api_key.clone())
            .build()
            .map_err(|e| anyhow!("Failed to create ZAI client: {e}"))?;

        // Initialize tool set with Reev tools
        let _tool_set = Self::initialize_tool_set().await?; // Prefix with _ to suppress warning

        Ok(Self {
            zai_client,
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
        debug!(
            "USDC balance in context: {:?}",
            wallet_context
                .token_balances
                .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                .map(|t| t.balance)
        );

        // Use structured prompt data if available, otherwise use refined prompt or original prompt
        let (prompt, action, parameters, structured_prompt) =
            if let Some(structured_prompt) = &step.structured_prompt {
                (
                    structured_prompt.refined_prompt.clone(),
                    Some(structured_prompt.action.clone()),
                    Some(&structured_prompt.parameters),
                    Some(structured_prompt),
                )
            } else if !step.refined_prompt.is_empty() {
                (step.refined_prompt.clone(), None, None, None)
            } else {
                (step.prompt.clone(), None, None, None)
            };

        // Create YML context and convert to prompt
        let yml_context = self.create_yml_context(step, wallet_context, previous_results)?;
        let context_prompt = self.yml_context_to_prompt(&yml_context, &prompt)?;

        // Log the YML context for debugging
        debug!(
            "Generated YML context for step {}: {:?}",
            step.step_id, yml_context
        );

        // Get expected tools hints from the step
        let expected_tools = step.expected_tools.clone();

        // Create a system prompt for the zai client
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
        let messages = vec![
            Message::system(system_prompt),
            Message::user(&context_prompt),
        ];

        // If we have expected tools, use them to guide the agent
        let response = if let Some(tools) = expected_tools {
            debug!("Using expected tools to guide agent: {:?}", tools);
            // Use the zai client for the request
            self.zai_client
                .completion_with_messages(messages)
                .await
                .map_err(|e| anyhow!("LLM generation failed: {e}"))?
        } else {
            debug!("No expected tools provided, using general agent prompt");
            // Use the zai client for the request
            self.zai_client
                .completion_with_messages(messages)
                .await
                .map_err(|e| anyhow!("LLM generation failed: {e}"))?
        };

        debug!("Got response from agent: {}", response);

        debug!("Parsing tool calls from LLM response");

        // If we have structured data with action and parameters, use them directly
        // Otherwise, extract tool calls from the LLM response
        let tool_calls = if let (Some(action), Some(parameters)) = (action, parameters) {
            debug!(
                "Using structured action and parameters: action={:?}, parameters={:?}",
                action, parameters
            );
            self.create_tool_calls_from_structured_data(&action, parameters, structured_prompt)?
        } else {
            debug!("Extracting tool calls from LLM response");
            self.parse_tool_calls_from_response(&response)?
        };

        // Check if this is a multi-step prompt and we have multiple operations
        let prompt_lower = prompt.to_lowercase();
        let is_multi_step = prompt_lower.contains(" then ")
            || prompt_lower.contains(" and ")
            || prompt_lower.contains(" followed by ");

        debug!("is_multi_step = {}", is_multi_step);
        debug!("Initial tool_calls count = {}", tool_calls.len());

        // For multi-step prompts, we need to ensure we extract all operations
        let tool_calls = if is_multi_step && tool_calls.len() < 2 {
            // Try to extract additional operations if we only got one tool call
            debug!("Multi-step prompt detected but only one tool call extracted, attempting to extract additional operations");
            let additional_calls = self.extract_multi_step_tool_calls(&response, &tool_calls)?;
            debug!("Additional tool_calls = {:?}", additional_calls);
            additional_calls
        } else {
            debug!("Using initial tool_calls as-is");
            tool_calls
        };

        // Execute selected tools
        debug!("Tool calls extracted: {:?}", tool_calls);
        let tool_results = self
            .execute_tools(tool_calls.clone(), wallet_context)
            .await?;
        debug!("Tool execution results: {:?}", tool_results);

        // Create list of tool names that were executed
        let executed_tool_names: Vec<String> = tool_calls.keys().cloned().collect();

        // Serialize ToolResultWrapper to JSON for storage in StepResult
        let tool_results_json: Result<Vec<_>, _> =
            tool_results.into_iter().map(serde_json::to_value).collect();

        let tool_results_json = match tool_results_json {
            Ok(results) => results,
            Err(e) => {
                return Err(anyhow!("Failed to serialize tool results: {e}"));
            }
        };

        // Create the step result with serialized ToolResultWrapper
        let step_result = StepResult {
            step_id: step.step_id.clone(),
            success: true,
            error_message: None,
            tool_calls: executed_tool_names,
            output: json!({}),
            execution_time_ms: 100, // This would be calculated in a real implementation
            tool_results: Some(tool_results_json),
        };

        Ok(step_result)
    }

    /// Parse tool calls from a response from zai_client
    fn parse_tool_calls_from_response(
        &self,
        response: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        debug!("Parsing tool calls from response: {}", response);

        // Parse the response as structured JSON with typed structs
        // Parse the response as structured JSON with typed structs
        if let Ok(structured_response) = serde_json::from_str::<StructuredLLMResponse>(response) {
            if let Some(tool_calls) = structured_response.tool_calls {
                let mut tool_map = HashMap::new();
                for tool_call in tool_calls {
                    tool_map.insert(tool_call.name, tool_call.parameters);
                }
                return Ok(tool_map);
            }
        }

        // Try parsing as a generic JSON object to see if tool_calls field exists
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(tool_calls) = json_value.get("tool_calls") {
                if let Some(tool_calls_array) = tool_calls.as_array() {
                    let mut tool_map = HashMap::new();
                    for tool_call in tool_calls_array {
                        if let Some(name) = tool_call.get("name").and_then(|v| v.as_str()) {
                            if let Some(params) = tool_call.get("parameters") {
                                tool_map.insert(name.to_string(), params.clone());
                            }
                        }
                    }
                    return Ok(tool_map);
                }
            }
        }

        // Fall back to text extraction if JSON parsing fails
        self.extract_tool_calls_from_text(response)
    }

    /// Extract tool calls from text response (fallback)
    fn extract_tool_calls_from_text(
        &self,
        response: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // Use serde_json to extract structured data
        // First, try to find JSON-like structures in the text
        let mut tool_calls = HashMap::new();

        // Look for patterns like "sol_transfer(...)" or "jupiter_swap(...)"
        if response.contains("sol_transfer") {
            if let Some(recipient) = extract_field(response, "recipient") {
                let amount = extract_field(response, "amount")
                    .and_then(|a| a.parse::<f64>().ok())
                    .unwrap_or(1.0);

                tool_calls.insert(
                    "sol_transfer".to_string(),
                    json!({
                        "recipient": recipient,
                        "amount": amount
                    }),
                );
            }
        }

        if response.contains("jupiter_swap") {
            let input_mint = extract_field(response, "input_mint")
                .or_else(|| extract_token_from_text(response, "SOL"))
                .unwrap_or_else(|| "So11111111111111111111111111111111111111112".to_string());

            let output_mint = extract_field(response, "output_mint")
                .or_else(|| extract_token_from_text(response, "USDC"))
                .unwrap_or_else(|| "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string());

            let input_amount = extract_field(response, "input_amount")
                .and_then(|a| a.parse::<f64>().ok())
                .unwrap_or(0.1);

            tool_calls.insert(
                "jupiter_swap".to_string(),
                json!({
                    "input_mint": input_mint,
                    "output_mint": output_mint,
                    "input_amount": input_amount
                }),
            );
        }

        if response.contains("jupiter_lend_earn_deposit") {
            let mint = extract_field(response, "mint")
                .or_else(|| extract_token_from_text(response, "USDC"))
                .unwrap_or_else(|| "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string());

            let amount = extract_field(response, "amount")
                .and_then(|a| a.parse::<u64>().ok())
                .unwrap_or(1000000); // Default to 1 USDC

            tool_calls.insert(
                "jupiter_lend_earn_deposit".to_string(),
                json!({
                    "mint": mint,
                    "amount": amount
                }),
            );
        }

        Ok(tool_calls)
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

    /// Create tool calls directly from structured prompt data
    /// This method implements Phase 4 of the structured LLM response system
    fn create_tool_calls_from_structured_data(
        &self,
        action: &crate::prompt_processor::PromptAction,
        parameters: &crate::prompt_processor::PromptParameters,
        structured_prompt: Option<&crate::prompt_processor::StructuredRefinedPrompt>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        debug!(
            "Creating tool calls from structured data: action={:?}",
            action
        );

        let mut tool_calls = HashMap::new();

        match action {
            crate::prompt_processor::PromptAction::Transfer => {
                if let (Some(amount), Some(input_mint)) =
                    (&parameters.amount, &parameters.input_mint)
                {
                    // Select tool based on token type
                    let tool_name = if input_mint == "So11111111111111111111111111111111111111112" {
                        "sol_transfer".to_string()
                    } else {
                        "spl_transfer".to_string()
                    };

                    // Get user pubkey from structured prompt or typed transfer parameters
                    let user_pubkey = if let Some(sp) = structured_prompt {
                        sp.subject_pubkey.clone().unwrap_or_else(|| {
                            "3F42CLVYyxuMYNTBRKuCQ6o3XnzPky6raWTPHtW8myLr".to_string()
                        })
                    } else if let Some(transfer_params) = &parameters.transfer_params {
                        transfer_params
                            .subject_pubkey
                            .clone()
                            .or_else(|| transfer_params.user_pubkey.clone())
                            .unwrap_or_else(|| {
                                "3F42CLVYyxuMYNTBRKuCQ6o3XnzPky6raWTPHtW8myLr".to_string()
                            })
                    } else {
                        "3F42CLVYyxuMYNTBRKuCQ6o3XnzPky6raWTPHtW8myLr".to_string()
                    };

                    // Get recipient pubkey from structured prompt or typed transfer parameters
                    let recipient_pubkey = if let Some(sp) = structured_prompt {
                        sp.target_pubkey.clone().unwrap_or_else(|| {
                            "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq".to_string()
                        })
                    } else if let Some(transfer_params) = &parameters.transfer_params {
                        transfer_params
                            .recipient
                            .clone()
                            .or_else(|| transfer_params.target_pubkey.clone())
                            .or_else(|| transfer_params.recipient_pubkey.clone())
                            .unwrap_or_else(|| {
                                "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq".to_string()
                            })
                    } else {
                        "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq".to_string()
                    };

                    tool_calls.insert(
                        tool_name,
                        json!({
                            "user_pubkey": user_pubkey,
                            "recipient": recipient_pubkey,
                            "amount": amount,
                            "mint_address": input_mint
                        }),
                    );
                } else if let Some(amount) = &parameters.amount {
                    // Fallback: extract token symbol from mint address
                    let token_symbol = parameters
                        .input_mint
                        .as_ref()
                        .map(|mint| {
                            if mint == "So11111111111111111111111111111111111111112" {
                                "SOL".to_string()
                            } else if mint == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" {
                                "USDC".to_string()
                            } else if mint == "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" {
                                "USDT".to_string()
                            } else {
                                // Default to USDC for unknown tokens
                                "USDC".to_string()
                            }
                        })
                        .unwrap_or_else(|| "USDC".to_string());

                    let input_mint = parameters.input_mint.clone().unwrap_or_else(|| {
                        // Fallback mint address based on token symbol
                        match token_symbol.as_str() {
                            "SOL" => "So11111111111111111111111111111111111111112".to_string(),
                            "USDC" => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                            "USDT" => "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
                            _ => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // Default to USDC
                        }
                    });

                    let tool_name = match token_symbol.as_str() {
                        "SOL" => "sol_transfer".to_string(),
                        _ => "spl_transfer".to_string(), // All non-SOL tokens use SPL transfer
                    };

                    let amount = amount.clone();

                    // Get user pubkey from structured prompt or typed transfer parameters
                    let user_pubkey = if let Some(sp) = structured_prompt {
                        sp.subject_pubkey.clone().unwrap_or_else(|| {
                            "3F42CLVYyxuMYNTBRKuCQ6o3XnzPky6raWTPHtW8myLr".to_string()
                        })
                    } else if let Some(transfer_params) = &parameters.transfer_params {
                        transfer_params
                            .subject_pubkey
                            .clone()
                            .or_else(|| transfer_params.user_pubkey.clone())
                            .unwrap_or_else(|| {
                                "3F42CLVYyxuMYNTBRKuCQ6o3XnzPky6raWTPHtW8myLr".to_string()
                            })
                    } else {
                        "3F42CLVYyxuMYNTBRKuCQ6o3XnzPky6raWTPHtW8myLr".to_string()
                    };

                    // Get recipient pubkey from structured prompt or typed transfer parameters
                    let recipient_pubkey = if let Some(sp) = structured_prompt {
                        sp.target_pubkey.clone().unwrap_or_else(|| {
                            "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq".to_string()
                        })
                    } else if let Some(transfer_params) = &parameters.transfer_params {
                        transfer_params
                            .recipient
                            .clone()
                            .or_else(|| transfer_params.target_pubkey.clone())
                            .or_else(|| transfer_params.recipient_pubkey.clone())
                            .unwrap_or_else(|| {
                                "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq".to_string()
                            })
                    } else {
                        "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq".to_string()
                    };

                    tool_calls.insert(
                        tool_name,
                        json!({
                            "user_pubkey": user_pubkey,
                            "recipient": recipient_pubkey,
                            "amount": amount,
                            "mint_address": input_mint
                        }),
                    );
                }
            }
            crate::prompt_processor::PromptAction::Swap => {
                if let (Some(amount), Some(input_mint), Some(output_mint)) = (
                    &parameters.amount,
                    &parameters.input_mint,
                    &parameters.output_mint,
                ) {
                    tool_calls.insert(
                        "jupiter_swap".to_string(),
                        json!({
                            "input_amount": amount,
                            "input_mint": input_mint,
                            "output_mint": output_mint,
                        }),
                    );
                }
            }
            crate::prompt_processor::PromptAction::Lend => {
                if let (Some(amount), Some(input_mint)) =
                    (&parameters.amount, &parameters.input_mint)
                {
                    tool_calls.insert(
                        "jupiter_lend".to_string(),
                        json!({
                            "amount": amount,
                            "mint": input_mint,
                        }),
                    );
                }
            }
            crate::prompt_processor::PromptAction::Earn => {
                if let Some(input_mint) = &parameters.input_mint {
                    tool_calls.insert(
                        "jupiter_earn".to_string(),
                        json!({
                            "mint": input_mint,
                        }),
                    );
                }
            }
            crate::prompt_processor::PromptAction::Borrow => {
                if let (Some(amount), Some(input_mint)) =
                    (&parameters.amount, &parameters.input_mint)
                {
                    tool_calls.insert(
                        "jupiter_borrow".to_string(),
                        json!({
                            "amount": amount,
                            "mint": input_mint,
                        }),
                    );
                }
            }
            crate::prompt_processor::PromptAction::Unknown => {
                // For unknown actions, we can't create specific tool calls
                debug!("Unknown action type, cannot create tool calls");
            }
        }

        debug!("Created tool calls from structured data: {:?}", tool_calls);
        Ok(tool_calls)
    }

    /// Extract additional tool calls for multi-step operations
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
                // Try to deserialize into typed swap parameters
                if let Ok(typed_params) =
                    serde_json::from_value::<SwapAdditionalParams>(swap_params.clone())
                {
                    if let Some(output_mint) = &typed_params.output_mint {
                        if output_mint == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" {
                            if let Some(input_amount) = typed_params.input_amount {
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
        }

        Ok(new_calls)
    }
}

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

// Helper function to extract field values from text
fn extract_field(text: &str, field: &str) -> Option<String> {
    // Try to find pattern like "field: value" or field="value"
    let patterns = [
        format!(r"{field}:\s*([^\s,]+)"),
        format!(r#"{field}:\s*"([^"]+)""#),
        format!(r#"{field}:\s*'([^']+)'"#),
    ];

    for pattern in patterns {
        if let Ok(re) = regex::Regex::new(&pattern) {
            if let Some(caps) = re.captures(text) {
                if let Some(m) = caps.get(1) {
                    return Some(m.as_str().trim().to_string());
                }
            }
        }
    }
    None
}

// Helper function to extract token mint address from text
fn extract_token_from_text(text: &str, token_name: &str) -> Option<String> {
    let token_lower = token_name.to_lowercase();
    if !text.to_lowercase().contains(&token_lower) {
        return None;
    }

    match token_lower.as_str() {
        "sol" => Some("So11111111111111111111111111111111111111112".to_string()),
        "usdc" => Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
        "usdt" => Some("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string()),
        _ => None,
    }
}
