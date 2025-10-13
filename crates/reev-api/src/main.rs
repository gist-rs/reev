use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use reev_runner::db::Db;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

/// API state containing database connection and execution state
#[derive(Clone)]
struct ApiState {
    db: Arc<Db>,
    executions: Arc<Mutex<HashMap<String, ExecutionState>>>,
    agent_configs: Arc<Mutex<HashMap<String, AgentConfig>>>,
}

/// Execution state for tracking benchmark runs
#[derive(Debug, Clone, Serialize)]
struct ExecutionState {
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

#[derive(Debug, Clone, Serialize)]
enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentConfig {
    agent_type: String,
    api_url: Option<String>,
    api_key: Option<String>,
}

/// Benchmark execution request
#[derive(Debug, Deserialize)]
struct BenchmarkExecutionRequest {
    agent: String,
    config: Option<AgentConfig>,
}

/// Execution response
#[derive(Debug, Serialize)]
struct ExecutionResponse {
    execution_id: String,
    status: String,
}

/// Health check response
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
    version: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "reev_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    let db_path =
        std::env::var("DATABASE_PATH").unwrap_or_else(|_| "db/reev_results.db".to_string());
    info!("Connecting to database at: {}", db_path);

    let db = Arc::new(Db::new(&db_path).await?);
    info!("Database connection established");

    // Create API state
    let state = ApiState {
        db,
        executions: Arc::new(Mutex::new(HashMap::new())),
        agent_configs: Arc::new(Mutex::new(HashMap::new())),
    };

    // Create router with state - simple approach for testing
    let app = Router::new()
        // Health check
        .route("/api/v1/health", get(health_check))
        // General routes
        .route("/api/v1/benchmarks", get(list_benchmarks))
        .route("/api/v1/agents", get(list_agents))
        .route("/api/v1/agent-performance", get(get_agent_performance))
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
        // Test endpoint without JSON
        .route("/api/v1/test", get(test_endpoint))
        // Test POST endpoint without JSON
        .route("/api/v1/test-post", post(test_post_endpoint))
        // Simple CORS layer
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string()) // Changed from 3000 to 3001 to avoid macOS Apple services conflict
        .parse()
        .unwrap_or(3001); // Changed from 3000 to 3001

    let addr = format!("0.0.0.0:{port}");
    info!("Starting API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("API server listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
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
async fn list_benchmarks() -> Json<Vec<String>> {
    let benchmarks = vec![
        "001-sol-transfer".to_string(),
        "002-spl-transfer".to_string(),
        "003-spl-transfer-fail".to_string(),
        "004-partial-score-spl-transfer".to_string(),
        "100-jup-swap-sol-usdc".to_string(),
        "110-jup-lend-deposit-sol".to_string(),
        "111-jup-lend-deposit-usdc".to_string(),
        "112-jup-lend-withdraw-sol".to_string(),
        "113-jup-lend-withdraw-usdc".to_string(),
        "114-jup-positions-and-earnings".to_string(),
        "115-jup-lend-mint-usdc".to_string(),
        "116-jup-lend-redeem-usdc".to_string(),
        "200-jup-swap-then-lend-deposit".to_string(),
    ];
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
async fn get_agent_performance(State(state): State<ApiState>) -> impl IntoResponse {
    info!("Getting agent performance summary");

    match state.db.get_agent_performance().await {
        Ok(summaries) => Json(summaries).into_response(),
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

/// Run a benchmark
/// Simple test endpoint
async fn test_endpoint() -> impl IntoResponse {
    Json(serde_json::json!({"status": "test working"}))
}

/// POST test endpoint without JSON
async fn test_post_endpoint() -> impl IntoResponse {
    Json(serde_json::json!({"status": "POST test working"}))
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
    db: &Db,
    benchmark_id: &str,
    agent: &str,
    score: f64,
) -> Result<()> {
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
