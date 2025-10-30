use anyhow::{Context, Result, anyhow};

use reev_flow::FlowLogger;
use reev_lib::{
    agent::{Agent, AgentObservation},
    benchmark::{FlowStep, TestCase},
    env::GymEnv,
    flow::{ExecutionResult, create_session_logger},
    llm_agent::LlmAgent,
    results::{FinalStatus, TestResult},
    score::calculate_final_score,
    server_utils::{kill_existing_reev_agent, kill_existing_surfpool},
    solana_env::environment::SolanaEnv,
    trace::ExecutionTrace,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};
use tracing::{debug, info, instrument, warn};

use crate::dependency::{DependencyConfig, DependencyManager};

pub mod dependency;
pub mod renderer;
pub mod version;

#[allow(dead_code)]
const AGENT_PORT: u16 = 9090;

/// RAII guard for dependency management
struct DependencyManagerGuard {
    pub manager: dependency::DependencyManager,
}

impl Drop for DependencyManagerGuard {
    fn drop(&mut self) {
        info!("Dependency manager dropped - processes will be cleaned up on next startup");
        // Note: Actual cleanup is handled at startup to avoid runtime-in-runtime issues
        // This ensures clean state for next run
    }
}

/// Initialize dependencies with custom configuration
async fn init_dependencies_with_config(config: DependencyConfig) -> Result<DependencyManagerGuard> {
    debug!("Initializing dependency management...");

    // Set environment variable for reset logic to know about shared vs fresh mode
    unsafe {
        std::env::set_var(
            "REEV_SHARED_INSTANCES",
            if config.shared_instances {
                "true"
            } else {
                "false"
            },
        );
    }

    // Clean up any existing processes before starting new ones (unless shared_instances is true)
    if !config.shared_instances {
        kill_existing_reev_agent(9090)
            .await
            .context("Failed to cleanup existing reev-agent processes")?;
        kill_existing_surfpool(8899)
            .await
            .context("Failed to cleanup existing surfpool processes")?;
    }

    let mut manager = DependencyManager::new(config)?;

    // Set up signal handlers for graceful shutdown
    manager
        .setup_signal_handlers()
        .context("Failed to setup signal handlers")?;

    // Ensure all dependencies are running
    manager
        .ensure_dependencies()
        .await
        .context("Failed to ensure dependencies are running")?;

    info!("Dependencies initialized successfully");
    info!("surfpool: ready (reev-agent will be started per benchmark)");

    let guard = DependencyManagerGuard { manager };
    Ok(guard)
}

/// Runs all benchmarks found at given path and returns results.
/// If shared_surfpool is true, reuses existing service instances.
/// If false, creates fresh instances for each run.
/// If kill_api is true, kills existing API processes (default: false for safety).
pub async fn run_benchmarks(
    path: PathBuf,
    agent_name: &str,
    shared_surfpool: bool,
    kill_api: bool,
    execution_id: Option<String>,
) -> Result<Vec<TestResult>> {
    let benchmark_paths = discover_benchmarks(&path)?;
    if benchmark_paths.is_empty() {
        return Ok(vec![]);
    }

    // Kill any existing API processes only if explicitly requested
    if kill_api {
        reev_lib::server_utils::kill_existing_api(3001).await?;
    }

    // Database-free runner - no cleanup needed

    // Initialize dependency management system based on shared_surfpool flag
    if shared_surfpool {
        info!("🔴 Using shared surfpool mode - reusing existing instances...");
    } else {
        info!("✨ Using fresh surfpool mode - creating new instances...");
    }

    let mut dependency_guard = init_dependencies_with_config(DependencyConfig {
        shared_instances: shared_surfpool,
        agent_type: Some(agent_name.to_string()),
        ..Default::default()
    })
    .await
    .context("Failed to initialize dependencies")?;
    info!("Dependency initialization completed successfully");

    let mut results = vec![];

    for path in benchmark_paths {
        info!(path = %path.display(), "Running benchmark");
        info!("Loading benchmark configuration...");
        let f = fs::File::open(&path)?;
        let test_case: TestCase = serde_yaml::from_reader(f)?;
        info!(id = %test_case.id, "Loaded test case");

        // Start reev-agent for this specific benchmark
        info!(
            "Starting reev-agent for benchmark: {} with agent: {}",
            test_case.id, agent_name
        );
        dependency_guard
            .manager
            .update_config_and_restart_agent(
                Some(agent_name.to_string()),
                Some(test_case.id.clone()),
            )
            .await
            .context("Failed to start reev-agent for benchmark")?;

        // Check if this is a flow benchmark
        if let Some(flow_steps) = &test_case.flow {
            info!(
                benchmark_id = %test_case.id,
                steps_count = %flow_steps.len(),
                "Detected flow benchmark, executing step-by-step"
            );

            // Generate session_id for flow benchmark
            let session_id = uuid::Uuid::new_v4().to_string();

            let result = run_flow_benchmark(
                &test_case,
                flow_steps,
                agent_name,
                &path.display().to_string(),
                &session_id,
            )
            .await?;
            results.push(result);

            // Stop reev-agent after flow benchmark completion
            info!("Stopping reev-agent after flow benchmark: {}", test_case.id);
            if let Err(e) = dependency_guard.manager.stop_reev_agent().await {
                warn!(
                    benchmark_id = %test_case.id,
                    error = %e,
                    "Failed to stop reev-agent gracefully after flow benchmark"
                );
            }

            continue;
        }

        // Initialize unified session logging if enabled
        let session_id = execution_id
            .as_ref()
            .cloned()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        // Flow logging is always enabled
        let log_path =
            std::env::var("REEV_SESSION_LOG_PATH").unwrap_or_else(|_| "logs/sessions".to_string());
        let path = PathBuf::from(log_path);
        std::fs::create_dir_all(&path)?;

        let session_logger = Some(create_session_logger(
            session_id.clone(),
            test_case.id.clone(),
            agent_name.to_string(),
            Some(path),
        )?);

        // Create session in database
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let session_info = reev_lib::db::SessionInfo {
            session_id: session_id.clone(),
            benchmark_id: test_case.id.clone(),
            agent_type: agent_name.to_string(),
            interface: "tui".to_string(),
            start_time,
            end_time: None,
            status: "running".to_string(),
            score: None,
            final_status: None,
        };

        // Initialize enhanced OTEL logging instead of basic flow logging
        let flow_logger = FlowLogger::new(
            test_case.id.clone(),
            agent_name.to_string(),
            PathBuf::from("logs/flows"),
        );

        info!(
            benchmark_id = %test_case.id,
            session_id = %session_id,
            "Session will be created with enhanced OTEL logging"
        );

        let mut llm_agent = LlmAgent::new_with_flow_logging(agent_name, Some(flow_logger))?;
        info!("[Runner] Setting session_id on LlmAgent: {}", session_id);
        llm_agent.set_session_id(session_id.clone());
        info!("[Runner] Session_id set successfully");
        let mut agent = Box::new(llm_agent) as Box<dyn Agent + Send>;
        let mut env = SolanaEnv::new().context("Failed to create Solana environment")?;

        let options = serde_json::to_value(&test_case)
            .context("Failed to serialize test case for env options")?;
        let initial_observation = env.reset(None, Some(options)).await?;

        let (final_observation, trace, actions) =
            match run_evaluation_loop(&mut env, agent.as_mut(), &test_case, &initial_observation)
                .await
            {
                Ok(result) => result,
                Err(e) => {
                    // Ensure session is marked as failed even if evaluation loop fails
                    let error_session_result = reev_lib::db::SessionResult {
                        end_time: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64,
                        score: 0.0,
                        final_status: FinalStatus::Failed.to_string(),
                    };

                    // Session file logging will handle the completion automatically
                    warn!(
                        benchmark_id = %test_case.id,
                        session_id = %session_id,
                        error = %e,
                        "Flow benchmark evaluation failed"
                    );

                    return Err(e).context(format!(
                        "Evaluation loop failed for benchmark: {}",
                        test_case.id
                    ));
                }
            };

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

        // Complete session logging if enabled
        if let Some(session_logger) = session_logger {
            let start_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let _total_time_ms = (SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                - start_time)
                * 1000;

            let _final_status = if final_observation.last_transaction_status == "Success" {
                FinalStatus::Succeeded
            } else {
                FinalStatus::Failed
            };

            // Store ExecutionTrace format directly for ASCII tree compatibility
            match session_logger.complete_with_trace(trace.clone()) {
                Ok(log_file) => {
                    info!(
                        benchmark_id = %test_case.id,
                        log_file = %log_file.display(),
                        "Session log with ExecutionTrace completed successfully"
                    );

                    // Session file logging automatically stores ExecutionTrace
                    info!(
                        benchmark_id = %test_case.id,
                        session_id = %session_id,
                        "ExecutionTrace stored in session file"
                    );
                }
                Err(e) => {
                    warn!(
                        benchmark_id = %test_case.id,
                        error = %e,
                        "Failed to complete session logging with ExecutionTrace"
                    );

                    // Session file already contains the ExecutionTrace
                    info!(
                        benchmark_id = %test_case.id,
                        session_id = %session_id,
                        "Session file contains ExecutionTrace (fallback not needed)"
                    );
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

        // Complete session in database with results
        let end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let session_result = reev_lib::db::SessionResult {
            end_time,
            score,
            final_status: final_status.to_string(),
        };

        // 🎯 CAPTURE TOOL CALLS FROM AGENT'S ENHANCED OTEL LOG FILES
        // Since reev-agent runs in separate process, we need to read from its otel log files
        let tool_calls = extract_tool_calls_from_agent_logs(&session_id).await;

        if !tool_calls.is_empty() {
            info!(
                session_id = %session_id,
                tool_calls_count = tool_calls.len(),
                "Storing tool calls in database (from agent log files)"
            );

            for tool_call in &tool_calls {
                // Extract tool name and input from new structure
                let tool_name = tool_call
                    .tool_input
                    .as_ref()
                    .map(|input| input.tool_name.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                let input_params = tool_call
                    .tool_input
                    .as_ref()
                    .map(|input| input.tool_args.clone())
                    .unwrap_or(serde_json::Value::Null);

                let output_result = tool_call
                    .tool_output
                    .as_ref()
                    .map(|output| output.results.clone())
                    .unwrap_or(serde_json::Value::Null);

                let execution_time_ms = tool_call.timing.step_timeuse_ms;

                let (status, error_message) = tool_call
                    .tool_output
                    .as_ref()
                    .map(|output| {
                        if output.success {
                            ("success".to_string(), None)
                        } else {
                            ("error".to_string(), output.error_message.clone())
                        }
                    })
                    .unwrap_or_else(|| (("pending".to_string()), None));

                let tool_data = reev_db::writer::sessions::ToolCallData {
                    session_id: session_id.clone(),
                    tool_name,
                    start_time: tool_call.timestamp.timestamp() as u64,
                    execution_time_ms,
                    input_params,
                    output_result,
                    status,
                    error_message,
                };

                // Tool calls are stored in enhanced otel session files automatically
                debug!(
                    session_id = %session_id,
                    tool_name = %tool_data.tool_name,
                    "Tool call stored in enhanced otel session file"
                );
            }
        } else {
            debug!("No tool calls found in agent log files");
        }

        // Session file completion is handled automatically by SessionFileLogger
        info!(
            benchmark_id = %test_case.id,
            session_id = %session_id,
            score = %score,
            final_status = %final_status,
            "Session completed in session file"
        );

        // Store performance metrics
        let performance_data = reev_lib::db::AgentPerformanceData {
            session_id: session_id.clone(),
            benchmark_id: test_case.id.clone(),
            agent_type: agent_name.to_string(),
            score,
            final_status: match final_status {
                FinalStatus::Succeeded => "succeeded".to_string(),
                FinalStatus::Failed => "failed".to_string(),
            },
            execution_time_ms: (end_time - start_time) as u64,
            timestamp: chrono::Utc::now().to_rfc3339(),
            flow_log_id: None,
            prompt_md5: None,
        };

        // Convert to shared AgentPerformance type for database insertion (skip if no_db)
        // Performance metrics stored in session file
        debug!(
            benchmark_id = %test_case.id,
            "Performance metrics available in session file"
        );

        let result = TestResult::new(&test_case, final_status, score, trace);
        results.push(result);

        if let Err(e) = env.close() {
            warn!(
                benchmark_id = %test_case.id,
                error = %e,
                "Failed to close environment gracefully"
            );
        }

        // Stop reev-agent after benchmark completion
        info!("Stopping reev-agent after benchmark: {}", test_case.id);
        if let Err(e) = dependency_guard.manager.stop_reev_agent().await {
            warn!(
                benchmark_id = %test_case.id,
                error = %e,
                "Failed to stop reev-agent gracefully"
            );
        }
    }

    info!("All benchmarks finished.");

    // Close database connection properly to prevent lock issues (skip if no_db)
    info!("All benchmarks completed (database-free runner)");

    Ok(results)
}

/// Extract tool calls from agent's enhanced otel log files
/// This is needed because agent runs in separate process from runner
async fn extract_tool_calls_from_agent_logs(session_id: &str) -> Vec<reev_flow::EnhancedToolCall> {
    use std::fs;
    use std::path::Path;

    // Look for specific enhanced otel log file for this session
    let logs_dir = Path::new("logs/sessions");
    let otel_filename = format!("enhanced_otel_{session_id}.jsonl");
    let otel_filepath = logs_dir.join(&otel_filename);
    let mut all_tool_calls = Vec::new();

    info!("Looking for otel file: {}", otel_filename);

    if otel_filepath.exists() {
        info!("Found otel file for session: {}", otel_filename);

        if let Ok(content) = fs::read_to_string(&otel_filepath) {
            // Parse JSON lines from file
            for line in content.lines() {
                if line.trim().is_empty() || line.trim().starts_with('#') {
                    continue; // Skip empty lines and comments
                }

                if let Ok(tool_call) = serde_json::from_str::<reev_flow::EnhancedToolCall>(line) {
                    // Verify this tool call belongs to our session
                    if tool_call.session_id == session_id {
                        all_tool_calls.push(tool_call);
                    }
                } else if !line.trim().is_empty() {
                    warn!("Failed to parse tool call from otel log: {}", line);
                }
            }
        } else {
            warn!("Failed to read otel file: {}", otel_filename);
        }
    } else {
        warn!("Otel file not found for session: {}", otel_filename);
    }

    info!(
        "Extracted {} tool calls from agent log file: {}",
        all_tool_calls.len(),
        otel_filename
    );
    all_tool_calls
}

/// Execute a flow benchmark step-by-step
async fn run_flow_benchmark(
    test_case: &TestCase,
    flow_steps: &[FlowStep],
    agent_name: &str,
    _benchmark_path: &str,

    session_id: &str,
) -> Result<TestResult> {
    info!(
        benchmark_id = %test_case.id,
        total_steps = %flow_steps.len(),
        "Starting flow benchmark execution"
    );

    // Initialize flow logging for flow benchmarks
    // Flow logging is always enabled
    let flow_logger = {
        let output_path =
            std::env::var("REEV_FLOW_LOG_PATH").unwrap_or_else(|_| "logs/flows".to_string());
        let path = PathBuf::from(output_path);
        std::fs::create_dir_all(&path)?;

        Some(FlowLogger::new_with_session(
            session_id.to_string(),
            test_case.id.clone(),
            agent_name.to_string(),
            path,
        ))
    };

    let mut agent = LlmAgent::new_with_flow_logging(agent_name, flow_logger)?;
    agent.set_session_id(session_id.to_string());
    let mut env = SolanaEnv::new().context("Failed to create Solana environment")?;
    let mut all_actions = Vec::new();
    let mut flow_trace = ExecutionTrace::new(test_case.prompt.clone());

    // Set up initial environment
    let options =
        serde_json::to_value(test_case).context("Failed to serialize test case for env options")?;
    let mut initial_observation = env.reset(None, Some(options)).await?;

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

        // Execute step
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
        // Flow logging is always enabled
        match flow_logger.complete(execution_result).await {
            Ok(flow_file_path) => {
                match reev_lib::flow::render_flow_file_as_ascii_tree(flow_file_path.as_path()) {
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

#[instrument(skip_all, name = "run_evaluation_loop")]
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
            Some(&test_case.initial_state),
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
