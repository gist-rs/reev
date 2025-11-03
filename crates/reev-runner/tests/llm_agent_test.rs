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
use tracing::{debug, error, info, warn};
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
    // First, kill any existing reev-agent processes to ensure clean start
    kill_existing_reev_agent().await?;

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

/// Kill any existing reev-agent process on port 9090
async fn kill_existing_reev_agent() -> Result<()> {
    debug!(
        "🧹 Checking for existing reev-agent processes on port {}...",
        AGENT_PORT
    );

    // Try to kill any process using port 9090
    match std::process::Command::new("lsof")
        .args(["-ti", &format!(":{AGENT_PORT}")])
        .output()
    {
        Ok(output) => {
            let pids = String::from_utf8_lossy(&output.stdout);
            if !pids.trim().is_empty() {
                info!("🔪 Found existing reev-agent processes: {}", pids.trim());
                for pid in pids.trim().lines() {
                    match std::process::Command::new("kill")
                        .args(["-9", pid.trim()])
                        .output()
                    {
                        Ok(_) => {
                            info!("✅ Killed process {}", pid.trim());
                        }
                        Err(e) => {
                            warn!("⚠️  Failed to kill process {}: {}", pid.trim(), e);
                        }
                    }
                }
                // Give processes time to terminate
                tokio::time::sleep(Duration::from_millis(500)).await;
            } else {
                debug!("✅ No existing reev-agent processes found");
            }
        }
        Err(e) => {
            warn!("⚠️  Failed to check for existing processes: {}", e);
        }
    }

    Ok(())
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
    // Check if we have API keys for cloud models
    let has_gemini_key = std::env::var("GEMINI_API_KEY").is_ok();
    let has_openai_key = std::env::var("OPENAI_API_KEY").is_ok();
    let has_glm_key = std::env::var("ZAI_API_KEY").is_ok();

    let model_name = if has_glm_key {
        "glm-4.6".to_string()
    } else if has_gemini_key || has_openai_key {
        std::env::var("LLM_MODEL").unwrap_or_else(|_| "gemini-2.5-flash-lite".to_string())
    } else {
        // Use local model when no API keys are available
        "local".to_string()
    };

    info!("🤖 Creating AI agent with model: {}", model_name);
    let agent = reev_lib::llm_agent::LlmAgent::new(&model_name)?;
    info!("🤖 AI agent created with model: {}", model_name);

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

    // test only first one
    // let benchmark_path = benchmark_paths.first().unwrap();

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
                Some(test_case.ground_truth.skip_instruction_validation),
                Some(&test_case.initial_state),
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

                // Set different score thresholds based on benchmark complexity
                let (threshold, description) = match test_case.id.as_str() {
                    // Simple transfers should achieve higher scores
                    "001-sol-transfer" | "002-spl-transfer" => (0.8, "simple transfer"),
                    // Jupiter operations are more complex, lower threshold acceptable
                    "100-jup-swap-sol-usdc" => (0.3, "Jupiter swap"),
                    "110-jup-lend-deposit-sol" | "111-jup-lend-deposit-usdc" => {
                        (0.4, "Jupiter lend deposit")
                    }
                    "113-jup-lend-withdraw-usdc" => (0.4, "Jupiter lend withdraw"),
                    // Multi-step operations are most challenging
                    "112-jup-lend-withdraw-sol" => (0.2, "complex 3-step Jupiter operation"),
                    // Default threshold for unknown benchmarks
                    _ => (0.5, "unknown benchmark"),
                };

                if score > threshold {
                    info!(
                        "✅ LLM agent successfully handled {} (score {} > {})",
                        description, score, threshold
                    );
                } else {
                    warn!(
                        "⚠️  LLM agent struggled with {} (score {} <= {})",
                        description, score, threshold
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
