use std::str::FromStr;

use crate::agent::{Agent, AgentAction, AgentObservation, LlmResponse};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tracing::{info, instrument};

/// An agent that uses a large language model to generate raw Solana instructions.
pub struct LlmAgent {
    client: Client,
    api_url: String,
    api_key: Option<String>,
    model_name: String,
}

impl LlmAgent {
    /// Creates a new `LlmAgent`.
    ///
    /// It initializes a `reqwest` client for making API calls.
    /// API configuration is loaded from environment variables.
    pub fn new(agent_name: &str) -> Result<Self> {
        info!("[LlmAgent] Initializing agent: '{agent_name}'");

        // Load base API URL from environment variables, falling back to a default.
        let base_url = std::env::var("LLM_API_URL")
            .unwrap_or_else(|_| "http://localhost:9090/gen/tx".to_string());
        info!("[LlmAgent] Using base URL: {base_url}");

        // Append `?mock=true` if the deterministic agent is selected.
        let api_url = if agent_name == "deterministic" {
            format!("{base_url}?mock=true")
        } else {
            base_url
        };

        // Map the public-facing agent name 'local' to the internal model name 'local-model'
        // that the reev-agent expects for locally-served models. Other names are passed through.
        let model_name = if agent_name == "local" {
            "local-model".to_string()
        } else {
            agent_name.to_string()
        };

        info!("[LlmAgent] Final API URL for agent '{agent_name}': {api_url}");
        info!("[LlmAgent] Model name being sent in payload: '{model_name}'");

        // Load API key from environment variables if it exists.
        let api_key = match std::env::var("LLM_API_KEY") {
            Ok(key) if !key.is_empty() => {
                info!("[LlmAgent] Using LLM_API_KEY from environment.");
                Some(key)
            }
            _ => {
                info!("[LlmAgent] WARNING: LLM_API_KEY environment variable not set or is empty.");
                None
            }
        };

        Ok(Self {
            client: Client::new(),
            api_url,
            api_key,
            model_name,
        })
    }
}

#[async_trait]
impl Agent for LlmAgent {
    #[instrument(skip(self, prompt, observation), name = "agent.get_action")]
    async fn get_action(
        &mut self,
        id: &str,
        prompt: &str,
        observation: &AgentObservation,
        fee_payer: Option<&String>,
        skip_instruction_validation: Option<bool>,
    ) -> Result<Vec<AgentAction>> {
        // 1. Serialize the full context to YAML to create the context prompt.
        let context_yaml = serde_yaml::to_string(&json!({
            "fee_payer_placeholder": fee_payer,
            "account_states": observation.account_states,
            "key_map": observation.key_map,
        }))
        .context("Failed to serialize full context to YAML")?;

        let context_prompt = format!("---\n\nCURRENT ON-CHAIN CONTEXT:\n{context_yaml}\n\n---");

        // 2. Create the final JSON payload for the API.
        let request_payload = json!({
            "id": id,
            "context_prompt": context_prompt,
            "prompt": prompt,
            "model_name": self.model_name,
        });

        // 3. Log the raw request for debugging.
        info!(
            "[LlmAgent] Sending raw request to LLM:\n{}",
            serde_json::to_string_pretty(&request_payload)?
        );

        // 4. Send the request to the LLM API.
        let mut request_builder = self.client.post(&self.api_url);
        if let Some(api_key) = &self.api_key {
            request_builder = request_builder.header("X-API-Key", api_key);
        }
        let response = request_builder
            .json(&request_payload)
            .send()
            .await
            .context("Failed to send request to LLM API")?;

        // 5. Handle API errors.
        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM API request failed with status {status}: {error_body}");
        }

        info!("[LlmAgent] Received successful response from LLM.");

        // 6. Check if we should skip instruction validation (API-based benchmark)
        if skip_instruction_validation.unwrap_or(false) {
            info!("[LlmAgent] Processing API-based benchmark response.");

            // For API benchmarks, we want to capture the response as successful tool execution
            // Create a mock action to indicate success for API benchmarks
            let mock_instruction = solana_sdk::instruction::Instruction {
                program_id: solana_sdk::pubkey::Pubkey::from_str(
                    "11111111111111111111111111111111",
                )?, // System program
                accounts: vec![],
                data: vec![1, 0, 0, 0], // Simple success indicator
            };
            return Ok(vec![AgentAction(mock_instruction)]);
        }

        // 7. Deserialize the response and extract the raw instructions.
        let llm_response: LlmResponse = response
            .json()
            .await
            .context("Failed to deserialize the LLM API response")?;

        // 8. Convert the raw instructions into a vector of native `AgentAction` and return it.
        let actions: Vec<AgentAction> = llm_response
            .result
            .text
            .into_iter()
            .map(|raw_ix| raw_ix.try_into())
            .collect::<Result<Vec<AgentAction>>>()?;

        info!(
            "[LlmAgent] Successfully parsed {} instruction(s).",
            actions.len()
        );
        tracing::info!(
            instruction_count = actions.len(),
            "LLM generated raw instruction(s)"
        );

        Ok(actions)
    }
}
