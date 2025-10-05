//! # Deterministic Agent Integration Tests
//!
//! This test file contains deterministic tests that validate the core functionality
//! of the reev framework without relying on external LLM services. These tests use
//! predefined instructions and mock perfect actions to ensure the system works
//! correctly end-to-end.
//!
//! The tests follow the same pattern as benchmarks_test.rs, using rstest to
//! dynamically generate test cases for each benchmark file.

#[path = "common/mod.rs"]
mod common;

use anyhow::Result;
use glob::glob;
use project_root::get_project_root;
use reev_lib::{agent::AgentAction, env::GymEnv, score::calculate_final_score};
use rstest::rstest;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

use common::helpers::{
    mock_perfect_instruction, prepare_jupiter_lend_deposit, prepare_jupiter_lend_deposit_usdc,
    prepare_jupiter_lend_withdraw_sol, prepare_jupiter_lend_withdraw_usdc, prepare_jupiter_swap,
    setup_env_for_benchmark,
};

const AGENT_PORT: u16 = 9090;

/// Kill any existing reev-agent process on port 9090
async fn kill_existing_reev_agent() -> Result<()> {
    info!(
        "🧹 Checking for existing reev-agent processes on port {}...",
        AGENT_PORT
    );

    // Try to kill any process using port 9090
    match Command::new("lsof")
        .args(["-ti", &format!(":{AGENT_PORT}")])
        .output()
    {
        Ok(output) => {
            let pids = String::from_utf8_lossy(&output.stdout);
            if !pids.trim().is_empty() {
                info!("🔪 Found existing reev-agent processes: {}", pids.trim());
                for pid in pids.trim().lines() {
                    match Command::new("kill").args(["-9", pid.trim()]).output() {
                        Ok(_) => {
                            info!("✅ Killed process {}", pid.trim());
                        }
                        Err(e) => {
                            warn!("⚠️  Failed to kill process {}: {}", pid.trim(), e);
                        }
                    }
                }
                // Give processes time to terminate
                sleep(Duration::from_millis(500)).await;
            } else {
                info!("✅ No existing reev-agent processes found");
            }
        }
        Err(e) => {
            warn!("⚠️  Failed to check for existing processes: {}", e);
        }
    }

    Ok(())
}

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

/// An integration test that validates all benchmarks with perfect instructions.
///
/// This test is parameterized by the `find_benchmark_files` function, which
/// provides the path to each benchmark file. The test asserts that a "perfect"
/// agent action results in a score of 1.0 for every benchmark, confirming their validity.
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn test_all_benchmarks_with_deterministic_agent(
    #[values(find_benchmark_files())] benchmark_paths: Vec<PathBuf>,
) -> Result<()> {
    // Initialize tracing for this test to ensure logs are captured when using `--nocapture`.
    let _ = tracing_subscriber::fmt::try_init();

    // Clean up any existing reev-agent processes before starting
    kill_existing_reev_agent().await?;

    for benchmark_path in benchmark_paths {
        info!(
            "🧪 Starting deterministic test for benchmark: {}",
            benchmark_path.display()
        );

        // 1. Set up the environment using the unified setup function.
        let (mut env, test_case, initial_observation) =
            setup_env_for_benchmark(&benchmark_path).await?;
        info!("✅ Environment setup complete for {}", test_case.id);

        // 2. Get the "perfect" action for this benchmark and execute.
        match test_case.id.as_str() {
            "112-JUP-LEND-WITHDRAW-SOL" => {
                info!("[Test] Jupiter SOL lend deposit-withdraw benchmark detected (3-step).");

                // --- Step 1: Deposit ---
                info!("Preparing Jupiter deposit (step 1)...");
                let deposit_instructions =
                    prepare_jupiter_lend_deposit(&env, &test_case, &initial_observation.key_map)
                        .await?;
                let deposit_actions: Vec<AgentAction> =
                    deposit_instructions.into_iter().map(AgentAction).collect();
                info!("Executing Jupiter deposit (step 1)...");
                let deposit_step_result =
                    env.step(deposit_actions.clone(), &test_case.ground_truth)?;
                info!("✅ Deposit complete.");

                // --- Step 2 & 3: Withdraw & Unwrap ---
                info!("Preparing Jupiter withdrawal (step 2 & 3)...");
                let (withdraw_instructions, unwrap_instruction) =
                    prepare_jupiter_lend_withdraw_sol(
                        &env,
                        &test_case,
                        &deposit_step_result.observation.key_map,
                    )
                    .await?;

                // Step 2: Execute Jupiter withdrawal
                let withdraw_actions: Vec<AgentAction> =
                    withdraw_instructions.into_iter().map(AgentAction).collect();
                info!("Executing Jupiter withdrawal (step 2)...");
                let _ = env.step(withdraw_actions.clone(), &test_case.ground_truth)?;
                info!("✅ Withdrawal complete.");

                // Step 3: Execute unwrap
                let unwrap_actions = vec![AgentAction(unwrap_instruction)];
                info!("Executing WSOL unwrap (step 3)...");
                let final_step_result =
                    env.step(unwrap_actions.clone(), &test_case.ground_truth)?;
                info!("✅ Unwrap complete.");

                // Score based on the final state after all steps.
                let all_actions = [deposit_actions, withdraw_actions, unwrap_actions].concat();
                let score = calculate_final_score(
                    &test_case,
                    &all_actions,
                    &initial_observation,
                    &final_step_result.observation,
                );
                info!(
                    "📊 Calculated score for '{}': {}",
                    benchmark_path.display(),
                    score
                );
                assert_eq!(
                    score,
                    1.0,
                    "Benchmark '{}' should be solvable with a perfect score, but got {}",
                    benchmark_path.display(),
                    score
                );
            }
            _ => {
                // Standard 1-step logic for all other benchmarks.
                let instructions = match test_case.id.as_str() {
                    "100-JUP-SWAP-SOL-USDC" => {
                        info!("[Test] Jupiter swap benchmark detected. Preparing environment...");
                        prepare_jupiter_swap(&env, &test_case, &initial_observation.key_map).await?
                    }
                    "110-JUP-LEND-DEPOSIT-SOL" => {
                        info!(
                            "[Test] Jupiter SOL lend benchmark detected. Preparing environment..."
                        );
                        prepare_jupiter_lend_deposit(&env, &test_case, &initial_observation.key_map)
                            .await?
                    }
                    "111-JUP-LEND-DEPOSIT-USDC" => {
                        info!(
                            "[Test] Jupiter USDC lend deposit benchmark detected. Preparing environment..."
                        );
                        prepare_jupiter_lend_deposit_usdc(
                            &env,
                            &test_case,
                            &initial_observation.key_map,
                        )
                        .await?
                    }
                    "113-JUP-LEND-WITHDRAW-USDC" => {
                        info!(
                            "[Test] Jupiter USDC lend withdraw benchmark detected. Preparing environment..."
                        );
                        prepare_jupiter_lend_withdraw_usdc(
                            &env,
                            &test_case,
                            &initial_observation.key_map,
                        )
                        .await?
                    }
                    _ => {
                        info!("[Test] Simple benchmark detected. Creating mock instruction...");
                        let instruction =
                            mock_perfect_instruction(&test_case, &initial_observation.key_map)?;
                        vec![instruction]
                    }
                };

                let actions: Vec<AgentAction> = instructions.into_iter().map(AgentAction).collect();
                let step_result = env.step(actions.clone(), &test_case.ground_truth)?;
                let score = calculate_final_score(
                    &test_case,
                    &actions,
                    &initial_observation,
                    &step_result.observation,
                );
                info!(
                    "📊 Calculated score for '{}': {}",
                    benchmark_path.display(),
                    score
                );
                assert_eq!(
                    score,
                    1.0,
                    "Benchmark '{}' should be solvable with a perfect score, but got {}",
                    benchmark_path.display(),
                    score
                );
            }
        }
        env.close()?;
    }

    Ok(())
}

/// Test deterministic agent on Jupiter Swap benchmark (complex DeFi task)
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn test_deterministic_agent_jupiter_swap_integration(
    #[values(find_benchmark_files())] benchmark_paths: Vec<PathBuf>,
) -> Result<()> {
    // Find the Jupiter swap benchmark
    let jupiter_swap_path = benchmark_paths
        .iter()
        .find(|path| path.to_str().unwrap().contains("100-jup-swap-sol-usdc"))
        .expect("Jupiter swap benchmark not found");

    let _ = tracing_subscriber::fmt::try_init();
    info!("🧪 Starting deterministic agent integration test for Jupiter Swap");

    // Clean up any existing reev-agent processes before starting
    kill_existing_reev_agent().await?;

    // Set up environment
    let (mut env, test_case, initial_observation) =
        setup_env_for_benchmark(jupiter_swap_path).await?;
    info!("✅ Environment setup complete for {}", test_case.id);

    // Prepare Jupiter swap
    info!("🔄 Preparing Jupiter swap with 0.1 SOL...");
    let instructions = prepare_jupiter_swap(&env, &test_case, &initial_observation.key_map).await?;
    let actions: Vec<AgentAction> = instructions.into_iter().map(AgentAction).collect();

    // Execute the swap
    info!("⚡ Executing Jupiter swap...");
    let step_result = env.step(actions.clone(), &test_case.ground_truth)?;
    info!("✅ Jupiter swap executed successfully");

    // Calculate and assert score
    let score = calculate_final_score(
        &test_case,
        &actions,
        &initial_observation,
        &step_result.observation,
    );
    info!("📊 Final score: {}", score);
    assert_eq!(
        score, 1.0,
        "Jupiter swap should achieve perfect score, got {score}"
    );

    env.close()?;
    info!("🎉 Jupiter swap deterministic test completed successfully!");
    Ok(())
}
