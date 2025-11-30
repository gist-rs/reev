//! Language Refiner for Phase 1 of V3 Plan
//!
//! This module implements language refinement functionality in Phase 1 of V3 plan.
//! It uses LLM to refine user prompts by fixing typos, normalizing terminology, and making
//! language clearer and more unambiguous.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use tracing::{debug, error, info, instrument, warn};

// Import prompts
use crate::prompts;
use prompts::prompt_processor::PROMPT_PROCESSOR_SYSTEM_PROMPT;

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
    /// Create a new language refiner
    pub fn new() -> Self {
        let model_name = std::env::var("GLM_MODEL").unwrap_or_else(|_| "glm-4".to_string());
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
        self.owner_wallet_address = Some(owner_wallet_address.to_string());
        info!(
            "Processing prompt: {}, sender: {:?}",
            prompt, self.owner_wallet_address
        );

        // If no API key is configured, return error as per V3 plan
        if self.api_key.is_none() {
            return Err(anyhow!("No API key configured for language refiner"));
        }

        // Check if this is a transfer request with "all" keyword
        let original_prompt = prompt.to_string();
        let is_all_transfer = original_prompt.to_lowercase().contains("all")
            && (original_prompt.to_lowercase().contains("transfer")
                || original_prompt.to_lowercase().contains("send"));

        let modified_prompt = if is_all_transfer {
            // This is a transfer with "all" keyword, calculate maximum transferable amount
            let owner_wallet_address = match &self.owner_wallet_address {
                Some(owner_wallet_address) => owner_wallet_address,
                None => panic!("Required owner_wallet_address"),
            };

            // Create wallet context to get balance
            let wallet_context = create_wallet_context(owner_wallet_address).await?;

            // Calculate gas reserve (0.001 SOL for now)
            let gas_reserve = 1_000_000u64; // 0.001 SOL in lamports

            // Calculate maximum transferable amount
            let max_amount = crate::utils::transfer_utils::calculate_max_transferable_amount(
                "", // Empty for SOL
                wallet_context.sol_balance,
                gas_reserve,
            );

            // Convert max_amount to SOL for display
            let max_amount_sol = max_amount as f64 / 1_000_000_000.0;

            info!(
                "Detected 'all' keyword, calculated max transferable amount: {} SOL",
                max_amount_sol
            );

            // Add calculated amount as context for the LLM to use
            format!(
                "Transferable amount: {max_amount_sol:.3} SOL. {original_prompt}"
            )
        } else {
            original_prompt.clone()
        };

        // Build LLM request for language refinement
        let request = LanguageRefineRequest {
            prompt: modified_prompt,
        };

        // Send request to LLM
        let response = self.send_refine_request(&request).await;

        // Handle LLM request failure
        let response = match response {
            Ok(response) => response,
            Err(e) => {
                warn!("LLM request failed: {}", e);
                // For "all" transfers, fallback to simple replacement
                if is_all_transfer {
                    let owner_wallet_address = match &self.owner_wallet_address {
                        Some(owner_wallet_address) => owner_wallet_address,
                        None => panic!("Required owner_wallet_address"),
                    };

                    // Create wallet context to get balance
                    let wallet_context = create_wallet_context(owner_wallet_address).await?;

                    // Calculate gas reserve (0.001 SOL for now)
                    let gas_reserve = 1_000_000u64; // 0.001 SOL in lamports

                    // Calculate maximum transferable amount
                    let max_amount =
                        crate::utils::transfer_utils::calculate_max_transferable_amount(
                            "", // Empty for SOL
                            wallet_context.sol_balance,
                            gas_reserve,
                        );

                    // Convert max_amount to SOL for display
                    let max_amount_sol = max_amount as f64 / 1_000_000_000.0;

                    // Return refined prompt with calculated amount
                    return Ok(RefinedPrompt::new_for_test(
                        original_prompt.to_string(),
                        original_prompt
                            .to_lowercase()
                            .replace("all", &format!("{max_amount_sol:.3}")),
                        true,
                    ));
                }

                return Err(anyhow!("LLM request failed: {e}"));
            }
        };

        // Parse response - response may already be a JSON string of LanguageRefineResponse
        // or a plain string that needs to be converted
        let response_obj = if response.starts_with('{') {
            // Response is JSON, parse it directly
            match serde_json::from_str::<LanguageRefineResponse>(&response) {
                Ok(r) => r,
                Err(e) => {
                    warn!("Failed to parse LLM JSON response: {}", e);
                    return Err(anyhow!("Failed to parse LLM response: {e}"));
                }
            }
        } else {
            // Response is plain text, create a LanguageRefineResponse from it
            let changed = response != request.prompt;
            LanguageRefineResponse {
                refined_prompt: response.clone(),
                changes_detected: changed,
                confidence: if changed { 0.8 } else { 0.95 },
            }
        };

        // Create refined prompt object
        let refined = RefinedPrompt::new_for_test(
            prompt.to_string(),
            response_obj.refined_prompt.clone(),
            response_obj.changes_detected,
        );
        info!("Processed prompt: {}", refined.refined);
        debug!("Original: {} -> Refined: {}", prompt, refined.refined);

        // Log the raw response for debugging
        debug!("LLM raw response: {}", response);

        Ok(RefinedPrompt {
            original: prompt.to_string(),
            refined: refined.refined,
            changes_detected: refined.changes_detected,
            confidence: refined.confidence,
        })
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
            "max_tokens": 200,
            "response_format": {"type": "json_object"}
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

        // Try to get reasoning_content first (for GLM model), then content
        if let Some(reasoning_content) = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("reasoning_content"))
            .and_then(|c| c.as_str())
        {
            debug!("Found reasoning_content from GLM, extracting refined prompt");
            // Extract the refined prompt from reasoning content
            // The GLM response contains analysis in Chinese, but the refined prompt should be in English
            // We need to extract the actual refined prompt from the reasoning text
            let refined = extract_refined_prompt_from_reasoning(reasoning_content, &request.prompt);
            debug!("Extracted refined prompt: {}", refined);

            // Create a valid LanguageRefineResponse from the extracted prompt
            Ok(serde_json::to_string(&LanguageRefineResponse {
                refined_prompt: refined,
                changes_detected: true,
                confidence: 0.9,
            })
            .unwrap())
        } else if let Some(content) = response_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
        {
            // Create a valid LanguageRefineResponse from the content
            Ok(serde_json::to_string(&LanguageRefineResponse {
                refined_prompt: content.to_string(),
                changes_detected: false,
                confidence: 0.95,
            })
            .unwrap())
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
    /// Whether changes were detected
    pub changes_detected: bool,
    /// Confidence in the refinement (0.0-1.0)
    confidence: f32,
}

/// Create wallet context from wallet address
async fn create_wallet_context(wallet_address: &str) -> Result<reev_types::flow::WalletContext> {
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

    // Get account balance
    let balance = client.get_balance(&pubkey).await?;
    info!(
        "Retrieved balance: {} lamports for address: {}",
        balance, wallet_address
    );

    // Create token balances map (empty for now)
    let _token_balances: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

    // Create wallet context
    Ok(WalletContext {
        owner: wallet_address.to_string(),
        sol_balance: balance,
        token_balances: std::collections::HashMap::new(),
        total_value_usd: balance as f64 / 1_000_000_000.0, // Simplified: 1 SOL = $1
        token_prices: std::collections::HashMap::new(),
    })
}

impl RefinedPrompt {
    /// Create a new refined prompt (for testing)
    pub fn new_for_test(original: String, refined: String, changes_detected: bool) -> Self {
        Self {
            original,
            refined,
            changes_detected,
            confidence: 0.8, // Default confidence for testing
        }
    }

    /// Get confidence of the refinement
    pub fn get_confidence(&self) -> f32 {
        self.confidence
    }
}

/// Extract the refined prompt from GLM reasoning content
fn extract_refined_prompt_from_reasoning(reasoning: &str, original_prompt: &str) -> String {
    // The GLM reasoning content contains analysis in Chinese
    // We need to properly extract refined prompt based on the JSON response format

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
        let after_key = &reasoning[start + "\"refined_prompt\":".len()..];
        if let Some(start_quote) = after_key.find('"') {
            let after_start_quote = &after_key[start_quote + 1..];
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

    // Check for common problematic patterns from GLM responses
    if reasoning.contains("The user wants me to refine the prompt") {
        // Extract the original prompt from the GLM response
        if let Some(start) = reasoning.find('"') {
            if let Some(end) = reasoning.rfind('"') {
                if end > start {
                    let original = reasoning[start + 1..end].to_string();
                    // Remove "The user wants me to refine prompt: " prefix if present
                    if let Some(stripped) =
                        original.strip_prefix("The user wants me to refine prompt: ")
                    {
                        return stripped.to_string();
                    }
                    return original;
                }
            }
        }
    }

    // If all else fails, check for any English text that looks like a prompt
    // Avoid returning the GLM analysis itself
    for line in lines {
        // If a line contains only ASCII characters and operation words
        if line.is_ascii() && line.len() > 10 {
            let trimmed = line.trim().trim_matches('"');
            // Check if it contains operation words and doesn't look like analysis
            if !trimmed.is_empty()
                && (trimmed.contains("swap")
                    || trimmed.contains("transfer")
                    || trimmed.contains("lend")
                    || trimmed.contains("send"))
                && !trimmed.contains("The user wants me")
                && !trimmed.contains("I should")
                && !trimmed.contains("This prompt")
            {
                return trimmed.to_string();
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
struct LanguageRefineResponse {
    /// Refined prompt
    refined_prompt: String,
    /// Whether changes were detected
    changes_detected: bool,
    /// Confidence in the refinement
    confidence: f32,
}
