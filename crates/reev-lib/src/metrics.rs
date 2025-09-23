use crate::agent::AgentObservation;
use crate::benchmark::{GroundTruth, StateAssertion};
use anyhow::{Result, anyhow};

/// Contains the calculated scores for a single test case evaluation.
#[derive(Debug, Default)]
pub struct QuantitativeScores {
    /// Task Success Rate (TSR): 1.0 if the agent achieved the goal, 0.0 otherwise.
    pub task_success_rate: f32,
    // Other metrics like TSA, PA, GCE will be added here later.
}

/// Calculates all quantitative metrics for a completed episode.
///
/// # Arguments
/// * `final_observation`: The last observation from the environment before termination.
/// * `ground_truth`: The ground truth from the `TestCase`.
///
/// # Returns
/// A `QuantitativeScores` struct containing the results.
pub fn calculate_quantitative_metrics(
    final_observation: &AgentObservation,
    ground_truth: &GroundTruth,
) -> Result<QuantitativeScores> {
    let task_success_rate = calculate_task_success_rate(final_observation, ground_truth)?;

    Ok(QuantitativeScores {
        task_success_rate,
        //.. other scores will be calculated here
    })
}

/// Calculates the Task Success Rate (TSR).
///
/// TSR is a binary metric: 1.0 if all final state assertions pass, 0.0 otherwise.
pub fn calculate_task_success_rate(
    final_observation: &AgentObservation,
    ground_truth: &GroundTruth,
) -> Result<f32> {
    // If there are no assertions, the task is considered trivially successful.
    if ground_truth.final_state_assertions.is_empty() {
        return Ok(1.0);
    }

    for assertion in &ground_truth.final_state_assertions {
        let passed = check_assertion(final_observation, assertion)?;
        if !passed {
            println!("[Metrics] Assertion FAILED: {assertion:?}");
            // If any single assertion fails, the entire task has failed.
            return Ok(0.0);
        }
        println!("[Metrics] Assertion PASSED: {assertion:?}");
    }

    // If all assertions were checked and none failed, the task was successful.
    Ok(1.0)
}

/// Checks if a single `StateAssertion` is met by the `final_observation`.
fn check_assertion(observation: &AgentObservation, assertion: &StateAssertion) -> Result<bool> {
    match assertion {
        StateAssertion::SolBalance { pubkey, expected } => {
            let account_value = observation.account_states.get(pubkey).ok_or_else(|| {
                anyhow!("Assertion failed: Account '{pubkey}' not found in final observation")
            })?;
            let account_state: crate::solana_env::MockAccountState =
                serde_json::from_value(account_value.clone())?;
            Ok(account_state.lamports == *expected)
        }
        StateAssertion::TokenAccountBalance { pubkey, expected } => {
            let account_value = observation.account_states.get(pubkey).ok_or_else(|| {
                anyhow!("Assertion failed: Account '{pubkey}' not found in final observation")
            })?;
            let account_state: crate::solana_env::MockAccountState =
                serde_json::from_value(account_value.clone())?;

            if let crate::solana_env::MockAccountData::SplToken(token_state) = account_state.data {
                Ok(token_state.amount == *expected)
            } else {
                Err(anyhow!(
                    "Assertion failed: Account '{pubkey}' is not an SPL Token account."
                ))
            }
        }
        // As more assertion types are defined, their checking logic will be added here.
        _ => {
            println!("[Metrics] WARNING: Unhandled assertion type encountered: {assertion:?}");
            // For now, we treat unhandled assertions as not passing.
            Ok(false)
        }
    }
}
