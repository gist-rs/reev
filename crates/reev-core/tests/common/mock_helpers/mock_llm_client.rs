//! Mock LLM Client for Tests
//!
//! This module provides a mock implementation of the LlmClient trait
//! for testing purposes, avoiding the need for actual LLM calls.

use async_trait::async_trait;
use reev_core::planner::LlmClient;
use serde_json::json;
use std::collections::HashMap;

/// Mock LLM client for testing
pub struct MockLLMClient {
    /// Whether to simulate success or failure
    simulate_success: bool,
    /// Predefined responses for specific prompts
    predefined_responses: HashMap<String, String>,
}

impl Default for MockLLMClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MockLLMClient {
    /// Create a new mock LLM client
    pub fn new() -> Self {
        Self {
            simulate_success: true,
            predefined_responses: HashMap::new(),
        }
    }

    /// Set whether to simulate success or failure
    pub fn with_success(mut self, success: bool) -> Self {
        self.simulate_success = success;
        self
    }

    /// Add a predefined response for a specific prompt
    pub fn with_response(mut self, prompt: &str, response: &str) -> Self {
        self.predefined_responses
            .insert(prompt.to_string(), response.to_string());
        self
    }

    /// Create a mock flow response for a swap operation
    fn create_swap_flow_response(&self, _prompt: &str) -> String {
        json!({
            "flow_id": uuid::Uuid::new_v4().to_string(),
            "user_prompt": _prompt,
            "subject_wallet_info": [{
                "pubkey": "USER_WALLET_PUBKEY",
                "lamports": 4000000000u32,
                "total_value_usd": 100.0,
                "tokens": [{
                    "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    "balance": 20000000
                }]
            }],
            "steps": [{
                "step_id": uuid::Uuid::new_v4().to_string(),
                "prompt": "swap SOL to USDC",
                "context": "Executing swap through Jupiter",
                "critical": true,
                "estimated_time_seconds": 30,
                "expected_tool_calls": [{
                    "tool_name": "JupiterSwap",
                    "critical": true
                }]
            }],
            "ground_truth": {
                "final_state_assertions": [{
                    "assertion_type": "SolBalanceChange",
                    "pubkey": "USER_WALLET_PUBKEY",
                    "expected_change_gte": -2000000000,
                    "error_tolerance": 0.01
                }],
                "expected_tool_calls": [{
                    "tool_name": "JupiterSwap",
                    "critical": true
                }]
            },
            "metadata": {
                "version": "1.0",
                "created_at": chrono::Utc::now().to_rfc3339()
            }
        })
        .to_string()
    }

    /// Create a mock flow response for a lend operation
    fn create_lend_flow_response(&self, _prompt: &str) -> String {
        json!({
            "flow_id": uuid::Uuid::new_v4().to_string(),
            "user_prompt": _prompt,
            "subject_wallet_info": [{
                "pubkey": "USER_WALLET_PUBKEY",
                "lamports": 4000000000u32,
                "total_value_usd": 100.0,
                "tokens": [{
                    "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    "balance": 20000000
                }]
            }],
            "steps": [{
                "step_id": uuid::Uuid::new_v4().to_string(),
                "prompt": "lend USDC to jupiter",
                "context": "Lending USDC through Jupiter",
                "critical": true,
                "estimated_time_seconds": 30,
                "expected_tool_calls": [{
                    "tool_name": "JupiterLendEarnDeposit",
                    "critical": true
                }]
            }],
            "ground_truth": {
                "final_state_assertions": [{
                    "assertion_type": "TokenBalanceChange",
                    "pubkey": "USER_WALLET_PUBKEY",
                    "expected_change_gte": -10000000,
                    "error_tolerance": 0.01
                }],
                "expected_tool_calls": [{
                    "tool_name": "JupiterLendEarnDeposit",
                    "critical": true
                }]
            },
            "metadata": {
                "version": "1.0",
                "created_at": chrono::Utc::now().to_rfc3339()
            }
        })
        .to_string()
    }

    /// Determine the appropriate response based on the prompt
    fn get_response_for_prompt(&self, prompt: &str) -> String {
        // Check for predefined responses first
        if let Some(response) = self.predefined_responses.get(prompt) {
            return response.clone();
        }

        // Generate appropriate response based on prompt content
        let prompt_lower = prompt.to_lowercase();
        if prompt_lower.contains("swap") || prompt_lower.contains("exchange") {
            self.create_swap_flow_response(prompt)
        } else if prompt_lower.contains("lend") || prompt_lower.contains("deposit") {
            self.create_lend_flow_response(prompt)
        } else {
            // Default response
            json!({
                "flow_id": uuid::Uuid::new_v4().to_string(),
                "user_prompt": prompt,
                "subject_wallet_info": [{
                    "pubkey": "USER_WALLET_PUBKEY",
                    "lamports": 4000000000u32,
                    "total_value_usd": 100.0,
                    "tokens": []
                }],
                "steps": [],
                "ground_truth": {
                    "final_state_assertions": [],
                    "expected_tool_calls": []
                },
                "metadata": {
                    "version": "1.0",
                    "created_at": chrono::Utc::now().to_rfc3339()
                }
            })
            .to_string()
        }
    }
}

#[async_trait]
impl LlmClient for MockLLMClient {
    async fn generate_flow(&self, prompt: &str) -> anyhow::Result<String> {
        if !self.simulate_success {
            return Err(anyhow::anyhow!("Mock LLM client failure for testing"));
        }

        Ok(self.get_response_for_prompt(prompt))
    }
}
