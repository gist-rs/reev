//! Benchmark management handlers

use crate::types::{ApiState, BenchmarkExecutionRequest};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono;
use reev_types::{ExecutionRequest, ExecutionResponse, ExecutionState, ExecutionStatus};
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;

/// List all available benchmarks
pub async fn list_benchmarks(
    State(state): State<ApiState>,
) -> Json<Vec<crate::types::BenchmarkInfo>> {
    // Query benchmarks directly from database instead of CLI
    match state.db.get_all_benchmarks().await {
        Ok(benchmark_data) => {
            let mut benchmarks = Vec::new();

            for data in benchmark_data {
                // Filter out position/earnings benchmarks (114-*) from web interface
                // These benchmarks use jupiter_earn tool which is restricted to specialized testing
                if data.benchmark_name.starts_with("114") {
                    continue;
                }

                // Parse YAML content to extract description and tags
                let (description, tags) =
                    match serde_yaml::from_str::<serde_yaml::Value>(&data.content) {
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

                            (description, tags)
                        }
                        Err(e) => {
                            error!(
                                "Failed to parse YAML content for {}: {}",
                                data.benchmark_name, e
                            );
                            ("Failed to parse description".to_string(), vec![])
                        }
                    };

                benchmarks.push(crate::types::BenchmarkInfo {
                    id: data.benchmark_name,
                    description,
                    tags,
                    prompt: data.prompt,
                });
            }

            benchmarks.sort_by(|a, b| a.id.cmp(&b.id));
            Json(benchmarks)
        }
        Err(e) => {
            error!("Failed to list benchmarks from database: {}", e);
            // Fallback to filesystem discovery
            fallback_filesystem_discovery()
        }
    }
}

/// Fallback method to discover benchmarks from filesystem
fn fallback_filesystem_discovery() -> Json<Vec<crate::types::BenchmarkInfo>> {
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

                        // Filter out position/earnings benchmarks (114-*) from web interface
                        if benchmark_id.starts_with("114") {
                            continue;
                        }

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

/// Run a benchmark
pub async fn run_benchmark(
    State(state): State<ApiState>,
    Path(benchmark_id): Path<String>,
    Json(request): Json<BenchmarkExecutionRequest>,
) -> impl IntoResponse {
    let execution_id = Uuid::new_v4().to_string();
    let _now = chrono::Utc::now();

    let execution_state = ExecutionState::new(
        execution_id.clone(),
        benchmark_id.clone(),
        request.agent.clone(),
    );

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
        "🔥  Starting benchmark execution: {} for agent: {}",
        benchmark_id, request.agent
    );

    // Start benchmark execution using the new BenchmarkExecutor
    let execution_request = ExecutionRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        execution_id: Some(execution_id.clone()),
        benchmark_path: format!("benchmarks/{benchmark_id}.yml"),
        agent: request.agent.clone(),
        priority: 0,
        timeout_seconds: 600, // 10 minutes default
        shared_surfpool: false,
        metadata: std::collections::HashMap::new(),
    };

    // Start benchmark execution using new BenchmarkExecutor
    let executor = state.benchmark_executor.clone();
    tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            match executor.execute_benchmark(execution_request).await {
                Ok(exec_id) => {
                    info!("Benchmark execution started: {}", exec_id);
                }
                Err(e) => {
                    error!("Benchmark execution failed: {}", e);
                }
            }
        })
    });

    Json(ExecutionResponse {
        execution_id,
        status: ExecutionStatus::Running,
        duration_ms: 0,
        result: None,
        error: None,
        logs: Vec::new(),
        tool_calls: Vec::new(),
    })
}

/// Get execution status
pub async fn get_execution_status(
    State(state): State<ApiState>,
    Path((benchmark_id, execution_id)): Path<(String, Option<String>)>,
) -> impl IntoResponse {
    get_execution_status_with_agent(State(state), Path((benchmark_id, execution_id)), None).await
}

pub async fn get_execution_status_with_agent(
    State(state): State<ApiState>,
    Path((benchmark_id, execution_id)): Path<(String, Option<String>)>,
    agent_type: Option<String>,
) -> impl IntoResponse {
    // If execution_id is provided, check in-memory executions first
    if let Some(ref exec_id) = execution_id {
        let executions = state.executions.lock().await;
        if let Some(execution) = executions.get(exec_id) {
            return Json(execution.clone()).into_response();
        }
        drop(executions);

        // If not in memory, try to load from database with the specific execution_id
        match state.db.get_session_log(exec_id).await {
            Ok(Some(log_content)) => {
                // Parse and format the execution trace
                match format_execution_trace(&log_content, exec_id.clone()) {
                    Ok(execution_state) => Json(execution_state).into_response(),
                    Err(e) => {
                        error!("Failed to format execution trace for {}: {}", exec_id, e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to format execution trace: {e}"),
                        )
                            .into_response()
                    }
                }
            }
            Ok(None) => (StatusCode::NOT_FOUND, "Execution not found in database").into_response(),
            Err(e) => {
                error!("Failed to get session log from database: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {e}"),
                )
                    .into_response()
            }
        }
    } else {
        // No execution_id provided - get the most recent execution for this benchmark
        let filter = reev_db::types::SessionFilter {
            benchmark_id: Some(benchmark_id.clone()),
            agent_type, // Use provided agent type or None for any agent
            interface: None,
            status: None,
            limit: Some(1), // Get the most recent
        };

        match state.db.list_sessions(&filter).await {
            Ok(sessions) => {
                if let Some(session) = sessions.first() {
                    match state.db.get_session_log(&session.session_id).await {
                        Ok(Some(log_content)) => {
                            match format_execution_trace(&log_content, session.session_id.clone()) {
                                Ok(execution_state) => Json(execution_state).into_response(),
                                Err(e) => {
                                    error!("Failed to format execution trace: {}", e);
                                    (
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        format!("Failed to format execution trace: {e}"),
                                    )
                                        .into_response()
                                }
                            }
                        }
                        Ok(None) => {
                            (StatusCode::NOT_FOUND, "No execution trace found").into_response()
                        }
                        Err(e) => {
                            error!("Failed to get session log: {}", e);
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Database error: {e}"),
                            )
                                .into_response()
                        }
                    }
                } else {
                    (StatusCode::NOT_FOUND, "No executions found for benchmark").into_response()
                }
            }
            Err(e) => {
                error!("Failed to list sessions: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {e}"),
                )
                    .into_response()
            }
        }
    }
}

/// Helper function to format execution trace from log content
fn format_execution_trace(
    log_content: &str,
    execution_id: String,
) -> Result<ExecutionState, Box<dyn std::error::Error + Send + Sync>> {
    // Parse the flow log and format it as readable trace
    match serde_json::from_str::<serde_json::Value>(log_content) {
        Ok(parsed) => {
            let mut formatted_trace = String::new();

            if let Some(prompt) = parsed.get("prompt").and_then(|v| v.as_str()) {
                formatted_trace.push_str(&format!("📝 Prompt: {prompt}\n\n"));
            }

            if let Some(steps) = parsed.get("steps").and_then(|v| v.as_array()) {
                for (i, step) in steps.iter().enumerate() {
                    formatted_trace.push_str(&format!("✓ Step {}\n", i + 1));

                    if let Some(action) = step.get("action") {
                        formatted_trace.push_str("   ├─ ACTION:\n");
                        if let Some(action_array) = action.as_array() {
                            for action_item in action_array {
                                if let Some(program_id) = action_item.get("program_id") {
                                    formatted_trace
                                        .push_str(&format!("      Program ID: {program_id}\n"));
                                }
                                if let Some(accounts) = action_item.get("accounts") {
                                    if let Some(accounts_array) = accounts.as_array() {
                                        formatted_trace.push_str("      Accounts:\n");
                                        for (idx, account) in accounts_array.iter().enumerate() {
                                            if let Some(pubkey) = account.get("pubkey") {
                                                let is_signer = account
                                                    .get("is_signer")
                                                    .and_then(|v| v.as_bool())
                                                    .unwrap_or(false);
                                                let is_writable = account
                                                    .get("is_writable")
                                                    .and_then(|v| v.as_bool())
                                                    .unwrap_or(false);
                                                let icon =
                                                    if is_signer { "🖋️" } else { "🖍️" };
                                                let arrow = if is_writable { "➕" } else { "➖" };
                                                formatted_trace.push_str(&format!(
                                                    "      [{idx}] {icon} {arrow} {pubkey}\n"
                                                ));
                                            }
                                        }
                                    }
                                }
                                if let Some(data) = action_item.get("data") {
                                    formatted_trace
                                        .push_str(&format!("      Data (Base58): {data}\n"));
                                }
                            }
                        }
                    }

                    if let Some(observation) = step.get("observation") {
                        formatted_trace.push_str("   └─ OBSERVATION: ");
                        if let Some(status) = observation.get("last_transaction_status") {
                            formatted_trace.push_str(&format!("{status}\n"));
                        }
                        if let Some(error) = observation.get("last_transaction_error") {
                            if !error.as_str().unwrap_or("").is_empty() {
                                formatted_trace.push_str(&format!("   Error: {error}\n"));
                            }
                        }
                    }
                    formatted_trace.push('\n');
                }
            }

            // Add final success message
            formatted_trace.push_str("✅ Execution completed - Full trace displayed above\n");

            // Extract benchmark_id from the trace if available
            let benchmark_id = parsed
                .get("benchmark_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let mut execution_state = ExecutionState::new(
                execution_id,
                benchmark_id,
                "deterministic".to_string(), // Default agent
            );
            execution_state.status = ExecutionStatus::Completed;
            execution_state.progress = Some(1.0);
            execution_state.add_metadata("trace", serde_json::Value::String(formatted_trace));
            Ok(execution_state)
        }
        Err(e) => Err(format!("Failed to parse execution trace: {e}").into()),
    }
}

/// Get execution status without execution_id (gets most recent for benchmark)
pub async fn get_execution_status_no_id(
    State(state): State<ApiState>,
    Path(benchmark_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let agent_type = params.get("agent").cloned();
    get_execution_status_with_agent(State(state), Path((benchmark_id, None)), agent_type).await
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
                ExecutionStatus::Running | ExecutionStatus::Queued
            ) {
                execution.status = ExecutionStatus::Failed;
                execution.updated_at = chrono::Utc::now();
                execution.error_message = Some("Execution stopped by user".to_string());
                info!("Stopped benchmark execution: {}", execution_id);
                Json(serde_json::json!({"status": "stopped"})).into_response()
            } else {
                (StatusCode::BAD_REQUEST, "Execution is not running").into_response()
            }
        }
        None => (StatusCode::NOT_FOUND, "Execution not found").into_response(),
    }
}
