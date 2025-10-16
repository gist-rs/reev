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
//! For simple benchmarks, it uses a "mock perfect" instruction. For complex
//! DeFi benchmarks like Jupiter swaps, it now uses a "smart test" approach where
//! the test itself calls the Jupiter API and pre-loads all necessary accounts
//! into the forked environment before execution, mimicking a perfect agent's setup.

#[path = "common/mod.rs"]
mod common;

use anyhow::{Context, Result, anyhow};
use glob::glob;
use project_root::get_project_root;
use reev_lib::{agent::AgentAction, env::GymEnv, score::calculate_final_score};
use rstest::rstest;
use std::{
    fs,
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::Duration,
};
use tokio::time::sleep;
use tracing::{error, info};

use common::helpers::{
    mock_perfect_instruction, prepare_jupiter_lend_deposit, prepare_jupiter_lend_deposit_usdc,
    prepare_jupiter_lend_withdraw_sol, prepare_jupiter_lend_withdraw_usdc, prepare_jupiter_swap,
    setup_env_for_benchmark,
};

/// RAII guard to ensure reev-agent process is killed after test
struct AgentProcessGuard {
    process: Child,
}

impl Drop for AgentProcessGuard {
    fn drop(&mut self) {
        info!("🧹 Shutting down reev-agent for test...");
        if let Err(e) = self.process.kill() {
            error!(error = ?e, "Failed to kill reev-agent process");
        }
    }
}

/// Start reev-agent process which hosts surfpool RPC server
async fn start_agent_for_test() -> Result<AgentProcessGuard> {
    let log_dir = PathBuf::from("logs");
    fs::create_dir_all(&log_dir)?;
    let log_file_path = log_dir.join("reev-agent-benchmark-test.log");
    let log_file = fs::File::create(&log_file_path)?;
    let stderr_log = log_file.try_clone()?;

    info!("🚀 Starting reev-agent for benchmark test...");
    let agent_process = Command::new("cargo")
        .args(["run", "--package", "reev-agent"])
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(stderr_log))
        .spawn()
        .context("Failed to spawn reev-agent process")?;

    let guard = AgentProcessGuard {
        process: agent_process,
    };

    // Health check - wait for agent to be ready
    let client = reqwest::Client::new();
    let health_check_url = "http://127.0.0.1:9090/health";
    for i in 0..30 {
        if let Ok(response) = client.get(health_check_url).send().await {
            if response.status().is_success() {
                info!("✅ reev-agent is healthy after {} attempts", i + 1);
                return Ok(guard);
            }
        }
        sleep(Duration::from_secs(1)).await;
    }

    Err(anyhow!(
        "Timed out waiting for reev-agent to become healthy"
    ))
}

/// Check if surfpool is available
async fn check_surfpool_available() -> Result<()> {
    let client = reqwest::Client::new();
    let rpc_url = "http://127.0.0.1:8899";

    for i in 0..15 {
        if let Ok(response) = client
            .post(rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getHealth"
            }))
            .send()
            .await
        {
            if response.status().is_success() {
                info!("✅ surfpool is available after {} attempts", i + 1);
                return Ok(());
            }
        }
        sleep(Duration::from_secs(1)).await;
    }

    Err(anyhow!("surfpool is not available"))
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
        .collect()
}

/// An integration test that runs against every solvable benchmark file.
///
/// This test is parameterized by the `find_benchmark_files` function, which
/// provides the path to each benchmark file. The test asserts that a "perfect"
/// agent action results in a score of 1.0 for every benchmark, confirming their validity.
///
/// Note: This test requires reev-agent and surfpool to be available. It will start
/// reev-agent automatically if needed and skip if surfpool cannot be reached.
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn test_all_benchmarks_are_solvable(
    #[values(find_benchmark_files())] benchmark_paths: Vec<PathBuf>,
) -> Result<()> {
    // Start reev-agent if not already running
    let _agent_guard = match start_agent_for_test().await {
        Ok(guard) => {
            info!("✅ Started reev-agent for test");
            Some(guard)
        }
        Err(e) => {
            info!(
                "⚠️ Could not start reev-agent: {}, checking if already running",
                e
            );
            None
        }
    };

    // Wait a bit for surfpool to be ready
    sleep(Duration::from_secs(3)).await;

    // Check if surfpool is available
    if let Err(e) = check_surfpool_available().await {
        info!("⚠️ Skipping benchmark test - surfpool not available: {}", e);
        info!("💡 To run this test, ensure surfpool is running or start reev-agent first");
        return Ok(());
    }
    for benchmark_path in benchmark_paths {
        // Initialize tracing for this test to ensure logs are captured when using `--nocapture`.
        let _ = tracing_subscriber::fmt::try_init();

        info!(
            "--- Starting test for benchmark: {} ---",
            benchmark_path.display()
        );

        // 1. Set up the environment using the unified setup function.
        let (mut env, test_case, initial_observation) =
            setup_env_for_benchmark(&benchmark_path).await?;
        info!("✅ Environment setup complete for {}", test_case.id);

        // 2. Get the "perfect" action for this benchmark and execute.
        if test_case.id == "112-JUP-LEND-WITHDRAW-SOL" {
            info!("[Test] Jupiter SOL lend deposit-withdraw benchmark detected (3-step).");

            // --- Step 1: Deposit ---
            info!("Preparing Jupiter deposit (step 1)...");
            let deposit_instructions =
                prepare_jupiter_lend_deposit(&env, &test_case, &initial_observation.key_map)
                    .await?;
            let deposit_actions: Vec<AgentAction> =
                deposit_instructions.into_iter().map(AgentAction).collect();
            info!("Executing Jupiter deposit (step 1)...");
            let deposit_step_result = env.step(deposit_actions.clone(), &test_case.ground_truth)?;
            info!("✅ Deposit complete.");

            // --- Step 2 & 3: Withdraw & Unwrap ---
            info!("Preparing Jupiter withdrawal (step 2 & 3)...");
            let (withdraw_instructions, unwrap_instruction) = prepare_jupiter_lend_withdraw_sol(
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
            let final_step_result = env.step(unwrap_actions.clone(), &test_case.ground_truth)?;
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
        } else {
            // Standard 1-step logic for all other benchmarks.
            let instructions = match test_case.id.as_str() {
                "100-JUP-SWAP-SOL-USDC" => {
                    info!("[Test] Jupiter swap benchmark detected. Preparing environment...");
                    prepare_jupiter_swap(&env, &test_case, &initial_observation.key_map).await?
                }
                "110-JUP-LEND-DEPOSIT-SOL" => {
                    info!("[Test] Jupiter SOL lend benchmark detected. Preparing environment...");
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
                    vec![mock_perfect_instruction(
                        &test_case,
                        &initial_observation.key_map,
                    )?]
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
            if test_case.id == "003-SPL-TRANSFER-FAIL" {
                assert_eq!(
                    score,
                    0.75,
                    "Benchmark '{}' should fail on-chain but have a perfect instruction score, but got {}",
                    benchmark_path.display(),
                    score
                );
            } else {
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
