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
    let instruction_score =
        calculate_instruction_score(test_case, actions, &initial_observation.key_map);
    let onchain_score = calculate_onchain_score(final_observation);

    let final_score =
        (instruction_score * INSTRUCTION_SCORE_WEIGHT) + (onchain_score * ONCHAIN_SCORE_WEIGHT);

    info!(
        instruction_score,
        onchain_score, final_score, "Final weighted score calculated."
    );
    final_score
}

/// Calculates a binary score based on the transaction's on-chain execution status.
fn calculate_onchain_score(final_observation: &AgentObservation) -> f64 {
    if final_observation.last_transaction_status == "Success" {
        debug!("On-chain score: 1.0 (Transaction Succeeded)");
        1.0
    } else {
        debug!("On-chain score: 0.0 (Transaction Failed)");
        0.0
    }
}
