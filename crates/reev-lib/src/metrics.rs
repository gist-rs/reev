use crate::agent::AgentObservation;
use crate::benchmark::{GroundTruth, StateAssertion};
use anyhow::{anyhow, Context, Result};
use solana_program::program_pack::Pack;
use solana_sdk::account::Account;

/// Contains the calculated scores for a single test case evaluation.
#[derive(Debug, Default)]
pub struct QuantitativeScores {
    pub task_success_rate: f32,
}

/// Calculates all quantitative metrics for a completed episode.
pub fn calculate_quantitative_metrics(
    final_observation: &AgentObservation,
    ground_truth: &GroundTruth,
) -> Result<QuantitativeScores> {
    let task_success_rate = calculate_task_success_rate(final_observation, ground_truth)?;
    Ok(QuantitativeScores { task_success_rate })
}

/// Calculates the Task Success Rate (TSR).
pub fn calculate_task_success_rate(
    final_observation: &AgentObservation,
    ground_truth: &GroundTruth,
) -> Result<f32> {
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
            Ok(token_account.amount == *expected)
        }
        // For now, any other assertion type is considered unhandled and will fail.
        _ => {
            println!("[Metrics] WARNING: Unhandled assertion type: {assertion:?}");
            Ok(false)
        }
    }
}
