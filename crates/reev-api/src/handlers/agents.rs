//! Agent management handlers
use crate::types::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use tracing::{error, info};

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

    let filter = reev_db::QueryFilter::new();
    match state.db.get_agent_performance(&filter).await {
        Ok(performances) => {
            // Debug logging for specific benchmark
            for performance in &performances {
                if performance.agent_type == "deterministic"
                    && performance.benchmark_id == "116-jup-lend-redeem-usdc"
                {
                    info!("🔍 [API_DEBUG] Latest 116-jup-lend-redeem-usdc result: score={}, status={}, timestamp={}",
                          performance.score, performance.final_status, performance.timestamp);
                }
            }
            Json::<Vec<reev_db::types::AgentPerformance>>(performances).into_response()
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
