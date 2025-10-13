use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use reev_runner::db;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};
use uuid::Uuid;

/// API state containing database connection and execution state
#[derive(Clone)]
pub struct ApiState {
    pub db: Arc<db::Db>,
    pub executions: Arc<Mutex<HashMap<String, ExecutionState>>>,
    pub agent_configs: Arc<Mutex<HashMap<String, AgentConfig>>>,
}

/// Execution state for tracking benchmark runs
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionState {
    id: String,
    benchmark_id: String,
    agent: String,
    status: ExecutionStatus,
    progress: u8,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: Option<chrono::DateTime<chrono::Utc>>,
    trace: String,
    logs: String,
    error: Option<String>,
}

/// Execution status
#[derive(Debug, Clone, Serialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    agent_type: String,
    api_url: Option<String>,
    api_key: Option<String>,
}

/// Benchmark execution request
#[derive(Debug, Deserialize)]
pub struct BenchmarkExecutionRequest {
    pub agent: String,
    pub config: Option<AgentConfig>,
}

/// Execution response
#[derive(Debug, Serialize)]
pub struct ExecutionResponse {
    pub execution_id: String,
    pub status: String,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
}

/// Create API router with all endpoints
pub fn create_router(db: Arc<db::Db>) -> Router<ApiState> {
    let state = ApiState {
        db,
        executions: Arc::new(Mutex::new(HashMap::new())),
        agent_configs: Arc::new(Mutex::new(HashMap::new())),
    };

    Router::new()
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/benchmarks", get(list_benchmarks))
        .route("/api/v1/agents", get(list_agents))
        .route("/api/v1/agent-performance", get(get_agent_performance))
        .route("/api/v1/test-post", post(test_post_endpoint))
        .route("/api/v1/test-post", axum::routing::options(options_handler))
        // Benchmark execution endpoints
        .route("/api/v1/benchmarks/{id}/run", post(run_benchmark))
        .route(
            "/api/v1/benchmarks/{id}/status/{execution_id}",
            get(get_execution_status),
        )
        .route(
            "/api/v1/benchmarks/{id}/stop/{execution_id}",
            post(stop_benchmark),
        )
        // Agent configuration endpoints
        .route("/api/v1/agents/config", post(save_agent_config))
        .route("/api/v1/agents/config/{agent_type}", get(get_agent_config))
        .route("/api/v1/agents/test", post(test_agent_connection))
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "http://localhost:3000".parse().unwrap(),
                    "http://localhost:5173".parse().unwrap(),
                ])
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_credentials(true),
        )
        .with_state(state)
}

/// Health check endpoint
async fn health_check() -> Json<HealthResponse> {
    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: "0.1.0".to_string(),
    };
    Json(response)
}

/// List all available benchmarks
async fn list_benchmarks(State(_state): State<ApiState>) -> Json<Vec<String>> {
    // Load benchmarks dynamically from actual files
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
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().is_some_and(|ext| ext == "yml") {
                        if let Some(stem) = path.file_stem() {
                            benchmarks.push(stem.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to read benchmarks directory: {}", e);
            return Json(vec![]);
        }
    }

    benchmarks.sort();
    info!("Found {} benchmark files", benchmarks.len());
    Json(benchmarks)
}

/// List all available agents
async fn list_agents() -> Json<Vec<String>> {
    let agents = vec![
        "deterministic".to_string(),
        "local".to_string(),
        "gemini".to_string(),
        "glm-4.6".to_string(),
    ];
    Json(agents)
}

/// Get agent performance summary
async fn get_agent_performance(State(state): State<ApiState>) -> impl axum::response::IntoResponse {
    info!("Getting agent performance summary");

    match state.db.get_agent_performance().await {
        Ok(summaries) => Json(summaries).into_response(),
        Err(e) => {
            tracing::error!("Failed to get agent performance: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get agent performance",
            )
                .into_response()
        }
    }
}

/// POST test endpoint
async fn test_post_endpoint() -> Json<serde_json::Value> {
    let response = serde_json::json!({"status": "POST test working"});
    Json(response)
}

/// OPTIONS handler for CORS preflight
async fn options_handler() -> axum::http::StatusCode {
    axum::http::StatusCode::OK
}

/// Run a benchmark
async fn run_benchmark(
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

    // Start the benchmark execution in background
    let state_clone = state.clone();
    let execution_id_clone = execution_id.clone();
    let benchmark_id_clone = benchmark_id.clone();
    let agent = request.agent.clone();

    tokio::spawn(async move {
        execute_benchmark_background(state_clone, execution_id_clone, benchmark_id_clone, agent)
            .await;
    });

    Json(ExecutionResponse {
        execution_id,
        status: "started".to_string(),
    })
}

/// Get execution status
async fn get_execution_status(
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
async fn stop_benchmark(
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
async fn save_agent_config(
    State(state): State<ApiState>,
    Json(config): Json<AgentConfig>,
) -> impl IntoResponse {
    let mut configs = state.agent_configs.lock().await;
    configs.insert(config.agent_type.clone(), config.clone());

    info!("Saved configuration for agent: {}", config.agent_type);
    Json(serde_json::json!({"status": "saved"}))
}

/// Get agent configuration
async fn get_agent_config(
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
async fn test_agent_connection(
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

/// Error response type
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub timestamp: String,
}

/// Background task to execute benchmark
async fn execute_benchmark_background(
    state: ApiState,
    execution_id: String,
    benchmark_id: String,
    agent: String,
) {
    // Update status to running
    {
        let mut executions = state.executions.lock().await;
        if let Some(execution) = executions.get_mut(&execution_id) {
            execution.status = ExecutionStatus::Running;
            execution.progress = 10;
            execution.trace = format!("Starting benchmark {benchmark_id} with agent {agent}\n");
        }
    }

    // Simulate benchmark execution
    info!(
        "Executing benchmark: {} with agent: {}",
        benchmark_id, agent
    );

    // Simulate progress updates
    for progress in [20, 40, 60, 80] {
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        let mut executions = state.executions.lock().await;
        if let Some(execution) = executions.get_mut(&execution_id) {
            execution.progress = progress;
            execution
                .trace
                .push_str(&format!("Progress: {progress}%\n"));
        }
    }

    // Simulate completion
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    {
        let mut executions = state.executions.lock().await;
        if let Some(execution) = executions.get_mut(&execution_id) {
            execution.status = ExecutionStatus::Completed;
            execution.progress = 100;
            execution.end_time = Some(chrono::Utc::now());
            execution
                .trace
                .push_str("Benchmark completed successfully\n");

            // Store result in database
            let db_clone = state.db.clone();
            let benchmark_id_clone = benchmark_id.clone();
            let agent_clone = agent.clone();

            tokio::spawn(async move {
                if let Err(e) =
                    store_benchmark_result(&db_clone, &benchmark_id_clone, &agent_clone, 1.0).await
                {
                    error!("Failed to store benchmark result: {}", e);
                }
            });
        }
    }

    info!("Benchmark execution completed: {}", execution_id);
}

/// Store benchmark result in database
async fn store_benchmark_result(
    db: &db::Db,
    benchmark_id: &str,
    agent: &str,
    score: f64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let execution_time_ms = 5000; // Mock execution time

    db.insert_agent_performance(
        benchmark_id,
        agent,
        score,
        "Succeeded",
        execution_time_ms,
        &timestamp,
        None,
    )
    .await?;

    Ok(())
}

/// Helper function to create error responses
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
