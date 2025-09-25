use anyhow::{Context, Result, anyhow};
use clap::Parser;
use dotenvy::dotenv;
use opentelemetry::global::{self};
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace as sdktrace;
use reev_lib::{
    agent::{Agent, AgentObservation},
    benchmark::{StateAssertion, TestCase},
    env::GymEnv,
    llm_agent::LlmAgent,
    results::{FinalStatus, TestResult},
    solana_env::SolanaEnv,
    trace::ExecutionTrace,
};
use std::{
    fs,
    path::PathBuf,
    process::{Child, Command},
};
use tokio::time::{Duration, sleep};
use tracing::subscriber;
use tracing_subscriber::{EnvFilter, Registry, prelude::*};

mod db;
mod renderer;

/// A simple RAII guard to ensure the `reev-agent` process is killed.
struct AgentProcessGuard {
    process: Child,
}

impl Drop for AgentProcessGuard {
    fn drop(&mut self) {
        println!("[reev-runner] Shutting down reev-agent...");
        if let Err(e) = self.process.kill() {
            eprintln!("[reev-runner] Failed to kill reev-agent process: {e}");
        }
    }
}

/// A command-line runner for the Reev evaluation framework.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to a specific benchmark YAML file or a directory containing multiple benchmarks.
    #[arg(default_value = "benchmarks/")]
    path: PathBuf,
}

/// Initializes the OpenTelemetry pipeline for tracing.
fn init_tracing() -> Result<sdktrace::SdkTracerProvider> {
    let provider = sdktrace::SdkTracerProvider::builder()
        .with_resource(Resource::builder().with_service_name("reev-runner").build())
        .build();
    let tracer = provider.tracer("reev-runner");
    global::set_tracer_provider(provider.clone());

    let subscriber = Registry::default()
        .with(EnvFilter::new("info,reev_lib=debug,reev_runner=debug"))
        .with(tracing_opentelemetry::layer().with_tracer(tracer));

    subscriber::set_global_default(subscriber)
        .context("Failed to set global default tracing subscriber")?;

    Ok(provider)
}

/// Calculates the final score based on the ground truth assertions.
fn calculate_score(test_case: &TestCase, final_observation: &AgentObservation) -> f64 {
    println!("\n[SCORING] Calculating score based on on-chain state assertions...");
    for assertion in &test_case.ground_truth.final_state_assertions {
        match assertion {
            StateAssertion::SolBalance { pubkey, expected } => {
                // Resolve the placeholder to the actual pubkey from the observation's key_map for logging.
                let target_pubkey_str = final_observation
                    .key_map
                    .get(pubkey)
                    .cloned()
                    .unwrap_or_else(|| pubkey.clone());

                if let Some(account_state) = final_observation.account_states.get(pubkey) {
                    if let Some(actual_lamports) =
                        account_state.get("lamports").and_then(|v| v.as_u64())
                    {
                        if actual_lamports == *expected {
                            println!(
                                "      ✅ Assertion PASSED: SOL balance for '{pubkey}' ({target_pubkey_str}) is {expected}."
                            );
                        } else {
                            println!(
                                "      ❌ Assertion FAILED: SOL balance for '{pubkey}' ({target_pubkey_str}). Expected: {expected}, Actual: {actual_lamports}"
                            );
                            return 0.0;
                        }
                    } else {
                        println!(
                            "      ❌ Assertion FAILED: Could not read lamports for '{pubkey}' ({target_pubkey_str})."
                        );
                        return 0.0;
                    }
                } else if *expected == 0 {
                    // If the account doesn't exist and the expected balance is 0, it's a pass.
                    println!(
                        "      ✅ Assertion PASSED: Account '{pubkey}' ({target_pubkey_str}) does not exist, matching expected balance of 0."
                    );
                } else {
                    println!(
                        "      ❌ Assertion FAILED: Account '{pubkey}' ({target_pubkey_str}) not found in final state."
                    );
                    return 0.0;
                }
            }
            StateAssertion::TokenAccountBalance { pubkey, expected } => {
                // Resolve the placeholder to the actual pubkey from the observation's key_map for logging.
                let target_pubkey_str = final_observation
                    .key_map
                    .get(pubkey)
                    .cloned()
                    .unwrap_or_else(|| pubkey.clone());

                if let Some(account_state) = final_observation.account_states.get(pubkey) {
                    if let Some(actual_amount) =
                        account_state.get("amount").and_then(|v| v.as_u64())
                    {
                        if actual_amount == *expected {
                            println!(
                                "      ✅ Assertion PASSED: Token balance for '{pubkey}' ({target_pubkey_str}) is {expected}."
                            );
                        } else {
                            println!(
                                "      ❌ Assertion FAILED: Token balance for '{pubkey}' ({target_pubkey_str}). Expected: {expected}, Actual: {actual_amount}"
                            );
                            return 0.0;
                        }
                    } else {
                        println!(
                            "      ❌ Assertion FAILED: Could not read token amount for '{pubkey}' ({target_pubkey_str})."
                        );
                        return 0.0;
                    }
                } else if *expected == 0 {
                    // If the account doesn't exist and the expected balance is 0, it's a pass.
                    println!(
                        "      ✅ Assertion PASSED: Account '{pubkey}' ({target_pubkey_str}) does not exist, matching expected balance of 0."
                    );
                } else {
                    println!(
                        "      ❌ Assertion FAILED: Account '{pubkey}' ({target_pubkey_str}) not found in final state."
                    );
                    return 0.0;
                }
            }
            StateAssertion::SolBalanceChange { .. } => {
                unimplemented!("SolBalanceChange assertion not yet implemented.")
            }
        }
    }
    println!("      All on-chain assertions passed.");
    1.0
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from a .env file in the current directory.
    dotenv().ok();

    let tracer_provider = init_tracing()?;

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        if let Some(workspace_root) = std::path::Path::new(&manifest_dir)
            .parent()
            .and_then(|p| p.parent())
        {
            std::env::set_current_dir(workspace_root)?;
        }
    }

    let cli = Cli::parse();
    println!("--- Reev Evaluation Runner ---");

    // --- Start Mock Agent ---
    println!("[reev-runner] Starting mock reev-agent...");
    let agent_process = Command::new("cargo")
        .args(["run", "-p", "reev-agent"])
        .spawn()
        .context("Failed to spawn reev-agent process. Is `cargo` in your PATH?")?;

    let _agent_guard = AgentProcessGuard {
        process: agent_process,
    };

    // --- Health Check ---
    println!("[reev-runner] Waiting for mock agent to be healthy...");
    let client = reqwest::Client::new();
    let health_check_url = "http://127.0.0.1:9090/health";
    let mut attempts = 0;
    loop {
        if attempts >= 20 {
            // Timeout after 10 seconds
            anyhow::bail!("Timed out waiting for reev-agent to become healthy.");
        }
        match client.get(health_check_url).send().await {
            Ok(response) if response.status().is_success() => {
                println!("[reev-runner] Mock agent is healthy.");
                break;
            }
            _ => {
                attempts += 1;
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
    // --- Mock Agent Started ---

    // Discover benchmark files.
    let benchmark_paths = if cli.path.is_dir() {
        fs::read_dir(&cli.path)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if path.is_file() && (path.extension()? == "yml" || path.extension()? == "yaml")
                    {
                        Some(path)
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>()
    } else if cli.path.is_file() {
        vec![cli.path]
    } else {
        return Err(anyhow!("Provided path is not a valid file or directory"));
    };

    if benchmark_paths.is_empty() {
        println!("[INFO] No benchmark files found to run.");
        return Ok(());
    }

    // Initialize database
    let db = db::Db::new("db/reev_results.db").await?;

    // Loop through and run each benchmark
    for path in benchmark_paths {
        println!(
            "\n================== Running Benchmark: {} ==================",
            path.display()
        );
        let f = fs::File::open(&path)?;
        let test_case: TestCase = serde_yaml::from_reader(f)?;
        println!("[LOADED] Test case: '{}'", test_case.id);

        // Instantiate a fresh agent and environment for each run to ensure isolation.
        let mut agent = LlmAgent::new()?;
        let mut env = SolanaEnv::new()?;

        let (final_observation, trace) =
            run_evaluation_loop(&mut env, &mut agent, &test_case).await?;
        let score = calculate_score(&test_case, &final_observation);
        let final_status = if score == 1.0 {
            FinalStatus::Succeeded
        } else {
            FinalStatus::Failed
        };

        if let Some(step) = trace.steps.first() {
            db.insert_result(
                &test_case.id,
                &test_case.prompt,
                &step.action,
                &final_observation,
                final_status,
                score,
            )
            .await?;
        }

        let result = TestResult::new(&test_case, final_status, trace);
        let tree_output = renderer::render_result_as_tree(&result);
        println!("{tree_output}");

        env.close()?;
    }

    println!("\n--- All Benchmarks Finished ---");
    tracer_provider.shutdown()?;
    Ok(())
}

#[tracing::instrument(skip_all, fields(benchmark_id = %test_case.id))]
async fn run_evaluation_loop(
    env: &mut SolanaEnv,
    agent: &mut (dyn Agent + Send),
    test_case: &TestCase,
) -> Result<(AgentObservation, ExecutionTrace)> {
    let initial_state_json = serde_json::to_value(&test_case.initial_state)?;
    let options = serde_json::json!({ "initial_state": initial_state_json });
    let observation = env.reset(None, Some(options))?;
    env.render();

    let mut trace = ExecutionTrace::new(test_case.prompt.clone());

    // In this model, we expect one action leading to one transaction.
    let fee_payer = env.fee_payer_placeholder();
    let action = agent
        .get_action(&test_case.prompt, &observation, fee_payer)
        .await?;
    let step_result = env.step(action.clone(), &test_case.ground_truth)?;
    env.render();

    let trace_step = reev_lib::trace::TraceStep {
        thought: None,
        action,
        observation: step_result.observation.clone(),
        info: step_result.info,
    };
    trace.add_step(trace_step);

    println!("\n--- Episode Finished ---");
    Ok((step_result.observation, trace))
}
