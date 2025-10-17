//! # Scoring System for Agent Evaluation
//!
//! This module implements the core scoring logic for evaluating agent performance
//! across different benchmark types. It uses a two-tiered approach that balances
//! reasoning quality with execution success.
//!
//! ## Scoring Philosophy
//!
//! The scoring system is designed to provide fair, granular assessment of agent
//! capabilities while maintaining robust anti-false-positive protection:
//!
//! ### Two-Tiered Scoring Formula
//! ```text
//! Final Score = (Instruction Score × 75%) + (On-Chain Score × 25%)
//! ```
//!
//! ### Component Breakdown
//! - **Instruction Score (75%)**: Evaluates how closely generated instructions match
//!   expected ground truth. Provides partial credit for correct reasoning.
//! - **On-Chain Score (25%)**: Binary success/failure based on transaction execution.
//!   Ensures agents can actually execute their plans.
//!
//! ## Special Cases
//!
//! ### API-First Protocols
//! For Jupiter and other complex protocols that use official APIs:
//! - `skip_instruction_validation: true` bypasses instruction matching
//! - Full score (1.0) awarded if API calls succeed
//! - Focuses on end results rather than instruction structure
//!
//! ### Flow Benchmarks
//! Multi-step workflows are scored per-step with:
//! - Individual step scores contributing to overall flow score
//! - Critical step failures carrying more weight
//! - Success criteria for partial credit scenarios

use crate::{
    agent::{AgentAction, AgentObservation},
    benchmark::TestCase,
    flow::ScoringBreakdown,
    instruction_score::calculate_instruction_score,
};
use tracing::{debug, info};

/// Weight for instruction quality in final score (75%)
///
/// This high weight ensures that agents with correct reasoning
/// receive substantial credit even if execution fails due to
/// external factors (network issues, timing, etc.).
const INSTRUCTION_SCORE_WEIGHT: f64 = 0.75;

/// Weight for on-chain execution in final score (25%)
///
/// This weight ensures agents can actually execute their plans
/// while not being overly punitive for execution failures
/// that might be beyond agent control.
const ONCHAIN_SCORE_WEIGHT: f64 = 0.25;

/// Calculates the final, comprehensive score for a test case.
///
/// This function implements the core two-tiered scoring algorithm that combines
/// instruction quality assessment with on-chain execution results.
///
/// ## Arguments
///
/// * `test_case` - The benchmark containing ground truth and scoring rules
/// * `actions` - Agent-generated actions to be evaluated
/// * `initial_observation` - Environment state before execution
/// * `final_observation` - Environment state after execution
///
/// ## Returns
///
/// A score between 0.0 and 1.0 where:
/// - 1.0 = Perfect performance (correct instructions and successful execution)
/// - 0.75 = Correct reasoning but execution failure
/// - 0.5 = Partially correct instructions
/// - 0.25 = Incorrect but plausible attempt
/// - 0.0 = Complete failure or no attempt
///
/// ## Special Handling
///
/// ### API-Based Benchmarks (`skip_instruction_validation: true`)
/// For Jupiter and other protocols using official APIs:
/// - Instruction score is automatically 1.0 (not applicable)
/// - Final score depends on API call success
/// - Focuses on end-state validation
///
/// ### Flow Benchmarks
/// Multi-step workflows are scored using:
/// - Per-step instruction matching
/// - Step-by-step execution validation
/// - Success criteria for partial credit
///
/// ## Example
///
/// ```rust
/// let score = calculate_final_score(
///     &test_case,
///     &agent_actions,
///     &initial_state,
///     &final_state,
/// );
/// assert!(score >= 0.0 && score <= 1.0);
/// ```
pub fn calculate_final_score(
    test_case: &TestCase,
    actions: &[AgentAction],
    initial_observation: &AgentObservation,
    final_observation: &AgentObservation,
) -> f64 {
    info!(
        "[SCORE] ==> Starting final score calculation for benchmark: {}",
        test_case.id
    );

    // Calculate instruction quality score
    let instruction_score = if test_case.ground_truth.skip_instruction_validation {
        info!("[SCORE] API-based benchmark detected, awarding full instruction score");
        1.0 // API protocols don't need instruction validation
    } else {
        calculate_instruction_score(test_case, actions, &initial_observation.key_map)
    };

    // Calculate on-chain execution score
    let onchain_score = calculate_onchain_score(
        final_observation,
        test_case.ground_truth.skip_instruction_validation,
    );

    // Apply scoring formula based on benchmark type
    let final_score = if test_case.ground_truth.skip_instruction_validation {
        // API benchmarks: full score if no crashes
        if final_observation.last_transaction_status == "Success" {
            1.0
        } else {
            0.0 // API call failed completely
        }
    } else {
        // Standard benchmarks: weighted combination
        (instruction_score * INSTRUCTION_SCORE_WEIGHT) + (onchain_score * ONCHAIN_SCORE_WEIGHT)
    };

    info!(
        instruction_score,
        onchain_score,
        final_score,
        benchmark_id = %test_case.id,
        api_based = %test_case.ground_truth.skip_instruction_validation,
        "Final score calculation completed"
    );

    // Ensure score is within valid bounds
    final_score.clamp(0.0, 1.0)
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
