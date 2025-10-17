use std::path::PathBuf;
use std::str::FromStr;

use crate::agent::{Agent, AgentAction, AgentObservation, LlmResponse, RawInstruction};
use crate::flow::{FlowLogger, LlmRequestContent, ToolCallContent, ToolResultStatus};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tracing::{debug, info, instrument, warn};

/// An agent that uses a large language model to generate raw Solana instructions.
pub struct LlmAgent {
    client: Client,
    api_url: String,
    api_key: Option<String>,
    model_name: String,
    pub flow_logger: Option<FlowLogger>,
    current_depth: u32,
    is_glm: bool,
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

        // Check for GLM environment variables
        let glm_api_key = std::env::var("GLM_API_KEY").ok();
        let glm_api_url = std::env::var("GLM_API_URL").ok();

        let (api_url, api_key, model_name, is_glm) = match (glm_api_key, glm_api_url) {
            (Some(key), Some(url)) if !key.is_empty() && !url.is_empty() => {
                info!("[LlmAgent] Using GLM 4.6 API with OpenAI compatibility");
                let final_url = if agent_name == "deterministic" {
                    format!("{url}?mock=true")
                } else {
                    url
                };
                (final_url, Some(key), "glm-4.6".to_string(), true)
            }
            (Some(_), None) => {
                anyhow::bail!("GLM_API_KEY is set but GLM_API_URL is missing. Please set both GLM_API_KEY and GLM_API_URL for GLM 4.6 support.");
            }
            (None, Some(_)) => {
                anyhow::bail!("GLM_API_URL is set but GLM_API_KEY is missing. Please set both GLM_API_KEY and GLM_API_URL for GLM 4.6 support.");
            }
            _ => {
                info!("[LlmAgent] GLM environment variables not found, using default LLM configuration");

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
                // Add support for "glm" alias to "glm-4.6"
                let model_name = match agent_name {
                    "glm" => "glm-4.6",
                    _ => agent_name,
                }
                .to_string();

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

                (api_url, api_key, model_name, false)
            }
        };

        info!("[LlmAgent] Final API URL for agent '{agent_name}': {api_url}");
        info!("[LlmAgent] Model name being sent in payload: '{model_name}'");
        if is_glm {
            info!("[LlmAgent] GLM 4.6 mode enabled with OpenAI-compatible API");
        }

        Ok(Self {
            client: Client::new(),
            api_url,
            api_key,
            model_name,
            flow_logger,
            current_depth: 0,
            is_glm,
        })
    }

    /// Get the model name for this agent
    pub fn model_name(&self) -> &str {
        &self.model_name
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
        if self.flow_logger.is_none() && crate::flow::is_flow_logging_enabled() {
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
        let request_payload = if self.is_glm {
            // GLM uses OpenAI-compatible format
            let full_prompt = format!("{}\n\n{}\n\n{}", context_prompt, prompt,
                "Generate Solana transactions as JSON array in the response. Each transaction should include program_id, accounts, and data fields.");

            json!({
                "model": self.model_name,
                "messages": [
                    {
                        "role": "user",
                        "content": full_prompt
                    }
                ],
                "temperature": 0.1,
                "max_tokens": 4000
            })
        } else {
            // Default reev API format
            json!({
                "id": id,
                "context_prompt": context_prompt,
                "prompt": prompt,
                "model_name": self.model_name,
            })
        };

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
            if self.is_glm {
                request_builder =
                    request_builder.header("Authorization", format!("Bearer {api_key}"));
            } else {
                request_builder = request_builder.header("X-API-Key", api_key);
            }
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

        let llm_response = if self.is_glm {
            // Parse OpenAI-compatible response from GLM
            let openai_response: serde_json::Value = serde_json::from_str(&llm_response_text)
                .context("Failed to deserialize GLM OpenAI-compatible response")?;

            info!(
                "[LlmAgent] Debug - Parsed GLM OpenAI response: {:?}",
                openai_response
            );

            // Extract content from OpenAI response format
            let content = openai_response
                .get("choices")
                .and_then(|choices| choices.get(0))
                .and_then(|choice| choice.get("message"))
                .and_then(|message| message.get("content"))
                .and_then(|content| content.as_str())
                .unwrap_or("");

            info!("[LlmAgent] Debug - GLM extracted content: {}", content);

            // Try to parse the content as JSON to extract transactions
            match serde_json::from_str::<serde_json::Value>(content) {
                Ok(json_content) => {
                    // Look for transactions array in the parsed content
                    if let Some(transactions) =
                        json_content.get("transactions").and_then(|t| t.as_array())
                    {
                        LlmResponse {
                            transactions: Some(
                                transactions
                                    .iter()
                                    .filter_map(|tx| serde_json::from_value(tx.clone()).ok())
                                    .collect(),
                            ),
                            result: None,
                            summary: json_content
                                .get("summary")
                                .and_then(|s| s.as_str())
                                .map(|s| s.to_string()),
                            signatures: None,
                            flows: None,
                        }
                    } else {
                        // Fallback: treat the entire content as a transactions array
                        if let Ok(transaction) =
                            serde_json::from_value::<RawInstruction>(json_content)
                        {
                            LlmResponse {
                                transactions: Some(vec![transaction]),
                                result: None,
                                summary: None,
                                signatures: None,
                                flows: None,
                            }
                        } else {
                            // If we can't parse as transaction, create empty response
                            LlmResponse {
                                transactions: None,
                                result: None,
                                summary: Some(content.to_string()),
                                signatures: None,
                                flows: None,
                            }
                        }
                    }
                }
                Err(_) => {
                    // If we can't parse as JSON, create a default response
                    warn!("[LlmAgent] Could not parse GLM response as JSON, using fallback");
                    LlmResponse {
                        transactions: None,
                        result: None,
                        summary: Some(content.to_string()),
                        signatures: None,
                        flows: None,
                    }
                }
            }
        } else {
            // Default reev API format
            serde_json::from_str(&llm_response_text)
                .context("Failed to deserialize the LLM API response")?
        };

        info!("[LlmAgent] Debug - Parsed LlmResponse: {:?}", llm_response);

        // 9. Handle both old and new response formats
        let actions: Vec<AgentAction> = if let Some(transactions) = llm_response.transactions {
            // New comprehensive format: direct transactions array
            if !transactions.is_empty() {
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
            } else {
                // Empty transactions array, try to extract from summary as fallback
                if let Some(summary) = &llm_response.summary {
                    info!(
                        "[LlmAgent] Transactions array empty, attempting to extract from summary"
                    );
                    match self.extract_transactions_from_summary(summary) {
                        Ok(extracted_actions) => {
                            if !extracted_actions.is_empty() {
                                info!(
                                    "[LlmAgent] Successfully extracted {} transactions from summary",
                                    extracted_actions.len()
                                );
                                return Ok(extracted_actions);
                            }
                        }
                        Err(e) => {
                            warn!(
                                "[LlmAgent] Failed to extract transactions from summary: {}",
                                e
                            );
                        }
                    }
                }
                info!("[LlmAgent] No transactions found in response");
                vec![]
            }
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
            // No instructions found, try summary as last resort
            if let Some(summary) = &llm_response.summary {
                info!("[LlmAgent] No transactions array, attempting to extract from summary");
                match self.extract_transactions_from_summary(summary) {
                    Ok(extracted_actions) => {
                        if !extracted_actions.is_empty() {
                            info!(
                                "[LlmAgent] Successfully extracted {} transactions from summary",
                                extracted_actions.len()
                            );
                            return Ok(extracted_actions);
                        }
                    }
                    Err(e) => {
                        warn!(
                            "[LlmAgent] Failed to extract transactions from summary: {}",
                            e
                        );
                    }
                }
            }
            info!("[LlmAgent] No instructions found in response");
            vec![]
        };

        debug!(
            "[LlmAgent] Successfully parsed {} instruction(s).",
            actions.len()
        );

        // 10. Log transaction signatures if available (for new format)
        if let Some(signatures) = llm_response.signatures {
            debug!("[LlmAgent] Transaction signatures: {:?}", signatures);
        }

        // 11. Log summary if available (for new format)
        if let Some(summary) = llm_response.summary {
            debug!("[LlmAgent] Agent summary: {}", summary);
        }

        // 12. Process flows from LlmResponse and log tool calls
        if let Some(flows) = llm_response.flows {
            debug!(
                "[LlmAgent] Processing {} tool calls from flows",
                flows.total_tool_calls
            );

            if let Some(flow_logger) = &mut self.flow_logger {
                for tool_call in &flows.tool_calls {
                    let tool_call_content = ToolCallContent {
                        tool_name: tool_call.tool_name.clone(),
                        tool_args: tool_call.tool_args.clone(),
                        execution_time_ms: tool_call.execution_time_ms,
                        result_status: match tool_call.result_status {
                            crate::agent::ToolResultStatus::Success => ToolResultStatus::Success,
                            crate::agent::ToolResultStatus::Error => ToolResultStatus::Error,
                            crate::agent::ToolResultStatus::Timeout => ToolResultStatus::Timeout,
                        },
                        result_data: tool_call.result_data.clone(),
                        error_message: tool_call.error_message.clone(),
                    };

                    // Log the tool call
                    flow_logger.log_tool_call(tool_call_content.clone(), tool_call.depth);

                    // Also log the tool result
                    flow_logger.log_tool_result(tool_call_content, tool_call.depth);
                }
            }
        }

        tracing::debug!(
            instruction_count = actions.len(),
            "LLM generated raw instruction(s)"
        );

        Ok(actions)
    }
}

impl LlmAgent {
    /// Extract transactions from a summary field that contains JSON-formatted transaction data
    fn extract_transactions_from_summary(&self, summary: &str) -> Result<Vec<AgentAction>> {
        use serde_json::Value;

        // Look for JSON blocks in the summary that contain transaction data
        // Pattern: ```json\n{...transactions...}\n```
        if let Some(json_start) = summary.find("```json") {
            let json_content = &summary[json_start + 7..];
            if let Some(json_end) = json_content.find("```") {
                let json_str = &json_content[..json_end];
                match serde_json::from_str::<Value>(json_str) {
                    Ok(parsed) => {
                        // Try to extract transactions array from the parsed JSON
                        if let Some(transactions) =
                            parsed.get("transactions").and_then(|t| t.as_array())
                        {
                            info!(
                                "[LlmAgent] Found {} transactions in summary JSON",
                                transactions.len()
                            );
                            let mut actions = Vec::new();
                            for transaction in transactions {
                                match serde_json::from_value::<RawInstruction>(transaction.clone())
                                {
                                    Ok(raw_ix) => match raw_ix.try_into() {
                                        Ok(action) => actions.push(action),
                                        Err(e) => {
                                            warn!(
                                                    "[LlmAgent] Failed to convert transaction to action: {}",
                                                    e
                                                );
                                        }
                                    },
                                    Err(e) => {
                                        warn!(
                                            "[LlmAgent] Failed to parse transaction from summary: {}",
                                            e
                                        );
                                    }
                                }
                            }
                            return Ok(actions);
                        }

                        // Also check for direct_transactions from tool responses
                        if let Some(direct_transactions) =
                            parsed.get("direct_transactions").and_then(|t| t.as_array())
                        {
                            info!(
                                "[LlmAgent] Found {} direct transactions from tool response",
                                direct_transactions.len()
                            );
                            let mut actions = Vec::new();
                            for transaction in direct_transactions {
                                match serde_json::from_value::<RawInstruction>(transaction.clone())
                                {
                                    Ok(raw_ix) => match raw_ix.try_into() {
                                        Ok(action) => actions.push(action),
                                        Err(e) => {
                                            warn!(
                                                    "[LlmAgent] Failed to convert direct transaction to action: {}",
                                                    e
                                                );
                                        }
                                    },
                                    Err(e) => {
                                        warn!(
                                            "[LlmAgent] Failed to parse direct transaction from tool: {}",
                                            e
                                        );
                                    }
                                }
                            }
                            return Ok(actions);
                        }
                    }
                    Err(e) => {
                        warn!("[LlmAgent] Failed to parse JSON from summary: {}", e);
                    }
                }
            }
        }

        // Fallback: try to find any JSON object that looks like a transaction
        // Look for patterns like "program_id" in the summary
        if summary.contains("program_id") && summary.contains("accounts") {
            // Try to extract individual transaction objects
            let mut actions = Vec::new();

            // Simple regex-like approach to find transaction-like JSON objects
            let lines: Vec<&str> = summary.lines().collect();
            let mut in_transaction = false;
            let mut transaction_lines = Vec::new();

            for line in lines {
                if line.trim().contains("\"program_id\"") {
                    in_transaction = true;
                    transaction_lines.clear();
                }

                if in_transaction {
                    transaction_lines.push(line);

                    // End of transaction object (simplified detection)
                    if line.trim().ends_with('}') && line.trim().starts_with('}') {
                        let transaction_json = transaction_lines.join("\n");
                        match serde_json::from_str::<RawInstruction>(&transaction_json) {
                            Ok(raw_ix) => match raw_ix.try_into() {
                                Ok(action) => actions.push(action),
                                Err(e) => {
                                    warn!(
                                        "[LlmAgent] Failed to convert transaction to action: {}",
                                        e
                                    );
                                }
                            },
                            Err(_) => {
                                // Try parsing as part of a larger object
                                // This is a fallback for malformed JSON
                            }
                        }
                        in_transaction = false;
                        transaction_lines.clear();
                    }
                }
            }

            if !actions.is_empty() {
                info!(
                    "[LlmAgent] Extracted {} transactions using fallback method",
                    actions.len()
                );
                return Ok(actions);
            }
        }

        Ok(Vec::new())
    }
}
