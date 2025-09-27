use anyhow::{Result, anyhow};
use project_root::get_project_root;
use reev_lib::{
    actions::spl_transfer, agent::AgentAction, benchmark::TestCase, env::GymEnv,
    score::calculate_score, solana_env::SolanaEnv,
};
use rstest::rstest;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use std::{collections::HashMap, fs, str::FromStr};

/// Helper to generate a correct SPL transfer `Instruction` for testing.
/// This mocks the behavior of a perfect agent.
fn instruction_from_ground_truth(key_map: &HashMap<String, String>) -> Result<Instruction> {
    let source_pubkey_str = key_map
        .get("USER_USDC_ATA")
        .ok_or_else(|| anyhow!("Pubkey placeholder 'USER_USDC_ATA' not found in key_map"))?;
    let destination_pubkey_str = key_map
        .get("RECIPIENT_USDC_ATA")
        .ok_or_else(|| anyhow!("Pubkey placeholder 'RECIPIENT_USDC_ATA' not found in key_map"))?;
    let authority_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .ok_or_else(|| anyhow!("Pubkey placeholder 'USER_WALLET_PUBKEY' not found in key_map"))?;

    let source_pubkey = Pubkey::from_str(source_pubkey_str)?;
    let destination_pubkey = Pubkey::from_str(destination_pubkey_str)?;
    let authority_pubkey = Pubkey::from_str(authority_pubkey_str)?;

    // Both test cases `002` and `003` use a 15 USDC transfer amount.
    let amount = 15_000_000;

    // Use the helper from `reev-lib` to create the instruction.
    spl_transfer::create_instruction(
        &source_pubkey,
        &destination_pubkey,
        &authority_pubkey,
        amount,
    )
}

/// Parameterized test for the scoring logic.
///
/// This single test function covers both passing and failing scenarios by using `rstest`.
/// Each `#[case]` defines a distinct test run with a specific benchmark file and an expected score.
///
/// # Arguments
/// * `file_path` - The path to the benchmark YAML file, relative to the workspace root.
/// * `expected_score` - The expected final score (1.0 for pass, 0.0 for fail).
#[rstest]
#[case(
    "benchmarks/002-spl-transfer.yml",
    1.0,
    "Correct assertion should pass"
)]
#[case(
    "benchmarks/003-spl-transfer-fail.yml",
    0.0,
    "Incorrect assertion should fail"
)]
#[tokio::test(flavor = "multi_thread")]
async fn test_scoring_logic(
    #[case] file_path: &str,
    #[case] expected_score: f64,
    #[case] description: &str,
) -> Result<()> {
    // 1. Construct an absolute path to the benchmark file from the project root.
    // This ensures the test can find the file regardless of the working directory.
    let project_root = get_project_root()?;
    let benchmark_path = project_root.join(file_path);

    // 2. Load the benchmark file.
    let f = fs::File::open(&benchmark_path)?;
    let test_case: TestCase = serde_yaml::from_reader(f)?;

    // 3. Set up the environment.
    let mut env = SolanaEnv::new()?;
    let initial_state_json = serde_json::to_value(&test_case.initial_state)?;
    let options = serde_json::json!({ "initial_state": initial_state_json });
    let initial_observation = env.reset(None, Some(options))?;

    // 4. Mock the agent's action.
    let instruction = instruction_from_ground_truth(&initial_observation.key_map)?;
    let action = AgentAction(instruction);

    // 5. Execute the transaction.
    let step_result = env.step(action, &test_case.ground_truth)?;

    // 6. Calculate the score using the centralized function from `reev-lib`.
    let score = calculate_score(&test_case, &initial_observation, &step_result.observation);

    // 7. Assert the score matches the expected outcome for this case.
    assert_eq!(
        score, expected_score,
        "Test case '{description}' failed: expected score of {expected_score}, but got {score}"
    );

    env.close()?;
    Ok(())
}
