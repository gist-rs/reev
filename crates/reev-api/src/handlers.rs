use crate::services::*;
use crate::types::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use reev_lib::db::BenchmarkYml;
use serde::{Deserialize, Serialize};
use serde_json::json;
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

    match state.db.get_agent_performance().await {
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
            if matches!(execution.status, ExecutionStatus::Running) {
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

    // Use session management to get logs for this benchmark
    let filter = reev_db::types::SessionFilter {
        benchmark_id: Some(benchmark_id.clone()),
        agent_type: None,
        interface: None,
        status: None,
        limit: None,
    };

    match state.db.list_sessions(&filter).await {
        Ok(sessions) => {
            if sessions.is_empty() {
                info!("No sessions found for benchmark: {}", benchmark_id);
                Json(json!({"message": "No sessions found", "sessions": []})).into_response()
            } else {
                // Get logs for each session
                let mut session_logs = Vec::new();
                for session in sessions {
                    match state.db.get_session_log(&session.session_id).await {
                        Ok(log_content) => {
                            session_logs.push(json!({
                                "session_id": session.session_id,
                                "agent_type": session.agent_type,
                                "interface": session.interface,
                                "status": session.status,
                                "score": session.score,
                                "final_status": session.final_status,
                                "log_content": log_content
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
                                "error": format!("Failed to retrieve log: {}", e)
                            }));
                        }
                    }
                }

                info!(
                    "Found {} sessions for benchmark: {}",
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
pub async fn get_transaction_logs(
    State(state): State<ApiState>,
    Path(benchmark_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting transaction logs for benchmark: {}", benchmark_id);

    // Get the most recent session for this benchmark
    let filter = reev_db::types::SessionFilter {
        benchmark_id: Some(benchmark_id.clone()),
        agent_type: None,
        interface: None,
        status: None,
        limit: Some(1), // Get the most recent session
    };

    match state.db.list_sessions(&filter).await {
        Ok(sessions) => {
            if let Some(session) = sessions.first() {
                info!("Found session for benchmark: {}", benchmark_id);

                // Get the session log which contains the execution trace
                match state.db.get_session_log(&session.session_id).await {
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

                                // Use existing backend transaction log extraction
                                let transaction_logs =
                                    crate::services::generate_transaction_logs(&test_result);

                                info!(
                                    "Extracted transaction logs for benchmark: {} ({} chars)",
                                    benchmark_id,
                                    transaction_logs.len()
                                );

                                if transaction_logs.trim().is_empty() {
                                    return (
                                        StatusCode::OK,
                                        Json(json!({
                                            "benchmark_id": benchmark_id,
                                            "transaction_logs": "",
                                            "message": "No transaction logs available"
                                        })),
                                    )
                                        .into_response();
                                }

                                (
                                    StatusCode::OK,
                                    Json(json!({
                                        "benchmark_id": benchmark_id,
                                        "transaction_logs": transaction_logs,
                                        "message": "Transaction logs extracted successfully"
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
                                        "message": "No valid ExecutionTrace data found"
                                    })),
                                )
                                    .into_response()
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
                        "message": "No execution data found"
                    })),
                )
                    .into_response()
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

    match state.db.list_sessions(&filter).await {
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
                        match state.db.get_session_log(&session.session_id).await {
                            Ok(log_content) => {
                                if log_content.trim().is_empty() {
                                    return (
                                        StatusCode::OK,
                                        [("Content-Type", "text/plain")],
                                        "📝 No execution data available".to_string(),
                                    )
                                        .into_response();
                                }

                                // Try to parse as ExecutionTrace and render as ASCII tree
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
                                            "Rendered ASCII tree for benchmark: {} ({} chars)",
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
                                    Err(e) => {
                                        warn!("Failed to parse log as ExecutionTrace: {}", e);
                                        // Return raw log if not a valid ExecutionTrace
                                        info!(
                                            "Returning raw execution log for benchmark: {} ({} chars)",
                                            benchmark_id,
                                            log_content.len()
                                        );
                                        (
                                            StatusCode::OK,
                                            [("Content-Type", "text/plain")],
                                            log_content,
                                        )
                                            .into_response()
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
                        match state.db.get_session_log(&session.session_id).await {
                            Ok(log_content) => {
                                if log_content.trim().is_empty() {
                                    return (
                                        StatusCode::OK,
                                        [("Content-Type", "text/plain")],
                                        "❌ Execution failed - No details available".to_string(),
                                    )
                                        .into_response();
                                }

                                // Try to parse as ExecutionTrace and render as ASCII tree even for failed executions
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
                                            "Rendered ASCII tree for failed benchmark: {} ({} chars)",
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
                                    Err(e) => {
                                        warn!(
                                            "Failed to parse failed log as ExecutionTrace: {}",
                                            e
                                        );
                                        // Return raw log if not a valid ExecutionTrace
                                        info!(
                                            "Returning raw failed execution log for benchmark: {} ({} chars)",
                                            benchmark_id,
                                            log_content.len()
                                        );
                                        (
                                            StatusCode::OK,
                                            [("Content-Type", "text/plain")],
                                            format!("❌ Execution Failed\n\n{log_content}"),
                                        )
                                            .into_response()
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
    let db_writer = &app_state.db;

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
    let db = &state.db;

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
    let db = &state.db;

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
    let db = &state.db;
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
