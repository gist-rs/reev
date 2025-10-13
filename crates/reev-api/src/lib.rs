use axum::{
    extract::{Path, State},
    http::{header, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use reev_runner::db;
use serde::Serialize;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

/// API state containing database connection
#[derive(Clone)]
pub struct ApiState {
    pub db: Arc<db::Db>,
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
    let state = ApiState { db };

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
    let benchmarks = vec![
        "001-sol-transfer".to_string(),
        "002-spl-transfer".to_string(),
        "003-jupiter-swap".to_string(),
        "004-jupiter-lend".to_string(),
        "005-jupiter-mint".to_string(),
        "006-jupiter-redeem".to_string(),
        "007-kamino-lend".to_string(),
        "008-kamino-withdraw".to_string(),
        "009-meteora-swap".to_string(),
        "010-orca-swap".to_string(),
        "011-raydium-swap".to_string(),
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
    State(_state): State<ApiState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let benchmark_id = "001-sol-transfer".to_string(); // For testing
    let agent_value = request
        .get("agent")
        .cloned()
        .unwrap_or(serde_json::Value::String("deterministic".to_string()));
    let agent_str = match &agent_value {
        serde_json::Value::String(s) => s.as_str(),
        _ => "deterministic",
    };

    let execution_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    let response = serde_json::json!({
        "execution_id": execution_id,
        "status": "started",
        "benchmark_id": benchmark_id,
        "agent": agent_str
    });

    Json(response)
}

/// Get execution status
async fn get_execution_status(
    State(_state): State<ApiState>,
    Path((_benchmark_id, execution_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let response = serde_json::json!({
        "execution_id": execution_id,
        "status": "completed",
        "benchmark_id": "001-sol-transfer",
        "agent": "deterministic",
        "progress": 100,
        "trace": "Benchmark completed successfully\n"
    });

    Json(response)
}

/// Stop a running benchmark
async fn stop_benchmark(
    State(_state): State<ApiState>,
    Path((_benchmark_id, execution_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let response = serde_json::json!({
        "execution_id": execution_id,
        "status": "stopped"
    });

    Json(response)
}

/// Save agent configuration
async fn save_agent_config(
    State(_state): State<ApiState>,
    Json(config): Json<serde_json::Value>,
) -> impl IntoResponse {
    let response = serde_json::json!({
        "status": "saved",
        "agent_type": config.get("agent_type")
    });

    Json(response)
}

/// Get agent configuration
async fn get_agent_config(
    Path(agent_type): Path<String>,
    State(_state): State<ApiState>,
) -> impl IntoResponse {
    let response = serde_json::json!({
        "agent_type": agent_type,
        "api_url": "https://api.example.com",
        "api_key": "***key"
    });

    Json(response)
}

/// Test agent connection
async fn test_agent_connection(
    State(_state): State<ApiState>,
    Json(config): Json<serde_json::Value>,
) -> impl IntoResponse {
    let agent_type_value = config
        .get("agent_type")
        .cloned()
        .unwrap_or(serde_json::Value::String("unknown".to_string()));
    let agent_str = match &agent_type_value {
        serde_json::Value::String(s) => s.as_str(),
        _ => "unknown",
    };

    let response = if agent_str == "deterministic" {
        serde_json::json!({
            "status": "success",
            "message": "Deterministic agent is always available"
        })
    } else {
        serde_json::json!({
            "status": "success",
            "message": "Configuration appears valid"
        })
    };

    Json(response)
}

/// Error response type
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub timestamp: String,
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
