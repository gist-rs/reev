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
    api_key: Option<String>,
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

        // Load API key from environment variables if it exists.
        let api_key = match std::env::var("LLM_API_KEY") {
            Ok(key) if !key.is_empty() => {
                println!("[LlmAgent] Using LLM_API_KEY from environment.");
                Some(key)
            }
            _ => {
                println!(
                    "[LlmAgent] WARNING: LLM_API_KEY environment variable not set or is empty."
                );
                None
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
        fee_payer: Option<&String>,
    ) -> Result<AgentAction> {
        // 1. Define the generation prompt, which provides static instructions to the LLM.
        const GENERATION_PROMPT: &str = r#"Your task is to generate a raw Solana instruction in JSON format based on the user's request and the provided on-chain context. Your response must be a JSON object with `program_id`, `accounts`, and `data` keys. Each account in the `accounts` array must have `pubkey`, `is_signer`, and `is_writable` fields. The `data` field must be a valid base58 encoded string.

---

**SPECIAL INSTRUCTIONS FOR NATIVE SOL TRANSFERS**

If the user requests a native SOL transfer, you MUST use the Solana System Program (`11111111111111111111111111111111`). The instruction `data` for a System Program transfer has a very specific format:

1.  **Instruction Index (4 bytes):** The value `2` as a little-endian `u32`. This is always `[2, 0, 0, 0]`.
2.  **Lamports (8 bytes):** The amount of lamports to transfer as a little-endian `u64`.

**Example:** To send 0.1 SOL (which is 100,000,000 lamports):
- The lamports value `100000000` as a little-endian `u64` is `[0, 225, 245, 5, 0, 0, 0, 0]`.
- The full data byte array is `[2, 0, 0, 0, 0, 225, 245, 5, 0, 0, 0, 0]`.
- You must base58 encode this byte array to create the `data` string. For this specific example, the result is `2Z4dY1Wp2j`.

Your `data` field for a 0.1 SOL transfer must be exactly "2Z4dY1Wp2j"."#;

        // 2. Serialize the full context to YAML to create the context prompt.
        let context_yaml = serde_yaml::to_string(&json!({
            "fee_payer_placeholder": fee_payer,
            "account_states": observation.account_states,
            "key_map": observation.key_map,
        }))
        .context("Failed to serialize full context to YAML")?;

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

        // 7. Send the request to the LLM API.
        let mut request_builder = self.client.post(&self.api_url);
        if let Some(api_key) = &self.api_key {
            request_builder = request_builder.header("X-API-Key", api_key);
        }
        let response = request_builder
            .json(&request_payload)
            .send()
            .await
            .context("Failed to send request to LLM API")?;

        // 8. Handle API errors.
        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM API request failed with status {status}: {error_body}");
        }

        println!("[LlmAgent] Received successful response from LLM.");

        // 9. Deserialize the response and extract the raw instruction.
        let llm_response: LlmResponse = response
            .json()
            .await
            .context("Failed to deserialize the LLM API response")?;

        // 10. Convert the raw instruction into a native `AgentAction` and return it.
        let action: AgentAction = llm_response.result.text.try_into()?;

        println!(
            "[LlmAgent] Successfully parsed instruction for program: {}",
            action.0.program_id
        );
        tracing::info!(program_id = %action.0.program_id, "LLM generated a raw instruction");

        Ok(action)
    }
}
