use anyhow::Result;
use reev_lib::{
    actions::spl_transfer,
    agent::AgentAction,
    benchmark::TestCase,
    env::GymEnv,
    // This function will need to be moved from `reev-runner/src/main.rs` to a new
    // module in `reev-lib` (e.g., `reev-lib/src/score.rs`) to be accessible here.
    // score::calculate_score,
    solana_env::SolanaEnv,
};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use std::{collections::HashMap, fs, str::FromStr};

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

/// Helper to generate a correct SPL transfer `Instruction` for testing.
fn instruction_from_ground_truth(
    _test_case: &TestCase,
    key_map: &HashMap<String, String>,
) -> Result<Instruction> {
    let source_pubkey_str = key_map.get("USER_USDC_ATA").ok_or_else(|| {
        anyhow::anyhow!("Pubkey placeholder 'USER_USDC_ATA' not found in key_map")
    })?;
    let destination_pubkey_str = key_map.get("RECIPIENT_USDC_ATA").ok_or_else(|| {
        anyhow::anyhow!("Pubkey placeholder 'RECIPIENT_USDC_ATA' not found in key_map")
    })?;
    let authority_pubkey_str = key_map.get("USER_WALLET_PUBKEY").ok_or_else(|| {
        anyhow::anyhow!("Pubkey placeholder 'USER_WALLET_PUBKEY' not found in key_map")
    })?;

    let source_pubkey = Pubkey::from_str(source_pubkey_str)?;
    let destination_pubkey = Pubkey::from_str(destination_pubkey_str)?;
    let authority_pubkey = Pubkey::from_str(authority_pubkey_str)?;

    // Both test cases `002` and `003` use a 15 USDC transfer amount.
    let amount = 15_000_000;

    spl_transfer::create_instruction(
        &source_pubkey,
        &destination_pubkey,
        &authority_pubkey,
        amount,
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn test_scoring_pass_case() -> Result<()> {
    // 1. Load the benchmark file.
    let f = fs::File::open("../../benchmarks/002-spl-transfer.yml")?;
    let test_case: TestCase = serde_yaml::from_reader(f)?;

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
    let f = fs::File::open("../../benchmarks/003-spl-transfer-fail.yml")?;
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
