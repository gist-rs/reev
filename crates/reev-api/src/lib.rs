use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use reev_runner::db;
use serde::Serialize;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
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
        .layer(CorsLayer::permissive())
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
