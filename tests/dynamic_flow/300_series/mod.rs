//! 300-Series Dynamic Flow Benchmark Tests
//!
//! This module contains comprehensive tests for the 300-series benchmarks
//! that demonstrate reev's dynamic flow capabilities through realistic DeFi scenarios.

pub mod benchmark_300_test;
pub mod benchmark_301_test;
pub mod benchmark_302_test;
pub mod benchmark_303_test;
pub mod benchmark_304_test;
pub mod benchmark_305_test;
pub mod integration_test;
pub mod otel_tracking_test;

use reev_orchestrator::OrchestratorGateway;
use reev_types::flow::WalletContext;
use serde_json::Value;
use std::collections::HashMap;

/// Mock wallet context for testing
pub fn create_test_wallet_context(sol_balance: u64, usdc_balance: u64) -> WalletContext {
    let mut context = WalletContext::new("test_wallet_12345".to_string());
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

/// Parse benchmark YAML file for testing
pub fn load_benchmark_yaml(benchmark_id: &str) -> Value {
    let file_path = format!("benchmarks/{}.yml", benchmark_id);
    let yaml_content = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|_| panic!("Failed to read benchmark file: {}", file_path));

    serde_yaml::from_str(&yaml_content)
        .unwrap_or_else(|_| panic!("Failed to parse benchmark YAML: {}", benchmark_id))
}

/// Common test utilities for 300-series benchmarks
pub struct TestUtils {
    pub gateway: OrchestratorGateway,
}

impl TestUtils {
    pub fn new() -> Self {
        Self {
            gateway: OrchestratorGateway::new(),
        }
    }

    /// Test tool call expectations from benchmark
    pub fn validate_tool_calls(benchmark: &Value, executed_tools: &[String]) -> bool {
        let expected_tools = benchmark
            .get("ground_truth")
            .and_then(|gt| gt.get("expected_tool_calls"))
            .and_then(|etc| etc.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|tool| tool.get("tool_name"))
                    .filter_map(|name| name.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // Check that all critical tools were executed
        let critical_tools: Vec<String> = benchmark
            .get("ground_truth")
            .and_then(|gt| gt.get("expected_tool_calls"))
            .and_then(|etc| etc.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter(|tool| {
                        tool.get("critical")
                            .and_then(|c| c.as_bool())
                            .unwrap_or(false)
                    })
                    .filter_map(|tool| tool.get("tool_name"))
                    .filter_map(|name| name.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // All critical tools must be executed
        for critical_tool in &critical_tools {
            if !executed_tools.contains(critical_tool) {
                return false;
            }
        }

        true
    }

    /// Test percentage calculation accuracy
    pub fn validate_percentage_calculation(
        prompt: &str,
        initial_sol: u64,
        used_sol: u64,
    ) -> (bool, f64) {
        let prompt_lower = prompt.to_lowercase();

        if prompt_lower.contains("50%") {
            let expected = (initial_sol as f64 * 0.5) as u64;
            let actual_percentage = (used_sol as f64 / initial_sol as f64) * 100.0;
            let is_accurate =
                (used_sol as i64 - expected as i64).abs() <= (initial_sol as f64 * 0.02) as i64; // ±2% tolerance
            (is_accurate, actual_percentage)
        } else if prompt_lower.contains("30%") {
            let expected = (initial_sol as f64 * 0.3) as u64;
            let actual_percentage = (used_sol as f64 / initial_sol as f64) * 100.0;
            let is_accurate =
                (used_sol as i64 - expected as i64).abs() <= (initial_sol as f64 * 0.02) as i64; // ±2% tolerance
            (is_accurate, actual_percentage)
        } else if prompt_lower.contains("70%") {
            let expected = (initial_sol as f64 * 0.7) as u64;
            let actual_percentage = (used_sol as f64 / initial_sol as f64) * 100.0;
            let is_accurate =
                (used_sol as i64 - expected as i64).abs() <= (initial_sol as f64 * 0.02) as i64; // ±2% tolerance
            (is_accurate, actual_percentage)
        } else {
            (true, 0.0) // No percentage constraint
        }
    }

    /// Test multiplication achievement (for benchmarks 300, 301)
    pub fn validate_multiplication_achievement(
        initial_usdc: u64,
        final_usdc: u64,
        target_multiplier: f64,
        tolerance: f64,
    ) -> (bool, f64) {
        let actual_multiplier = final_usdc as f64 / initial_usdc as f64;
        let min_multiplier = target_multiplier - tolerance;
        let achieved = actual_multiplier >= min_multiplier;
        (achieved, actual_multiplier)
    }

    /// Test tool sequence complexity
    pub fn validate_tool_sequence(
        benchmark: &Value,
        executed_sequence: &[String],
    ) -> (bool, usize) {
        let min_steps = benchmark
            .get("ground_truth")
            .and_then(|gt| gt.get("expected_flow_complexity"))
            .and_then(|efc| efc.as_sequence())
            .and_then(|seq| {
                seq.iter().find_map(|item| {
                    item.get("type")
                        .and_then(|t| t.as_str())
                        .filter(|&t| t == "multi_step_execution")
                        .and_then(|_| item.get("min_steps"))
                        .and_then(|ms| ms.as_u64())
                        .map(|ms| ms as usize)
                })
            })
            .unwrap_or(1);

        let actual_steps = executed_sequence.len();
        let meets_minimum = actual_steps >= min_steps;
        (meets_minimum, actual_steps)
    }
}
