//! Benchmark management handlers

use crate::services::*;
use crate::types::{
    ApiState, BenchmarkExecutionRequest, ExecutionResponse, ExecutionState, ExecutionStatus,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono;
use tracing::{error, info};
use uuid::Uuid;

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
    // First check in-memory executions (for running/recent executions)
    {
        let executions = state.executions.lock().await;
        if let Some(execution) = executions.get(&execution_id) {
            return Json(execution.clone()).into_response();
        }
    }

    // If not in memory, try to load from database
    // Look for session logs with execution_id as session_id
    match state.db.get_session_log(&execution_id).await {
        Ok(Some(log_content)) => {
            // Parse the flow log and format it nicely
            match serde_json::from_str::<reev_flow::FlowLog>(&log_content) {
                Ok(flow_log) => {
                    // Format the flow log as a readable trace
                    let mut formatted_trace = String::new();

                    // Add prompt from first event if available
                    if let Some(event) = flow_log.events.first() {
                        if let Some(prompt) = &event.content.data.get("prompt") {
                            formatted_trace.push_str(&format!("📝 Prompt: {prompt}\n\n"));
                        }
                    }

                    // Add steps - format all events
                    for (i, event) in flow_log.events.iter().enumerate() {
                        if let reev_flow::FlowEventType::BenchmarkStateChange = event.event_type {
                            formatted_trace.push_str(&format!("✓ Step {}\n", i + 1));
                            if let Some(action) = &event.content.data.get("action") {
                                formatted_trace
                                    .push_str(&format!("   ├─ ACTION: {action}\n"));
                            }
                            if let Some(observation) = &event.content.data.get("observation") {
                                formatted_trace
                                    .push_str(&format!("   └─ OBSERVATION: {observation}\n"));
                            }
                            formatted_trace.push('\n');
                        }
                    }

                    // Add final result
                    if let Some(result) = &flow_log.final_result {
                        formatted_trace.push_str(&format!(
                            "✅ Execution completed - Score: {:.1}%\n",
                            result.score * 100.0
                        ));
                    }

                    // Extract benchmark_id from the flow log
                    let benchmark_id = flow_log.benchmark_id.clone();

                    let execution_state = ExecutionState {
                        id: execution_id,
                        benchmark_id,
                        agent: flow_log.agent_type,
                        status: if flow_log
                            .final_result
                            .as_ref()
                            .map(|r| r.success)
                            .unwrap_or(false)
                        {
                            ExecutionStatus::Completed
                        } else {
                            ExecutionStatus::Failed
                        },
                        progress: 100,
                        start_time: chrono::DateTime::from_timestamp(
                            flow_log
                                .start_time
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs() as i64,
                            0,
                        )
                        .unwrap_or_else(chrono::Utc::now),
                        end_time: flow_log.end_time.map(|et| {
                            chrono::DateTime::from_timestamp(
                                et.duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs() as i64,
                                0,
                            )
                            .unwrap_or_else(chrono::Utc::now)
                        }),
                        trace: formatted_trace,
                        logs: String::new(),
                        error: None,
                    };

                    Json(execution_state).into_response()
                }
                Err(e) => {
                    error!("Failed to parse flow log JSON: {}", e);
                    // Return raw content as fallback
                    let execution_state = ExecutionState {
                        id: execution_id,
                        benchmark_id: "unknown".to_string(),
                        agent: "deterministic".to_string(),
                        status: ExecutionStatus::Completed,
                        progress: 100,
                        start_time: chrono::Utc::now(),
                        end_time: Some(chrono::Utc::now()),
                        trace: format!("Error parsing flow log: {e}\nRaw data:\n{log_content}"),
                        logs: String::new(),
                        error: Some(format!("Failed to parse flow log: {e}")),
                    };
                    Json(execution_state).into_response()
                }
            }
        }
        Ok(None) => {
            // Try to find by looking for agent_performance records
            match state
                .db
                .get_agent_performance(&reev_db::QueryFilter::new())
                .await
            {
                Ok(performances) => {
                    for perf in performances {
                        if perf.session_id == execution_id {
                            let execution_state = ExecutionState {
                                id: execution_id,
                                benchmark_id: perf.benchmark_id,
                                agent: perf.agent_type,
                                status: ExecutionStatus::Completed,
                                progress: 100,
                                start_time: chrono::Utc::now(),
                                end_time: Some(chrono::Utc::now()),
                                trace: format!(
                                    "Benchmark completed with score: {:.1}%",
                                    perf.score * 100.0
                                ),
                                logs: String::new(),
                                error: None,
                            };
                            return Json(execution_state).into_response();
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to query agent performance: {}", e);
                }
            }

            (StatusCode::NOT_FOUND, "Execution not found in database").into_response()
        }
        Err(e) => {
            error!("Failed to get session log from database: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {e}"),
            )
                .into_response()
        }
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
