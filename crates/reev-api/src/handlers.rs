use crate::services::*;
use crate::types::ExecutionStatus;
use crate::types::*;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use reev_lib::db::BenchmarkYml;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tracing::warn;
use tracing::{error, info};
use uuid::Uuid;

/// Health check endpoint
pub async fn health_check() -> Json<HealthResponse> {
    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: "0.1.0".to_string(),
    };
    Json(response)
}

/// List all available benchmarks
pub async fn list_benchmarks(
    State(_state): State<ApiState>,
) -> Json<Vec<crate::types::BenchmarkInfo>> {
    // Load benchmarks dynamically from actual YAML files
    let project_root = match project_root::get_project_root() {
        Ok(root) => root,
        Err(e) => {
            error!("Failed to get project root: {}", e);
            return Json(vec![]);
        }
    };

    let benchmarks_dir = project_root.join("benchmarks");

    if !benchmarks_dir.exists() {
        error!("Benchmarks directory not found: {:?}", benchmarks_dir);
        return Json(vec![]);
    }

    let mut benchmarks = Vec::new();
    match std::fs::read_dir(&benchmarks_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "yml") {
                    if let Some(stem) = path.file_stem() {
                        let benchmark_id = stem.to_string_lossy().to_string();

                        // Parse YAML file to extract full benchmark info
                        match std::fs::read_to_string(&path) {
                            Ok(yaml_content) => {
                                match serde_yaml::from_str::<serde_yaml::Value>(&yaml_content) {
                                    Ok(yaml) => {
                                        let description = yaml
                                            .get("description")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("No description available")
                                            .to_string();

                                        let tags = yaml
                                            .get("tags")
                                            .and_then(|v| v.as_sequence())
                                            .map(|seq| {
                                                seq.iter()
                                                    .filter_map(|v| v.as_str())
                                                    .map(|s| s.to_string())
                                                    .collect()
                                            })
                                            .unwrap_or_default();

                                        let prompt = yaml
                                            .get("prompt")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();

                                        benchmarks.push(crate::types::BenchmarkInfo {
                                            id: benchmark_id,
                                            description,
                                            tags,
                                            prompt,
                                        });
                                    }
                                    Err(e) => {
                                        error!("Failed to parse YAML file {:?}: {}", path, e);
                                        // Fallback to minimal info
                                        benchmarks.push(crate::types::BenchmarkInfo {
                                            id: benchmark_id,
                                            description: "Failed to parse description".to_string(),
                                            tags: vec![],
                                            prompt: "".to_string(),
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to read YAML file {:?}: {}", path, e);
                                // Fallback to minimal info
                                benchmarks.push(crate::types::BenchmarkInfo {
                                    id: benchmark_id,
                                    description: "Failed to read description".to_string(),
                                    tags: vec![],
                                    prompt: "".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to read benchmarks directory: {}", e);
        }
    }

    benchmarks.sort_by(|a, b| a.id.cmp(&b.id));
    Json(benchmarks)
}

/// List all available agents
pub async fn list_agents() -> Json<Vec<String>> {
    let agents = vec![
        "deterministic".to_string(),
        "local".to_string(),
        "gemini".to_string(),
        "glm-4.6".to_string(),
    ];
    Json(agents)
}

/// Get agent performance summary
pub async fn get_agent_performance(State(state): State<ApiState>) -> impl IntoResponse {
    info!("Getting agent performance summary");

    match state.db.lock().await.get_agent_performance().await {
        Ok(summaries) => {
            // Debug logging for specific benchmark
            for summary in &summaries {
                if summary.agent_type == "deterministic" {
                    let latest_result = summary
                        .results
                        .iter()
                        .filter(|r| r.benchmark_id == "116-jup-lend-redeem-usdc")
                        .max_by(|a, b| a.timestamp.cmp(&b.timestamp));

                    if let Some(result) = latest_result {
                        info!("🔍 [API_DEBUG] Latest 116-jup-lend-redeem-usdc result: score={}, status={}, timestamp={}",
                              result.score, result.final_status, result.timestamp);
                    }
                }
            }
            Json::<Vec<reev_db::AgentPerformanceSummary>>(summaries).into_response()
        }
        Err(e) => {
            error!("Failed to get agent performance: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get agent performance",
            )
                .into_response()
        }
    }
}

/// Simple test endpoint
pub async fn test_endpoint() -> impl IntoResponse {
    Json(serde_json::json!({"status": "test working"}))
}

/// POST test endpoint without JSON
pub async fn test_post_endpoint() -> impl IntoResponse {
    Json(serde_json::json!({"status": "POST test working"}))
}

/// Run a benchmark
pub async fn run_benchmark(
    State(state): State<ApiState>,
    Path(benchmark_id): Path<String>,
    Json(request): Json<BenchmarkExecutionRequest>,
) -> impl IntoResponse {
    let execution_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    let execution_state = ExecutionState {
        id: execution_id.clone(),
        benchmark_id: benchmark_id.clone(),
        agent: request.agent.clone(),
        status: ExecutionStatus::Pending,
        progress: 0,
        start_time: now,
        end_time: None,
        trace: String::new(),
        logs: String::new(),
        error: None,
    };

    // Store execution state
    {
        let mut executions = state.executions.lock().await;
        executions.insert(execution_id.clone(), execution_state);
    }

    // Save agent configuration if provided
    if let Some(config) = request.config {
        let mut configs = state.agent_configs.lock().await;
        configs.insert(request.agent.clone(), config);
    }

    info!(
        "Starting benchmark execution: {} for agent: {}",
        benchmark_id, request.agent
    );

    // Start the benchmark execution in background using blocking task for non-Send dependencies
    let state_clone = state.clone();
    let execution_id_clone = execution_id.clone();
    let benchmark_id_clone = benchmark_id.clone();
    let agent = request.agent.clone();

    tokio::spawn(async move {
        tokio::task::spawn_blocking(move || {
            // Use a blocking runtime for the benchmark runner
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                execute_benchmark_background(
                    state_clone,
                    execution_id_clone,
                    benchmark_id_clone,
                    agent,
                )
                .await;
            })
        })
        .await
        .unwrap_or_else(|e| {
            error!("Benchmark execution task failed: {}", e);
        });
    });

    Json(ExecutionResponse {
        execution_id,
        status: "started".to_string(),
    })
}

/// Get execution status
pub async fn get_execution_status(
    State(state): State<ApiState>,
    Path((_benchmark_id, execution_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let executions = state.executions.lock().await;

    match executions.get(&execution_id) {
        Some(execution) => Json(execution.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, "Execution not found").into_response(),
    }
}

/// Stop a running benchmark
pub async fn stop_benchmark(
    State(state): State<ApiState>,
    Path((_benchmark_id, execution_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let mut executions = state.executions.lock().await;

    match executions.get_mut(&execution_id) {
        Some(execution) => {
            if matches!(
                execution.status,
                ExecutionStatus::Running | ExecutionStatus::Pending
            ) {
                execution.status = ExecutionStatus::Failed;
                execution.end_time = Some(chrono::Utc::now());
                execution.error = Some("Execution stopped by user".to_string());
                info!("Stopped benchmark execution: {}", execution_id);
                Json(serde_json::json!({"status": "stopped"})).into_response()
            } else {
                (StatusCode::BAD_REQUEST, "Execution is not running").into_response()
            }
        }
        None => (StatusCode::NOT_FOUND, "Execution not found").into_response(),
    }
}

/// Save agent configuration
pub async fn save_agent_config(
    State(state): State<ApiState>,
    Json(config): Json<AgentConfig>,
) -> impl IntoResponse {
    let mut configs = state.agent_configs.lock().await;
    configs.insert(config.agent_type.clone(), config.clone());

    info!("Saved configuration for agent: {}", config.agent_type);
    Json(serde_json::json!({"status": "saved"}))
}

/// Get agent configuration
pub async fn get_agent_config(
    Path(agent_type): Path<String>,
    State(state): State<ApiState>,
) -> impl IntoResponse {
    let configs = state.agent_configs.lock().await;

    match configs.get(&agent_type) {
        Some(config) => {
            // Mask API key for security
            let mut masked_config = config.clone();
            if let Some(ref api_key) = masked_config.api_key {
                if api_key.len() > 4 {
                    masked_config.api_key = Some(format!("***{}", &api_key[api_key.len() - 4..]));
                } else {
                    masked_config.api_key = Some("***".to_string());
                }
            }
            Json(masked_config).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Configuration not found").into_response(),
    }
}

/// Test agent connection
pub async fn test_agent_connection(
    State(_state): State<ApiState>,
    Json(config): Json<AgentConfig>,
) -> impl IntoResponse {
    // For now, just validate the configuration format
    if config.agent_type == "deterministic" {
        Json(serde_json::json!({
            "status": "success",
            "message": "Deterministic agent is always available"
        }))
    } else {
        // Validate that API URL and API Key are provided for LLM agents
        match (&config.api_url, &config.api_key) {
            (Some(url), Some(key)) if !url.is_empty() && !key.is_empty() => {
                Json(serde_json::json!({
                    "status": "success",
                    "message": "Configuration appears valid"
                }))
            }
            _ => Json(serde_json::json!({
                "status": "error",
                "message": "API URL and API Key are required for LLM agents"
            })),
        }
    }
}

/// Get flow logs for a benchmark
pub async fn get_flow_log(
    State(state): State<ApiState>,
    Path(benchmark_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting session logs for benchmark: {}", benchmark_id);

    // First check for active executions (like execution trace does)
    let executions = state.executions.lock().await;
    let mut active_executions = Vec::new();

    for (execution_id, execution) in executions.iter() {
        if execution.benchmark_id == benchmark_id {
            let is_running = execution.status == ExecutionStatus::Running
                || execution.status == ExecutionStatus::Pending;
            info!(
                "Execution trace debug: execution_id={}, status={:?}, is_running={}, benchmark_id={}",
                execution_id, execution.status, is_running, benchmark_id
            );
            active_executions.push(json!({
                "session_id": execution_id,
                "agent_type": execution.agent,
                "interface": "web",
                "status": format!("{:?}", execution.status).to_lowercase(),
                "score": null,
                "final_status": execution.status,
                "log_content": execution.trace.clone(),
                "is_running": is_running,
                "progress": execution.progress
            }));
        }
    }
    drop(executions);

    // If there are active executions, return them
    if !active_executions.is_empty() {
        info!(
            "Found {} active executions for benchmark: {}",
            active_executions.len(),
            benchmark_id
        );
        return Json(json!({
            "benchmark_id": benchmark_id,
            "sessions": active_executions
        }))
        .into_response();
    }

    // If no active executions, look for completed sessions
    let filter = reev_db::types::SessionFilter {
        benchmark_id: Some(benchmark_id.clone()),
        agent_type: None,
        interface: None,
        status: None,
        limit: None,
    };

    match state.db.lock().await.list_sessions(&filter).await {
        Ok(sessions) => {
            if sessions.is_empty() {
                info!(
                    "No sessions or active executions found for benchmark: {}",
                    benchmark_id
                );
                Json(json!({"message": "No sessions found", "sessions": []})).into_response()
            } else {
                // Get logs for each session
                let mut session_logs = Vec::new();
                for session in sessions {
                    match state
                        .db
                        .lock()
                        .await
                        .get_session_log(&session.session_id)
                        .await
                    {
                        Ok(log_content) => {
                            session_logs.push(json!({
                                "session_id": session.session_id,
                                "agent_type": session.agent_type,
                                "interface": session.interface,
                                "status": session.status,
                                "score": session.score,
                                "final_status": session.final_status,
                                "log_content": log_content,
                                "is_running": false
                            }));
                        }
                        Err(e) => {
                            warn!(
                                "Failed to get log for session {}: {}",
                                session.session_id, e
                            );
                            session_logs.push(json!({
                                "session_id": session.session_id,
                                "agent_type": session.agent_type,
                                "interface": session.interface,
                                "status": session.status,
                                "score": session.score,
                                "final_status": session.final_status,
                                "error": format!("Failed to retrieve log: {}", e),
                                "is_running": false
                            }));
                        }
                    }
                }

                info!(
                    "Found {} completed sessions for benchmark: {}",
                    session_logs.len(),
                    benchmark_id
                );
                Json(json!({
                    "benchmark_id": benchmark_id,
                    "sessions": session_logs
                }))
                .into_response()
            }
        }
        Err(e) => {
            error!(
                "Failed to get sessions for benchmark {}: {}",
                benchmark_id, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

/// Get transaction logs for a benchmark
pub async fn get_transaction_logs_demo(
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    info!("Generating demo transaction logs with visualization");

    // Check format parameter: yaml or plain (yaml is default)
    let format_param = params
        .get("format")
        .map_or("yaml".to_string(), |v| v.clone());
    let use_yaml = format_param == "yaml";

    // Check show_cu parameter: true or false (false is default)
    let show_cu_param = params
        .get("show_cu")
        .map_or("false".to_string(), |v| v.clone());
    let show_cu = show_cu_param == "true";

    let demo_logs = if use_yaml {
        // Generate YAML format demo
        let mock_logs = vec![
            "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]".to_string(),
            "Program log: CreateIdempotent".to_string(),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]".to_string(),
            "Program log: Instruction: GetAccountDataSize".to_string(),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1569 of 997595 compute units".to_string(),
            "Program return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=".to_string(),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
        ];

        match crate::services::generate_transaction_logs_yaml(&mock_logs, show_cu) {
            Ok(yaml_logs) => yaml_logs,
            Err(e) => {
                error!("Failed to generate YAML logs: {}", e);
                format!("Error generating YAML tree: {e}")
            }
        }
    } else {
        // Generate plain format demo
        let mock_logs = vec![
            "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]".to_string(),
            "Program log: CreateIdempotent".to_string(),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]".to_string(),
            "Program log: Instruction: GetAccountDataSize".to_string(),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1569 of 997595 compute units".to_string(),
            "Program return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=".to_string(),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
        ];

        let mut plain = String::new();
        plain.push_str("Step 1:\n");
        for log in &mock_logs {
            plain.push_str(&format!("  {log}\n"));
        }
        plain
    };

    (
        StatusCode::OK,
        Json(json!({
            "benchmark_id": "demo-jupiter-swap",
            "transaction_logs": demo_logs,
            "format": if use_yaml { "yaml" } else { "plain" },
            "message": "Demo transaction logs generated successfully"
        })),
    )
        .into_response()
}

/// Get transaction logs for a benchmark
pub async fn get_transaction_logs(
    State(state): State<ApiState>,
    Path(benchmark_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    info!("Getting transaction logs for benchmark: {}", benchmark_id);

    // First check for active executions (like execution trace does)
    let executions = state.executions.lock().await;
    info!("DEBUG: Total executions in memory: {}", executions.len());
    for (id, exec) in executions.iter() {
        info!(
            "DEBUG: Execution {} -> benchmark: {}, status: {:?}",
            id, exec.benchmark_id, exec.status
        );
    }
    for (_execution_id, execution) in executions.iter() {
        if execution.benchmark_id == benchmark_id {
            let is_running = execution.status == ExecutionStatus::Running
                || execution.status == ExecutionStatus::Pending;
            info!(
                "Found execution for benchmark: {} (status: {:?})",
                benchmark_id, execution.status
            );

            // Handle running executions like execution trace - return raw trace or loading message
            if is_running {
                info!(
                    "DEBUG: Handling running execution {} with trace length: {}",
                    _execution_id,
                    execution.trace.len()
                );
                // Check format parameter: yaml or plain (yaml is default)
                let format_param = params
                    .get("format")
                    .map_or("yaml".to_string(), |v| v.clone());

                // Check show_cu parameter: true or false (false is default)
                let show_cu_param = params
                    .get("show_cu")
                    .map_or("false".to_string(), |v| v.clone());
                let show_cu = show_cu_param == "true";

                let transaction_logs = if execution.trace.trim().is_empty() {
                    "🔄 Loading transaction logs...\n\n⏳ Execution in progress - please wait"
                        .to_string()
                } else {
                    // Try to parse and extract transaction logs from running execution
                    match serde_json::from_str::<reev_lib::trace::ExecutionTrace>(&execution.trace)
                    {
                        Ok(trace) => {
                            // Create a TestResult from the trace to use existing extraction logic
                            let test_result = reev_lib::results::TestResult::new(
                                &reev_lib::benchmark::TestCase {
                                    id: benchmark_id.clone(),
                                    description: format!("Transaction logs for {benchmark_id}"),
                                    tags: vec!["api".to_string()],
                                    initial_state: vec![],
                                    prompt: trace.prompt.clone(),
                                    flow: None,
                                    ground_truth: reev_lib::benchmark::GroundTruth {
                                        transaction_status: "unknown".to_string(),
                                        final_state_assertions: vec![],
                                        expected_instructions: vec![],
                                        skip_instruction_validation: false,
                                    },
                                },
                                reev_lib::results::FinalStatus::Succeeded,
                                1.0, // Default score for running execution
                                trace,
                            );

                            // Use appropriate transaction log extraction
                            let logs = if format_param == "yaml" {
                                match crate::services::generate_transaction_logs_yaml(
                                    &test_result
                                        .trace
                                        .steps
                                        .iter()
                                        .flat_map(|step| &step.observation.last_transaction_logs)
                                        .cloned()
                                        .collect::<Vec<_>>(),
                                    show_cu,
                                ) {
                                    Ok(yaml_logs) => yaml_logs,
                                    Err(e) => {
                                        error!(
                                            "Failed to generate YAML logs from execution: {}",
                                            e
                                        );
                                        format!("Error generating YAML tree: {e}")
                                    }
                                }
                            } else {
                                crate::services::generate_transaction_logs(&test_result)
                            };

                            // Add status indicator for running executions
                            if !logs.trim().is_empty() {
                                format!(
                                    "{logs}\n\n⏳ Execution in progress - logs may be incomplete"
                                )
                            } else {
                                logs
                            }
                        }
                        Err(_) => {
                            // Failed to parse - show raw trace with processing message
                            format!("🔄 Processing transaction logs...\n\n⚠️ Unable to parse execution trace - still running\n\nRaw trace preview:\n{}",
                                &execution.trace[..execution.trace.len().min(500)])
                        }
                    }
                };

                info!(
                    "Returning transaction logs from running execution for benchmark: {} ({} chars, format: {}, show_cu: {})",
                    benchmark_id,
                    transaction_logs.len(),
                    format_param,
                    show_cu
                );

                return (
                    StatusCode::OK,
                    Json(json!({
                        "benchmark_id": benchmark_id,
                        "transaction_logs": transaction_logs,
                        "format": format_param,
                        "show_cu": show_cu,
                        "message": "Transaction logs from active execution (may be incomplete)",
                        "is_running": true
                    })),
                )
                    .into_response();
            }

            // For completed executions, try to parse the trace
            info!(
                "DEBUG: Completed execution {} found, attempting to parse trace (length: {} chars)",
                _execution_id,
                execution.trace.len()
            );

            if let Ok(trace) =
                serde_json::from_str::<reev_lib::trace::ExecutionTrace>(&execution.trace)
            {
                // Create a TestResult from the trace to use existing extraction logic
                let test_result = reev_lib::results::TestResult::new(
                    &reev_lib::benchmark::TestCase {
                        id: benchmark_id.clone(),
                        description: format!("Transaction logs for {benchmark_id}"),
                        tags: vec!["api".to_string()],
                        initial_state: vec![],
                        prompt: trace.prompt.clone(),
                        flow: None,
                        ground_truth: reev_lib::benchmark::GroundTruth {
                            transaction_status: "unknown".to_string(),
                            final_state_assertions: vec![],
                            expected_instructions: vec![],
                            skip_instruction_validation: false,
                        },
                    },
                    reev_lib::results::FinalStatus::Succeeded,
                    1.0,
                    trace,
                );

                // Check format parameter: yaml or plain (yaml is default)
                let format_param = params
                    .get("format")
                    .map_or("yaml".to_string(), |v| v.clone());
                let use_yaml = format_param == "yaml";

                // Check show_cu parameter: true or false (false is default)
                let show_cu_param = params
                    .get("show_cu")
                    .map_or("false".to_string(), |v| v.clone());
                let show_cu = show_cu_param == "true";

                // Use appropriate transaction log extraction
                let transaction_logs = if use_yaml {
                    match crate::services::generate_transaction_logs_yaml(
                        &test_result
                            .trace
                            .steps
                            .iter()
                            .flat_map(|step| &step.observation.last_transaction_logs)
                            .cloned()
                            .collect::<Vec<_>>(),
                        show_cu,
                    ) {
                        Ok(yaml_logs) => yaml_logs,
                        Err(e) => {
                            error!("Failed to generate YAML logs from execution: {}", e);
                            format!("Error generating YAML tree: {e}")
                        }
                    }
                } else {
                    crate::services::generate_transaction_logs(&test_result)
                };

                info!(
                    "Extracted transaction logs from completed execution for benchmark: {} ({} chars, format: {}, show_cu: {})",
                    benchmark_id,
                    transaction_logs.len(),
                    if use_yaml { "yaml" } else { "plain" },
                    show_cu
                );

                return (
                    StatusCode::OK,
                    Json(json!({
                        "benchmark_id": benchmark_id,
                        "transaction_logs": transaction_logs,
                        "format": if use_yaml { "yaml" } else { "plain" },
                        "show_cu": show_cu,
                        "message": "Transaction logs from completed execution",
                        "is_running": false
                    })),
                )
                    .into_response();
            } else {
                warn!(
                    "Failed to parse trace for completed execution: {} - JSON parsing error, trace length: {}",
                    _execution_id,
                    execution.trace.len()
                );
            }
        }
    }

    if executions.is_empty() {
        info!(
            "No executions found in memory for benchmark: {}",
            benchmark_id
        );
    } else {
        info!(
            "Checked {} executions, none matched benchmark: {}",
            executions.len(),
            benchmark_id
        );
    }
    drop(executions);

    // Get the most recent session for this benchmark
    let filter = reev_db::types::SessionFilter {
        benchmark_id: Some(benchmark_id.clone()),
        agent_type: None,
        interface: None,
        status: None,
        limit: Some(1), // Get the most recent session
    };

    match state.db.lock().await.list_sessions(&filter).await {
        Ok(sessions) => {
            if let Some(session) = sessions.first() {
                info!("Found session for benchmark: {}", benchmark_id);

                // Get the session log which contains the execution trace
                match state
                    .db
                    .lock()
                    .await
                    .get_session_log(&session.session_id)
                    .await
                {
                    Ok(log_content) => {
                        // Try to parse as ExecutionTrace and extract transaction logs
                        match serde_json::from_str::<reev_lib::trace::ExecutionTrace>(&log_content)
                        {
                            Ok(trace) => {
                                // Create a TestResult from the trace to use existing extraction logic
                                let test_result = reev_lib::results::TestResult::new(
                                    &reev_lib::benchmark::TestCase {
                                        id: benchmark_id.clone(),
                                        description: format!("Transaction logs for {benchmark_id}"),
                                        tags: vec!["api".to_string()],
                                        initial_state: vec![],
                                        prompt: trace.prompt.clone(),
                                        flow: None,
                                        ground_truth: reev_lib::benchmark::GroundTruth {
                                            transaction_status: "unknown".to_string(),
                                            final_state_assertions: vec![],
                                            expected_instructions: vec![],
                                            skip_instruction_validation: false,
                                        },
                                    },
                                    reev_lib::results::FinalStatus::Succeeded,
                                    session.score.unwrap_or(1.0),
                                    trace,
                                );

                                // Check format parameter: yaml or plain (yaml is default)
                                let format_param = params
                                    .get("format")
                                    .map_or("yaml".to_string(), |v| v.clone());
                                let use_yaml = format_param == "yaml";

                                // Check show_cu parameter: true or false (false is default)
                                let show_cu_param = params
                                    .get("show_cu")
                                    .map_or("false".to_string(), |v| v.clone());
                                let show_cu = show_cu_param == "true";

                                // Use appropriate transaction log extraction
                                let transaction_logs = if use_yaml {
                                    match crate::services::generate_transaction_logs_yaml(
                                        &test_result
                                            .trace
                                            .steps
                                            .iter()
                                            .flat_map(|step| {
                                                &step.observation.last_transaction_logs
                                            })
                                            .cloned()
                                            .collect::<Vec<_>>(),
                                        show_cu,
                                    ) {
                                        Ok(yaml_logs) => yaml_logs,
                                        Err(e) => {
                                            error!("Failed to generate YAML logs: {}", e);
                                            format!("Error generating YAML tree: {e}")
                                        }
                                    }
                                } else {
                                    crate::services::generate_transaction_logs(&test_result)
                                };

                                info!(
                                    "Extracted transaction logs for benchmark: {} ({} chars, format: {}, show_cu: {})",
                                    benchmark_id,
                                    transaction_logs.len(),
                                    if use_yaml { "yaml" } else { "plain" },
                                    show_cu
                                );

                                if transaction_logs.trim().is_empty() {
                                    return (
                                        StatusCode::OK,
                                        Json(json!({
                                            "benchmark_id": benchmark_id,
                                            "transaction_logs": "",
                                            "format": format_param,
                                            "show_cu": show_cu,
                                            "message": "No transaction logs available",
                                            "is_running": false
                                        })),
                                    )
                                        .into_response();
                                }

                                (
                                    StatusCode::OK,
                                    Json(json!({
                                        "benchmark_id": benchmark_id,
                                        "format": format_param,
                                        "show_cu": show_cu,
                                        "message": "Transaction logs extracted successfully",
                                        "transaction_logs": transaction_logs,
                                        "is_running": false
                                    })),
                                )
                                    .into_response()
                            }
                            Err(e) => {
                                warn!("Failed to parse log as ExecutionTrace: {}", e);

                                (
                                    StatusCode::OK,
                                    Json(json!({
                                        "benchmark_id": benchmark_id,
                                        "transaction_logs": "",
                                        "format": if params.get("format").is_some_and(|f| f == "tree") { "tree" } else { "plain" },
                                        "show_cu": params.get("show_cu").unwrap_or(&"false".to_string()) == "true",
                                        "message": "No valid ExecutionTrace data found",
                                        "is_running": false
                                    })),
                                ).into_response()
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to get session log: {}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "error": "Failed to retrieve session log"
                            })),
                        )
                            .into_response()
                    }
                }
            } else {
                info!("No sessions found for benchmark: {}", benchmark_id);
                (
                    StatusCode::OK,
                    Json(json!({
                        "benchmark_id": benchmark_id,
                        "transaction_logs": "",
                        "format": if params.get("format").is_some_and(|f| f == "tree") { "tree" } else { "plain" },
                        "show_cu": params.get("show_cu").unwrap_or(&"false".to_string()) == "true",
                        "message": format!("No sessions found for benchmark: {}", benchmark_id),
                        "is_running": false
                    })),
                ).into_response()
            }
        }
        Err(e) => {
            error!("Failed to list sessions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Database error"
                })),
            )
                .into_response()
        }
    }
}

/// Get ASCII tree directly from YML TestResult in database
pub async fn get_ascii_tree_direct(
    State(state): State<ApiState>,
    Path((benchmark_id, agent_type)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting execution log for benchmark: {} by agent: {}",
        benchmark_id, agent_type
    );

    // Get the most recent session for this benchmark and agent
    let filter = reev_db::types::SessionFilter {
        benchmark_id: Some(benchmark_id.clone()),
        agent_type: Some(agent_type.clone()),
        interface: None,
        status: None,
        limit: Some(1), // Get the most recent session
    };

    match state.db.lock().await.list_sessions(&filter).await {
        Ok(sessions) => {
            if let Some(session) = sessions.first() {
                info!(
                    "Found session for benchmark: {} by agent: {} with status: {}",
                    benchmark_id,
                    agent_type,
                    session.final_status.as_deref().unwrap_or("Unknown")
                );

                // Check execution status
                match session.final_status.as_deref() {
                    Some("Running") | Some("running") => (
                        StatusCode::OK,
                        [("Content-Type", "text/plain")],
                        "⏳ Execution in progress...".to_string(),
                    )
                        .into_response(),
                    Some("Completed") | Some("Succeeded") | Some("completed")
                    | Some("succeeded") => {
                        // Get the session log which contains the full execution output
                        match state
                            .db
                            .lock()
                            .await
                            .get_session_log(&session.session_id)
                            .await
                        {
                            Ok(log_content) => {
                                if log_content.trim().is_empty() {
                                    return (
                                        StatusCode::OK,
                                        [("Content-Type", "text/plain")],
                                        "📝 No execution data available".to_string(),
                                    )
                                        .into_response();
                                }

                                // Try to parse as ExecutionTrace first (new format)
                                match serde_json::from_str::<reev_lib::trace::ExecutionTrace>(
                                    &log_content,
                                ) {
                                    Ok(trace) => {
                                        // Create a TestResult from the trace for rendering
                                        let test_result = reev_lib::results::TestResult::new(
                                            &reev_lib::benchmark::TestCase {
                                                id: benchmark_id.clone(),
                                                description: format!(
                                                    "Execution result for {benchmark_id}"
                                                ),
                                                tags: vec!["web".to_string()],
                                                initial_state: vec![],
                                                prompt: trace.prompt.clone(),
                                                flow: None,
                                                ground_truth: reev_lib::benchmark::GroundTruth {
                                                    transaction_status: "unknown".to_string(),
                                                    final_state_assertions: vec![],
                                                    expected_instructions: vec![],
                                                    skip_instruction_validation: false,
                                                },
                                            },
                                            reev_lib::results::FinalStatus::Succeeded,
                                            session.score.unwrap_or(1.0),
                                            trace,
                                        );

                                        // Render as ASCII tree using the existing renderer
                                        let ascii_tree =
                                            reev_runner::renderer::render_result_as_tree(
                                                &test_result,
                                            );

                                        info!(
                                            "Rendered ASCII tree for benchmark: {} ({} chars) - New format",
                                            benchmark_id,
                                            ascii_tree.len()
                                        );
                                        (
                                            StatusCode::OK,
                                            [("Content-Type", "text/plain")],
                                            ascii_tree,
                                        )
                                            .into_response()
                                    }
                                    Err(_) => {
                                        // Try to parse as old SessionLog format and extract ExecutionTrace
                                        match serde_json::from_str::<
                                            reev_lib::session_logger::SessionLog,
                                        >(&log_content)
                                        {
                                            Ok(session_log) => {
                                                // Check if final_result contains ExecutionTrace in data field
                                                if let Some(final_result) =
                                                    &session_log.final_result
                                                {
                                                    if let Ok(trace) = serde_json::from_value::<
                                                        reev_lib::trace::ExecutionTrace,
                                                    >(
                                                        final_result.data.clone()
                                                    ) {
                                                        // Create a TestResult from the extracted trace
                                                        let test_result = reev_lib::results::TestResult::new(
                                                            &reev_lib::benchmark::TestCase {
                                                                id: benchmark_id.clone(),
                                                                description: format!(
                                                                    "Execution result for {benchmark_id}"
                                                                ),
                                                                tags: vec!["tui".to_string()],
                                                                initial_state: vec![],
                                                                prompt: trace.prompt.clone(),
                                                                flow: None,
                                                                ground_truth: reev_lib::benchmark::GroundTruth {
                                                                    transaction_status: "unknown".to_string(),
                                                                    final_state_assertions: vec![],
                                                                    expected_instructions: vec![],
                                                                    skip_instruction_validation: false,
                                                                },
                                                            },
                                                            reev_lib::results::FinalStatus::Succeeded,
                                                            final_result.score,
                                                            trace,
                                                        );

                                                        // Render as ASCII tree using the existing renderer
                                                        let ascii_tree =
                                                            reev_runner::renderer::render_result_as_tree(
                                                                &test_result,
                                                            );

                                                        info!(
                                                            "Rendered ASCII tree for benchmark: {} ({} chars) - Migrated format",
                                                            benchmark_id,
                                                            ascii_tree.len()
                                                        );
                                                        (
                                                            StatusCode::OK,
                                                            [("Content-Type", "text/plain")],
                                                            ascii_tree,
                                                        )
                                                            .into_response()
                                                    } else {
                                                        warn!("Failed to extract ExecutionTrace from SessionLog data");
                                                        (
                                                            StatusCode::OK,
                                                            [("Content-Type", "text/plain")],
                                                            format!("❌ Legacy session format detected - unable to extract ExecutionTrace\n\nRaw log preview:\n{}", &log_content[..log_content.len().min(500)]),
                                                        )
                                                            .into_response()
                                                    }
                                                } else {
                                                    warn!("SessionLog has no final_result");
                                                    (
                                                        StatusCode::OK,
                                                        [("Content-Type", "text/plain")],
                                                        format!("❌ Legacy session format detected - no final_result available\n\nRaw log preview:\n{}", &log_content[..log_content.len().min(500)]),
                                                    )
                                                        .into_response()
                                                }
                                            }
                                            Err(e) => {
                                                warn!("Failed to parse log as both ExecutionTrace and SessionLog: {}", e);
                                                // Return raw log if not a valid format
                                                info!(
                                                    "Returning raw execution log for benchmark: {} ({} chars)",
                                                    benchmark_id,
                                                    log_content.len()
                                                );
                                                (
                                                    StatusCode::OK,
                                                    [("Content-Type", "text/plain")],
                                                    format!("❌ Unknown session format - cannot render ASCII tree\n\nRaw log:\n{}", &log_content[..log_content.len().min(1000)]),
                                                )
                                                    .into_response()
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to get session log: {}", e);
                                (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    [("Content-Type", "text/plain")],
                                    "❌ Failed to retrieve execution data".to_string(),
                                )
                                    .into_response()
                            }
                        }
                    }
                    Some("Failed") | Some("failed") | Some("Error") | Some("error") => {
                        // Get the session log even for failed executions to show error details
                        match state
                            .db
                            .lock()
                            .await
                            .get_session_log(&session.session_id)
                            .await
                        {
                            Ok(log_content) => {
                                if log_content.trim().is_empty() {
                                    return (
                                        StatusCode::OK,
                                        [("Content-Type", "text/plain")],
                                        "❌ Execution failed - No details available".to_string(),
                                    )
                                        .into_response();
                                }

                                // Try to parse as ExecutionTrace first (new format)
                                match serde_json::from_str::<reev_lib::trace::ExecutionTrace>(
                                    &log_content,
                                ) {
                                    Ok(trace) => {
                                        // Create a TestResult from the trace for rendering
                                        let test_result = reev_lib::results::TestResult::new(
                                            &reev_lib::benchmark::TestCase {
                                                id: benchmark_id.clone(),
                                                description: format!(
                                                    "Failed execution result for {benchmark_id}"
                                                ),
                                                tags: vec!["web".to_string()],
                                                initial_state: vec![],
                                                prompt: trace.prompt.clone(),
                                                flow: None,
                                                ground_truth: reev_lib::benchmark::GroundTruth {
                                                    transaction_status: "unknown".to_string(),
                                                    final_state_assertions: vec![],
                                                    expected_instructions: vec![],
                                                    skip_instruction_validation: false,
                                                },
                                            },
                                            reev_lib::results::FinalStatus::Failed,
                                            session.score.unwrap_or(0.0),
                                            trace,
                                        );

                                        // Render as ASCII tree using the existing renderer
                                        let ascii_tree =
                                            reev_runner::renderer::render_result_as_tree(
                                                &test_result,
                                            );

                                        info!(
                                            "Rendered ASCII tree for failed benchmark: {} ({} chars) - New format",
                                            benchmark_id,
                                            ascii_tree.len()
                                        );
                                        (
                                            StatusCode::OK,
                                            [("Content-Type", "text/plain")],
                                            format!("❌ Execution Failed\n\n{ascii_tree}"),
                                        )
                                            .into_response()
                                    }
                                    Err(_) => {
                                        // Try to parse as old SessionLog format and extract ExecutionTrace
                                        match serde_json::from_str::<
                                            reev_lib::session_logger::SessionLog,
                                        >(&log_content)
                                        {
                                            Ok(session_log) => {
                                                // Check if final_result contains ExecutionTrace in data field
                                                if let Some(final_result) =
                                                    &session_log.final_result
                                                {
                                                    if let Ok(trace) = serde_json::from_value::<
                                                        reev_lib::trace::ExecutionTrace,
                                                    >(
                                                        final_result.data.clone()
                                                    ) {
                                                        // Create a TestResult from the extracted trace
                                                        let test_result = reev_lib::results::TestResult::new(
                                                            &reev_lib::benchmark::TestCase {
                                                                id: benchmark_id.clone(),
                                                                description: format!(
                                                                    "Failed execution result for {benchmark_id}"
                                                                ),
                                                                tags: vec!["tui".to_string()],
                                                                initial_state: vec![],
                                                                prompt: trace.prompt.clone(),
                                                                flow: None,
                                                                ground_truth: reev_lib::benchmark::GroundTruth {
                                                                    transaction_status: "unknown".to_string(),
                                                                    final_state_assertions: vec![],
                                                                    expected_instructions: vec![],
                                                                    skip_instruction_validation: false,
                                                                },
                                                            },
                                                            reev_lib::results::FinalStatus::Failed,
                                                            final_result.score,
                                                            trace,
                                                        );

                                                        // Render as ASCII tree using the existing renderer
                                                        let ascii_tree =
                                                            reev_runner::renderer::render_result_as_tree(
                                                                &test_result,
                                                            );

                                                        info!(
                                                            "Rendered ASCII tree for failed benchmark: {} ({} chars) - Migrated format",
                                                            benchmark_id,
                                                            ascii_tree.len()
                                                        );
                                                        (
                                                            StatusCode::OK,
                                                            [("Content-Type", "text/plain")],
                                                            format!("❌ Execution Failed\n\n{ascii_tree}"),
                                                        )
                                                            .into_response()
                                                    } else {
                                                        warn!("Failed to extract ExecutionTrace from failed SessionLog data");
                                                        (
                                                            StatusCode::OK,
                                                            [("Content-Type", "text/plain")],
                                                            format!("❌ Failed execution - Legacy session format detected, unable to extract ExecutionTrace\n\nRaw log preview:\n{}", &log_content[..log_content.len().min(500)]),
                                                        )
                                                            .into_response()
                                                    }
                                                } else {
                                                    warn!("Failed SessionLog has no final_result");
                                                    (
                                                        StatusCode::OK,
                                                        [("Content-Type", "text/plain")],
                                                        format!("❌ Failed execution - Legacy session format detected, no final_result available\n\nRaw log preview:\n{}", &log_content[..log_content.len().min(500)]),
                                                    )
                                                        .into_response()
                                                }
                                            }
                                            Err(e) => {
                                                warn!("Failed to parse failed log as both ExecutionTrace and SessionLog: {}", e);
                                                // Return raw log if not a valid format
                                                info!(
                                                    "Returning raw failed execution log for benchmark: {} ({} chars)",
                                                    benchmark_id,
                                                    log_content.len()
                                                );
                                                (
                                                    StatusCode::OK,
                                                    [("Content-Type", "text/plain")],
                                                    format!("❌ Failed execution - Unknown session format, cannot render ASCII tree\n\nRaw log:\n{}", &log_content[..log_content.len().min(1000)]),
                                                )
                                                    .into_response()
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to get session log for failed execution: {}", e);
                                (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    [("Content-Type", "text/plain")],
                                    "❌ Failed to retrieve error details".to_string(),
                                )
                                    .into_response()
                            }
                        }
                    }
                    _ => {
                        info!(
                            "Unknown session status: {} for benchmark: {}",
                            session.final_status.as_deref().unwrap_or("None"),
                            benchmark_id
                        );
                        (
                            StatusCode::OK,
                            [("Content-Type", "text/plain")],
                            "❓ Unknown execution status".to_string(),
                        )
                            .into_response()
                    }
                }
            } else {
                info!(
                    "No sessions found for benchmark: {} by agent: {}",
                    benchmark_id, agent_type
                );
                (
                    StatusCode::OK,
                    [("Content-Type", "text/plain")],
                    "📭 No execution data found".to_string(),
                )
                    .into_response()
            }
        }
        Err(e) => {
            error!("Failed to list sessions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("Content-Type", "text/plain")],
                "❌ Database error".to_string(),
            )
                .into_response()
        }
    }
}

/// Render TestResult as ASCII tree
pub async fn render_ascii_tree(Json(test_result): Json<serde_json::Value>) -> impl IntoResponse {
    info!("Rendering ASCII tree for TestResult");

    // Parse the TestResult from JSON
    let test_result: reev_lib::results::TestResult = match serde_json::from_value(test_result) {
        Ok(result) => result,
        Err(e) => {
            error!("Failed to parse TestResult: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid TestResult format").into_response();
        }
    };

    // Render as ASCII tree
    let ascii_tree = reev_runner::renderer::render_result_as_tree(&test_result);

    info!("Successfully rendered ASCII tree");
    (StatusCode::OK, [("Content-Type", "text/plain")], ascii_tree).into_response()
}

/// Parse YML to TestResult
pub async fn parse_yml_to_testresult(yml_content: String) -> impl IntoResponse {
    info!("Parsing YML to TestResult");
    info!("YML content length: {} chars", yml_content.len());
    info!(
        "YML content preview: {}",
        &yml_content[..yml_content.len().min(200)]
    );

    // Log the first few lines to understand the format
    let lines: Vec<&str> = yml_content.lines().take(5).collect();
    info!("YML first 5 lines: {:?}", lines);

    // Parse YML to TestResult object
    let test_result: reev_lib::results::TestResult = match serde_yaml::from_str(&yml_content) {
        Ok(result) => {
            info!("Successfully parsed YML to TestResult");
            result
        }
        Err(e) => {
            error!("Failed to parse YML to TestResult: {}", e);
            error!(
                "YML content that failed: {}",
                &yml_content[..yml_content.len().min(500)]
            );
            return (StatusCode::BAD_REQUEST, format!("Invalid YML format: {e}")).into_response();
        }
    };

    info!("Successfully parsed YML to TestResult");
    Json(test_result).into_response()
}

/// Get ASCII tree from current execution state (temporary fix)
pub async fn get_ascii_tree_from_state(
    State(state): State<ApiState>,
    Path((benchmark_id, agent_type)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting ASCII tree from execution state for benchmark: {} by agent: {}",
        benchmark_id, agent_type
    );

    let executions = state.executions.lock().await;

    // Find the most recent execution for this benchmark and agent
    let mut matching_execution = None;
    for execution in executions.values() {
        if execution.benchmark_id == benchmark_id && execution.agent == agent_type {
            match matching_execution {
                None => matching_execution = Some(execution),
                Some(current) => {
                    if execution.start_time > current.start_time {
                        matching_execution = Some(execution);
                    }
                }
            }
        }
    }

    match matching_execution {
        Some(execution) => {
            info!(
                "Found execution with status: {}",
                match execution.status {
                    ExecutionStatus::Pending => "Pending",
                    ExecutionStatus::Running => "Running",
                    ExecutionStatus::Completed => "Completed",
                    ExecutionStatus::Failed => "Failed",
                }
            );

            if !execution.trace.is_empty() {
                info!(
                    "Returning ASCII tree trace ({} chars)",
                    execution.trace.len()
                );
                (
                    StatusCode::OK,
                    [("Content-Type", "text/plain")],
                    execution.trace.clone(),
                )
                    .into_response()
            } else {
                (
                    StatusCode::NOT_FOUND,
                    "No trace available for this execution",
                )
                    .into_response()
            }
        }
        None => {
            info!(
                "No execution found for benchmark: {} by agent: {}",
                benchmark_id, agent_type
            );
            (StatusCode::NOT_FOUND, "No execution found").into_response()
        }
    }
}

/// Upsert YML content to database
pub async fn upsert_yml(
    State(app_state): State<ApiState>,
    Json(payload): Json<UpsertYmlRequest>,
) -> impl IntoResponse {
    let db_writer = &app_state.db.lock().await;

    // Validate YML content
    let benchmark_data: BenchmarkYml = match serde_yaml::from_str(&payload.yml_content) {
        Ok(data) => data,
        Err(e) => {
            error!("Invalid YAML format: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Invalid YAML format: {}", e)
                })),
            )
                .into_response();
        }
    };

    // Upsert to database
    let prompt_md5 = match db_writer
        .upsert_benchmark(
            &benchmark_data.id,
            &benchmark_data.prompt,
            &payload.yml_content,
        )
        .await
    {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to upsert benchmark: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to upsert benchmark: {}", e)
                })),
            )
                .into_response();
        }
    };

    info!("Upserted benchmark with MD5: {}", prompt_md5);

    (
        StatusCode::OK,
        Json(UpsertYmlResponse {
            success: true,
            benchmark_id: prompt_md5,
            message: "Benchmark upserted successfully".to_string(),
        }),
    )
        .into_response()
}

/// Request body for upsert_yml endpoint
#[derive(Debug, Deserialize)]
pub struct UpsertYmlRequest {
    pub yml_content: String,
}

/// Response body for upsert_yml endpoint
#[derive(Debug, Serialize)]
pub struct UpsertYmlResponse {
    pub success: bool,
    pub benchmark_id: String,
    pub message: String,
}

/// Helper function to create error responses
#[allow(dead_code)]
pub fn create_error_response(
    status: StatusCode,
    message: String,
) -> (StatusCode, Json<ErrorResponse>) {
    let response = ErrorResponse {
        error: status.as_str().to_string(),
        message,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    (status, Json(response))
}

/// Debug endpoint to check benchmarks table
pub async fn debug_benchmarks(State(state): State<ApiState>) -> impl IntoResponse {
    let db = &state.db.lock().await;

    // Get all benchmarks from database
    match db.get_all_benchmarks().await {
        Ok(benchmarks) => {
            let debug_info: Vec<serde_json::Value> = benchmarks
                .into_iter()
                .map(|b| {
                    serde_json::json!({
                        "id": b.id,
                        "prompt_preview": b.prompt.chars().take(50).collect::<String>(),
                        "benchmark_name": b.benchmark_name,
                        "created_at": b.created_at,
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "count": debug_info.len(),
                    "benchmarks": debug_info
                })),
            )
        }
        Err(e) => {
            error!("Failed to get benchmarks: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to get benchmarks: {}", e)
                })),
            )
        }
    }
}

/// Test ON CONFLICT behavior with simple data
pub async fn test_on_conflict(State(state): State<ApiState>) -> impl IntoResponse {
    let db = &state.db.lock().await;

    // Test 1: Insert first record using existing upsert function
    let result1 = db
        .upsert_benchmark("test-conflict-1", "test-prompt-1", "test-content-1")
        .await;

    // Test 2: Insert second record with SAME benchmark_name AND SAME prompt (should trigger conflict)
    let result2 = db
        .upsert_benchmark(
            "test-conflict-1", // Same benchmark_name as first record
            "test-prompt-1",   // SAME prompt - should generate same MD5
            "test-content-1-updated",
        )
        .await;

    // Check results using existing database functions
    let total_records = db.get_all_benchmark_count().await.unwrap_or(0);

    let success = total_records == 1;
    let message = if success {
        "✅ Turso ON CONFLICT works correctly - no duplicates created"
    } else {
        "❌ Turso ON CONFLICT failed - duplicates created"
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": success,
            "message": message,
            "test_results": {
                "first_insert_result": format!("{:?}", result1),
                "second_insert_result": format!("{:?}", result2),
                "total_records": total_records,
                "expected_records": 1
            }
        })),
    )
}

/// Sync benchmarks from filesystem to database
pub async fn sync_benchmarks(State(state): State<ApiState>) -> impl IntoResponse {
    let db = &state.db.lock().await;
    let benchmarks_dir = "benchmarks";

    info!(
        "Starting manual benchmark sync from directory: {}",
        benchmarks_dir
    );

    match db.sync_benchmarks_from_dir(benchmarks_dir).await {
        Ok(synced_count) => {
            info!(
                "Successfully synced {:?} benchmarks to database",
                synced_count
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "synced_count": synced_count,
                    "message": format!("Successfully synced {:?} benchmarks from {}", synced_count, benchmarks_dir)
                })),
            )
        }
        Err(e) => {
            error!("Failed to sync benchmarks: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to sync benchmarks: {}", e)
                })),
            )
        }
    }
}

/// Manual test endpoint to test prompt MD5 lookup
pub async fn test_prompt_md5_lookup(
    State(_state): State<ApiState>,
    Json(_request): Json<serde_json::Value>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "success": false,
            "error": "Test endpoint not implemented"
        })),
    )
}
