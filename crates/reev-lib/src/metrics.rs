use crate::agent::AgentObservation;
use crate::benchmark::{GroundTruth, StateAssertion};
use crate::trace::ExecutionTrace;
use anyhow::{anyhow, Context, Result};
use solana_program::program_pack::Pack;
use solana_sdk::account::Account;

/// Contains the calculated scores for a single test case evaluation.
use serde::{Deserialize, Serialize};

/// Contains the calculated scores for a single test case evaluation.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct QuantitativeScores {
    pub task_success_rate: f32,
    /// Tool Selection Accuracy (TSA)
    pub tool_selection_accuracy: f32,
    /// Parameterization Accuracy (PA)
    pub parameterization_accuracy: f32,
}

/// Calculates all quantitative metrics for a completed episode.
pub fn calculate_quantitative_metrics(
    final_observation: &AgentObservation,
    trace: &ExecutionTrace,
    ground_truth: &GroundTruth,
) -> Result<QuantitativeScores> {
    let task_success_rate = calculate_task_success_rate(final_observation, ground_truth)?;
    let tool_selection_accuracy = calculate_tool_selection_accuracy(trace, ground_truth)?;
    let parameterization_accuracy = calculate_parameterization_accuracy(trace, ground_truth)?;

    Ok(QuantitativeScores {
        task_success_rate,
        tool_selection_accuracy,
        parameterization_accuracy,
    })
}

/// Calculates the Task Success Rate (TSR).
pub fn calculate_task_success_rate(
    final_observation: &AgentObservation,
    ground_truth: &GroundTruth,
) -> Result<f32> {
    // An unsuccessful transaction is an automatic failure, regardless of assertions.
    if final_observation.last_transaction_status != "Success" {
        println!(
            "[Metrics] Task FAILED: Last transaction status was '{}'",
            final_observation.last_transaction_status
        );
        return Ok(0.0);
    }

    if ground_truth.final_state_assertions.is_empty() {
        return Ok(1.0);
    }
    for assertion in &ground_truth.final_state_assertions {
        if !check_assertion(final_observation, assertion)? {
            println!("[Metrics] Assertion FAILED: {assertion:?}");
            return Ok(0.0);
        }
        println!("[Metrics] Assertion PASSED: {assertion:?}");
    }
    Ok(1.0)
}

/// Checks if a single `StateAssertion` is met by the `final_observation`.
fn check_assertion(observation: &AgentObservation, assertion: &StateAssertion) -> Result<bool> {
    match assertion {
        StateAssertion::SolBalance { pubkey, expected } => {
            let account_value = observation.account_states.get(pubkey).ok_or_else(|| {
                anyhow!("Assertion failed: Account '{pubkey}' not found in final observation")
            })?;
            // The observation now contains the full Account object, serialized to a Value.
            let account: Account = serde_json::from_value(account_value.clone())
                .context(format!("Failed to deserialize account state for {pubkey}"))?;
            Ok(account.lamports == *expected)
        }
        StateAssertion::TokenAccountBalance { pubkey, expected } => {
            let account_value = observation.account_states.get(pubkey).ok_or_else(|| {
                anyhow!("Assertion failed: Account '{pubkey}' not found in final observation")
            })?;
            let account: Account = serde_json::from_value(account_value.clone())
                .context(format!("Failed to deserialize account state for {pubkey}"))?;
            let token_account = spl_token::state::Account::unpack(&account.data)
                .context("Failed to unpack SPL token account data")?;
            let actual_amount = token_account.amount;
            println!("[Metrics] Checking TokenAccountBalance for {pubkey}: Expected={expected}, Actual={actual_amount}");
            Ok(actual_amount == *expected)
        }
        // For now, any other assertion type is considered unhandled and will fail.
        _ => {
            println!("[Metrics] WARNING: Unhandled assertion type: {assertion:?}");
            Ok(false)
        }
    }
}

/// Calculates Tool Selection Accuracy (TSA).
/// Compares the agent's tool calls against the ground truth.
fn calculate_tool_selection_accuracy(
    trace: &ExecutionTrace,
    ground_truth: &GroundTruth,
) -> Result<f32> {
    let expected_calls = &ground_truth.expected_tool_calls;
    if expected_calls.is_empty() {
        return Ok(1.0); // No tools to select, so 100% accuracy.
    }

    let actual_calls: Vec<_> = trace
        .steps
        .iter()
        .map(|s| &s.action)
        // The `DummyAgent` might add a `no_op` at the end, which is not part of the expected calls.
        .filter(|a| a.tool_name != "no_op")
        .collect();

    let mut correct_selections = 0;
    for i in 0..expected_calls.len() {
        if let Some(actual_call) = actual_calls.get(i) {
            if actual_call.tool_name == expected_calls[i].tool_name {
                correct_selections += 1;
            }
        }
    }

    Ok(correct_selections as f32 / expected_calls.len() as f32)
}

/// Calculates Parameterization Accuracy (PA).
/// For correctly chosen tools, it checks if the parameters were correct.
fn calculate_parameterization_accuracy(
    trace: &ExecutionTrace,
    ground_truth: &GroundTruth,
) -> Result<f32> {
    let expected_calls = &ground_truth.expected_tool_calls;
    if expected_calls.is_empty() {
        return Ok(1.0);
    }

    let actual_calls: Vec<_> = trace
        .steps
        .iter()
        .map(|s| &s.action)
        .filter(|a| a.tool_name != "no_op")
        .collect();

    let mut correct_selections = 0;
    let mut correct_params = 0;

    for i in 0..expected_calls.len() {
        if let Some(actual_call) = actual_calls.get(i) {
            if actual_call.tool_name == expected_calls[i].tool_name {
                correct_selections += 1;
                // Use serde_json::Value's PartialEq implementation for a deep comparison.
                if actual_call.parameters == expected_calls[i].parameters {
                    correct_params += 1;
                }
            }
        }
    }

    if correct_selections == 0 {
        return Ok(0.0); // No tools were selected correctly, so 0% param accuracy.
    }

    Ok(correct_params as f32 / correct_selections as f32)
}
