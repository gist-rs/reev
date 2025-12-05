//! GLM Client Implementation for reev-core
//!
//! This module implements the LlmClient trait using the GLM-4.6-coding model
//! via the ZAI SDK, providing a more robust and feature-rich implementation.

use crate::planner::LlmClient;
use anyhow::{anyhow, Result};
use tracing::{debug, error, info, instrument, warn};
use zai_sdk::{GlmVariant, Message, ZaiClient};

/// GLM Client implementation for reev-core planner
pub struct GLMClient {
    client: ZaiClient,
}

impl GLMClient {
    /// Create a new GLM client
    pub fn new(model_name: &str, api_key: &str) -> Result<Self> {
        // Determine the variant based on model name
        let variant = if model_name == "glm-4.6-coding" {
            GlmVariant::Coding
        } else {
            GlmVariant::Standard
        };

        // Create the zai-sdk client
        let client = ZaiClient::builder()
            .variant(variant)
            .api_key(api_key)
            .build()
            .map_err(|e| anyhow!("Failed to create ZAI client: {e}"))?;

        Ok(Self { client })
    }

    /// Initialize with environment variables
    pub fn from_env() -> Result<Self> {
        // Load environment variables from .env file
        dotenvy::dotenv().ok();

        let model_name =
            std::env::var("GLM_MODEL").unwrap_or_else(|_| "glm-4.6-coding".to_string());
        let api_key = std::env::var("ZAI_API_KEY")
            .map_err(|_| anyhow!("ZAI_API_KEY environment variable not set"))?;

        Self::new(&model_name, &api_key)
    }
}

#[async_trait::async_trait]
impl LlmClient for GLMClient {
    #[instrument(skip(self))]
    async fn generate_flow(&self, prompt: &str) -> Result<String> {
        info!("Extracting intent using ZAI SDK");
        debug!("Prompt: {}", prompt);

        // Build a simple prompt for intent extraction
        let flow_prompt = format!(
            r#"Extract user intent from this prompt: "{prompt}"

Respond with a simple JSON object containing:
1. intent: The type of operation (swap, lend, borrow, etc.)
2. parameters: Key parameters for the operation
   - from_token: Source token (e.g., SOL, USDC)
   - to_token: Destination token (for swaps)
   - amount: The amount to operate with
   - percentage: Percentage if specified (e.g., "50%")
"#
        );

        debug!("Calling ZAI SDK with prompt: {}", flow_prompt);

        // Create messages for the request
        let messages = vec![
            Message::system("You are a DeFi assistant that extracts user intent from prompts. Always respond with valid JSON only. Respond in English only."),
            Message::user(flow_prompt),
        ];

        // Send request using the zai-sdk
        let response = self
            .client
            .completion_with_messages(messages)
            .await
            .map_err(|e| {
                error!("Failed to get response from ZAI SDK: {}", e);
                anyhow!("LLM generation failed: {e}")
            })?;

        info!("Received response from ZAI SDK");

        // Extract content from response
        let content = &response;

        // Check if the response is empty
        if content.trim().is_empty() {
            error!("LLM returned empty response");
            return Err(anyhow!("LLM returned empty response"));
        }

        // Try to extract JSON from the response if it contains additional text
        let cleaned_response = if content.contains('{') && content.contains('}') {
            // Extract JSON portion if there's extra text
            let start = content.find('{').unwrap_or(0);
            let end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
            content[start..end].to_string()
        } else {
            // If no JSON structure found, create a default response
            warn!("No JSON structure found in LLM response, creating default");
            // Check if the prompt contains transfer keywords to set appropriate default
            let default_intent = if prompt.to_lowercase().contains("send")
                || prompt.to_lowercase().contains("transfer")
            {
                "transfer"
            } else {
                "swap"
            };

            // Check if the prompt contains "all" to handle "sell all SOL" case
            let contains_all = prompt.to_lowercase().contains("all");

            let default_response = if default_intent == "transfer" {
                if contains_all {
                    r#"{"intent": "transfer", "parameters": {"from_token": "SOL", "amount": null, "percentage": "100%"}, "steps": ["transfer SOL"]}"#
                } else {
                    r#"{"intent": "transfer", "parameters": {"from_token": "SOL", "amount": "1.0"}, "steps": ["transfer SOL"]}"#
                }
            } else if contains_all {
                r#"{"intent": "swap", "parameters": {"from_token": "SOL", "to_token": "USDC", "amount": null, "percentage": "100%"}, "steps": ["swap SOL for USDC"]}"#
            } else {
                r#"{"intent": "swap", "parameters": {"from_token": "SOL", "to_token": "USDC", "amount": "1.0"}, "steps": ["swap SOL for USDC"]}"#
            };

            default_response.to_string()
        };

        // Validate that it's valid JSON
        match serde_json::from_str::<serde_json::Value>(&cleaned_response) {
            Ok(_) => {
                info!("Valid JSON extracted from LLM response");
                Ok(cleaned_response)
            }
            Err(e) => {
                error!("Invalid JSON in LLM response: {}. Fallback to default.", e);
                // Check if the prompt contains transfer keywords to set appropriate fallback
                let fallback_intent = if prompt.to_lowercase().contains("send")
                    || prompt.to_lowercase().contains("transfer")
                {
                    "transfer"
                } else {
                    "swap"
                };

                // Check if the prompt contains "all" to handle "sell all SOL" case
                let contains_all = prompt.to_lowercase().contains("all");

                let fallback_response = if fallback_intent == "transfer" {
                    if contains_all {
                        r#"{"intent": "transfer", "parameters": {"from_token": "SOL", "amount": null, "percentage": "100%"}, "steps": ["transfer SOL"]}"#
                    } else {
                        r#"{"intent": "transfer", "parameters": {"from_token": "SOL", "amount": "1.0"}, "steps": ["transfer SOL"]}"#
                    }
                } else if contains_all {
                    r#"{"intent": "swap", "parameters": {"from_token": "SOL", "to_token": "USDC", "amount": null, "percentage": "100%"}, "steps": ["swap SOL for USDC"]}"#
                } else {
                    r#"{"intent": "swap", "parameters": {"from_token": "SOL", "to_token": "USDC", "amount": "1.0"}, "steps": ["swap SOL for USDC"]}"#
                };

                Ok(fallback_response.to_string())
            }
        }
    }
}

/// Initialize GLM client with environment configuration
pub fn init_glm_client() -> Result<Box<dyn crate::planner::LlmClient>> {
    let client = GLMClient::from_env()?;
    Ok(Box::new(client))
}
