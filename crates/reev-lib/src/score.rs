use crate::{
    agent::{AgentAction, AgentObservation},
    benchmark::TestCase,
    instruction_score::calculate_instruction_score,
};
use tracing::{debug, info};

/// Weights for the final combined score.
const INSTRUCTION_SCORE_WEIGHT: f64 = 0.75;
const ONCHAIN_SCORE_WEIGHT: f64 = 0.25;

/// Calculates the final, comprehensive score for a test case.
///
/// This function combines two different scores, each with its own weight:
/// 1.  **Instruction Score (75%):** How closely the agent's generated transaction matches the
///     ground truth `expected_instructions`. This provides granular, partial credit for the
///     agent's reasoning.
/// 2.  **On-Chain Score (25%):** A binary score based on whether the transaction executed
///     successfully on `surfpool`.
///
/// `final_state_assertions` are NOT used for scoring; they are for post-run diagnostics.
pub fn calculate_final_score(
    test_case: &TestCase,
    actions: &[AgentAction],
    initial_observation: &AgentObservation,
    final_observation: &AgentObservation,
) -> f64 {
    info!(
        "[SCORE] ==> Entering calculate_final_score, which calls the CORRECT instruction scorer."
    );

    // Skip instruction validation if the benchmark is API-based
    let instruction_score = if test_case.ground_truth.skip_instruction_validation {
        info!("[SCORE] Skipping instruction validation for API-based benchmark");
        1.0 // Give full score for instruction part since it's not applicable
    } else {
        calculate_instruction_score(test_case, actions, &initial_observation.key_map)
    };
    let onchain_score = calculate_onchain_score(
        final_observation,
        test_case.ground_truth.skip_instruction_validation,
    );

    // For API-based benchmarks, we give full score since they don't need transactions
    let final_score = if test_case.ground_truth.skip_instruction_validation {
        1.0 // API benchmarks get full score if they don't crash
    } else {
        (instruction_score * INSTRUCTION_SCORE_WEIGHT) + (onchain_score * ONCHAIN_SCORE_WEIGHT)
    };

    info!(
        instruction_score,
        onchain_score, final_score, "Final weighted score calculated."
    );
    final_score
}

/// Calculates a binary score based on the transaction's on-chain execution status.
fn calculate_onchain_score(
    final_observation: &AgentObservation,
    skip_instruction_validation: bool,
) -> f64 {
    if skip_instruction_validation {
        debug!("On-chain score: 1.0 (API-based benchmark - no transaction needed)");
        1.0
    } else if final_observation.last_transaction_status == "Success" {
        debug!("On-chain score: 1.0 (Transaction Succeeded)");
        1.0
    } else {
        debug!("On-chain score: 0.0 (Transaction Failed)");
        0.0
    }
}
