use crate::{
    agent::AgentObservation,
    benchmark::{StateAssertion, TestCase},
};
use tracing::{debug, info};

/// Calculates the final score for a test case based on its on-chain assertions.
///
/// This function iterates through all `final_state_assertions` defined in the
/// benchmark's ground truth. It compares the expected state with the actual
/// final state observed in the environment.
///
/// # Arguments
/// * `test_case` - The benchmark test case, containing the ground truth assertions.
/// * `initial_observation` - The state of the accounts *before* the transaction was executed.
/// * `final_observation` - The state of the accounts *after* the transaction was executed.
///
/// # Returns
/// A score of `1.0` if all assertions pass, and `0.0` if any assertion fails.
pub fn calculate_score(
    test_case: &TestCase,
    initial_observation: &AgentObservation,
    final_observation: &AgentObservation,
) -> f64 {
    info!(?final_observation, "Final observation for scoring");
    debug!("Calculating score based on on-chain state assertions...");
    for assertion in &test_case.ground_truth.final_state_assertions {
        let pass = match assertion {
            StateAssertion::SolBalance { pubkey, expected } => {
                if let Some(account_state) = final_observation.account_states.get(pubkey) {
                    if let Some(actual) = account_state.get("lamports").and_then(|v| v.as_u64()) {
                        if actual == *expected {
                            debug!(pubkey, expected, actual, "SolBalance assertion PASSED");
                            true
                        } else {
                            debug!(pubkey, expected, actual, "SolBalance assertion FAILED");
                            false
                        }
                    } else {
                        debug!(
                            pubkey,
                            expected, "SolBalance assertion FAILED: lamports not found"
                        );
                        false
                    }
                } else if *expected == 0 {
                    // If the account doesn't exist, its balance is implicitly 0.
                    debug!(
                        pubkey,
                        expected, "SolBalance assertion PASSED: account not found"
                    );
                    true
                } else {
                    debug!(
                        pubkey,
                        expected, "SolBalance assertion FAILED: account not found"
                    );
                    false
                }
            }
            StateAssertion::TokenAccountBalance { pubkey, expected } => {
                if let Some(account_state) = final_observation.account_states.get(pubkey) {
                    if let Some(actual) = account_state.get("amount").and_then(|v| v.as_u64()) {
                        if actual == *expected {
                            debug!(
                                pubkey,
                                expected, actual, "TokenAccountBalance assertion PASSED"
                            );
                            true
                        } else {
                            debug!(
                                pubkey,
                                expected, actual, "TokenAccountBalance assertion FAILED"
                            );
                            false
                        }
                    } else {
                        debug!(
                            pubkey,
                            expected, "TokenAccountBalance assertion FAILED: amount not found"
                        );
                        false
                    }
                } else if *expected == 0 {
                    // If the token account doesn't exist, its balance is implicitly 0.
                    debug!(
                        pubkey,
                        expected, "TokenAccountBalance assertion PASSED: account not found"
                    );
                    true
                } else {
                    debug!(
                        pubkey,
                        expected, "TokenAccountBalance assertion FAILED: account not found"
                    );
                    false
                }
            }
            StateAssertion::SolBalanceChange {
                pubkey,
                expected_change_gte,
            } => {
                let initial_balance = initial_observation
                    .account_states
                    .get(pubkey)
                    .and_then(|v| v.get("lamports"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let final_balance = final_observation
                    .account_states
                    .get(pubkey)
                    .and_then(|v| v.get("lamports"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let actual_change = final_balance as i64 - initial_balance as i64;

                if actual_change >= *expected_change_gte {
                    debug!(
                        pubkey,
                        expected_change_gte, actual_change, "SolBalanceChange assertion PASSED"
                    );
                    true
                } else {
                    debug!(
                        pubkey,
                        expected_change_gte, actual_change, "SolBalanceChange assertion FAILED"
                    );
                    false
                }
            }
        };

        // If any assertion fails, the score is immediately 0.
        if !pass {
            return 0.0;
        }
    }
    debug!("All on-chain assertions passed");
    1.0
}
