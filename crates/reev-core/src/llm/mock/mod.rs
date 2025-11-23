//! Mock GLM Client for Tests
//!
//! This module provides a mock implementation of LlmClient trait
//! for testing purposes, avoiding the need for API keys and network calls.

use crate::planner::LlmClient;
use anyhow::Result;

/// Mock GLM client for testing
pub struct MockGLMClient;

impl MockGLMClient {
    /// Create a new mock GLM client
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockGLMClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl LlmClient for MockGLMClient {
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
    total_value_usd: 300.0
steps:
  - step_id: "1"
    prompt: "Swap SOL to USDC"
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
      critical: true
metadata:
  version: "1.0"
  created_at: "2023-01-01T00:00:00Z"
  category: "swap"
  complexity_score: 3
  tags: ["defi", "jupiter"]"#
            )
        } else if prompt.to_lowercase().contains("lend") {
            format!(
                r#"flow_id: {flow_id}
user_prompt: "{prompt}"
subject_wallet_info:
  - pubkey: "USER_WALLET_PUBKEY"
    lamports: 4000000000
    tokens:
      - mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        amount: 20000000
    total_value_usd: 300.0
steps:
  - step_id: "1"
    prompt: "Lend USDC to Jupiter"
    context: "Depositing USDC in Jupiter for yield"
    critical: true
    estimated_time_seconds: 5
    expected_tool_calls:
      - tool_name: "JupiterLendEarnDeposit"
        critical: true
ground_truth:
  final_state_assertions:
    - type: "TokenBalanceChange"
      pubkey: "USER_WALLET_PUBKEY"
      mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
      expected_change_gte: -20000000
      error_tolerance: 0.01
  expected_tool_calls:
    - tool_name: "JupiterLendEarnDeposit"
      critical: true
metadata:
  version: "1.0"
  created_at: "2023-01-01T00:00:00Z"
  category: "lend"
  complexity_score: 3
  tags: ["defi", "jupiter"]"#
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
    total_value_usd: 300.0
steps:
  - step_id: "1"
    prompt: "Default action"
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
      critical: false
metadata:
  version: "1.0"
  created_at: "2023-01-01T00:00:00Z"
  category: "default"
  complexity_score: 1
  tags: ["default"]"#
            )
        };

        Ok(yml_response)
    }
}

/// Initialize a mock GLM client for testing
pub fn init_mock_glm_client() -> Result<Box<dyn LlmClient>> {
    Ok(Box::new(MockGLMClient::new()))
}
