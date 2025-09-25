use crate::agent::{Agent, AgentAction, AgentObservation, LlmResponse};
use anyhow::{Context, Result};
use async_trait::async_trait;
use regex::Regex;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
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
    #[instrument(skip(self, prompt, observation), name = "agent.get_action")]
    async fn get_action(
        &mut self,
        prompt: &str,
        observation: &AgentObservation,
    ) -> Result<AgentAction> {
        // 1. Define the generation prompt, which provides static instructions to the LLM.
        const GENERATION_PROMPT: &str = "Your task is to generate a raw Solana instruction in JSON format based on the user's request and the provided on-chain context. Your response must be a JSON object with `program_id`, `accounts`, and `data` keys.";

        // 2. Extract account placeholders from the user prompt to identify relevant accounts.
        let re = Regex::new(r"\(([A-Z_0-9]+)\)")
            .context("Failed to compile regex for placeholder extraction")?;
        let relevant_placeholders: Vec<String> = re
            .captures_iter(prompt)
            .map(|cap| cap[1].to_string())
            .collect();

        // 3. Filter the full on-chain state to create a minimal context for the LLM.
        //    This includes only the accounts mentioned in the prompt and only the fields
        //    necessary for generating the instruction.
        let mut filtered_account_states = HashMap::new();
        let mut filtered_key_map = HashMap::new();

        for placeholder in &relevant_placeholders {
            if let Some(pubkey) = observation.key_map.get(placeholder) {
                filtered_key_map.insert(placeholder.clone(), pubkey.clone());
            }

            if let Some(state) = observation.account_states.get(placeholder) {
                let mut minimal_state = serde_json::Map::new();
                // Include lamports for any account.
                if let Some(lamports) = state.get("lamports") {
                    minimal_state.insert("lamports".to_string(), lamports.clone());
                }
                // For token accounts, parse and include the token data.
                if let Some(data) = state.get("data") {
                    if let Some(data_str) = data.as_str() {
                        if let Ok(token_data) = serde_json::from_str::<Value>(data_str) {
                            minimal_state.insert("token_data".to_string(), token_data);
                        }
                    }
                }
                filtered_account_states.insert(placeholder.clone(), Value::Object(minimal_state));
            }
        }

        // 4. Serialize the minimal context to YAML to create the context prompt.
        let context_yaml = serde_yaml::to_string(&json!({
            "account_states": filtered_account_states,
            "key_map": filtered_key_map,
        }))
        .context("Failed to serialize minimal context to YAML")?;

        let context_prompt = format!("---\n\nCURRENT ON-CHAIN CONTEXT:\n{context_yaml}\n\n---");

        // 5. Create the final JSON payload for the API.
        let request_payload = json!({
            "generation_prompt": GENERATION_PROMPT,
            "context_prompt": context_prompt,
            "prompt": prompt,
        });

        // 6. Log the raw request for debugging.
        println!(
            "[LlmAgent] Sending raw request to LLM:\n{}",
            serde_json::to_string_pretty(&request_payload)?
        );

        // 4. Send the request to the LLM API.
        let response = self
            .client
            .post(&self.api_url)
            .bearer_auth(&self.api_key)
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

        println!("[LlmAgent] Received successful response from LLM.");

        // 6. Deserialize the response and extract the raw instruction.
        let llm_response: LlmResponse = response
            .json()
            .await
            .context("Failed to deserialize the LLM API response")?;

        // 7. Convert the raw instruction into a native `AgentAction` and return it.
        let action: AgentAction = llm_response.result.text.try_into()?;

        println!(
            "[LlmAgent] Successfully parsed instruction for program: {}",
            action.0.program_id
        );
        tracing::info!(program_id = %action.0.program_id, "LLM generated a raw instruction");

        Ok(action)
    }
}
