use crate::services::*;
use crate::types::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use reev_lib::db::BenchmarkYml;
use serde::{Deserialize, Serialize};
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
            Json(summaries).into_response()
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
    info!("Getting YML flow logs for benchmark: {}", benchmark_id);

    match state.db.get_yml_flow_logs(&benchmark_id).await {
        Ok(yml_logs) => {
            info!(
                "Found {} YML logs for benchmark: {}",
                yml_logs.len(),
                benchmark_id
            );
            for (i, log) in yml_logs.iter().enumerate() {
                let preview = if log.is_empty() {
                    "<empty>".to_string()
                } else {
                    log[..log.len().min(100)].to_string()
                };
                info!("YML log {}: length={}, preview={}", i, log.len(), preview);
            }
            Json(yml_logs).into_response()
        }
        Err(e) => {
            error!("Failed to get YML flow logs: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get flow logs").into_response()
        }
    }
}

/// Get ASCII tree directly from YML TestResult in database
pub async fn get_ascii_tree_direct(
    State(state): State<ApiState>,
    Path((benchmark_id, agent_type)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting ASCII tree for benchmark: {} by agent: {}",
        benchmark_id, agent_type
    );

    // Try to get YML TestResult from database first
    match state
        .db
        .get_yml_testresult(&benchmark_id, &agent_type)
        .await
    {
        Ok(Some(yml_content)) => {
            info!("Found YML TestResult for benchmark: {}", benchmark_id);

            // Parse YML to TestResult
            let test_result: reev_lib::results::TestResult =
                match serde_yaml::from_str(&yml_content) {
                    Ok(result) => result,
                    Err(e) => {
                        error!("Failed to parse YML to TestResult: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to parse YML: {e}"),
                        )
                            .into_response();
                    }
                };

            // Render as ASCII tree
            let ascii_tree = reev_runner::renderer::render_result_as_tree(&test_result);

            info!(
                "Successfully rendered ASCII tree for benchmark: {}",
                benchmark_id
            );
            return (StatusCode::OK, [("Content-Type", "text/plain")], ascii_tree).into_response();
        }
        Ok(None) => {
            info!("No YML TestResult found in database, falling back to execution state");
        }
        Err(e) => {
            error!("Failed to query YML TestResult from database: {}", e);
            info!("Falling back to execution state");
        }
    }

    // Fallback: Get ASCII tree from current execution state
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
                    "Returning ASCII tree trace from execution state ({} chars)",
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
        .upsert_benchmark(&benchmark_data.prompt, &payload.yml_content)
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
