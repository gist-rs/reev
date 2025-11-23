//! Test utilities for planner module

use anyhow::Result;
use reev_core::planner::LlmClient;

#[cfg(test)]
pub struct MockLlmClient;

#[async_trait::async_trait]
impl LlmClient for MockLlmClient {
    async fn generate_flow(&self, _prompt: &str) -> Result<String> {
        Ok(r#"
flow_id: "test-flow-id"
user_prompt: "swap 1 SOL to USDC"
subject_wallet_info:
  pubkey: "11111111111111111111111111111112"
  lamports: 1000000000
  total_value_usd: 150.0
  tokens:
    - mint: "So11111111111111111111111111111111111111112"
      balance: 1000000000
      decimals: 9
      symbol: "SOL"
steps:
  - step_id: "swap"
    prompt: "swap 1 SOL to USDC"
    context: "Exchange 1 SOL for USDC"
    critical: true
ground_truth:
  final_state_assertions:
    - assertion_type: "SolBalanceChange"
      pubkey: "11111111111111111111111111111112"
      expected_change_gte: -1010000000
  error_tolerance: 0.01
metadata:
  category: "swap"
  complexity_score: 1
  tags: []
  version: "1.0"
created_at: "2023-01-01T00:00:00Z"
"#
        .to_string())
    }
}
