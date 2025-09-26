use anyhow::Result;
use reev_lib::{
    agent::AgentAction,
    benchmark::{BenchmarkAccountMeta, TestCase},
    env::GymEnv,
    // This function will need to be moved from `reev-runner/src/main.rs` to a new
    // module in `reev-lib` (e.g., `reev-lib/src/score.rs`) to be accessible here.
    // score::calculate_score,
    solana_env::SolanaEnv,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use std::{collections::HashMap, fs, str::FromStr};
use tracing::info;

// NOTE: The following two functions are duplicates of logic in `main.rs` and `db.rs`.
// This is a temporary measure. The ideal solution is to refactor `calculate_score`
// into `reev-lib` so it can be called directly from both the runner and this test.

/// Calculates the final score based on the ground truth assertions.
fn calculate_score(
    test_case: &TestCase,
    final_observation: &reev_lib::agent::AgentObservation,
) -> f64 {
    for assertion in &test_case.ground_truth.final_state_assertions {
        match assertion {
            reev_lib::benchmark::StateAssertion::SolBalance { pubkey, expected } => {
                if let Some(account_state) = final_observation.account_states.get(pubkey) {
                    if let Some(actual_lamports) =
                        account_state.get("lamports").and_then(|v| v.as_u64())
                    {
                        if actual_lamports != *expected {
                            return 0.0; // FAILED
                        }
                    } else {
                        return 0.0; // FAILED
                    }
                } else if *expected != 0 {
                    return 0.0; // FAILED
                }
            }
            reev_lib::benchmark::StateAssertion::TokenAccountBalance { pubkey, expected } => {
                if let Some(account_state) = final_observation.account_states.get(pubkey) {
                    if let Some(actual_amount) =
                        account_state.get("amount").and_then(|v| v.as_u64())
                    {
                        if actual_amount != *expected {
                            return 0.0; // FAILED
                        }
                    } else {
                        return 0.0; // FAILED
                    }
                } else if *expected != 0 {
                    return 0.0; // FAILED
                }
            }
            _ => unimplemented!(),
        }
    }
    1.0 // PASSED
}

/// Helper to convert the benchmark's `ExpectedInstruction` into a native `Instruction`.
fn instruction_from_ground_truth(
    test_case: &TestCase,
    key_map: &HashMap<String, String>,
) -> Result<Instruction> {
    let gt_instruction = test_case
        .ground_truth
        .expected_instruction
        .as_ref()
        .expect("Test case must have an expected_instruction for this test.");

    let program_id = Pubkey::from_str(&gt_instruction.program_id)?;

    let accounts = gt_instruction
        .accounts
        .iter()
        .map(
            |acc: &BenchmarkAccountMeta| -> Result<AccountMeta, anyhow::Error> {
                let pubkey_str = key_map.get(&acc.pubkey).ok_or_else(|| {
                    anyhow::anyhow!("Pubkey placeholder '{}' not found in key_map", acc.pubkey)
                })?;
                let pubkey = Pubkey::from_str(pubkey_str)?;

                Ok(if acc.is_writable {
                    AccountMeta::new(pubkey, acc.is_signer)
                } else {
                    AccountMeta::new_readonly(pubkey, acc.is_signer)
                })
            },
        )
        .collect::<Result<Vec<_>>>()?;

    let data = bs58::decode(&gt_instruction.data).into_vec()?;

    let instruction = Instruction {
        program_id,
        accounts,
        data,
    };
    Ok(instruction)
}

#[tokio::test(flavor = "multi_thread")]
async fn test_scoring_pass_case() -> Result<()> {
    // 1. Load the benchmark file.
    let f = fs::File::open("tests/benchmarks/002-spl-transfer.yml")?;
    let test_case: TestCase = serde_yaml::from_reader(f)?;

    info!(
        "[DEBUG] Ground truth instruction: {:#?}",
        test_case.ground_truth.expected_instruction
    );

    // 2. Set up the environment.
    let mut env = SolanaEnv::new()?;
    let initial_state_json = serde_json::to_value(&test_case.initial_state)?;
    let options = serde_json::json!({ "initial_state": initial_state_json });
    let observation = env.reset(None, Some(options))?;

    // 3. Mock the agent's action by using the ground truth instruction.
    let instruction = instruction_from_ground_truth(&test_case, &observation.key_map)?;
    let action = AgentAction(instruction);

    // 4. Execute the transaction.
    let step_result = env.step(action, &test_case.ground_truth)?;

    // 5. Calculate the score using the local, duplicated logic.
    let score = calculate_score(&test_case, &step_result.observation);

    // 6. Assert the score is 1.0.
    assert_eq!(score, 1.0, "Score for the pass case should be 1.0");

    env.close()?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_scoring_fail_case() -> Result<()> {
    // 1. Load the benchmark file.
    let f = fs::File::open("tests/benchmarks/003-spl-transfer-fail.yml")?;
    let test_case: TestCase = serde_yaml::from_reader(f)?;

    // 2. Set up the environment.
    let mut env = SolanaEnv::new()?;
    let initial_state_json = serde_json::to_value(&test_case.initial_state)?;
    let options = serde_json::json!({ "initial_state": initial_state_json });
    let observation = env.reset(None, Some(options))?;

    // 3. Mock the agent's action. The transaction itself is valid.
    let instruction = instruction_from_ground_truth(&test_case, &observation.key_map)?;
    let action = AgentAction(instruction);

    // 4. Execute the transaction.
    let step_result = env.step(action, &test_case.ground_truth)?;

    // 5. Calculate the score. The logic should find a mismatch with the assertion.
    let score = calculate_score(&test_case, &step_result.observation);

    // 6. Assert the score is 0.0.
    assert_eq!(score, 0.0, "Score for the fail case should be 0.0");

    env.close()?;
    Ok(())
}
