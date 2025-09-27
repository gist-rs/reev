//! # Benchmark Sanity Check Test
//!
//! This test file ensures that all defined benchmarks in the `benchmarks/`
//! directory are valid and "solvable" by a perfect agent.
//!
//! It dynamically discovers every `.yml` file and runs a test for each one.
//! For each benchmark, it:
//! 1. Sets up the initial on-chain state.
//! 2. Generates the "perfect" instruction required to solve the task.
//! 3. Executes the instruction in the simulated environment.
//! 4. Calculates the score based on the benchmark's ground truth assertions.
//! 5. Asserts that the final score is 1.0.
//!
//! This acts as a critical integration test. If any benchmark file has a configuration
//! that makes it impossible to achieve a perfect score, this test will fail,
//! alerting us to a problem with the benchmark's definition, not the agent's logic.

mod common;

use anyhow::Result;
use glob::glob;
use project_root::get_project_root;
use reev_lib::{agent::AgentAction, env::GymEnv, score::calculate_score};
use rstest::rstest;
use std::path::PathBuf;

use common::{mock_perfect_instruction, setup_env_for_benchmark};

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
            !path
                .to_str()
                .unwrap()
                .ends_with("003-spl-transfer-fail.yml")
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
        // 1. Set up the environment from the benchmark file.
        let (mut env, test_case, initial_observation) = setup_env_for_benchmark(&benchmark_path)?;

        // 2. Create the "perfect" action for this benchmark.
        let instruction = mock_perfect_instruction(&test_case, &initial_observation.key_map)?;
        let action = AgentAction(instruction);

        // 3. Execute the transaction in the environment.
        let step_result = env.step(action, &test_case.ground_truth)?;

        // 4. Calculate the score.
        let score = calculate_score(&test_case, &initial_observation, &step_result.observation);

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
