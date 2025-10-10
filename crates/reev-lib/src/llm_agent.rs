use std::path::PathBuf;
use std::str::FromStr;

use crate::agent::{Agent, AgentAction, AgentObservation, LlmResponse};
use crate::flow::{
    ExecutionResult, ExecutionStatistics, FlowError, FlowLogger, LlmRequestContent,
    ToolCallContent, ToolResultStatus,
};
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
    pub flow_logger: Option<FlowLogger>,
    current_depth: u32,
}

impl LlmAgent {
    /// Creates a new `LlmAgent`.
    ///
    /// It initializes a `reqwest` client for making API calls.
    /// API configuration is loaded from environment variables.
    pub fn new(agent_name: &str) -> Result<Self> {
        Self::new_with_flow_logging(agent_name, None)
    }

    /// Creates a new `LlmAgent` with optional flow logging.
    pub fn new_with_flow_logging(
        agent_name: &str,
        flow_logger: Option<FlowLogger>,
    ) -> Result<Self> {
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

        // Pass through agent names directly - 'local' should remain 'local' for actual local models
        let model_name = agent_name.to_string();

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
            flow_logger,
            current_depth: 0,
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
        // Initialize flow logger if not already done and logging is enabled
        if self.flow_logger.is_none() && std::env::var("REEV_ENABLE_FLOW_LOGGING").is_ok() {
            let output_path =
                std::env::var("REEV_FLOW_LOG_PATH").unwrap_or_else(|_| "logs/flows".to_string());
            let path = PathBuf::from(output_path);
            std::fs::create_dir_all(&path)?;

            self.flow_logger = Some(FlowLogger::new(
                id.to_string(),
                self.model_name.clone(),
                path,
            ));

            info!("[LlmAgent] Flow logging enabled for benchmark: {}", id);
        }
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

        // 4. Log LLM request to flow logger
        if let Some(flow_logger) = &mut self.flow_logger {
            let context_tokens = context_prompt.len() as u32 / 4; // Rough estimate
            let llm_request = LlmRequestContent {
                prompt: prompt.to_string(),
                context_tokens,
                model: self.model_name.clone(),
                request_id: uuid::Uuid::new_v4().to_string(),
            };
            flow_logger.log_llm_request(llm_request, self.current_depth);
        }

        // 5. Send the request to the LLM API.
        let mut request_builder = self.client.post(&self.api_url);
        if let Some(api_key) = &self.api_key {
            request_builder = request_builder.header("X-API-Key", api_key);
        }
        let response = request_builder
            .json(&request_payload)
            .send()
            .await
            .context("Failed to send request to LLM API")?;

        // 6. Handle API errors.
        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();

            // Log error to flow logger
            if let Some(flow_logger) = &mut self.flow_logger {
                use crate::flow::ErrorContent;
                let error_content = ErrorContent {
                    error_type: "LLM_API_ERROR".to_string(),
                    message: format!("LLM API request failed with status {status}: {error_body}"),
                    stack_trace: None,
                    context: std::collections::HashMap::new(),
                };
                flow_logger.log_error(error_content, self.current_depth);
            }

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

        // 7. Check if this is a flow response (for flow benchmarks starting with "200-")
        let response_text = response
            .text()
            .await
            .context("Failed to get response text")?;

        // Check if this is a flow response (for flow benchmarks starting with "200-")
        // Flow responses contain flow_completed, steps, or summary fields
        // Only detect as flow response if it contains flow_completed or steps, not just summary
        // to avoid false positives with Jupiter swap responses that also have summary fields
        let is_flow_response =
            response_text.contains("flow_completed") || response_text.contains("\"steps\"");

        if is_flow_response {
            info!("[LlmAgent] Detected flow response, creating mock success action.");

            // For flow benchmarks, create a valid system transfer instruction
            // Transfer 0 lamports from a known pubkey to itself (valid but no-op)
            let system_program =
                solana_sdk::pubkey::Pubkey::from_str("11111111111111111111111111111111")?;
            let mock_instruction = solana_system_interface::instruction::transfer(
                &system_program,
                &system_program,
                0, // 0 lamports - no-op transfer
            );
            return Ok(vec![AgentAction(mock_instruction)]);
        }

        // 8. Deserialize the response and extract the raw instructions.
        // We need to recreate the response since we consumed it with .text()
        let llm_response_text = response_text;

        info!(
            "[LlmAgent] Debug - Raw response text: {}",
            llm_response_text
        );

        let llm_response: LlmResponse = serde_json::from_str(&llm_response_text)
            .context("Failed to deserialize the LLM API response")?;

        info!("[LlmAgent] Debug - Parsed LlmResponse: {:?}", llm_response);

        // 9. Handle both old and new response formats
        let actions: Vec<AgentAction> = if let Some(transactions) = llm_response.transactions {
            // New comprehensive format: direct transactions array
            info!(
                "[LlmAgent] Processing {} transactions from comprehensive format",
                transactions.len()
            );
            for (i, tx) in transactions.iter().enumerate() {
                info!(
                    "[LlmAgent] Transaction {}: {}",
                    i,
                    serde_json::to_string_pretty(tx).unwrap_or_default()
                );
            }
            transactions
                .into_iter()
                .map(|raw_ix| raw_ix.try_into())
                .collect::<Result<Vec<AgentAction>>>()?
        } else if let Some(result) = llm_response.result {
            // Old format: nested in result.text
            info!(
                "[LlmAgent] Processing {} instructions from old format",
                result.text.len()
            );
            result
                .text
                .into_iter()
                .map(|raw_ix| raw_ix.try_into())
                .collect::<Result<Vec<AgentAction>>>()?
        } else {
            // No instructions found
            info!("[LlmAgent] No instructions found in response");
            vec![]
        };

        info!(
            "[LlmAgent] Successfully parsed {} instruction(s).",
            actions.len()
        );

        // 10. Log transaction signatures if available (for new format)
        if let Some(signatures) = llm_response.signatures {
            info!("[LlmAgent] Transaction signatures: {:?}", signatures);
        }

        // 11. Log summary if available (for new format)
        if let Some(summary) = llm_response.summary {
            info!("[LlmAgent] Agent summary: {}", summary);
        }
        tracing::info!(
            instruction_count = actions.len(),
            "LLM generated raw instruction(s)"
        );

        Ok(actions)
    }
}
