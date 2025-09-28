//! # Core Testing Philosophy: Surfpool + Real Mainnet Programs
//!
//! All integration tests in the `reev` framework operate on `surfpool`, a high-speed
//! local Solana test validator. `surfpool` instantly forks Solana mainnet, meaning
//! any on-chain account not explicitly mocked in the test setup is fetched live from
//! mainnet. This allows tests to interact with real, deployed programs (like SPL Token
//! or Jupiter) without any mocking of program logic. Test assertions are based on the
//! real outcomes of these transactions. This approach ensures that a passing test gives
//! a strong signal of real-world viability.

//! # Benchmark Sanity Check Test
//!
//! This test file ensures that all defined benchmarks in the `benchmarks/`
//! directory are valid and "solvable" by a perfect agent.
//!
//! It dynamically discovers every `.yml` file and runs a test for each one.
//! All complex setup logic is now centralized in `reev-lib`, so this test uses a
//! single, unified setup function (`setup_env_for_benchmark`) for all cases.

#[path = "common/mod.rs"]
mod common;

use anyhow::Result;
use glob::glob;
use project_root::get_project_root;
use reev_lib::{agent::AgentAction, env::GymEnv, score::calculate_score};
use rstest::rstest;
use std::path::PathBuf;
use tracing::info;

use common::helpers::{mock_perfect_instruction, setup_env_for_benchmark};

/// Dynamically discovers all solvable `.yml` files in the `benchmarks` directory.
///
/// This function is used by `rstest` to generate a test case for each file.
/// It explicitly filters out any benchmark known to be unsolvable by design
/// (e.g., those with intentionally incorrect assertions).
fn find_benchmark_files() -> Vec<PathBuf> {
    let root = get_project_root().unwrap();
    let pattern = root.join("benchmarks/*.yml");
    glob(pattern.to_str().unwrap())
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .filter(|path| {
            let path_str = path.to_str().unwrap();
            !path_str.ends_with("003-spl-transfer-fail.yml")
        })
        .collect()
}

/// An integration test that runs against every solvable benchmark file.
///
/// This test is parameterized by the `find_benchmark_files` function, which
/// provides the path to each benchmark file. The test asserts that a "perfect"
/// agent action results in a score of 1.0 for every benchmark, confirming their validity.
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn test_all_benchmarks_are_solvable(
    #[values(find_benchmark_files())] benchmark_paths: Vec<PathBuf>,
) -> Result<()> {
    for benchmark_path in benchmark_paths {
        // Initialize tracing for this test to ensure logs are captured when using `--nocapture`.
        let _ = tracing_subscriber::fmt::try_init();

        info!(
            "--- Starting test for benchmark: {} ---",
            benchmark_path.display()
        );

        // 1. Set up the environment using the unified setup function.
        // All complex logic is now handled within `SolanaEnv::reset`.
        let (mut env, test_case, initial_observation) = setup_env_for_benchmark(&benchmark_path)?;
        info!("✅ Environment setup complete for {}", test_case.id);

        // 2. Create the "perfect" action for this benchmark.
        let instruction = mock_perfect_instruction(&test_case, &initial_observation.key_map)?;
        let action = AgentAction(instruction);
        info!("✅ Mock instruction created for {}", test_case.id);

        // 3. Execute the transaction in the environment.
        let step_result = env.step(action, &test_case.ground_truth)?;
        info!(
            "✅ Transaction executed for {}. Status: {}",
            test_case.id, step_result.observation.last_transaction_status
        );

        // 4. Calculate the score.
        let score = calculate_score(&test_case, &initial_observation, &step_result.observation);
        info!(
            "📊 Calculated score for '{}': {}",
            benchmark_path.display(),
            score
        );

        // 5. Assert that the benchmark is solvable with a perfect score.
        assert_eq!(
            score,
            1.0,
            "Benchmark '{}' should be solvable with a perfect score, but got {}",
            benchmark_path.display(),
            score
        );
        env.close()?;
    }

    Ok(())
}
