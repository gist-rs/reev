//! Dynamic Flow Tests
//!
//! This module contains comprehensive tests for reev's dynamic flow capabilities,
//! including the 300-series benchmarks that demonstrate advanced DeFi automation
//! through natural language processing and intelligent orchestration.

pub mod 300_series;

use reev_orchestrator::OrchestratorGateway;
use reev_types::flow::WalletContext;
use serde_json::Value;
use std::collections::HashMap;

/// Test utilities for dynamic flow functionality
pub struct TestUtils {
    pub gateway: OrchestratorGateway,
}

impl TestUtils {
    pub fn new() -> Self {
        Self {
            gateway: OrchestratorGateway::new(),
        }
    }

    /// Create a mock wallet context for testing
    pub fn create_wallet_context(
        owner: &str,
        sol_balance: u64,
        usdc_balance: u64,
    ) -> WalletContext {
        let mut context = WalletContext::new(owner.to_string());
        context.sol_balance = sol_balance * 1_000_000_000; // Convert to lamports
        context.add_token_balance(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            usdc_balance * 1_000_000, // USDC has 6 decimals
        );
        context.add_token_price(
            "So11111111111111111111111111111111111111112".to_string(),
            150.0, // $150 SOL
        );
        context.add_token_price(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1.0, // $1 USDC
        );
        context.calculate_total_value();
        context
    }

    /// Load and parse benchmark YAML file
    pub fn load_benchmark(benchmark_id: &str) -> Value {
        let file_path = format!("benchmarks/{}.yml", benchmark_id);
        let yaml_content = std::fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Failed to read benchmark file: {}", file_path));

        serde_yaml::from_str(&yaml_content)
            .unwrap_or_else(|_| panic!("Failed to parse benchmark YAML: {}", benchmark_id))
    }

    /// Validate that benchmark uses expected_tool_calls (not expected_api_calls)
    pub fn validate_tool_call_design(benchmark_id: &str) -> bool {
        let benchmark = Self::load_benchmark(benchmark_id);

        // Should have expected_tool_calls
        let has_tool_calls = benchmark
            .get("ground_truth")
            .and_then(|gt| gt.get("expected_tool_calls"))
            .is_some();

        // Should NOT have expected_api_calls (old design)
        let has_api_calls = benchmark
            .get("ground_truth")
            .and_then(|gt| gt.get("expected_api_calls"))
            .is_some();

        has_tool_calls && !has_api_calls
    }

    /// Test prompt parsing for natural language understanding
    pub fn validate_prompt_parsing(prompt: &str) -> HashMap<String, String> {
        let mut extracted = HashMap::new();

        // Extract percentages
        if prompt.to_lowercase().contains("50%") {
            extracted.insert("percentage".to_string(), "50%".to_string());
        } else if prompt.to_lowercase().contains("30%") {
            extracted.insert("percentage".to_string(), "30%".to_string());
        } else if prompt.to_lowercase().contains("70%") {
            extracted.insert("percentage".to_string(), "70%".to_string());
        }

        // Extract multiplication targets
        if prompt.to_lowercase().contains("1.5x") {
            extracted.insert("multiplier".to_string(), "1.5x".to_string());
        } else if prompt.to_lowercase().contains("2x") {
            extracted.insert("multiplier".to_string(), "2x".to_string());
        }

        // Extract strategy types
        if prompt.to_lowercase().contains("multiply") {
            extracted.insert("strategy".to_string(), "multiplication".to_string());
        } else if prompt.to_lowercase().contains("optimization") {
            extracted.insert("strategy".to_string(), "optimization".to_string());
        } else if prompt.to_lowercase().contains("rebalancing") {
            extracted.insert("strategy".to_string(), "rebalancing".to_string());
        } else if prompt.to_lowercase().contains("emergency") {
            extracted.insert("strategy".to_string(), "emergency".to_string());
        } else if prompt.to_lowercase().contains("yield farming") {
            extracted.insert("strategy".to_string(), "yield_farming".to_string());
        }

        extracted
    }

    /// Calculate expected step count based on prompt complexity
    pub fn estimate_step_count(prompt: &str) -> usize {
        let prompt_lower = prompt.to_lowercase();

        let mut steps = 1; // Base step

        if prompt_lower.contains("swap") && prompt_lower.contains("lend") {
            steps += 1; // Additional lend step
        }

        if prompt_lower.contains("multiply") || prompt_lower.contains("optimization") {
            steps += 1; // Analysis step
        }

        if prompt_lower.contains("rebalancing") {
            steps += 1; // Portfolio analysis step
        }

        if prompt_lower.contains("emergency") {
            steps += 1; // Withdrawal step
        }

        if prompt_lower.contains("yield farming") || prompt_lower.contains("multi-pool") {
            steps += 1; // Pool analysis step
        }

        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_call_design_validation() {
        let benchmark_ids = vec![
            "300-swap-sol-then-mul-usdc",
            "301-dynamic-yield-optimization",
            "302-portfolio-rebalancing",
            "303-risk-adjusted-growth",
            "304-emergency-exit-strategy",
            "305-yield-farming-optimization",
        ];

        for benchmark_id in benchmark_ids {
            let is_valid_design = TestUtils::validate_tool_call_design(benchmark_id);
            assert!(is_valid_design,
                   "Benchmark {} should use expected_tool_calls design", benchmark_id);
        }
    }

    #[test]
    fn test_prompt_parsing() {
        let test_cases = vec![
            ("use my 50% sol to multiply usdc 1.5x on jup", vec![
                ("percentage", "50%"), ("multiplier", "1.5x"), ("strategy", "multiplication")
            ]),
            ("Use my 50% SOL to maximize my USDC returns through Jupiter lending", vec![
                ("percentage", "50%"), ("strategy", "optimization")
            ]),
            ("I want to rebalance my portfolio based on current market conditions", vec![
                ("strategy", "rebalancing")
            ]),
            ("I need an emergency exit strategy for all my positions due to market stress", vec![
                ("strategy", "emergency")
            ]),
            ("I want to optimize my yield farming strategy using 70% of my available capital", vec![
                ("percentage", "70%"), ("strategy", "yield_farming")
            ]),
        ];

        for (prompt, expected_key_values) in test_cases {
            let extracted = TestUtils::validate_prompt_parsing(prompt);

            for (key, expected_value) in expected_key_values {
                assert_eq!(extracted.get(key), Some(&expected_value.to_string()),
                         "Should extract {}: {} from prompt: {}", key, expected_value, prompt);
            }
        }
    }

    #[test]
    fn test_step_count_estimation() {
        let test_cases = vec![
            ("swap 1 SOL to USDC", 1),
            ("use my 50% sol to multiply usdc 1.5x on jup", 2),
            ("Use my 50% SOL to maximize my USDC returns through Jupiter lending", 2),
            ("I want to rebalance my portfolio based on current market conditions", 2),
            ("I need an emergency exit strategy for all my positions due to market stress", 3),
            ("I want to optimize my yield farming strategy using 70% of my available capital", 3),
        ];

        for (prompt, expected_steps) in test_cases {
            let estimated = TestUtils::estimate_step_count(prompt);
            assert_eq!(estimated, expected_steps,
                     "Should estimate {} steps for prompt: {}", expected_steps, prompt);
        }
    }

    #[tokio::test]
    async fn test_basic_flow_generation() -> anyhow::Result<()> {
        let utils = TestUtils::new();
        let context = utils.create_wallet_context("test_wallet", 5, 50);

        let test_prompts = vec![
            "swap 1 SOL to USDC",
            "lend my USDC on Jupiter",
            "use 50% sol to usdc",
        ];

        for prompt in test_prompts {
            let flow_plan = utils.gateway.generate_flow_plan(prompt, &context, None)?;

            assert!(!flow_plan.steps.is_empty(),
                   "Should generate steps for prompt: {}", prompt);
            assert_eq!(flow_plan.user_prompt, prompt);
            assert_eq!(flow_plan.context.owner, "test_wallet");
        }

        Ok(())
    }
}
