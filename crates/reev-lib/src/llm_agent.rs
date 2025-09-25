use crate::agent::{Agent, AgentAction, AgentObservation, LlmResponse};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tracing::instrument;

/// An agent that uses a large language model to generate raw Solana instructions.
pub struct LlmAgent {
    client: Client,
    api_url: String,
    api_key: String,
}

impl LlmAgent {
    /// Creates a new `LlmAgent`.
    ///
    /// It initializes a `reqwest` client for making API calls.
    /// API configuration is loaded from environment variables.
    pub fn new() -> Result<Self> {
        println!("[LlmAgent] Initializing...");

        // Load API URL from environment variables
        let api_url = match std::env::var("LLM_API_URL") {
            Ok(url) => {
                println!("[LlmAgent] Using LLM_API_URL from environment.");
                url
            }
            Err(_) => {
                let default_url = "http://localhost:9090/gen/tx".to_string();
                println!("[LlmAgent] LLM_API_URL not set, using default: {default_url}");
                default_url
            }
        };

        // Load API key from environment variables
        let api_key = match std::env::var("LLM_API_KEY") {
            Ok(key) => {
                println!("[LlmAgent] Using LLM_API_KEY from environment.");
                key
            }
            Err(_) => {
                println!("[LlmAgent] WARNING: LLM_API_KEY environment variable not set.");
                "NONE".to_string()
            }
        };

        Ok(Self {
            client: Client::new(),
            api_url,
            api_key,
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
            "Based on the following observation, what is the next Solana instruction to execute? Provide the response as a JSON object with `program_id`, `accounts`, and `data` keys.\n\nObservation:\n{}",
            serde_json::to_string_pretty(observation)?
        );
        println!("[LlmAgent] Generating prompt...");
        tracing::debug!(prompt = %prompt, "Generated LLM prompt");

        // 2. Send the prompt to the LLM API.
        println!("[LlmAgent] Sending prompt to LLM at {}...", self.api_url);

        let response = self
            .client
            .post(&self.api_url)
            .bearer_auth(&self.api_key)
            // The user's server code expects a `Json(payload)` of `GenTextRequest`,
            // which likely contains a `prompt` field.
            .json(&json!({ "prompt": prompt }))
            .send()
            .await
            .context("Failed to send request to LLM API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM API request failed with status {status}: {error_body}");
        }

        println!("[LlmAgent] Received successful response from LLM.");

        // 3. Deserialize the entire third-party response using the structs defined in `agent.rs`.
        // This now correctly expects the `{"result": {"text": {...}}}` structure.
        let llm_response = response
            .json::<LlmResponse>()
            .await
            .context("Failed to deserialize the third-party LLM API response")?;

        // 4. Extract the inner instruction and convert it into a native AgentAction.
        // The `TryFrom` implementation handles the complex parsing and decoding (e.g., base58 for data).
        let action: AgentAction = llm_response.result.text.try_into()?;

        // 5. Return the parsed action.
        println!(
            "[LlmAgent] Successfully parsed instruction for program: {}",
            action.0.program_id
        );
        tracing::info!(program_id = %action.0.program_id, "LLM generated a raw instruction");

        Ok(action)
    }
}
