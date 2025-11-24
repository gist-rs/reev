//! Mock LLM Client for Tests
//!
//! This module provides a mock implementation of LlmClient trait
//! for testing purposes, avoiding the need for API keys and network calls.

use anyhow::Result;
use reev_core::planner::LlmClient;
// serde_json::json is not used in this file

/// Mock LLM client for testing
pub struct MockLLMClient;

impl MockLLMClient {
    /// Create a new mock LLM client
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockLLMClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl LlmClient for MockLLMClient {
    async fn generate_flow(&self, prompt: &str) -> Result<String> {
        // Generate a mock YML flow based on the prompt
        let flow_id = uuid::Uuid::new_v4();

        let yml_response = if prompt.to_lowercase().contains("swap") {
            format!(
                r#"flow_id: {flow_id}
user_prompt: "{prompt}"
subject_wallet_info:
  - pubkey: "USER_WALLET_PUBKEY"
    lamports: 4000000000
    tokens:
      - mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        amount: 20000000
steps:
  - step_id: "1"
    prompt: "{prompt}"
    context: "Converting SOL to USDC for better yield"
    critical: true
    estimated_time_seconds: 5
    expected_tool_calls:
      - tool_name: "JupiterSwap"
        critical: true
ground_truth:
  final_state_assertions:
    - type: "SolBalanceChange"
      pubkey: "USER_WALLET_PUBKEY"
      expected_change_gte: -200500000
      error_tolerance: 0.01
  expected_tool_calls:
    - tool_name: "JupiterSwap"
      critical: true"#
            )
        } else {
            // Default flow for unrecognized prompts
            format!(
                r#"flow_id: {flow_id}
user_prompt: "{prompt}"
subject_wallet_info:
  - pubkey: "USER_WALLET_PUBKEY"
    lamports: 4000000000
    tokens:
      - mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        amount: 20000000
steps:
  - step_id: "1"
    prompt: "{prompt}"
    context: "Executing default action"
    critical: true
    estimated_time_seconds: 5
    expected_tool_calls:
      - tool_name: "GetAccountBalance"
        critical: false
ground_truth:
  final_state_assertions:
    - type: "SolBalanceChange"
      pubkey: "USER_WALLET_PUBKEY"
      expected_change_gte: 0
      error_tolerance: 0.01
  expected_tool_calls:
    - tool_name: "GetAccountBalance"
      critical: false"#
            )
        };

        Ok(yml_response)
    }
}

/// Initialize a mock LLM client for testing
pub fn init_mock_llm_client() -> Result<Box<dyn LlmClient>> {
    Ok(Box::new(MockLLMClient::new()))
}

/// Create a new mock LLM client for testing
pub fn new_mock_llm_client() -> Box<dyn LlmClient> {
    Box::new(MockLLMClient::new())
}
