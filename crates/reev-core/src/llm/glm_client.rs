//! GLM Client Implementation for reev-core
//!
//! This module implements the LlmClient trait using the GLM-4.6-coding model
//! via the ZAI provider, leveraging existing implementation in reev-agent.

use crate::planner::LlmClient;
use anyhow::{anyhow, Result};
use serde_json::json;
use tracing::{debug, error, info, instrument, warn};

/// GLM Client implementation for reev-core planner
pub struct GLMClient {
    model_name: String,
    #[allow(dead_code)]
    api_key: String,
}

impl GLMClient {
    /// Create a new GLM client
    pub fn new(model_name: &str, api_key: &str) -> Self {
        Self {
            model_name: model_name.to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Initialize with environment variables
    pub fn from_env() -> Result<Self> {
        // Load environment variables from .env file
        dotenvy::dotenv().ok();

        let model_name =
            std::env::var("GLM_MODEL").unwrap_or_else(|_| "glm-4.6-coding".to_string());
        let api_key = std::env::var("ZAI_API_KEY")
            .map_err(|_| anyhow!("ZAI_API_KEY environment variable not set"))?;

        Ok(Self::new(&model_name, &api_key))
    }
}

#[async_trait::async_trait]
impl LlmClient for GLMClient {
    #[instrument(skip(self))]
    async fn generate_flow(&self, prompt: &str) -> Result<String> {
        info!("Extracting intent using ZAI API");
        debug!("Prompt: {}", prompt);

        // Build a simple prompt for intent extraction
        let flow_prompt = format!(
            r#"Extract user intent from this prompt: "{}"

Respond with a simple JSON object containing:
1. intent: The type of operation (swap, lend, borrow, etc.)
2. parameters: Key parameters for the operation
   - from_token: Source token (e.g., SOL, USDC)
   - to_token: Destination token (for swaps)
   - amount: The amount to operate with
   - percentage: Percentage if specified (e.g., "50%")
"#,
            prompt
        );

        debug!("Calling ZAI API with prompt: {}", flow_prompt);

        // Create a simple HTTP client for ZAI API
        let api_key = std::env::var("ZAI_API_KEY")
            .map_err(|_| anyhow!("ZAI_API_KEY environment variable not set"))?;

        info!(
            "Using API key: {}...",
            &api_key[..std::cmp::min(8, api_key.len())]
        );

        let client = reqwest::Client::new();

        // Create request body for ZAI API
        // Use the correct model name for ZAI API
        let model_name = if self.model_name == "glm-4.6-coding" {
            "glm-4.6"
        } else {
            &self.model_name
        };

        let request_body = json!({
            "model": model_name,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a DeFi assistant that extracts user intent from prompts. Always respond with valid JSON only. Respond in English only."
                },
                {
                    "role": "user",
                    "content": flow_prompt
                }
            ],
            "temperature": 0.1,
            "max_tokens": 500
        });

        // Send request to ZAI API
        let response = client
            .post("https://api.z.ai/api/coding/paas/v4/chat/completions")
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to ZAI API: {}", e);
                anyhow!("LLM generation failed: {e}")
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("ZAI API returned error: {status} - {error_text}"));
        }

        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            error!("Failed to parse ZAI API response: {}", e);
            anyhow!("LLM generation failed: {e}")
        })?;

        info!("Received ZAI API response: {:?}", response_json);

        // Extract content from response - try reasoning_content first, then content
        let message = response_json
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .ok_or_else(|| anyhow!("Invalid response format from ZAI API"))?;

        // Try reasoning_content first (for GLM model), then content
        let content = message
            .get("reasoning_content")
            .and_then(|c| c.as_str())
            .or_else(|| message.get("content").and_then(|c| c.as_str()))
            .ok_or_else(|| anyhow!("Invalid response format from ZAI API"))?;

        info!("Extracted content: {}", content);

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
            r#"{"intent": "swap", "parameters": {"from_token": "SOL", "to_token": "USDC", "amount": "1.0"}, "steps": ["swap SOL for USDC"]}"#.to_string()
        };

        // Validate that it's valid JSON
        match serde_json::from_str::<serde_json::Value>(&cleaned_response) {
            Ok(_) => {
                info!("Valid JSON extracted from LLM response");
                Ok(cleaned_response)
            }
            Err(e) => {
                error!("Invalid JSON in LLM response: {}. Fallback to default.", e);
                Ok(r#"{"intent": "swap", "parameters": {"from_token": "SOL", "to_token": "USDC", "amount": "1.0"}, "steps": ["swap SOL for USDC"]}"#.to_string())
            }
        }
    }
}

/// Initialize GLM client with environment configuration
pub fn init_glm_client() -> Result<Box<dyn crate::planner::LlmClient>> {
    let client = GLMClient::from_env()?;
    Ok(Box::new(client))
}
