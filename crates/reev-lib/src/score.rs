use crate::{
    agent::AgentObservation,
    benchmark::{StateAssertion, TestCase},
};
use tracing::{debug, info, warn};

/// Calculates the final score for a test case based on transaction success and on-chain assertions.
///
/// This function implements a "Transaction Quality" scoring model:
/// 1. It first checks if the transaction was successful. If the transaction failed for any
///    reason, the score is immediately `0.0`.
/// 2. If the transaction was successful, it then proceeds to validate all `final_state_assertions`
///    defined in the benchmark's ground truth.
/// 3. If all assertions pass, the score is `1.0`. If any assertion fails, the score is `0.0`.
///
/// # Arguments
/// * `test_case` - The benchmark test case, containing the ground truth assertions.
/// * `initial_observation` - The state of the accounts *before* the transaction was executed.
/// * `final_observation` - The state of the accounts *after* the transaction was executed.
///
/// # Returns
/// A score of `1.0` if the transaction succeeded and all assertions pass. Otherwise, returns `0.0`.
pub fn calculate_score(
    test_case: &TestCase,
    initial_observation: &AgentObservation,
    final_observation: &AgentObservation,
) -> f64 {
    info!(?final_observation, "Final observation for scoring");

    // 1. Primary check: Was the transaction successful?
    if final_observation.last_transaction_status != "Success" {
        debug!(
            "Scoring FAILED: Transaction status was '{}', not 'Success'.",
            final_observation.last_transaction_status
        );
        return 0.0;
    }
    debug!("Transaction status was 'Success'. Proceeding to state assertions.");

    // 2. If successful, check all on-chain state assertions.
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
            StateAssertion::TokenAccountBalance {
                pubkey,
                expected,
                expected_gte,
                address_derivation: _,
            } => {
                let actual = final_observation
                    .account_states
                    .get(pubkey)
                    .and_then(|account_state| {
                        account_state.get("amount").and_then(|v| {
                            v.as_u64()
                                .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
                        })
                    })
                    .unwrap_or(0);

                let mut all_passed = true;

                if let Some(expected_val) = expected {
                    if actual != *expected_val {
                        debug!(
                            pubkey,
                            expected = expected_val,
                            actual,
                            "TokenAccountBalance `expected` assertion FAILED"
                        );
                        all_passed = false;
                    } else {
                        debug!(
                            pubkey,
                            expected = expected_val,
                            actual,
                            "TokenAccountBalance `expected` assertion PASSED"
                        );
                    }
                }

                if let Some(expected_gte_val) = expected_gte {
                    if actual < *expected_gte_val {
                        debug!(
                            pubkey,
                            expected_gte = expected_gte_val,
                            actual,
                            "TokenAccountBalance `expected_gte` assertion FAILED"
                        );
                        all_passed = false;
                    } else {
                        debug!(
                            pubkey,
                            expected_gte = expected_gte_val,
                            actual,
                            "TokenAccountBalance `expected_gte` assertion PASSED"
                        );
                    }
                }

                all_passed
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

        // If any assertion fails, return 0.0 immediately.
        if !pass {
            warn!(
                "\n------------------------\nScoring assertion failed on test case '{}': {:?}\n------------------------",
                test_case.id, assertion
            );
            return 0.0;
        }
    }

    // If we get here, the transaction was successful AND all assertions passed.
    debug!("All on-chain assertions passed for successful transaction.");
    1.0
}
