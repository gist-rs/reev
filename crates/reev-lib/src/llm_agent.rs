use crate::agent::{Agent, AgentAction, AgentObservation};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::instrument;

/// Represents the JSON payload sent to the LLM API.
#[derive(Serialize)]
struct LlmRequestPayload {
    model: String,
    prompt: String,
    // Add other parameters like temperature, max_tokens, etc., as needed.
}

/// Represents the expected JSON structure of a successful LLM API response.
/// This is a simplified example; a real implementation would need to handle
/// various response formats and potential errors.
#[derive(Deserialize)]
struct LlmResponse {
    tool_name: String,
    parameters: HashMap<String, Value>,
}

/// An agent that uses a large language model to decide the next action.
pub struct LlmAgent {
    client: Client,
    api_url: String,
    api_key: String,
}

impl LlmAgent {
    /// Creates a new `LlmAgent`.
    ///
    /// It initializes a `reqwest` client and a `tokio` runtime.
    /// API configuration is loaded from environment variables.
    pub fn new() -> Result<Self> {
        // In a real application, the API key should be loaded securely,
        // for example, from environment variables or a secrets management service.
        let api_key =
            std::env::var("LLM_API_KEY").unwrap_or_else(|_| "YOUR_API_KEY_HERE".to_string());
        let api_url = std::env::var("LLM_API_URL")
            .unwrap_or_else(|_| "https://api.example.com/v1/chat/completions".to_string());

        if api_key == "YOUR_API_KEY_HERE" {
            println!("[LlmAgent] WARNING: Using a placeholder API key. Please set the LLM_API_KEY environment variable.");
        }

        Ok(Self {
            client: Client::new(),
            api_url,
            api_key,
        })
    }

    /// Asynchronously calls the LLM API with a given prompt.
    async fn call_llm_api(&self, prompt: String) -> Result<AgentAction> {
        let payload = LlmRequestPayload {
            model: "gpt-4-turbo".to_string(), // This can also be made configurable
            prompt,
        };

        let response = self
            .client
            .post(&self.api_url)
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .context("Failed to send request to LLM API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Could not retrieve error body".to_string());
            anyhow::bail!("LLM API request failed with status {status}: {error_body}");
        }

        let llm_response = response
            .json::<LlmResponse>()
            .await
            .context("Failed to deserialize LLM API response")?;

        Ok(AgentAction {
            tool_name: llm_response.tool_name,
            parameters: llm_response.parameters,
        })
    }
}

#[async_trait]
impl Agent for LlmAgent {
    #[instrument(skip(self, observation), name = "agent.get_action")]
    async fn get_action(&mut self, observation: &AgentObservation) -> Result<AgentAction> {
        // 1. Serialize the observation into a prompt. A real implementation would
        //    include conversation history and more sophisticated prompt engineering.
        let prompt = format!(
            "Based on the following observation, what is the next tool to call? Provide the response as a JSON object with `tool_name` and `parameters` keys.\n\nObservation:\n{}",
            serde_json::to_string_pretty(observation)?
        );
        println!("[LlmAgent] Generating prompt...");
        tracing::debug!(prompt = %prompt, "Generated LLM prompt");

        // 2. Send the prompt to the LLM API.
        println!("[LlmAgent] Sending prompt to LLM...");
        let action = self.call_llm_api(prompt).await?;

        // 3. Return the parsed action.
        println!("[LlmAgent] Parsed action: {}", action.tool_name);
        tracing::info!(tool_name = %action.tool_name, "LLM decided on action");

        Ok(action)
    }
}
