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

    match state.db.get_agent_performance_summary().await {
        Ok(summaries) => {
            info!(
                "✅ Successfully got {} agent performance summaries",
                summaries.len()
            );
            // Debug logging for specific benchmark
            for summary in &summaries {
                info!(
                    "🔍 [API_DEBUG] {} agent: {} benchmarks, avg_score={:.2}, success_rate={:.2}",
                    summary.agent_type,
                    summary.total_benchmarks,
                    summary.average_score,
                    summary.success_rate
                );
            }
            Json::<Vec<reev_db::types::AgentPerformanceSummary>>(summaries).into_response()
        }
        Err(e) => {
            error!("❌ Failed to get agent performance summary: {}", e);
            error!("❌ Error details: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json::<serde_json::Value>(serde_json::json!({
                    "error": "Failed to get agent performance summary",
                    "details": e.to_string()
                })),
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

/// Debug endpoint to check raw agent performance data
pub async fn debug_agent_performance_raw(State(state): State<ApiState>) -> impl IntoResponse {
    info!("DEBUG: Getting raw agent performance data");

    // Use the original flat query to see what's in the database
    let filter = reev_db::QueryFilter::new();
    match state.db.get_agent_performance(&filter).await {
        Ok(performances) => {
            info!("DEBUG: Got {} raw performance records", performances.len());
            return Json::<serde_json::Value>(serde_json::json!({
                "count": performances.len(),
                "data": performances
            }))
            .into_response();
        }
        Err(e) => {
            error!("DEBUG: Failed to get raw agent performance: {}", e);
            return Json::<serde_json::Value>(serde_json::json!({
                "error": "Failed to get raw agent performance",
                "details": e.to_string()
            }))
            .into_response();
        }
    }
}

/// Debug endpoint to manually insert test performance data
pub async fn debug_insert_test_data(State(state): State<ApiState>) -> impl IntoResponse {
    info!("DEBUG: Manually inserting test performance data");

    // Create test performance data
    let test_performance = reev_db::types::AgentPerformance {
        id: None,
        session_id: "test-session-001".to_string(),
        benchmark_id: "001-sol-transfer".to_string(),
        agent_type: "deterministic".to_string(),
        score: 1.0,
        final_status: "completed".to_string(),
        execution_time_ms: Some(1000),
        timestamp: chrono::Utc::now().to_rfc3339(),
        flow_log_id: None,
        prompt_md5: Some("".to_string()),
        additional_metrics: std::collections::HashMap::new(),
    };

    match state.db.insert_agent_performance(&test_performance).await {
        Ok(_) => {
            info!("DEBUG: Successfully inserted test performance data");
            Json::<serde_json::Value>(serde_json::json!({
                "status": "success",
                "message": "Test performance data inserted successfully"
            }))
            .into_response()
        }
        Err(e) => {
            error!("DEBUG: Failed to insert test performance data: {}", e);
            Json::<serde_json::Value>(serde_json::json!({
                "status": "error",
                "message": "Failed to insert test performance data",
                "details": e.to_string()
            }))
            .into_response()
        }
    }
}

/// Debug endpoint to check execution_sessions table
pub async fn debug_execution_sessions(State(state): State<ApiState>) -> impl IntoResponse {
    info!("DEBUG: Getting execution sessions data");

    // Query execution sessions directly to see what's in the database
    let filter = reev_db::types::SessionFilter {
        benchmark_id: None,
        agent_type: None,
        interface: None,
        status: None,
        limit: Some(10),
    };

    match state.db.list_sessions(&filter).await {
        Ok(sessions) => {
            info!("DEBUG: Got {} execution sessions", sessions.len());
            Json::<serde_json::Value>(serde_json::json!({
                "count": sessions.len(),
                "data": sessions
            }))
            .into_response()
        }
        Err(e) => {
            error!("DEBUG: Failed to get execution sessions: {}", e);
            Json::<serde_json::Value>(serde_json::json!({
                "error": "Failed to get execution sessions",
                "details": e.to_string()
            }))
            .into_response()
        }
    }
}
