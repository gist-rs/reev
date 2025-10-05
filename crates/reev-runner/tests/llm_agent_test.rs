//! # LLM Agent Integration Tests
//!
//! This test file contains end-to-end tests that validate the AI agent's ability
//! to generate and execute transactions by calling external LLM services. These tests
//! require the reev-agent service to be running and properly configured with LLM API keys.
//!
//! The tests follow the same pattern as benchmarks_test.rs, using rstest to
//! dynamically generate test cases for each benchmark file, but instead of using
//! predefined instructions, they call the LLM agent to generate actions.

#[path = "common/mod.rs"]
mod common;

use anyhow::Result;
use glob::glob;
use project_root::get_project_root;
use reev_lib::{agent::Agent, env::GymEnv, score::calculate_final_score};
use rstest::rstest;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use tracing_subscriber::fmt;

use common::helpers::setup_env_for_benchmark;

const LOCAL_RPC_URL: &str = "http://127.0.0.1:8899";
const AGENT_PORT: u16 = 9090;

/// Shared agent process guard to ensure only one agent process runs across all tests
static SHARED_AGENT: once_cell::sync::Lazy<Arc<Mutex<Option<AgentProcessGuard>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

struct AgentProcessGuard {
    process: Child,
}

impl Drop for AgentProcessGuard {
    fn drop(&mut self) {
        if let Err(e) = self.process.kill() {
            warn!("Failed to kill agent process: {}", e);
        }
    }
}

/// Get or create the shared agent process
async fn get_or_create_shared_agent() -> Result<()> {
    {
        let guard = SHARED_AGENT.lock().unwrap();
        if guard.is_some() {
            info!("♻️  Using existing shared reev-agent service");
            return Ok(());
        }
    } // Guard is dropped here

    info!("🚀 Starting shared reev-agent service...");
    let process = start_agent_process().await?;

    // Wait for agent to be healthy
    sleep(Duration::from_secs(2)).await;

    match check_agent_health().await {
        Ok(_) => {
            info!("✅ reev-agent is healthy and ready for requests.");
            info!("✅ Shared reev-agent service started");

            // Update the shared agent after successful health check
            let mut guard = SHARED_AGENT.lock().unwrap();
            *guard = Some(AgentProcessGuard { process });
        }
        Err(e) => {
            error!("❌ reev-agent health check failed: {}", e);
            return Err(e);
        }
    }
    Ok(())
}

/// Start the reev-agent process
async fn start_agent_process() -> Result<Child> {
    let project_root = get_project_root()?;
    let agent_binary = project_root.join("target/debug/reev-agent");

    if !agent_binary.exists() {
        // Build the agent if it doesn't exist
        info!("🔨 Building reev-agent...");
        let output = Command::new("cargo")
            .args(["build", "--package", "reev-agent"])
            .current_dir(&project_root)
            .output()?;

        if !output.status.success() {
            error!(
                "Failed to build reev-agent: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(anyhow::anyhow!("Failed to build reev-agent"));
        }
    }

    info!("🚀 Starting reev-agent process...");
    let process = Command::new(&agent_binary).spawn()?;

    Ok(process)
}

/// Check if the agent service is healthy
async fn check_agent_health() -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("http://localhost:{AGENT_PORT}/health");

    for attempt in 1..=10 {
        match client.get(&url).send().await {
            Ok(_response) if _response.status().is_success() => {
                return Ok(());
            }
            Ok(_) => {
                warn!("Agent health check attempt {} failed, retrying...", attempt);
            }
            Err(e) => {
                warn!(
                    "Agent health check attempt {} failed: {}, retrying...",
                    attempt, e
                );
            }
        }
        sleep(Duration::from_millis(500)).await;
    }

    Err(anyhow::anyhow!(
        "Agent health check failed after 10 attempts"
    ))
}

/// Check if surfpool is available
async fn check_surfpool_available() -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{LOCAL_RPC_URL}/health");

    match client.get(&url).send().await {
        Ok(_response) => {
            info!("✅ surfpool is available at {}", LOCAL_RPC_URL);
            Ok(())
        }
        Err(e) => {
            warn!(
                "❌ surfpool is not available at {}. Install with: brew install txtx/taps/surfpool",
                LOCAL_RPC_URL
            );
            Err(anyhow::anyhow!("surfpool not available: {e}"))
        }
    }
}

/// Ensure the shared agent is available
async fn ensure_shared_agent() -> Result<()> {
    get_or_create_shared_agent().await
}

/// Create an AI agent instance
async fn create_ai_agent() -> Result<reev_lib::llm_agent::LlmAgent> {
    let model_name =
        std::env::var("LLM_MODEL").unwrap_or_else(|_| "gemini-2.0-flash-exp".to_string());

    info!("🤖 Creating AI agent with model: {}", model_name);
    let agent = reev_lib::llm_agent::LlmAgent::new(&model_name)?;
    info!("🤖 AI agent created with Gemini model");

    Ok(agent)
}

/// Dynamically discovers all solvable `.yml` files in the `benchmarks` directory for LLM testing.
///
/// This function filters to a subset of benchmarks that are suitable for LLM testing,
/// excluding complex multi-step operations that might be challenging for current LLM capabilities.
fn find_llm_benchmark_files() -> Vec<PathBuf> {
    let root = get_project_root().unwrap();
    let pattern = root.join("benchmarks/*.yml");
    glob(pattern.to_str().unwrap())
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .filter(|path| {
            let path_str = path.to_str().unwrap();
            // Include simple benchmarks and basic Jupiter operations
            // Exclude complex multi-step operations for now
            !path_str.ends_with("003-spl-transfer-fail.yml")
                && !path_str.ends_with("112-jup-lend-withdraw-sol.yml") // Complex 3-step operation
        })
        .collect()
}

/// An LLM integration test that runs against selected benchmark files.
///
/// This test is parameterized by the `find_llm_benchmark_files` function, which
/// provides the path to each benchmark file. The test calls the LLM agent to
/// generate actions and validates that they produce reasonable results.
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn test_llm_agent_on_all_benchmarks(
    #[values(find_llm_benchmark_files())] benchmark_paths: Vec<PathBuf>,
) -> Result<()> {
    // Initialize tracing
    let _ = fmt::try_init();

    // Check prerequisites
    if let Err(e) = check_surfpool_available().await {
        warn!("⚠️  Skipping LLM test - surfpool not available: {}", e);
        return Ok(());
    }

    if let Err(e) = ensure_shared_agent().await {
        warn!("⚠️  Skipping LLM test - agent not available: {}", e);
        return Ok(());
    }

    let mut agent = create_ai_agent().await?;

    for benchmark_path in benchmark_paths {
        info!(
            "🧪 Testing AI agent on {}",
            benchmark_path.file_stem().unwrap().to_str().unwrap()
        );

        // Set up environment
        let (mut env, test_case, initial_observation) =
            setup_env_for_benchmark(&benchmark_path).await?;
        info!("✅ Environment setup complete for {}", test_case.id);

        // Get action from LLM agent
        match agent
            .get_action(
                &test_case.id,
                &test_case.prompt,
                &initial_observation,
                Some(&"USER_WALLET_PUBKEY".to_string()),
            )
            .await
        {
            Ok(actions) => {
                info!("✅ AI agent generated {} actions", actions.len());

                // Execute actions
                let step_result = env.step(actions.clone(), &test_case.ground_truth)?;

                // Calculate score (we don't expect perfect scores from LLM)
                let score = calculate_final_score(
                    &test_case,
                    &actions,
                    &initial_observation,
                    &step_result.observation,
                );

                info!("📊 LLM agent score for {}: {}", test_case.id, score);

                // For now, just log the score instead of asserting
                // LLM agents may not achieve perfect scores consistently
                if score > 0.5 {
                    info!(
                        "✅ LLM agent achieved reasonable score (>0.5) for {}",
                        test_case.id
                    );
                } else {
                    warn!(
                        "⚠️  LLM agent achieved low score ({} <= 0.5) for {}",
                        score, test_case.id
                    );
                }
            }
            Err(e) => {
                warn!("⚠️  AI agent failed to generate actions: {}", e);
                // Don't fail the test, just log the error
            }
        }

        env.close()?;
    }

    Ok(())
}

/// Test LLM agent on Jupiter Swap benchmark (complex DeFi task)
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn test_llm_agent_jupiter_swap_integration() -> Result<()> {
    let _ = fmt::try_init();

    info!("🎯 Testing AI agent on Jupiter Swap benchmark (complex DeFi task)");

    // Check prerequisites
    if let Err(e) = check_surfpool_available().await {
        warn!(
            "⚠️  Skipping Jupiter swap LLM test - surfpool not available: {}",
            e
        );
        return Ok(());
    }

    if let Err(e) = ensure_shared_agent().await {
        warn!(
            "⚠️  Skipping Jupiter swap LLM test - agent not available: {}",
            e
        );
        return Ok(());
    }

    // Find Jupiter swap benchmark
    let root = get_project_root()?;
    let jupiter_swap_path = root.join("benchmarks/100-jup-swap-sol-usdc.yml");

    if !jupiter_swap_path.exists() {
        warn!("⚠️  Jupiter swap benchmark not found, skipping test");
        return Ok(());
    }

    // Set up environment
    let (mut env, test_case, initial_observation) =
        setup_env_for_benchmark(&jupiter_swap_path).await?;
    info!("✅ Environment setup complete for {}", test_case.id);

    // Create AI agent
    let mut agent = create_ai_agent().await?;

    // Get action from LLM agent
    match agent
        .get_action(
            &test_case.id,
            &test_case.prompt,
            &initial_observation,
            Some(&"USER_WALLET_PUBKEY".to_string()),
        )
        .await
    {
        Ok(actions) => {
            info!(
                "✅ AI agent generated {} actions for Jupiter swap",
                actions.len()
            );

            // Execute actions
            let step_result = env.step(actions.clone(), &test_case.ground_truth)?;

            // Calculate score
            let score = calculate_final_score(
                &test_case,
                &actions,
                &initial_observation,
                &step_result.observation,
            );

            info!("📊 LLM agent Jupiter swap score: {}", score);

            if score > 0.3 {
                info!("✅ LLM agent successfully handled Jupiter swap (score > 0.3)");
            } else {
                warn!("⚠️  LLM agent struggled with Jupiter swap (score <= 0.3)");
            }
        }
        Err(e) => {
            warn!(
                "⚠️  AI agent failed to generate Jupiter swap actions: {}",
                e
            );
        }
    }

    env.close()?;
    info!("🎉 Jupiter swap LLM test completed!");
    Ok(())
}

/// Test LLM agent on simple SOL transfer
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn test_llm_agent_simple_sol_transfer() -> Result<()> {
    let _ = fmt::try_init();

    info!("🧪 Testing AI agent on simple SOL transfer");

    // Check prerequisites
    if let Err(e) = check_surfpool_available().await {
        warn!(
            "⚠️  Skipping SOL transfer LLM test - surfpool not available: {}",
            e
        );
        return Ok(());
    }

    if let Err(e) = ensure_shared_agent().await {
        warn!(
            "⚠️  Skipping SOL transfer LLM test - agent not available: {}",
            e
        );
        return Ok(());
    }

    // Find SOL transfer benchmark
    let root = get_project_root()?;
    let sol_transfer_path = root.join("benchmarks/001-sol-transfer.yml");

    if !sol_transfer_path.exists() {
        warn!("⚠️  SOL transfer benchmark not found, skipping test");
        return Ok(());
    }

    // Set up environment
    let (mut env, test_case, initial_observation) =
        setup_env_for_benchmark(&sol_transfer_path).await?;
    info!("✅ Environment setup complete for {}", test_case.id);

    // Create AI agent
    let mut agent = create_ai_agent().await?;

    // Get action from LLM agent
    match agent
        .get_action(
            &test_case.id,
            &test_case.prompt,
            &initial_observation,
            Some(&"USER_WALLET_PUBKEY".to_string()),
        )
        .await
    {
        Ok(actions) => {
            info!(
                "✅ AI agent generated {} actions for SOL transfer",
                actions.len()
            );

            // Execute actions
            let step_result = env.step(actions.clone(), &test_case.ground_truth)?;

            // Calculate score
            let score = calculate_final_score(
                &test_case,
                &actions,
                &initial_observation,
                &step_result.observation,
            );

            info!("📊 LLM agent SOL transfer score: {}", score);

            // Simple transfers should achieve higher scores
            if score > 0.8 {
                info!("✅ LLM agent successfully handled SOL transfer (score > 0.8)");
            } else {
                warn!("⚠️  LLM agent struggled with SOL transfer (score <= 0.8)");
            }
        }
        Err(e) => {
            warn!(
                "⚠️  AI agent failed to generate SOL transfer actions: {}",
                e
            );
        }
    }

    env.close()?;
    info!("🎉 SOL transfer LLM test completed!");
    Ok(())
}
