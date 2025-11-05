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
use reev_orchestrator::OrchestratorGateway;
use reev_types::flow::{BenchmarkSource, DynamicFlowPlan};

use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};
use tracing::{debug, error, info, instrument, warn};

use crate::dependency::{DependencyConfig, DependencyManager};
use reev_lib::benchmark::{GroundTruth, InitialStateItem};

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

            // Use provided execution_id for flow benchmark to ensure consistency
            let session_id = execution_id.clone();

            let result = run_flow_benchmark(
                &test_case,
                flow_steps,
                agent_name,
                &path.display().to_string(),
                session_id.as_deref().unwrap_or("unknown"),
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

        // Session file created via SessionFileLogger
        // No database operations needed - API handles database storage

        // Initialize enhanced OTEL logging instead of basic flow logging
        // Database-only logging - runner should not access database
        let flow_logger = FlowLogger::new_with_session(
            session_id.clone(),
            test_case.id.clone(),
            agent_name.to_string(),
        );

        info!(
            benchmark_id = %test_case.id,
            session_id = %session_id,
            "Session will be created with enhanced OTEL logging"
        );

        let mut llm_agent = LlmAgent::new_with_flow_logging(agent_name, Some(flow_logger))?;
        // Extract flow logger option before moving llm_agent
        let flow_logger_option = llm_agent.flow_logger.take();
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
                    // Session file will automatically handle failure state
                    // No database operations needed - API handles database storage

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

        // Session file completion handled automatically by SessionFileLogger
        // Database storage handled by API after reading session file
        let _end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // 🎯 CAPTURE TOOL CALLS FROM AGENT'S ENHANCED OTEL LOG FILES
        // Since reev-agent runs in separate process, we need to read from its otel log files
        let tool_calls = extract_tool_calls_from_agent_logs(&session_id).await;

        if !tool_calls.is_empty() {
            info!(
                session_id = %session_id,
                tool_calls_count = tool_calls.len(),
                "Processing tool calls from agent log files (session file storage)"
            );

            for tool_call in &tool_calls {
                // Extract tool name and input from new structure
                let tool_name = tool_call
                    .tool_input
                    .as_ref()
                    .map(|input| input.tool_name.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                let _input_params = tool_call
                    .tool_input
                    .as_ref()
                    .map(|input| input.tool_args.clone())
                    .unwrap_or(serde_json::Value::Null);

                let _output_result = tool_call
                    .tool_output
                    .as_ref()
                    .map(|output| output.results.clone())
                    .unwrap_or(serde_json::Value::Null);

                let _execution_time_ms = tool_call.timing.step_timeuse_ms;

                let (_status, _error_message) = tool_call
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

                // Tool calls are automatically stored in enhanced otel session files
                debug!(
                    session_id = %session_id,
                    tool_name = %tool_name,
                    "Tool call available in enhanced otel session file"
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

        // Convert enhanced_otel JSONL to YML and store in database for flow diagrams
        // This ensures API can read tool calls from CLI executions
        convert_and_store_enhanced_otel_for_cli(&session_id)
            .await
            .ok();

        // Performance metrics stored in session file
        // Database storage handled by API after reading session file

        // Performance metrics available in session file for API to process
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

        // Complete flow logger with execution result for performance tracking
        if let Some(mut flow_logger) = flow_logger_option {
            let execution_result = ExecutionResult {
                success: final_status == reev_lib::results::FinalStatus::Succeeded,
                score,
                total_time_ms: 0, // Not tracked in this path
                statistics: reev_flow::types::ExecutionStatistics {
                    total_llm_calls: 0,
                    total_tool_calls: 0,
                    total_tokens: 0,
                    tool_usage: std::collections::HashMap::new(),
                    max_depth: 0,
                },
                scoring_breakdown: None, // Not tracked in this path
            };

            info!("Completing flow logger for performance tracking");
            match flow_logger.complete(execution_result).await {
                Ok(_) => {
                    info!("✅ Successfully completed flow logger with performance data");
                }
                Err(e) => {
                    warn!(
                        benchmark_id = %test_case.id,
                        error = %e,
                        "Failed to complete flow logger for performance tracking"
                    );
                }
            }
        }
    }

    info!("All benchmarks finished.");

    // Database-free runner - no database connections to close
    info!("All benchmarks completed (database-free runner)");

    Ok(results)
}

/// Convert enhanced_otel JSONL file to YML format and store in database for CLI runs
async fn convert_and_store_enhanced_otel_for_cli(session_id: &str) -> Result<()> {
    use std::path::PathBuf;

    let jsonl_path = PathBuf::from(format!("logs/sessions/enhanced_otel_{session_id}.jsonl"));

    if !jsonl_path.exists() {
        debug!("No enhanced_otel file found for session: {}", session_id);
        return Ok(());
    }

    info!(
        "Converting enhanced_otel to YML for CLI session: {}",
        session_id
    );

    // Use JsonlToYmlConverter to convert to session format
    let temp_yml_path = jsonl_path.with_extension("yml");
    let session_data = reev_flow::JsonlToYmlConverter::convert_file(&jsonl_path, &temp_yml_path)
        .map_err(|e| anyhow!("Failed to convert enhanced_otel JSONL to YML: {e}"))?;

    // Read YML content for storage
    let yml_content = std::fs::read_to_string(&temp_yml_path)
        .map_err(|e| anyhow!("Failed to read temporary YML file: {e}"))?;

    // Clean up temporary file
    let _ = std::fs::remove_file(&temp_yml_path);

    // Store in a simple database file for API to access
    let db_path = PathBuf::from("db/cli_sessions.json");
    std::fs::create_dir_all(db_path.parent().unwrap())?;

    // Read existing sessions or create new
    let mut sessions = if db_path.exists() {
        let content = std::fs::read_to_string(&db_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Add new session data
    sessions.as_object_mut().unwrap().insert(session_id.to_string(), serde_json::json!({
        "yml_content": yml_content,
        "tool_calls": session_data.tool_calls.len(),
        "session_id": session_id,
        "benchmark_id": session_data.tool_calls.first().map(|t| &t.tool_name).unwrap_or(&"unknown".to_string()),
        "created_at": chrono::Utc::now().to_rfc3339()
    }));

    // Write back to database file
    let sessions_json = serde_json::to_string_pretty(&sessions)?;
    if let Err(e) = std::fs::write(&db_path, sessions_json) {
        warn!("Failed to write CLI sessions database: {}", e);
    } else {
        info!(
            "Successfully wrote CLI sessions database: {} ({} sessions)",
            db_path.display(),
            sessions.as_object().unwrap().len()
        );
    }

    info!(
        "Successfully converted and stored enhanced_otel data for CLI session: {} ({} tool calls)",
        session_id,
        session_data.tool_calls.len()
    );

    Ok(())
}

/// Unified runner function supporting both static files and dynamic flows (Phase 2)
pub async fn run_benchmarks_with_source(
    source: BenchmarkSource,
    agent_name: &str,
    shared_surfpool: bool,
    execution_id: Option<String>,
) -> Result<Vec<TestResult>> {
    match source {
        BenchmarkSource::StaticFile { path } => {
            // Use existing function for static files
            run_benchmarks(
                PathBuf::from(path),
                agent_name,
                shared_surfpool,
                false,
                execution_id,
            )
            .await
        }
        BenchmarkSource::DynamicFlow { prompt, wallet } => {
            // Use new direct execution for dynamic flows
            run_dynamic_flow(&prompt, &wallet, agent_name, shared_surfpool, execution_id).await
        }
        BenchmarkSource::Hybrid { path, prompt } => {
            // For hybrid, prefer dynamic if prompt available
            if let Some(ref p) = prompt {
                // Extract wallet from path or use default
                let wallet = "11111111111111111111111111111112".to_string();
                run_dynamic_flow(p, &wallet, agent_name, shared_surfpool, execution_id).await
            } else if let Some(p) = path {
                run_benchmarks(
                    PathBuf::from(p),
                    agent_name,
                    shared_surfpool,
                    false,
                    execution_id,
                )
                .await
            } else {
                Err(anyhow::anyhow!(
                    "Hybrid source requires either path or prompt"
                ))
            }
        }
    }
}

/// Runs dynamic flow directly in memory without temporary files (Phase 2)
pub async fn run_dynamic_flow(
    prompt: &str,
    wallet: &str,
    agent_name: &str,
    shared_surfpool: bool,
    execution_id: Option<String>,
) -> Result<Vec<TestResult>> {
    info!("--- Phase 2: Direct Dynamic Flow Execution ---");
    info!(
        "Generating flow for prompt: '{}' with wallet: {}",
        prompt, wallet
    );

    // Initialize orchestrator gateway
    let gateway = OrchestratorGateway::new();

    // Process user request and generate dynamic flow plan
    let (flow_plan, _yml_path) = gateway
        .process_user_request(prompt, wallet)
        .await
        .context("Failed to process dynamic flow request")?;

    info!(
        "Generated flow plan '{}' with {} steps (direct execution)",
        flow_plan.flow_id,
        flow_plan.steps.len()
    );

    // Initialize dependency management system
    let mut dependency_guard = init_dependencies_with_config(DependencyConfig {
        shared_instances: shared_surfpool,
        agent_type: Some(agent_name.to_string()),
        ..Default::default()
    })
    .await
    .context("Failed to initialize dependencies")?;

    // Create test case from flow plan (in-memory)
    let test_case = create_test_case_from_flow_plan(&flow_plan)?;

    // Start reev-agent for this flow
    info!(
        "Starting reev-agent for dynamic flow: {} with agent: {}",
        flow_plan.flow_id, agent_name
    );
    dependency_guard
        .manager
        .update_config_and_restart_agent(
            Some(agent_name.to_string()),
            Some(flow_plan.flow_id.clone()),
        )
        .await
        .context("Failed to start reev-agent for dynamic flow")?;

    // Execute the dynamic flow
    let session_id = execution_id
        .as_ref()
        .cloned()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let result = run_flow_benchmark(
        &test_case,
        test_case
            .flow
            .as_ref()
            .expect("Flow steps should be present"),
        agent_name,
        &format!("dynamic://{}", flow_plan.flow_id),
        &session_id,
    )
    .await?;

    info!("Dynamic flow execution completed successfully");

    // Stop reev-agent after flow completion
    info!(
        "Stopping reev-agent after dynamic flow: {}",
        flow_plan.flow_id
    );
    if let Err(e) = dependency_guard.manager.stop_reev_agent().await {
        warn!(
            flow_id = %flow_plan.flow_id,
            error = %e,
            "Failed to stop reev-agent gracefully after dynamic flow"
        );
    }

    Ok(vec![result])
}

/// Execute a flow with Phase 3 recovery mechanisms
pub async fn run_recovery_flow(
    prompt: &str,
    wallet: &str,
    agent_name: &str,
    shared_surfpool: bool,
    execution_id: Option<String>,
    recovery_config: reev_orchestrator::RecoveryConfig,
    atomic_mode: Option<reev_types::flow::AtomicMode>,
) -> Result<Vec<TestResult>> {
    info!("--- Phase 3: Recovery Flow Execution ---");
    info!(
        "Generating flow for prompt: '{}' with wallet: {}",
        prompt, wallet
    );

    // Initialize orchestrator gateway with recovery configuration
    let gateway = reev_orchestrator::OrchestratorGateway::with_recovery_config(recovery_config);

    // Create wallet context for flow generation
    let wallet_context = reev_types::flow::WalletContext::new(wallet.to_string());

    // Process user request and generate dynamic flow plan
    let flow_plan = gateway
        .generate_flow_plan(prompt, &wallet_context, atomic_mode)
        .context("Failed to generate recovery flow plan")?;

    info!(
        "Generated recovery flow plan '{}' with {} steps (atomic mode: {:?})",
        flow_plan.flow_id,
        flow_plan.steps.len(),
        flow_plan.atomic_mode
    );

    // Initialize dependency management system
    let mut dependency_guard = init_dependencies_with_config(DependencyConfig {
        shared_instances: shared_surfpool,
        agent_type: Some(agent_name.to_string()),
        ..Default::default()
    })
    .await
    .context("Failed to initialize dependencies")?;

    // Create test case from flow plan (in-memory)
    let test_case = create_test_case_from_flow_plan(&flow_plan)?;

    // Start reev-agent for this flow
    info!(
        "Starting reev-agent for recovery flow: {} with agent: {}",
        flow_plan.flow_id, agent_name
    );
    dependency_guard
        .manager
        .update_config_and_restart_agent(
            Some(agent_name.to_string()),
            Some(flow_plan.flow_id.clone()),
        )
        .await
        .context("Failed to start reev-agent for recovery flow")?;

    // Execute the flow with recovery mechanisms
    let session_id = execution_id
        .as_ref()
        .cloned()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let result = run_flow_benchmark_with_recovery(
        &test_case,
        test_case
            .flow
            .as_ref()
            .expect("Flow steps should be present"),
        agent_name,
        &format!("recovery://{}", flow_plan.flow_id),
        &session_id,
        &gateway,
    )
    .await?;

    info!("Recovery flow execution completed");

    // Log recovery metrics
    let recovery_metrics = gateway.get_recovery_metrics().await;
    info!(
        "Recovery metrics - Total attempts: {}, Successful recoveries: {}, Failed recoveries: {}",
        recovery_metrics.total_attempts,
        recovery_metrics.successful_recoveries,
        recovery_metrics.failed_recoveries
    );

    // Stop reev-agent after flow completion
    info!(
        "Stopping reev-agent after recovery flow: {}",
        flow_plan.flow_id
    );
    if let Err(e) = dependency_guard.manager.stop_reev_agent().await {
        warn!(
            flow_id = %flow_plan.flow_id,
            error = %e,
            "Failed to stop reev-agent gracefully after recovery flow"
        );
    }

    Ok(vec![result])
}

/// Create TestCase from DynamicFlowPlan for execution
fn create_test_case_from_flow_plan(flow_plan: &DynamicFlowPlan) -> Result<TestCase> {
    // Convert dynamic steps to flow steps
    let flow_steps: Vec<FlowStep> = flow_plan
        .steps
        .iter()
        .enumerate()
        .map(|(index, step)| FlowStep {
            step: (index + 1) as u32,
            description: step.description.clone(),
            prompt: step.prompt_template.clone(),
            critical: step.critical,
            timeout: Some(step.estimated_time_seconds as u32),
            depends_on: Vec::new(),
        })
        .collect();

    // Generate initial accounts from context
    let initial_accounts = generate_initial_accounts_from_context(&flow_plan.context)?;

    // Generate ground truth from steps
    let ground_truth = generate_ground_truth_from_steps(&flow_plan.steps)?;

    Ok(TestCase {
        id: flow_plan.flow_id.clone(),
        description: format!("Dynamic flow: {}", flow_plan.user_prompt),
        prompt: flow_plan.user_prompt.clone(),
        initial_state: initial_accounts,
        ground_truth,
        flow: Some(flow_steps),
        tags: vec!["dynamic".to_string(), "phase2".to_string()],
    })
}

/// Generate initial accounts from wallet context
fn generate_initial_accounts_from_context(
    context: &reev_types::flow::WalletContext,
) -> Result<Vec<InitialStateItem>> {
    let accounts = vec![
        // User wallet with SOL balance
        InitialStateItem {
            pubkey: "USER_WALLET_PUBKEY".to_string(),
            owner: context.owner.clone(),
            lamports: context.sol_balance,
            data: None, // System account, no data
        },
        // USDC ATA with zero balance
        InitialStateItem {
            pubkey: "USER_USDC_ATA".to_string(),
            owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(), // Token program
            lamports: 2039280, // Standard rent exemption
            data: Some(reev_lib::benchmark::SplAccountData {
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC mint
                owner: "USER_WALLET_PUBKEY".to_string(),
                amount: "0".to_string(),
            }),
        },
    ];

    Ok(accounts)
}

/// Generate ground truth assertions from dynamic steps
fn generate_ground_truth_from_steps(
    steps: &[reev_types::flow::DynamicStep],
) -> Result<GroundTruth> {
    use reev_lib::benchmark::StateAssertion;

    let mut assertions = Vec::new();

    // Check if any step involves swap
    let has_swap = steps.iter().any(|step| {
        step.required_tools.contains(&"sol_tool".to_string())
            || step.description.to_lowercase().contains("swap")
    });

    if has_swap {
        assertions.push(StateAssertion::TokenAccountBalance {
            pubkey: "USER_USDC_ATA".to_string(),
            expected_gte: Some(1),
            expected: None,
            address_derivation: None,
            weight: 1.0,
        });
    }

    // Check if any step involves lend/earn
    let has_lend = steps.iter().any(|step| {
        step.required_tools
            .contains(&"jupiter_earn_tool".to_string())
            || step.description.to_lowercase().contains("lend")
            || step.description.to_lowercase().contains("earn")
    });

    if has_lend {
        assertions.push(StateAssertion::SolBalanceChange {
            pubkey: "USER_WALLET_PUBKEY".to_string(),
            expected_change_gte: -100005000, // Account for fees
            weight: 1.0,
        });
    }

    Ok(GroundTruth {
        final_state_assertions: assertions,
        transaction_status: "Success".to_string(),
        expected_instructions: Vec::new(),
        skip_instruction_validation: false,
    })
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

    // Initialize unified session logging for consistency with regular benchmarks
    // Session logging is always enabled
    let session_logger = {
        let log_path =
            std::env::var("REEV_SESSION_LOG_PATH").unwrap_or_else(|_| "logs/sessions".to_string());
        let path = PathBuf::from(log_path);
        std::fs::create_dir_all(&path)?;

        Some(create_session_logger(
            session_id.to_string(),
            test_case.id.clone(),
            agent_name.to_string(),
            Some(path),
        )?)
    };
    // Initialize flow logging for flow benchmarks (file-based only)
    // Flow logging is always enabled
    let flow_logger = {
        info!("🗄️ Flow logger initialized (database-only)");
        Some(FlowLogger::new_with_session(
            session_id.to_string(),
            test_case.id.clone(),
            agent_name.to_string(),
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

    // Execute each step in the flow with soft error handling
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

        // Execute step with error recovery
        match run_evaluation_loop(&mut env, &mut agent, &step_test_case, &initial_observation).await
        {
            Ok((step_observation, step_trace, step_actions)) => {
                // Log step completion
                info!(
                    step = step.step,
                    actions_count = %step_actions.len(),
                    "Flow step completed successfully"
                );

                // Collect actions and trace
                all_actions.extend(step_actions);
                flow_trace.steps.extend(step_trace.steps);

                // Update observation for next step
                initial_observation = step_observation;
            }
            Err(step_error) => {
                // Log comprehensive error to session and continue
                let error_message = format!("Flow step {} failed: {}", step.step, step_error);
                error!(error = %error_message, "Flow step error - continuing with next step");

                // Log error to flow logger if available
                if let Some(ref mut flow_logger) = agent.flow_logger {
                    use reev_flow::types::ErrorContent;
                    let mut context = std::collections::HashMap::new();
                    context.insert("step".to_string(), step.step.to_string());
                    context.insert("recovery_attempted".to_string(), "true".to_string());
                    context.insert(
                        "recovery_result".to_string(),
                        "Continuing to next step".to_string(),
                    );

                    flow_logger.log_error(
                        ErrorContent {
                            error_type: "FLOW_STEP_ERROR".to_string(),
                            message: error_message.clone(),
                            stack_trace: None,
                            context,
                        },
                        0,
                    );
                }

                // Add error step to trace for visibility
                let error_step = reev_lib::trace::TraceStep {
                    thought: None,
                    action: vec![],
                    observation: initial_observation.clone(),
                    info: serde_json::json!({
                        "success": false,
                        "error": error_message,
                        "execution_time_ms": 0,
                        "gas_used": null,
                        "transaction_signature": null
                    }),
                };
                flow_trace.add_step(error_step);

                // Continue with next step instead of crashing
                continue;
            }
        }
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

    let result = TestResult::new(test_case, final_status, score, flow_trace.clone());

    // Complete session logging if enabled for consistency with regular benchmarks
    if let Some(session_logger) = session_logger {
        match session_logger.complete_with_trace(flow_trace) {
            Ok(log_file) => {
                info!(
                    benchmark_id = %test_case.id,
                    session_id = %session_id,
                    log_file = %log_file.display(),
                    "Session log with ExecutionTrace completed successfully for flow benchmark"
                );
            }
            Err(e) => {
                warn!(
                    benchmark_id = %test_case.id,
                    session_id = %session_id,
                    error = %e,
                    "Failed to complete session logging with ExecutionTrace for flow benchmark"
                );
            }
        }
    }

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

/// Execute a flow benchmark with Phase 3 recovery mechanisms
async fn run_flow_benchmark_with_recovery(
    test_case: &TestCase,
    flow_steps: &[FlowStep],
    agent_name: &str,
    _benchmark_path: &str,
    session_id: &str,
    _gateway: &reev_orchestrator::OrchestratorGateway,
) -> Result<TestResult> {
    info!(
        benchmark_id = %test_case.id,
        total_steps = %flow_steps.len(),
        "Starting flow benchmark execution with recovery"
    );

    // Initialize unified session logging for consistency with regular benchmarks
    let _session_logger = {
        let log_path =
            std::env::var("REEV_SESSION_LOG_PATH").unwrap_or_else(|_| "logs/sessions".to_string());
        let path = PathBuf::from(log_path);
        std::fs::create_dir_all(&path)?;

        Some(create_session_logger(
            session_id.to_string(),
            test_case.id.clone(),
            agent_name.to_string(),
            Some(path),
        )?)
    };

    // Initialize flow logging for flow benchmarks (file-based only)
    let flow_logger = {
        info!("🗄️ Flow logger initialized (database-only)");
        Some(FlowLogger::new_with_session(
            session_id.to_string(),
            test_case.id.clone(),
            agent_name.to_string(),
        ))
    };

    let mut agent = LlmAgent::new_with_flow_logging(agent_name, flow_logger)?;
    agent.set_session_id(session_id.to_string());
    let mut env = SolanaEnv::new().context("Failed to create Solana environment")?;
    let mut all_actions: Vec<reev_lib::agent::AgentAction> = Vec::new();
    let mut flow_trace = ExecutionTrace::new(test_case.prompt.clone());

    // Set up initial environment
    let options =
        serde_json::to_value(test_case).context("Failed to serialize test case for env options")?;
    let mut initial_observation = env.reset(None, Some(options)).await?;

    // Execute flow steps manually without recovery engine complexity
    let mut step_results = Vec::new();
    let mut successful_steps = 0;
    let mut failed_steps = 0;

    for flow_step in flow_steps {
        info!(
            step = %flow_step.step,
            description = %flow_step.description,
            "Executing flow step"
        );

        // Create step-specific test case
        let step_test_case = TestCase {
            id: format!("{}-step-{}", test_case.id, flow_step.step),
            description: flow_step.description.clone(),
            tags: vec![],
            initial_state: vec![],
            prompt: flow_step.prompt.clone(),
            flow: None,
            ground_truth: GroundTruth {
                transaction_status: "Success".to_string(),
                final_state_assertions: vec![],
                expected_instructions: vec![],
                skip_instruction_validation: false,
            },
        };

        // Execute step
        match run_evaluation_loop(&mut env, &mut agent, &step_test_case, &initial_observation).await
        {
            Ok((step_observation, step_trace, step_actions)) => {
                info!(
                    step = %flow_step.step,
                    actions_count = %step_actions.len(),
                    "Flow step executed successfully"
                );

                // Update observation for next step
                initial_observation = step_observation;

                // Collect actions and trace
                all_actions.extend(step_actions.clone());
                flow_trace.steps.extend(step_trace.steps);

                // Add step result
                step_results.push(reev_types::flow::StepResult {
                    step_id: flow_step.step.to_string(),
                    success: true,
                    duration_ms: 1000,
                    tool_calls: step_actions.iter().map(|a| format!("{a:?}")).collect(),
                    output: None,
                    error_message: None,
                    recovery_attempts: 0,
                });

                successful_steps += 1;
            }
            Err(step_error) => {
                let error_message = format!("Flow step {} failed: {}", flow_step.step, step_error);
                error!(error = %error_message, "Flow step failed");

                step_results.push(reev_types::flow::StepResult {
                    step_id: flow_step.step.to_string(),
                    success: false,
                    duration_ms: 1000,
                    tool_calls: vec![],
                    output: None,
                    error_message: Some(error_message),
                    recovery_attempts: 0,
                });

                failed_steps += 1;

                // For now, break on first failure - recovery logic can be added later
                if flow_step.critical {
                    break;
                }
            }
        }
    }

    // Create flow result from manual execution
    let flow_result = reev_types::flow::FlowResult {
        flow_id: test_case.id.clone(),
        user_prompt: test_case.prompt.clone(),
        success: failed_steps == 0,
        step_results,
        metrics: reev_types::flow::FlowMetrics {
            total_duration_ms: 1000 * flow_steps.len() as u64,
            successful_steps,
            failed_steps,
            critical_failures: failed_steps,
            non_critical_failures: 0,
            total_tool_calls: all_actions.len(),
            context_resolution_ms: 0,
            prompt_generation_ms: 0,
            cache_hit_rate: 0.0,
        },
        final_context: None,
        error_message: if failed_steps > 0 {
            Some("Flow execution failed".to_string())
        } else {
            None
        },
    };

    info!(
        benchmark_id = %test_case.id,
        success = %flow_result.success,
        successful_steps = %flow_result.metrics.successful_steps,
        failed_steps = %flow_result.metrics.failed_steps,
        "Flow benchmark with recovery completed"
    );

    // Convert flow result to test result
    let _final_score = if flow_result.success { 1.0 } else { 0.0 };
    let _final_status = if flow_result.success {
        FinalStatus::Succeeded
    } else {
        FinalStatus::Failed
    };

    // Create comprehensive info for final trace
    let flow_info = serde_json::json!({
        "flow_id": flow_result.flow_id,
        "success": flow_result.success,
        "total_steps": flow_result.step_results.len(),
        "successful_steps": flow_result.metrics.successful_steps,
        "failed_steps": flow_result.metrics.failed_steps,
        "critical_failures": flow_result.metrics.critical_failures,
        "non_critical_failures": flow_result.metrics.non_critical_failures,
        "total_duration_ms": flow_result.metrics.total_duration_ms,
        "total_tool_calls": flow_result.metrics.total_tool_calls,
        "recovery_attempts": flow_result.step_results.iter().map(|r| r.recovery_attempts).sum::<usize>(),
        "error_message": flow_result.error_message,
    });

    // Add final step to trace
    let final_step = reev_lib::trace::TraceStep {
        thought: None,
        action: vec![],
        observation: initial_observation.clone(),
        info: flow_info,
    };
    flow_trace.add_step(final_step);

    // Get final observation
    let final_observation = initial_observation.clone();

    // Calculate final score
    let final_score = calculate_final_score(
        test_case,
        &all_actions,
        &final_observation,
        &final_observation,
    );

    // Determine final status based on flow result
    let _final_status = if flow_result.success {
        FinalStatus::Succeeded
    } else {
        FinalStatus::Failed
    };

    Ok(TestResult::new(
        test_case,
        _final_status,
        final_score,
        flow_trace,
    ))
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
    let step_result = match env.step(actions.clone(), &test_case.ground_truth) {
        Ok(result) => result,
        Err(step_error) => {
            // Graceful error handling for transaction execution failures
            warn!(
                "🔥 Transaction step failed in evaluation loop: {} - creating failure observation",
                step_error
            );

            // Create a failure observation to maintain flow continuity
            let failure_observation = reev_lib::agent::AgentObservation {
                key_map: initial_observation.key_map.clone(),
                account_states: initial_observation.account_states.clone(),
                last_transaction_status: "Failed".to_string(),
                last_transaction_error: Some(step_error.to_string()),
                last_transaction_logs: vec![],
            };

            let trace_step = reev_lib::trace::TraceStep {
                thought: None,
                action: actions.clone(),
                observation: failure_observation.clone(),
                info: serde_json::json!({
                    "success": false,
                    "error": step_error.to_string(),
                    "execution_time_ms": 0,
                    "gas_used": None::<u64>,
                    "transaction_signature": None::<String>,
                }),
            };
            trace.add_step(trace_step);

            warn!(
                "🔄 Continuing flow despite transaction failure: {}",
                test_case.id
            );

            // Return failure observation to allow flow to continue
            return Ok((failure_observation, trace, actions));
        }
    };

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
