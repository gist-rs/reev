use anyhow::{Context, Result, anyhow};
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
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};
use tokio::time::{Duration, sleep};
use tracing::{debug, error, info, instrument};

pub mod db;
pub mod renderer;

/// A simple RAII guard to ensure the `reev-agent` process is killed.
struct AgentProcessGuard {
    process: Child,
}

impl Drop for AgentProcessGuard {
    fn drop(&mut self) {
        info!("Shutting down reev-agent...");
        if let Err(e) = self.process.kill() {
            error!(error = ?e, "Failed to kill reev-agent process");
        }
    }
}

/// Starts the `reev-agent` process, redirects its output to a log file,
/// and performs a health check. Returns a guard that will kill the process
/// when it goes out of scope.
async fn start_agent() -> Result<AgentProcessGuard> {
    let log_dir = PathBuf::from("logs");
    fs::create_dir_all(&log_dir)?;
    let log_file_path = log_dir.join("reev-agent.log");
    let log_file = File::create(&log_file_path)?;
    let stderr_log = log_file.try_clone()?;

    info!(log_path = %log_file_path.display(), "Starting reev-agent...");

    info!("Building and running reev-agent from source...");
    let agent_process = Command::new("cargo")
        .args(["run", "--package", "reev-agent"])
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(stderr_log))
        .spawn()
        .context("Failed to spawn reev-agent process using 'cargo run'")?;

    let guard = AgentProcessGuard {
        process: agent_process,
    };

    info!("Waiting for reev-agent to be healthy...");
    let client = reqwest::Client::new();
    let health_check_url = "http://127.0.0.1:9090/health";
    let mut attempts = 0;
    loop {
        if attempts >= 20 {
            return Err(anyhow!(
                "Timed out waiting for reev-agent to become healthy."
            ));
        }
        match client.get(health_check_url).send().await {
            Ok(response) if response.status().is_success() => {
                info!("reev-agent is healthy.");
                break;
            }
            _ => {
                attempts += 1;
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    Ok(guard)
}

/// Runs all benchmarks found at the given path and returns the results.
pub async fn run_benchmarks(path: PathBuf, agent_name: &str) -> Result<Vec<TestResult>> {
    let benchmark_paths = discover_benchmarks(&path)?;
    if benchmark_paths.is_empty() {
        return Ok(vec![]);
    }

    // Start the reev-agent service. The `_agent_guard` will ensure it's
    // shut down when this function returns, keeping the service alive for all benchmarks.
    let _agent_guard = start_agent().await?;

    let db = db::Db::new("db/reev_results.db").await?;
    let mut results = vec![];

    for path in benchmark_paths {
        info!(path = %path.display(), "Running benchmark");
        let f = fs::File::open(&path)?;
        let test_case: TestCase = serde_yaml::from_reader(f)?;
        info!(id = %test_case.id, "Loaded test case");

        let mut agent = LlmAgent::new(agent_name)?;
        let mut env = SolanaEnv::new()?;

        let (initial_observation, final_observation, trace) =
            run_evaluation_loop(&mut env, &mut agent, &test_case).await?;
        let score = calculate_score(&test_case, &initial_observation, &final_observation);
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
        results.push(result);

        env.close()?;
    }
    info!("All benchmarks finished.");
    Ok(results)
}

/// Discovers benchmark files from a given path.
fn discover_benchmarks(path: &Path) -> Result<Vec<PathBuf>> {
    let mut benchmark_paths = vec![];
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && (path.extension() == Some("yml".as_ref())
                    || path.extension() == Some("yaml".as_ref()))
            {
                benchmark_paths.push(path);
            }
        }
    } else if path.is_file() {
        benchmark_paths.push(path.to_path_buf());
    } else {
        return Err(anyhow!("Provided path is not a valid file or directory"));
    }

    if benchmark_paths.is_empty() {
        info!("No benchmark files found to run.");
    }

    benchmark_paths.sort();
    Ok(benchmark_paths)
}

#[instrument(skip_all, fields(benchmark_id = %test_case.id))]
async fn run_evaluation_loop(
    env: &mut SolanaEnv,
    agent: &mut (dyn Agent + Send),
    test_case: &TestCase,
) -> Result<(AgentObservation, AgentObservation, ExecutionTrace)> {
    let initial_state_json = serde_json::to_value(&test_case.initial_state)?;
    let options = serde_json::json!({
        "id": test_case.id,
        "initial_state": initial_state_json
    });
    let initial_observation = env.reset(None, Some(options))?;

    let mut trace = ExecutionTrace::new(test_case.prompt.clone());

    let fee_payer = env.fee_payer_placeholder();
    let action = agent
        .get_action(
            &test_case.id,
            &test_case.prompt,
            &initial_observation,
            Some(&fee_payer.to_owned()),
        )
        .await?;
    let step_result = env.step(action.clone(), &test_case.ground_truth)?;

    let trace_step = reev_lib::trace::TraceStep {
        thought: None,
        action,
        observation: step_result.observation.clone(),
        info: step_result.info,
    };
    trace.add_step(trace_step);
    info!("Episode finished.");
    Ok((initial_observation, step_result.observation, trace))
}

/// Calculates the final score based on the ground truth assertions.
fn calculate_score(
    test_case: &TestCase,
    initial_observation: &AgentObservation,
    final_observation: &AgentObservation,
) -> f64 {
    debug!("Calculating score based on on-chain state assertions...");
    for assertion in &test_case.ground_truth.final_state_assertions {
        let pass = match assertion {
            StateAssertion::SolBalance { pubkey, expected } => {
                if let Some(account_state) = final_observation.account_states.get(pubkey) {
                    if let Some(actual) = account_state.get("lamports").and_then(|v| v.as_u64()) {
                        if actual == *expected {
                            debug!(pubkey, expected, actual, "SolBalance assertion PASSED");
                            true
                        } else {
                            debug!(pubkey, expected, actual, "SolBalance assertion FAILED");
                            false
                        }
                    } else {
                        debug!(
                            pubkey,
                            expected, "SolBalance assertion FAILED: lamports not found"
                        );
                        false
                    }
                } else if *expected == 0 {
                    debug!(
                        pubkey,
                        expected, "SolBalance assertion PASSED: account not found"
                    );
                    true
                } else {
                    debug!(
                        pubkey,
                        expected, "SolBalance assertion FAILED: account not found"
                    );
                    false
                }
            }
            StateAssertion::TokenAccountBalance { pubkey, expected } => {
                if let Some(account_state) = final_observation.account_states.get(pubkey) {
                    if let Some(actual) = account_state.get("amount").and_then(|v| v.as_u64()) {
                        if actual == *expected {
                            debug!(
                                pubkey,
                                expected, actual, "TokenAccountBalance assertion PASSED"
                            );
                            true
                        } else {
                            debug!(
                                pubkey,
                                expected, actual, "TokenAccountBalance assertion FAILED"
                            );
                            false
                        }
                    } else {
                        debug!(
                            pubkey,
                            expected, "TokenAccountBalance assertion FAILED: amount not found"
                        );
                        false
                    }
                } else if *expected == 0 {
                    debug!(
                        pubkey,
                        expected, "TokenAccountBalance assertion PASSED: account not found"
                    );
                    true
                } else {
                    debug!(
                        pubkey,
                        expected, "TokenAccountBalance assertion FAILED: account not found"
                    );
                    false
                }
            }
            StateAssertion::SolBalanceChange {
                pubkey,
                expected_change_gte,
            } => {
                let initial_balance = initial_observation
                    .account_states
                    .get(pubkey)
                    .and_then(|v| v.get("lamports"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let final_balance = final_observation
                    .account_states
                    .get(pubkey)
                    .and_then(|v| v.get("lamports"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let actual_change = final_balance as i64 - initial_balance as i64;

                if actual_change >= *expected_change_gte {
                    debug!(
                        pubkey,
                        expected_change_gte, actual_change, "SolBalanceChange assertion PASSED"
                    );
                    true
                } else {
                    debug!(
                        pubkey,
                        expected_change_gte, actual_change, "SolBalanceChange assertion FAILED"
                    );
                    false
                }
            }
        };

        if !pass {
            return 0.0;
        }
    }
    debug!("All on-chain assertions passed");
    1.0
}
