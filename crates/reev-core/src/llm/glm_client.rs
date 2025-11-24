//! GLM Client Implementation for reev-core
//!
//! This module implements the LlmClient trait using the GLM-4.6-coding model
//! via the ZAI provider, leveraging existing implementation in reev-agent.

use crate::llm::prompt_templates::FlowPromptTemplate;
use crate::planner::LlmClient;
use anyhow::{anyhow, Result};
use serde_json::json;
use tracing::{debug, error, info, instrument};

/// GLM Client implementation for reev-core planner
pub struct GLMClient {
    model_name: String,
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

        // Build structured prompt for intent extraction
        let flow_prompt = FlowPromptTemplate::build_flow_prompt(prompt);

        debug!("Calling ZAI API with structured prompt");

        // Create a simple HTTP client for ZAI API
        let api_key = std::env::var("ZAI_API_KEY")
            .map_err(|_| anyhow!("ZAI_API_KEY environment variable not set"))?;

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
                    "content": "You are a DeFi assistant that extracts user intent and parameters from prompts."
                },
                {
                    "role": "user",
                    "content": flow_prompt
                }
            ],
            "temperature": 0.1,
            "max_tokens": 1000
        });

        // Send request to ZAI API
        let response = client
            .post("https://api.z.ai/api/coding/paas/v4/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
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
            return Err(anyhow!(
                "ZAI API returned error: {} - {}",
                status,
                error_text
            ));
        }

        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            error!("Failed to parse ZAI API response: {}", e);
            anyhow!("LLM generation failed: {e}")
        })?;

        debug!("Received ZAI response: {:?}", response_json);

        // Extract content from response
        let content = response_json
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .ok_or_else(|| anyhow!("Invalid response format from ZAI API"))?;

        debug!("Extracted content: {}", content);

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
            content.to_string()
        };

        Ok(cleaned_response)
    }
}

/// Initialize GLM client with environment configuration
pub fn init_glm_client() -> Result<Box<dyn crate::planner::LlmClient>> {
    let client = GLMClient::from_env()?;
    Ok(Box::new(client))
}
