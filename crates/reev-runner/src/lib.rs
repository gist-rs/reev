use anyhow::{Context, Result, anyhow};
use reev_lib::{
    agent::{Agent, AgentObservation},
    benchmark::{FlowStep, TestCase},
    env::GymEnv,
    flow::{ExecutionResult, FlowLogger},
    llm_agent::LlmAgent,
    results::{FinalStatus, TestResult},
    score::{calculate_detailed_score, calculate_final_score},
    server_utils::{kill_existing_reev_agent, kill_existing_surfpool},
    solana_env::environment::SolanaEnv,
    test_scenarios,
    trace::ExecutionTrace,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};
use tracing::{info, instrument, warn};

pub mod db;
pub mod dependency;
pub mod renderer;

#[allow(dead_code)]
const AGENT_PORT: u16 = 9090;

/// RAII guard for dependency management
struct DependencyManagerGuard {
    #[allow(dead_code)]
    manager: dependency::DependencyManager,
}

impl Drop for DependencyManagerGuard {
    fn drop(&mut self) {
        info!("Dependency manager dropped - processes will be cleaned up on next startup");
        // Note: Actual cleanup is handled at startup to avoid runtime-in-runtime issues
        // This ensures clean state for the next run
    }
}

/// Initialize dependency management and ensure services are running
async fn init_dependencies() -> Result<(DependencyManagerGuard, dependency::DependencyUrls)> {
    info!("Initializing dependency management...");

    // Clean up any existing processes before starting new ones
    kill_existing_reev_agent(9090)
        .await
        .context("Failed to cleanup existing reev-agent processes")?;
    kill_existing_surfpool(8899)
        .await
        .context("Failed to cleanup existing surfpool processes")?;

    let config = dependency::DependencyConfig::from_env();
    let mut manager = dependency::DependencyManager::new(config)
        .context("Failed to create dependency manager")?;

    // Set up signal handlers for graceful shutdown
    manager
        .setup_signal_handlers()
        .context("Failed to setup signal handlers")?;

    // Ensure all dependencies are running
    let urls = manager
        .ensure_dependencies()
        .await
        .context("Failed to ensure dependencies are running")?;

    info!("Dependencies initialized successfully");
    info!("reev-agent: {}", urls.reev_agent);
    info!("surfpool: {}", urls.surfpool_rpc);

    let guard = DependencyManagerGuard { manager };
    Ok((guard, urls))
}

/// Runs all benchmarks found at the given path and returns the results.
pub async fn run_benchmarks(path: PathBuf, agent_name: &str) -> Result<Vec<TestResult>> {
    let benchmark_paths = discover_benchmarks(&path)?;
    if benchmark_paths.is_empty() {
        return Ok(vec![]);
    }

    // Initialize dependency management system
    let _dependency_guard = init_dependencies()
        .await
        .context("Failed to initialize dependencies")?;

    let db = db::Db::new("db/reev_results.db").await?;
    let mut results = vec![];

    for path in benchmark_paths {
        info!(path = %path.display(), "Running benchmark");
        let f = fs::File::open(&path)?;
        let test_case: TestCase = serde_yaml::from_reader(f)?;
        info!(id = %test_case.id, "Loaded test case");

        // Check if this is a flow benchmark
        if let Some(flow_steps) = &test_case.flow {
            info!(
                benchmark_id = %test_case.id,
                steps_count = %flow_steps.len(),
                "Detected flow benchmark, executing step-by-step"
            );

            let result = run_flow_benchmark(
                &test_case,
                flow_steps,
                agent_name,
                &path.display().to_string(),
            )
            .await?;
            results.push(result);
            continue;
        }

        // Initialize flow logging if enabled
        let flow_logger = if std::env::var("REEV_ENABLE_FLOW_LOGGING").is_ok() {
            let output_path =
                std::env::var("REEV_FLOW_LOG_PATH").unwrap_or_else(|_| "logs/flows".to_string());
            let path = PathBuf::from(output_path);
            std::fs::create_dir_all(&path)?;
            Some(FlowLogger::new(
                test_case.id.clone(),
                agent_name.to_string(),
                path,
            ))
        } else {
            None
        };

        let mut agent = LlmAgent::new_with_flow_logging(agent_name, flow_logger)?;
        let mut env = SolanaEnv::new().context("Failed to create Solana environment")?;

        let options = serde_json::to_value(&test_case)
            .context("Failed to serialize test case for env options")?;
        let mut initial_observation = env.reset(None, Some(options)).await?;
        test_scenarios::setup_spl_scenario(&mut env, &test_case, &mut initial_observation)
            .await
            .context("Failed to set up SPL scenario")?;

        let (final_observation, trace, actions) =
            run_evaluation_loop(&mut env, &mut agent, &test_case, &initial_observation)
                .await
                .with_context(|| {
                    format!("Evaluation loop failed for benchmark: {}", test_case.id)
                })?;

        // Use the new comprehensive scoring function from reev-lib.
        // Use the new comprehensive scoring function.
        let score = calculate_final_score(
            &test_case,
            &actions,
            &initial_observation,
            &final_observation,
        );

        info!(
            benchmark_id = %test_case.id,
            score = %score,
            instructions_count = %actions.len(),
            "Benchmark scoring completed"
        );

        // Complete flow logging if enabled
        if let Some(mut flow_logger) = agent.flow_logger.take() {
            let start_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let total_time_ms = (SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                - start_time)
                * 1000;

            let statistics = flow_logger.get_current_statistics();
            let final_status = if final_observation.last_transaction_status == "Success" {
                FinalStatus::Succeeded
            } else {
                FinalStatus::Failed
            };

            // Calculate detailed scoring breakdown
            let scoring_breakdown = calculate_detailed_score(
                &test_case,
                &actions,
                &initial_observation,
                &final_observation,
            );

            let execution_result = ExecutionResult {
                success: final_status == FinalStatus::Succeeded,
                score,
                total_time_ms,
                statistics,
                scoring_breakdown: Some(scoring_breakdown),
            };

            if let Err(e) = flow_logger.complete(execution_result) {
                warn!(
                    benchmark_id = %test_case.id,
                    error = %e,
                    "Failed to complete flow logging"
                );
            } else {
                // Successfully completed flow logging, render the flow as ASCII tree
                info!(
                    benchmark_id = %test_case.id,
                    "Flow log completed, rendering as ASCII tree"
                );

                // Find the most recent flow log file for this benchmark
                let logs_path = if let Ok(flow_logs_dir) = std::env::var("REEV_FLOW_LOG_PATH") {
                    std::path::PathBuf::from(flow_logs_dir)
                } else {
                    // Default to logs/flows directory
                    std::path::PathBuf::from("logs/flows")
                };

                if let Ok(entries) = std::fs::read_dir(&logs_path) {
                    let mut latest_flow_file: Option<(std::path::PathBuf, std::time::SystemTime)> =
                        None;

                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            if filename.contains(&test_case.id) && filename.contains("local") {
                                if let Ok(metadata) = std::fs::metadata(&path) {
                                    if let Ok(modified) = metadata.modified() {
                                        match &latest_flow_file {
                                            None => latest_flow_file = Some((path, modified)),
                                            Some((_, latest_time)) => {
                                                if modified > *latest_time {
                                                    latest_flow_file = Some((path, modified));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some((flow_file_path, _)) = latest_flow_file {
                        match reev_lib::flow::render_flow_file_as_ascii_tree(&flow_file_path) {
                            Ok(tree_output) => {
                                info!("\n🌊 Flow Execution Details:\n{tree_output}");
                            }
                            Err(e) => {
                                warn!(
                                    benchmark_id = %test_case.id,
                                    error = %e,
                                    "Failed to render flow log as ASCII tree"
                                );
                            }
                        }
                    }
                }
            }
        }

        // A score >= 0.75 means the instruction was perfect, even if it failed on-chain.
        // This is the primary signal for agent success.
        let final_status = if score >= 0.75 {
            FinalStatus::Succeeded
        } else {
            FinalStatus::Failed
        };

        if let Some(step) = trace.steps.first() {
            // Serialize the Vec<AgentAction> to a JSON string for database storage.
            let action_json = serde_json::to_string(&step.action)
                .context("Failed to serialize action vector to JSON for DB insertion")?;
            db.insert_result(
                &test_case.id,
                &test_case.prompt,
                &action_json,
                &final_observation,
                final_status,
                score,
            )
            .await?;
        }

        let result = TestResult::new(&test_case, final_status, score, trace);
        results.push(result);

        if let Err(e) = env.close() {
            warn!(
                benchmark_id = %test_case.id,
                error = %e,
                "Failed to close environment gracefully"
            );
        }
    }
    info!("All benchmarks finished.");
    Ok(results)
}

/// Execute a flow benchmark step-by-step
async fn run_flow_benchmark(
    test_case: &TestCase,
    flow_steps: &[FlowStep],
    agent_name: &str,
    _benchmark_path: &str,
) -> Result<TestResult> {
    info!(
        benchmark_id = %test_case.id,
        total_steps = %flow_steps.len(),
        "Starting flow benchmark execution"
    );

    // Initialize flow logging for flow benchmarks
    let flow_logger = if std::env::var("REEV_ENABLE_FLOW_LOGGING").is_ok() {
        let output_path =
            std::env::var("REEV_FLOW_LOG_PATH").unwrap_or_else(|_| "logs/flows".to_string());
        let path = PathBuf::from(output_path);
        std::fs::create_dir_all(&path)?;
        Some(FlowLogger::new(
            test_case.id.clone(),
            agent_name.to_string(),
            path,
        ))
    } else {
        None
    };

    let mut agent = LlmAgent::new_with_flow_logging(agent_name, flow_logger)?;
    let mut env = SolanaEnv::new().context("Failed to create Solana environment")?;
    let mut all_actions = Vec::new();
    let mut flow_trace = ExecutionTrace::new(test_case.prompt.clone());

    // Set up initial environment
    let options =
        serde_json::to_value(test_case).context("Failed to serialize test case for env options")?;
    let mut initial_observation = env.reset(None, Some(options)).await?;
    test_scenarios::setup_spl_scenario(&mut env, test_case, &mut initial_observation)
        .await
        .context("Failed to set up SPL scenario")?;

    // Execute each step in the flow
    for step in flow_steps.iter() {
        info!(
            step = step.step,
            description = %step.description,
            "Executing flow step"
        );

        // Create a step-specific test case
        let step_test_case = TestCase {
            id: format!("{}-step-{}", test_case.id, step.step),
            description: step.description.clone(),
            tags: test_case.tags.clone(),
            initial_state: test_case.initial_state.clone(),
            prompt: step.prompt.clone(),
            flow: None, // No nested flows
            ground_truth: test_case.ground_truth.clone(),
        };

        // Execute the step
        let (step_observation, step_trace, step_actions) =
            run_evaluation_loop(&mut env, &mut agent, &step_test_case, &initial_observation)
                .await
                .with_context(|| {
                    format!(
                        "Flow step {} failed for benchmark: {}",
                        step.step, test_case.id
                    )
                })?;

        // Log step completion before moving actions
        info!(
            step = step.step,
            actions_count = %step_actions.len(),
            "Flow step completed"
        );

        // Collect actions and trace
        all_actions.extend(step_actions);
        flow_trace.steps.extend(step_trace.steps);

        // Update observation for next step
        initial_observation = step_observation;
    }

    // Calculate final score for the entire flow
    let final_observation = initial_observation.clone();
    let score = calculate_final_score(
        test_case,
        &all_actions,
        &initial_observation,
        &final_observation,
    );

    info!(
        benchmark_id = %test_case.id,
        score = %score,
        total_actions = %all_actions.len(),
        "Flow benchmark completed"
    );

    // Determine final status
    let final_status = if score >= 0.6 {
        // Use min_score from ground_truth if available
        FinalStatus::Succeeded
    } else {
        FinalStatus::Failed
    };

    // Complete flow logging if enabled
    if let Some(mut flow_logger) = agent.flow_logger.take() {
        let total_time_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let statistics = flow_logger.get_current_statistics();

        // Create a simple scoring breakdown for flow benchmarks
        let scoring_breakdown = reev_lib::flow::ScoringBreakdown {
            instruction_score: if score >= 0.75 { 1.0 } else { score },
            onchain_score: if final_status == FinalStatus::Succeeded {
                1.0
            } else {
                0.0
            },
            final_score: score,
            issues: if score < 1.0 {
                vec![format!("Flow execution scored {:.1}%", score * 100.0)]
            } else {
                vec![]
            },
            mismatches: vec![],
        };

        let execution_result = ExecutionResult {
            success: final_status == FinalStatus::Succeeded,
            score,
            total_time_ms,
            statistics,
            scoring_breakdown: Some(scoring_breakdown),
        };

        // Auto-render flow as ASCII tree after completion
        if std::env::var("REEV_ENABLE_FLOW_LOGGING").is_ok() {
            match flow_logger.complete(execution_result) {
                Ok(flow_file_path) => {
                    match reev_lib::flow::render_flow_file_as_ascii_tree(&flow_file_path) {
                        Ok(tree_output) => {
                            info!("\n{}", tree_output);
                        }
                        Err(e) => {
                            warn!(
                                benchmark_id = %test_case.id,
                                error = %e,
                                "Failed to render flow as ASCII tree"
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        benchmark_id = %test_case.id,
                        error = %e,
                        "Failed to complete flow logging"
                    );
                }
            }
        }
    }

    let result = TestResult::new(test_case, final_status, score, flow_trace);

    // Close environment
    if let Err(e) = env.close() {
        warn!(
            benchmark_id = %test_case.id,
            error = %e,
            "Failed to close environment gracefully after flow execution"
        );
    }

    Ok(result)
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
    initial_observation: &AgentObservation,
) -> Result<(
    AgentObservation,
    ExecutionTrace,
    Vec<reev_lib::agent::AgentAction>,
)> {
    let mut trace = ExecutionTrace::new(test_case.prompt.clone());

    let fee_payer = env.fee_payer_placeholder();
    // The agent now returns a vector of actions.
    let actions = agent
        .get_action(
            &test_case.id,
            &test_case.prompt,
            initial_observation,
            Some(&fee_payer.to_owned()),
            Some(test_case.ground_truth.skip_instruction_validation),
        )
        .await?;

    // The environment's step function now takes a vector of actions to be bundled
    // into a single transaction.
    let step_result = env.step(actions.clone(), &test_case.ground_truth)?;

    let trace_step = reev_lib::trace::TraceStep {
        thought: None,
        action: actions.clone(),
        observation: step_result.observation.clone(),
        info: step_result.info,
    };
    trace.add_step(trace_step);
    info!("Episode finished.");
    Ok((step_result.observation, trace, actions))
}
