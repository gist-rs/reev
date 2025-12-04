//! Prompt Processor for Phase 1 of V3 Plan
//!
//! This module implements language refinement functionality in Phase 1 of V3 plan.
//! It uses LLM to refine user prompts by fixing typos, normalizing terminology, and making
//! language clearer and more unambiguous.

use std::str::FromStr;

use anyhow::{anyhow, Result};
// Removed unused regex import
use serde::{Deserialize, Serialize};

use solana_sdk::pubkey::Pubkey;
use tracing::{debug, error, info, instrument, warn};

// Import prompts
use crate::prompts;
use prompts::prompt_processor::PROMPT_PROCESSOR_SYSTEM_PROMPT;

// Import modules
pub mod max_amount_calculator;
pub mod types;
pub mod validation;

// Re-export types
pub use max_amount_calculator::{MaxAmountCalculation, MaxAmountCalculator};
pub use types::{
    PromptAction, PromptParameters, StructuredRefineRequest, StructuredRefineResponse,
    StructuredRefinedPrompt, ValidationResult,
};
pub use validation::{
    calculate_confidence_score, validate_structured_response,
    validate_structured_response_with_max_amounts,
};

/// Prompt processor for refining user prompts and handling special cases
pub struct PromptProcessor {
    /// API key for LLM service
    api_key: Option<String>,
    /// Model name for LLM
    model_name: String,
    /// Owner wallet address
    owner_wallet_address: Option<String>,
}

impl Default for PromptProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptProcessor {
    /// Create a new prompt processor
    pub fn new() -> Self {
        let model_name =
            std::env::var("GLM_MODEL").unwrap_or_else(|_| "glm-4.6-coding".to_string());
        let api_key = std::env::var("ZAI_API_KEY").ok();

        Self {
            api_key,
            model_name,
            owner_wallet_address: None,
        }
    }

    /// Create a prompt processor with custom configuration
    pub fn with_config(api_key: Option<String>, model_name: String) -> Self {
        Self {
            api_key,
            model_name,
            owner_wallet_address: None,
        }
    }

    /// Process a user prompt using LLM with special handling for "all" keyword
    #[instrument(skip(self))]
    pub async fn process_prompt(
        &mut self,
        prompt: &str,
        owner_wallet_address: &str,
    ) -> Result<RefinedPrompt> {
        // Try using the new structured response system
        match self
            .process_prompt_structured(prompt, owner_wallet_address)
            .await
        {
            Ok(structured_prompt) => {
                info!("Structured response processing successful");
                info!(
                    "Structured refined prompt: {}",
                    structured_prompt.refined_prompt
                );
                // Convert StructuredRefinedPrompt to RefinedPrompt for fallback
                let refined_prompt = RefinedPrompt {
                    original: structured_prompt.original_prompt.clone(),
                    refined: structured_prompt.refined_prompt.clone(),
                    confidence: structured_prompt.confidence,
                    usable_amount: structured_prompt.usable_amount,
                };
                info!("Created refined prompt: {}", refined_prompt.refined);
                Ok(refined_prompt)
            }
            Err(e) => {
                warn!(
                    "Failed to process with structured response, falling back to legacy: {}",
                    e
                );
                info!("Falling back to legacy processing for prompt: {}", prompt);
                self.process_prompt_legacy(prompt, owner_wallet_address)
                    .await
            }
        }
    }

    /// Legacy method for processing prompts (fallback)
    pub async fn process_prompt_legacy(
        &mut self,
        prompt: &str,
        owner_wallet_address: &str,
    ) -> Result<RefinedPrompt> {
        self.owner_wallet_address = Some(owner_wallet_address.to_string());
        info!(
            "Processing prompt: {}, sender: {:?}",
            prompt, self.owner_wallet_address
        );

        // If no API key is configured, return error as per V3 plan
        if self.api_key.is_none() {
            return Err(anyhow!("No API key configured for prompt processor"));
        }

        info!(
            "Starting structured response processing for prompt: {}",
            prompt
        );

        // Check if this is a request with "all" keyword for any operation type
        let original_prompt = prompt.to_string();
        let is_all_keyword = original_prompt.to_lowercase().contains("all");

        // Build LLM request for language refinement
        let (request, max_amount_decimal) = if is_all_keyword {
            info!("Detected 'all' keyword in prompt, calculating max amount");
            // This is a transfer with "all" keyword, calculate maximum transferable amount
            let owner_wallet_address = match &self.owner_wallet_address {
                Some(owner_wallet_address) => owner_wallet_address,
                None => panic!("Required owner_wallet_address"),
            };

            // Create wallet context to get balance
            let wallet_context = create_wallet_context(owner_wallet_address).await?;

            // Calculate gas reserve based on operation type
            // For Jupiter swaps, we need more reserve due to account creation fees
            let is_swap_operation = original_prompt.to_lowercase().contains("swap");
            let gas_reserve = if is_swap_operation {
                reev_lib::constants::amounts::tokens::sol::JUPITER_SWAP_FEE_RESERVE
            // 0.01 SOL for Jupiter swaps (account creation fees)
            } else {
                reev_lib::constants::amounts::tokens::sol::ONE_MILLI // 0.001 SOL for transfers
            };

            // Calculate maximum transferable amount
            let max_amount = crate::utils::transfer_utils::calculate_max_transferable_amount(
                "", // Empty for SOL
                wallet_context.sol_balance,
                gas_reserve,
            );

            // Convert max_amount to token units for display
            let max_amount_decimal = max_amount as f64 / 1_000_000_000.0;

            info!(
                "Detected 'all' keyword, calculated max transferable amount: {}",
                max_amount_decimal
            );

            info!("Creating structured prompt with max amount");

            // Create structured prompt for LLM using template approach
            let structured_prompt = build_refinement_prompt(&original_prompt, max_amount_decimal);
            info!(
                "Created structured prompt with max amount: {}",
                structured_prompt
            );

            // Build LLM request for language refinement
            let request = LanguageRefineRequest {
                prompt: structured_prompt,
            };

            (request, Some(max_amount_decimal))
        } else {
            // No "all" keyword, use original prompt directly
            let request = LanguageRefineRequest {
                prompt: original_prompt,
            };

            (request, None)
        };

        info!("Sending request to LLM, is_all_keyword: {}", is_all_keyword);
        // Send request to LLM
        let response = self.send_refine_request(&request).await;

        // Just return the response or error directly
        let response = response?;
        info!("Received structured response from LLM: {}", response);
        debug!(
            "Raw LLM response (first 500 chars): {}",
            &response[..response.len().min(500)]
        );
        info!("Received structured response from LLM: {}", response);
        // Parse response - response may already be a JSON string of LanguageRefineResponse
        // or a plain string that needs to be converted
        let response_obj = if response.starts_with('{') {
            // Response is JSON, parse it directly
            match serde_json::from_str::<LanguageRefineResponse>(&response) {
                Ok(r) => r,
                Err(e) => {
                    warn!("Failed to parse LLM JSON response: {}", e);

                    // Try to extract JSON from the response if it contains additional text
                    let cleaned_response = if response.contains('{') && response.contains('}') {
                        // Extract JSON portion if there's extra text
                        let start = response.find('{').unwrap_or(0);
                        let end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
                        response[start..end].to_string()
                    } else {
                        response.clone()
                    };

                    // Try parsing of cleaned response
                    match serde_json::from_str::<LanguageRefineResponse>(&cleaned_response) {
                        Ok(r) => r,
                        Err(e2) => {
                            error!("Failed to parse cleaned LLM JSON response: {}", e2);
                            // Fall back to plain text response
                            let changed = response != request.prompt;
                            LanguageRefineResponse {
                                refined_prompt: response.clone(),
                                confidence: if changed { 0.8 } else { 0.95 },
                            }
                        }
                    }
                }
            }
        } else {
            // Response is plain text, create a LanguageRefineResponse from it
            let changed = response != request.prompt;
            LanguageRefineResponse {
                refined_prompt: response.clone(),
                confidence: if changed { 0.8 } else { 0.95 },
            }
        };

        // Create refined prompt object
        let refined = if is_all_keyword {
            RefinedPrompt {
                original: prompt.to_string(),
                refined: response_obj.refined_prompt.clone(),
                confidence: response_obj.confidence,
                usable_amount: max_amount_decimal,
            }
        } else {
            RefinedPrompt {
                original: prompt.to_string(),
                refined: response_obj.refined_prompt.clone(),
                confidence: response_obj.confidence,
                usable_amount: None,
            }
        };
        info!("Processed prompt: {}", refined.refined);
        debug!("Original: {} -> Refined: {}", prompt, refined.refined);

        // Log the raw response for debugging
        debug!("LLM raw response: {}", response);

        Ok(RefinedPrompt {
            original: prompt.to_string(),
            refined: refined.refined,
            confidence: refined.confidence,
            usable_amount: max_amount_decimal,
        })
    }

    /// Process prompt with structured LLM response
    /// This method implements Phase 1 of the structured LLM response system
    #[instrument(skip(self))]
    pub async fn process_prompt_structured(
        &mut self,
        prompt: &str,
        owner_wallet_address: &str,
    ) -> Result<StructuredRefinedPrompt> {
        self.owner_wallet_address = Some(owner_wallet_address.to_string());
        info!(
            "Processing prompt with structured response: {}, sender: {:?}",
            prompt, self.owner_wallet_address
        );

        // If no API key is configured, return error as per V3 plan
        if self.api_key.is_none() {
            return Err(anyhow!("No API key configured for prompt processor"));
        }

        // Check if this is a request with "all" keyword for any operation type
        let original_prompt = prompt.to_string();
        let _is_all_keyword = original_prompt.to_lowercase().contains("all");

        // Calculate max amounts for all action types using MaxAmountCalculator
        let owner_wallet_address = match &self.owner_wallet_address {
            Some(owner_wallet_address) => owner_wallet_address,
            None => panic!("Required owner_wallet_address"),
        };

        let max_amounts =
            MaxAmountCalculator::calculate_all_max_amounts(owner_wallet_address).await?;
        let max_amounts_yml = MaxAmountCalculator::format_as_yml_prompt(&max_amounts)?;

        info!("Calculated max amounts for all action types");
        debug!("Max amounts YML: {}", max_amounts_yml);

        // Build structured LLM request
        let request = StructuredRefineRequest {
            prompt: original_prompt,
            owner_wallet_address: Some(owner_wallet_address.to_string()),
            max_amount: None, // We'll provide all max amounts in the prompt
            max_amounts_yml: Some(max_amounts_yml.clone()),
        };

        // Send request to LLM
        info!("Sending structured refine request to LLM");
        let response = self
            .send_structured_refine_request(&request, &max_amounts_yml)
            .await;

        // Just return the response or error directly
        let response = response?;
        info!("Received structured response from LLM: {}", response);
        debug!(
            "Raw LLM response (first 500 chars): {}",
            &response[..response.len().min(500)]
        );
        // Parse response - response may already be a JSON string of StructuredRefineResponse
        // or a plain string that needs to be converted
        let response_obj = if response.trim().starts_with('{') {
            // Response is JSON, parse it directly
            match serde_json::from_str::<serde_json::Value>(&response) {
                Ok(json_value) => {
                    // Extract refined_prompt if present
                    let refined_prompt = json_value
                        .get("refined_prompt")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&response)
                        .to_string();

                    // Try to extract action if present
                    let action = json_value
                        .get("action")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    // Extract other optional fields
                    let subject_pubkey = json_value
                        .get("subject_pubkey")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .or_else(|| Some(owner_wallet_address.to_string()));

                    let target_pubkey = json_value
                        .get("target_pubkey")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let param_json = json_value.get("parameters");

                    let parameters = if let Some(param_value) = param_json {
                        match serde_json::from_value::<PromptParameters>(param_value.clone()) {
                            Ok(params) => {
                                debug!("Successfully parsed PromptParameters: {:?}", params);
                                params
                            }
                            Err(e) => {
                                error!(
                                    "Failed to parse PromptParameters: {e}. LLM response: {param_value}"
                                );
                                return Err(anyhow!("Invalid parameters in LLM response: {e}"));
                            }
                        }
                    } else {
                        error!("No parameters field found in LLM response");
                        return Err(anyhow!("Missing parameters field in LLM response"));
                    };

                    let confidence = json_value
                        .get("confidence")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.7); // Use higher default confidence

                    // If action is unknown, try to extract it from refined_prompt
                    let action = if action == "unknown" && !refined_prompt.is_empty() {
                        Self::extract_action_from_prompt(&refined_prompt)
                    } else {
                        action
                    };

                    StructuredRefineResponse {
                        refined_prompt,
                        action,
                        subject_pubkey,
                        target_pubkey,
                        parameters,
                        confidence,
                    }
                }
                Err(_) => {
                    // No fallback - return error if LLM response is invalid
                    return Err(anyhow!(
                        "LLM returned invalid JSON response: {response}. Please try again."
                    ));
                }
            }
        } else {
            // No fallback for plain text - require valid JSON
            return Err(anyhow!(
                "LLM did not return valid JSON response: {response}. Please try again."
            ));
        };

        // Check if LLM detected "all" keyword and extracted a usable amount
        let usable_amount = if prompt.to_lowercase().contains("all")
            && response_obj.parameters.amount.is_some()
            && response_obj
                .parameters
                .amount
                .as_ref()
                .unwrap_or(&"0".to_string())
                != "all"
        {
            // LLM has calculated a usable amount from max_amounts
            response_obj
                .parameters
                .amount
                .as_ref()
                .and_then(|a| a.parse::<f64>().ok())
        } else {
            None
        };

        // Convert to StructuredRefinedPrompt
        let mut structured_prompt = response_obj
            .to_structured_prompt(prompt.to_string(), usable_amount)
            .map_err(|e| anyhow!("Failed to convert structured response: {e}"))?;

        // Validate response with max amounts
        let validation_result = validate_structured_response_with_max_amounts(
            &structured_prompt,
            prompt,
            &max_amounts_yml,
        );
        match validation_result {
            ValidationResult::Valid => {
                info!("Structured response validation passed");
            }
            ValidationResult::Invalid(issues) => {
                warn!("Structured response validation failed: {:?}", issues);

                // Calculate confidence score
                let confidence = calculate_confidence_score(&structured_prompt, prompt);
                structured_prompt.confidence = confidence;
            }
        }

        Ok(structured_prompt)
    }

    /// Extract action type from prompt
    fn extract_action_from_prompt(prompt: &str) -> String {
        let prompt_lower = prompt.to_lowercase();
        if prompt_lower.contains("transfer") || prompt_lower.contains("send") {
            "transfer".to_string()
        } else if prompt_lower.contains("swap") {
            "swap".to_string()
        } else if prompt_lower.contains("lend") {
            "lend".to_string()
        } else if prompt_lower.contains("earn") {
            "earn".to_string()
        } else if prompt_lower.contains("borrow") {
            "borrow".to_string()
        } else {
            // Try to match partial words for typos
            if (prompt_lower.contains("tras") && prompt_lower.contains("fer"))
                || (prompt_lower.contains("trasnfer"))
            {
                "transfer".to_string()
            } else if prompt_lower.contains("swp") {
                "swap".to_string()
            } else {
                "unknown".to_string()
            }
        }
    }

    // REMOVED: extract_target_pubkey_from_prompt - LLM should always extract pubkeys

    // REMOVED: extract_parameters_from_prompt - LLM should always extract parameters

    /// Send structured refine request to LLM
    #[instrument(skip(self))]
    async fn send_structured_refine_request(
        &self,
        request: &StructuredRefineRequest,
        max_amounts_yml: &str,
    ) -> Result<String> {
        info!("Sending structured refine request to LLM");
        let client = reqwest::Client::new();

        // Build the structured system prompt
        let system_prompt = self.build_structured_system_prompt();

        // Build user prompt with max amounts YML
        let user_prompt = serde_json::json!({
            "prompt": request.prompt,
            "owner_wallet_address": request.owner_wallet_address,
            "max_amount": request.max_amount,
            "max_amounts_yml": max_amounts_yml
        })
        .to_string();

        // Use the correct model name for ZAI API
        let model_name = if self.model_name == "glm-4.6-coding" {
            "glm-4.6"
        } else {
            &self.model_name
        };

        // Create request body
        let request_body = serde_json::json!({
            "model": model_name,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": user_prompt
                }
            ],
            "temperature": 0.3,
            "max_tokens": 1000
        });

        // Send request - use the same URL as the working implementation
        let url = "https://api.z.ai/api/coding/paas/v4/chat/completions";
        info!("Sending structured request to LLM API at URL: {}", url);
        let response = client
            .post(url)
            .header(
                "Authorization",
                format!(
                    "Bearer {}",
                    self.api_key
                        .as_ref()
                        .ok_or_else(|| anyhow!("No API key configured"))?
                ),
            )
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        // Check response status
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("LLM API error: {status} - {error_text}"));
        }

        // Parse response
        let response_body: serde_json::Value = response.json().await?;

        // Extract content
        let content = response_body
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .ok_or_else(|| anyhow!("Invalid response format from LLM"))?;

        // Debug output
        info!("LLM response: {}", content);

        // Parse JSON response
        let json_result = serde_json::from_str::<serde_json::Value>(content);
        match json_result {
            Ok(json) => {
                info!("Parsed JSON response: {:?}", json);
            }
            Err(e) => {
                info!("Failed to parse JSON: {}, response was: {}", e, content);
            }
        }

        Ok(content.to_string())
    }

    /// Build structured system prompt for LLM
    fn build_structured_system_prompt(&self) -> String {
        // Use structured prompt system prompt
        crate::prompts::prompt_processor::STRUCTURED_PROMPT_SYSTEM_PROMPT.to_string()
    }

    /// Send request to LLM for language refinement
    async fn send_refine_request(&self, request: &LanguageRefineRequest) -> Result<String> {
        let client = reqwest::Client::new();
        let url = "https://api.z.ai/api/coding/paas/v4/chat/completions";

        // Use system prompt from prompts module
        let system_prompt = PROMPT_PROCESSOR_SYSTEM_PROMPT;

        // Use the correct model name for ZAI API
        let model_name = if self.model_name == "glm-4.6-coding" {
            "glm-4.6"
        } else {
            &self.model_name
        };

        let body = serde_json::json!({
            "model": model_name,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": format!("Refine this prompt: {}", request.prompt)}
            ],
            "temperature": 0.1,
            "max_tokens": 500
        });

        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.as_ref().unwrap()),
            )
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to LLM: {e}"))?;

        let status = response.status();
        debug!("Response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            debug!("Error response: {}", error_text);
            return Err(anyhow!("LLM request failed with status: {status}"));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read LLM response: {e}"))?;

        debug!("Raw response text length: {}", response_text.len());
        debug!("Raw response text: {}", response_text);

        // Extract the content from the response
        let response_json: serde_json::Value =
            serde_json::from_str(&response_text).map_err(|e| {
                error!("JSON parsing error: {}", e);
                error!(
                    "First 200 chars of response: {}",
                    &response_text[..response_text.len().min(200)]
                );
                anyhow!("Failed to parse JSON: {e}")
            })?;

        // Try to get content first, then reasoning_content (for GLM model)
        if let Some(content) = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
        {
            // Check if the content is valid JSON for our LanguageRefineResponse
            // Try to parse the content as JSON for our LanguageRefineResponse
            if let Ok(_lang_response) = serde_json::from_str::<LanguageRefineResponse>(content) {
                // Valid JSON response, use it directly
                debug!("Valid JSON response from LLM: {}", content);
                Ok(content.to_string())
            } else if content.contains('{') && content.contains('}') {
                // Try to extract JSON from content if there's additional text
                let start = content.find('{').unwrap_or(0);
                let end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
                let json_content = &content[start..end];

                debug!("Extracted JSON from content: {}", json_content);

                // Check if this is a partial response with just refined_prompt
                if let Ok(partial) = serde_json::from_str::<serde_json::Value>(json_content) {
                    if let Some(refined_prompt) =
                        partial.get("refined_prompt").and_then(|v| v.as_str())
                    {
                        // Create a complete LanguageRefineResponse from the partial
                        let response = LanguageRefineResponse {
                            refined_prompt: refined_prompt.to_string(),
                            confidence: 0.9,
                        };
                        return serde_json::to_string(&response)
                            .map_err(|e| anyhow!("Failed to serialize response: {e}"));
                    }
                }

                Ok(json_content.to_string())
            } else {
                // Not JSON, treat as plain text response
                debug!("Plain text response from LLM: {}", content);

                // Create a valid LanguageRefineResponse from the content
                let response = LanguageRefineResponse {
                    refined_prompt: content.to_string(),
                    confidence: if content != request.prompt { 0.8 } else { 0.95 },
                };

                serde_json::to_string(&response)
                    .map_err(|e| anyhow!("Failed to serialize response: {e}"))
            }
        } else if let Some(reasoning_content) = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("reasoning_content"))
            .and_then(|c| c.as_str())
        {
            // Extract the refined prompt from reasoning content
            // The GLM response contains analysis in Chinese, but the refined prompt should be in English
            // We need to extract the actual refined prompt from the reasoning text
            let refined = extract_refined_prompt_from_reasoning(reasoning_content, &request.prompt);
            debug!("Extracted refined prompt: {}", refined);

            // Create a valid LanguageRefineResponse from the extracted prompt
            let response = LanguageRefineResponse {
                refined_prompt: refined,
                confidence: 0.9,
            };

            serde_json::to_string(&response)
                .map_err(|e| anyhow!("Failed to serialize response: {e}"))
        } else {
            Err(anyhow!("Invalid LLM response format"))
        }
    }

    // Rule-based refiner removed as per V3 plan
    // All language refinement must be handled by the LLM
}

/// Result of prompt refinement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinedPrompt {
    /// Original prompt
    pub original: String,
    /// Refined prompt
    pub refined: String,
    /// Confidence in the refinement (0.0-1.0)
    pub confidence: f32,
    /// Usable amount for transfers (when "all" keyword was used)
    pub usable_amount: Option<f64>,
}

/// Create wallet context from wallet address
async fn create_wallet_context(wallet_address: &str) -> Result<reev_types::flow::WalletContext> {
    use reev_types::benchmark::TokenBalance;
    use reev_types::flow::WalletContext;
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    info!("Creating wallet context for address: {}", wallet_address);

    // Parse wallet address
    let pubkey =
        Pubkey::from_str(wallet_address).map_err(|e| anyhow!("Invalid wallet address: {e}"))?;

    // Create RPC client to query balance
    let client = RpcClient::new("http://localhost:8899".to_string());

    // Get account balance with fallback
    let balance = match client.get_balance(&pubkey).await {
        Ok(balance) => {
            info!(
                "Retrieved balance: {} lamports for address: {}",
                balance, wallet_address
            );
            balance
        }
        Err(e) => {
            warn!(
                "Failed to get balance for address: {}, using default. Error: {}",
                wallet_address, e
            );
            // Default to 1 SOL for testing purposes
            1_000_000_000
        }
    };

    // Create token balances map
    let mut token_balances = std::collections::HashMap::new();

    // Query actual token balances from surfpool for common tokens

    // Common SPL token mints
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let usdt_mint = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";

    // Query USDC balance
    if let Ok(Some(usdc_balance)) =
        get_token_balance_from_surfpool(&client, &pubkey, usdc_mint).await
    {
        token_balances.insert(
            usdc_mint.to_string(),
            TokenBalance::new(usdc_mint.to_string(), usdc_balance)
                .with_decimals(6)
                .with_symbol("USDC".to_string()),
        );
        info!(
            "Retrieved USDC balance: {} tokens",
            usdc_balance as f64 / 1_000_000.0
        );
    } else {
        // Add default USDC balance if query fails
        token_balances.insert(
            usdc_mint.to_string(),
            TokenBalance::new(usdc_mint.to_string(), 100_000_000)
                .with_decimals(6)
                .with_symbol("USDC".to_string()),
        );
    }

    // Query USDT balance
    if let Ok(Some(usdt_balance)) =
        get_token_balance_from_surfpool(&client, &pubkey, usdt_mint).await
    {
        token_balances.insert(
            usdt_mint.to_string(),
            TokenBalance::new(usdt_mint.to_string(), usdt_balance)
                .with_decimals(6)
                .with_symbol("USDT".to_string()),
        );
        info!(
            "Retrieved USDT balance: {} tokens",
            usdt_balance as f64 / 1_000_000.0
        );
    } else {
        // Add default USDT balance if query fails
        token_balances.insert(
            usdt_mint.to_string(),
            TokenBalance::new(usdt_mint.to_string(), 100_000_000)
                .with_decimals(6)
                .with_symbol("USDT".to_string()),
        );
    }

    // Create wallet context
    Ok(WalletContext {
        owner: wallet_address.to_string(),
        sol_balance: balance,
        token_balances,
        total_value_usd: balance as f64 / 1_000_000_000.0, // Simplified: 1 SOL = $1
        token_prices: std::collections::HashMap::new(),
    })
}

/// Query token balance from surfpool
/// Get token balance from surfpool blockchain
async fn get_token_balance_from_surfpool(
    client: &solana_client::nonblocking::rpc_client::RpcClient,
    pubkey: &solana_sdk::pubkey::Pubkey,
    mint: &str,
) -> Result<Option<u64>> {
    // Parse mint address
    let mint_pubkey = Pubkey::from_str(mint).map_err(|e| anyhow!("Invalid mint address: {e}"))?;

    // Get associated token account address
    let ata = spl_associated_token_account::get_associated_token_address(pubkey, &mint_pubkey);

    // Query token account balance
    match client.get_token_account_balance(&ata).await {
        Ok(balance) => {
            // Extract amount from UiTokenAmount
            let amount = balance.amount;
            // Parse amount string to u64
            match amount.parse::<u64>() {
                Ok(parsed_amount) => Ok(Some(parsed_amount)),
                Err(e) => {
                    warn!("Failed to parse token amount: {}", e);
                    Ok(None)
                }
            }
        }
        Err(_) => {
            // Token account might not exist, return None
            warn!("No token account found for mint: {}", mint);
            Ok(None)
        }
    }
}

impl RefinedPrompt {
    /// Create a new refined prompt with default values for testing
    ///
    /// This constructor should only be used in tests.
    /// Production code should use the full struct initialization.
    pub fn new_for_test(original: String, refined: String) -> Self {
        Self {
            original,
            refined,
            confidence: 0.8, // Default confidence for testing
            usable_amount: None,
        }
    }

    /// Create a new refined prompt with usable amount for testing
    ///
    /// This constructor should only be used in tests.
    /// Production code should use the full struct initialization.
    pub fn new_for_test_with_amount(original: String, refined: String, usable_amount: f64) -> Self {
        Self {
            original,
            refined,
            confidence: 0.8, // Default confidence for testing
            usable_amount: Some(usable_amount),
        }
    }

    /// Get the confidence level of refinement
    pub fn get_confidence(&self) -> f32 {
        self.confidence
    }
}

/// Structured YML prompt for LLM to refine amounts when "all" keyword is used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferAmountRefinementRequest {
    /// The original user prompt
    pub original_prompt: String,
    /// Maximum usable amount in token units (after gas reserve)
    pub usable_amount: f64,
    /// Instruction for the LLM
    pub instruction: String,
}

impl TransferAmountRefinementRequest {
    /// Create a new refinement request
    pub fn new(original_prompt: String, usable_amount: f64) -> Self {
        Self {
            original_prompt,
            usable_amount,
            instruction: "Replace 'all' with the usable amount in the prompt. You MUST respond with valid JSON: {\"refined_prompt\": \"your refined prompt here\"}".to_string(),
        }
    }

    /// Convert to YAML string for LLM
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| anyhow!("Failed to serialize refinement request: {e}"))
    }
}

/// Build a structured prompt for amount refinement when "all" keyword is used
pub fn build_refinement_prompt(original_prompt: &str, usable_amount: f64) -> String {
    let prompt = format!(
        r#"You are a DeFi assistant that refines prompts with "all" keyword.

Replace "all" with the usable amount in the prompt below.

Original Prompt: "{original_prompt}"
Usable Amount: {usable_amount}

Respond with valid JSON ONLY:
{{
"refined_prompt": "replace 'all' with the usable amount"
}}

Example:
Original Prompt: "swap all SOL for USDC"
Usable Amount: 4.999
Response: {{
"refined_prompt": "swap 4.999 SOL for USDC"
}}"#
    );

    info!("Built refinement prompt for 'all' keyword: {}", prompt);
    prompt
}

/// Extract the refined prompt from LLM response
/// This function prioritizes JSON responses and falls back to pattern matching
fn extract_refined_prompt_from_reasoning(reasoning: &str, original_prompt: &str) -> String {
    // First, check if the reasoning contains JSON that we can parse directly
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(reasoning) {
        // If the entire reasoning is valid JSON, check if it has refined_prompt field
        if let Some(refined_prompt) = json_value.get("refined_prompt").and_then(|v| v.as_str()) {
            return refined_prompt.to_string();
        }
    }

    // If not direct JSON, try to extract from text
    // Look for "refined_prompt" key in the reasoning
    if let Some(start) = reasoning.find("\"refined_prompt\":") {
        let after_keyword = &reasoning[start + "\"refined_prompt\":".len()..];
        if let Some(start_quote) = after_keyword.find('"') {
            let after_start_quote = &after_keyword[start_quote + 1..];
            if let Some(end_quote) = after_start_quote.find('"') {
                let refined = after_start_quote[..end_quote].to_string();
                // Check if it looks like a valid prompt
                if refined.len() > 5
                    && (refined.contains("swap")
                        || refined.contains("transfer")
                        || refined.contains("lend")
                        || refined.contains("send"))
                {
                    return refined;
                }
            }
        }
    }

    // Look for patterns like "优化后的提示应该是：" (The refined prompt should be:)
    let lines: Vec<&str> = reasoning.lines().collect();

    // Look for patterns in the reasoning that indicate the refined prompt
    for line in lines.iter().rev() {
        if line.contains("优化后的提示应该是") {
            // Extract the refined prompt after the colon
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if end > start {
                        let refined = line[start + 1..end].to_string();
                        // Check if it looks like a valid prompt
                        if refined.len() > 5
                            && (refined.contains("swap")
                                || refined.contains("transfer")
                                || refined.contains("lend")
                                || refined.contains("send"))
                        {
                            return refined;
                        }
                    }
                }
            }
        }
    }

    // If we can't find a proper refined prompt, return the original prompt unchanged
    // This is better than returning the GLM analysis which would break the system
    original_prompt.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LanguageRefineRequest {
    /// Original prompt to refine
    prompt: String,
}

/// Response from language refinement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageRefineResponse {
    /// Refined prompt
    pub refined_prompt: String,
    /// Confidence in the refinement (0.0-1.0)
    pub confidence: f32,
}
