//! Structured prompt processor for Phase 1 of V3 Plan
//!
//! This module implements the structured prompt processor with max amount calculation.
//! It uses serde_yaml for structured serialization/deserialization.

use crate::prompt_processor::max_amount_calculator::MaxAmountCalculator;
use crate::prompt_processor::types::{
    StructuredRefineRequest, StructuredRefineResponse, StructuredRefinedPrompt, ValidationResult,
};
use crate::prompt_processor::validation::validate_structured_response_with_max_amounts;
use crate::prompts::prompt_processor::STRUCTURED_PROMPT_SYSTEM_PROMPT;
use anyhow::{anyhow, Result};

use serde_json;
use std::env;
use tracing::{debug, error, info, warn};

/// Structured prompt processor for handling refined prompts with max amounts
#[derive(Debug, Clone)]
pub struct StructuredProcessor {
    /// API key for LLM service
    api_key: String,
    /// Model name for LLM
    model_name: String,
}

impl Default for StructuredProcessor {
    fn default() -> Self {
        let model_name = env::var("GLM_MODEL").unwrap_or_else(|_| "glm-4.6-coding".to_string());
        let api_key = env::var("ZAI_API_KEY").unwrap_or_else(|_| "test_key".to_string()); // Use test key for unit tests

        Self {
            api_key,
            model_name,
        }
    }
}

impl StructuredProcessor {
    /// Create a new structured processor
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a structured processor with custom configuration
    pub fn with_config(api_key: String, model_name: String) -> Self {
        Self {
            api_key,
            model_name,
        }
    }

    /// Process prompt with structured LLM response and max amounts
    pub async fn process_prompt_structured(
        &self,
        prompt: &str,
        owner_wallet_address: &str,
    ) -> Result<StructuredRefinedPrompt> {
        info!(
            "Processing prompt with structured response: {}, sender: {}",
            prompt, owner_wallet_address
        );

        // Calculate max amounts for all action types using MaxAmountCalculator with prompt context
        let max_amounts = MaxAmountCalculator::calculate_all_max_amounts_with_context(
            owner_wallet_address,
            prompt,
        )
        .await?;
        let max_amounts_yml = MaxAmountCalculator::format_as_yml_prompt(&max_amounts)?;

        info!("Calculated max amounts for all action types");
        debug!("Max amounts YML: {}", max_amounts_yml);

        // Build structured LLM request
        let request = StructuredRefineRequest {
            prompt: prompt.to_string(),
            owner_wallet_address: Some(owner_wallet_address.to_string()),
            max_amount: None, // We'll provide all max amounts in the prompt
            max_amounts_yml: Some(max_amounts_yml.clone()),
        };

        // Send request to LLM
        info!("Sending structured refine request to LLM");
        let response = self
            .send_structured_refine_request(&request, &max_amounts_yml)
            .await?;

        info!("Received structured response from LLM");
        debug!(
            "Raw LLM response (first 500 chars): {}",
            &response[..response.len().min(500)]
        );

        // Parse JSON response using serde_json
        let response_obj: StructuredRefineResponse = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse LLM JSON response: {e}"))?;

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

        // Ensure subject_pubkey is set to owner wallet address
        if structured_prompt.subject_pubkey.is_none() {
            structured_prompt.subject_pubkey = Some(owner_wallet_address.to_string());
        }

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
                // For invalid responses, return error instead of adjusting confidence
                return Err(anyhow!("Validation failed: {issues:?}"));
            }
        }

        Ok(structured_prompt)
    }

    /// Send structured refine request to LLM
    async fn send_structured_refine_request(
        &self,
        request: &StructuredRefineRequest,
        max_amounts_yml: &str,
    ) -> Result<String> {
        // Build the system prompt with max amounts
        let system_prompt = self.build_structured_system_prompt(max_amounts_yml);

        // Create a client
        let client = reqwest::Client::new();

        // Prepare the request payload
        let payload = serde_json::json!({
            "model": self.model_name,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": serde_json::json!({
                    "prompt": request.prompt,
                    "owner_wallet_address": request.owner_wallet_address,
                    "max_amounts_yml": max_amounts_yml
                }).to_string()}
            ],
            "temperature": 0.2
        });

        info!("Sending request to LLM: model={}", self.model_name);
        debug!(
            "Request payload: {}",
            serde_json::to_string_pretty(&payload)?
        );

        // Use correct API endpoint and model name for ZAI API
        let api_endpoint = "https://api.z.ai/api/coding/paas/v4/chat/completions";
        let model_name = if self.model_name == "glm-4.6-coding" {
            "glm-4.6"
        } else {
            &self.model_name
        };

        // Update the model in the payload
        let mut fixed_payload = payload.clone();
        if let Some(obj) = fixed_payload.as_object_mut() {
            obj.insert(
                "model".to_string(),
                serde_json::Value::String(model_name.to_string()),
            );
        }

        // Send request to LLM
        let response = client
            .post(api_endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&fixed_payload)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to LLM: {e}"))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            error!("LLM API error: {}", error_text);
            return Err(anyhow!("LLM API error: {error_text}"));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read LLM response: {e}"))?;

        info!("Received response from LLM");
        debug!("Response text: {}", response_text);

        // Parse response to extract content
        let json_response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse LLM response JSON: {e}"))?;

        let content = json_response
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Invalid response format: missing content"))?;

        Ok(content.to_string())
    }

    /// Build structured system prompt with max amounts
    fn build_structured_system_prompt(&self, max_amounts_yml: &str) -> String {
        format!(
            r#"{STRUCTURED_PROMPT_SYSTEM_PROMPT}

Max amounts provided:
{max_amounts_yml}
"#
        )
    }
}
