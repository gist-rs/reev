//! # Core Testing Philosophy: Surfpool + Real Mainnet Programs
//!
//! All integration tests in the `reev` framework operate on `surfpool`, a high-speed
//! local Solana test validator. `surfpool` instantly forks Solana mainnet, meaning
//! any on-chain account not explicitly mocked in the test setup is fetched live from
//! mainnet. This allows tests to interact with real, deployed programs (like SPL Token
//! or Jupiter) without any mocking of program logic. Test assertions are based on the
//! real outcomes of these transactions. This approach ensures that a passing test gives
//! a strong signal of real-world viability.

//! # Scoring Logic Unit Test
//!
//! This test file is dedicated to verifying the correctness of the `calculate_score` function.
//! It uses a minimal set of benchmarks to confirm that the scoring logic correctly
//! identifies passing and failing outcomes based on on-chain state assertions.

#[path = "common/mod.rs"]
mod common;

use anyhow::Result;
use project_root::get_project_root;
use reev_lib::{agent::AgentAction, env::GymEnv, score::calculate_score};
use rstest::rstest;

use common::helpers::{mock_perfect_instruction, setup_env_for_benchmark};

/// A focused unit test for the `calculate_score` function.
///
/// This test verifies the two most important scenarios:
/// 1. A benchmark where the final state **matches** the assertions should receive a score of 1.0.
/// 2. A benchmark where the final state **does not match** the assertions should receive a score of 0.0.
///
/// It uses the SPL transfer benchmarks as they provide a clear and direct way to test the assertion logic,
/// as the transaction itself is valid in both cases, but the ground truth differs.
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
    // 1. Set up the environment from the benchmark file.
    let project_root = get_project_root()?;
    let benchmark_path = project_root.join(file_path);
    let (mut env, test_case, initial_observation) = setup_env_for_benchmark(&benchmark_path)?;

    // 2. Create the "perfect" action for this benchmark.
    let instruction = mock_perfect_instruction(&test_case, &initial_observation.key_map)?;
    let action = AgentAction(instruction);

    // 3. Execute the transaction in the environment.
    let step_result = env.step(action, &test_case.ground_truth)?;

    // 4. Calculate the score using the centralized function from `reev-lib`.
    let score = calculate_score(&test_case, &initial_observation, &step_result.observation);

    // 5. Assert the score matches the expected outcome for this case.
    assert_eq!(
        score, expected_score,
        "Test case '{description}' failed: expected score of {expected_score}, but got {score}"
    );

    env.close()?;
    Ok(())
}
