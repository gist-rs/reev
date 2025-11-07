//! Recovery Metrics Handler
//!
//! This module provides the handler for retrieving recovery flow metrics.

use axum::{extract::State, response::IntoResponse, Json};
use chrono::Utc;
use serde_json::json;
use tracing::{info, instrument};

use crate::types::ApiState;

/// Get recovery metrics
#[instrument(skip_all)]
pub async fn get_recovery_metrics(State(_state): State<ApiState>) -> impl IntoResponse {
    info!("Getting recovery metrics");

    // Simple approach: return empty metrics for now
    // TODO: Implement real metrics collection when orchestrator metrics are thread-safe
    let metrics_json = json!({
        "total_flows": 0,
        "successful_flows": 0,
        "failed_flows": 0,
        "recovered_flows": 0,
        "total_recovery_time_ms": 0,
        "average_recovery_time_ms": 0,
        "success_rate": 0.0,
        "strategies_used": {
            "retry_attempts": 0,
            "alternative_flows_used": 0,
            "user_fulfillment_used": 0
        },
        "last_updated": Utc::now().to_rfc3339()
    });

    Json(metrics_json).into_response()
}
