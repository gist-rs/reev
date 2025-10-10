use crate::{
    agent::{AgentAction, AgentObservation},
    benchmark::TestCase,
    flow::ScoringBreakdown,
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

/// Calculates detailed scoring breakdown for analysis
pub fn calculate_detailed_score(
    test_case: &TestCase,
    actions: &[AgentAction],
    initial_observation: &AgentObservation,
    final_observation: &AgentObservation,
) -> ScoringBreakdown {
    let instruction_score = if test_case.ground_truth.skip_instruction_validation {
        1.0
    } else {
        calculate_instruction_score(test_case, actions, &initial_observation.key_map)
    };

    let onchain_score = calculate_onchain_score(
        final_observation,
        test_case.ground_truth.skip_instruction_validation,
    );

    let final_score = if test_case.ground_truth.skip_instruction_validation {
        1.0
    } else {
        (instruction_score * INSTRUCTION_SCORE_WEIGHT) + (onchain_score * ONCHAIN_SCORE_WEIGHT)
    };

    let mut issues = Vec::new();
    let mut mismatches = Vec::new();

    // Analyze instruction score issues
    if instruction_score < 1.0 && !test_case.ground_truth.skip_instruction_validation {
        let lost_instruction_points = (1.0 - instruction_score) * 100.0;
        if lost_instruction_points > 20.0 {
            issues.push(format!(
                "Instruction matching lost {lost_instruction_points:.1} points"
            ));
            mismatches.push("Program ID, accounts, or instruction data mismatches".to_string());
        } else {
            mismatches.push("Minor instruction format differences".to_string());
        }
    }

    // Analyze on-chain execution issues
    if onchain_score < 1.0 && !test_case.ground_truth.skip_instruction_validation {
        issues.push("Transaction failed on-chain execution".to_string());
        if let Some(error) = &final_observation.last_transaction_error {
            mismatches.push(format!("On-chain error: {error}"));
        }
    }

    ScoringBreakdown {
        instruction_score,
        onchain_score,
        final_score,
        issues,
        mismatches,
    }
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
