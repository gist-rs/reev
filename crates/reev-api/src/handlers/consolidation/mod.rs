//! Consolidation Session Handler Module
//!
//! This module provides handlers for retrieving and managing consolidated sessions
//! from the PingPong consolidation pipeline.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use reev_db::writer::DatabaseWriterTrait;
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument, warn};

use crate::types::ApiState;

/// Query parameters for consolidation status
#[derive(Debug, Deserialize)]
pub struct ConsolidationStatusQuery {
    /// Include detailed metadata in response
    pub include_metadata: Option<bool>,
}

/// Consolidation session response
#[derive(Debug, Serialize)]
pub struct ConsolidatedSessionResponse {
    /// Consolidated session ID
    pub session_id: String,
    /// Execution ID that produced this consolidation
    pub execution_id: String,
    /// Consolidated content (ping-pong format)
    pub content: String,
    /// Consolidation metadata
    pub metadata: ConsolidationMetadata,
    /// Timestamp when consolidation was created
    pub created_at: String,
    /// Whether this is a successful consolidation
    pub success: bool,
}

/// Consolidation metadata
#[derive(Debug, Serialize)]
pub struct ConsolidationMetadata {
    /// Average score across all sessions
    pub avg_score: Option<f64>,
    /// Total number of tools executed
    pub total_tools: Option<i64>,
    /// Success rate percentage
    pub success_rate: Option<f64>,
    /// Execution duration in milliseconds
    pub execution_duration_ms: Option<i64>,
    /// Number of original sessions consolidated
    pub session_count: usize,
}

/// Consolidation status response
#[derive(Debug, Serialize)]
pub struct ConsolidationStatusResponse {
    /// Execution ID
    pub execution_id: String,
    /// Whether consolidation is complete
    pub complete: bool,
    /// Consolidated session ID if available
    pub consolidated_session_id: Option<String>,
    /// Consolidation status (pending, in_progress, completed, failed, timeout)
    pub status: String,
    /// Status message
    pub message: String,
    /// Timestamp of last update
    pub updated_at: String,
    /// Consolidation metadata if available
    pub metadata: Option<ConsolidationMetadata>,
}

/// Error response for consolidation operations
#[derive(Debug, Serialize)]
pub struct ConsolidationError {
    pub error: String,
    pub execution_id: Option<String>,
    pub session_id: Option<String>,
    pub timestamp: String,
}

/// Get consolidated session by session ID
#[instrument(skip_all, fields(session_id = %session_id))]
pub async fn get_consolidated_session(
    State(state): State<ApiState>,
    Path(session_id): Path<String>,
) -> axum::response::Response {
    info!("Retrieving consolidated session: {}", session_id);

    match state.db.get_consolidated_session(&session_id).await {
        Ok(Some(content)) => {
            // Parse the consolidated content to extract metadata
            let consolidated_data: serde_json::Value = match serde_json::from_str(&content) {
                Ok(data) => data,
                Err(e) => {
                    warn!("Failed to parse consolidated content as JSON: {}", e);
                    // Try as YAML fallback
                    match serde_yaml::from_str::<serde_json::Value>(&content) {
                        Ok(data) => data,
                        Err(_) => {
                            error!("Consolidated content is neither valid JSON nor YAML");
                            return error_response(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Invalid consolidated content format",
                                Some(session_id.clone()),
                                None,
                            );
                        }
                    }
                }
            };

            // Extract execution ID from consolidated content
            let execution_id = consolidated_data
                .get("execution_id")
                .and_then(|v| v.as_str())
                .unwrap_or(&session_id)
                .to_string();

            // Create basic metadata from content
            let metadata = extract_consolidation_metadata(&consolidated_data);

            let response = ConsolidatedSessionResponse {
                session_id: session_id.clone(),
                execution_id,
                content,
                metadata,
                created_at: Utc::now().to_rfc3339(),
                success: true,
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => {
            warn!("Consolidated session not found: {}", session_id);
            return error_response(
                StatusCode::NOT_FOUND,
                "Consolidated session not found",
                Some(session_id),
                None,
            );
        }
        Err(e) => {
            error!(error = %e, "Failed to retrieve consolidated session: {}", session_id);
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve consolidated session",
                Some(session_id),
                None,
            );
        }
    }
}

/// Get consolidated session for execution
#[instrument(skip_all, fields(execution_id = %execution_id))]
pub async fn get_execution_consolidated_session(
    State(state): State<ApiState>,
    Path(execution_id): Path<String>,
) -> axum::response::Response {
    info!(
        "Retrieving consolidated session for execution: {}",
        execution_id
    );

    // Query consolidated sessions by execution_id
    match state
        .db
        .get_consolidated_sessions_by_execution(&execution_id)
        .await
    {
        Ok(sessions) => {
            if sessions.is_empty() {
                warn!(
                    "No consolidated sessions found for execution: {}",
                    execution_id
                );
                return error_response(
                    StatusCode::NOT_FOUND,
                    "No consolidated sessions found for this execution",
                    None,
                    Some(execution_id),
                );
            } else {
                // Return the most recent consolidated session
                let session = &sessions[0]; // Assuming ordered by creation time desc

                let metadata = ConsolidationMetadata {
                    avg_score: session.avg_score,
                    total_tools: session.total_tools,
                    success_rate: session.success_rate,
                    execution_duration_ms: session.execution_duration_ms,
                    session_count: session.original_session_ids.split(',').count(),
                };

                let response = ConsolidatedSessionResponse {
                    session_id: session.consolidated_session_id.clone(),
                    execution_id: execution_id.clone(),
                    content: session.consolidated_content.clone(),
                    metadata,
                    created_at: Utc::now().to_rfc3339(),
                    success: true,
                };

                (StatusCode::OK, Json(response)).into_response()
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to query consolidated sessions for execution: {}", execution_id);
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to check consolidation status",
                None,
                Some(execution_id),
            );
        }
    }
}

/// Get consolidation status for execution
#[instrument(skip_all, fields(execution_id = %execution_id))]
pub async fn get_consolidation_status(
    State(state): State<ApiState>,
    Path(execution_id): Path<String>,
    Query(query): Query<ConsolidationStatusQuery>,
) -> axum::response::Response {
    info!(
        "Checking consolidation status for execution: {}",
        execution_id
    );

    // Check for existing consolidated session
    match state
        .db
        .get_consolidated_sessions_by_execution(&execution_id)
        .await
    {
        Ok(sessions) => {
            if sessions.is_empty() {
                // No consolidation found, check if there are step sessions
                match state.db.get_sessions_for_consolidation(&execution_id).await {
                    Ok(step_sessions) => {
                        if step_sessions.is_empty() {
                            // No step sessions either
                            let response = ConsolidationStatusResponse {
                                execution_id: execution_id.clone(),
                                complete: false,
                                consolidated_session_id: None,
                                status: "pending".to_string(),
                                message: "No step sessions found for consolidation".to_string(),
                                updated_at: Utc::now().to_rfc3339(),
                                metadata: None,
                            };
                            return (StatusCode::OK, Json(response)).into_response();
                        } else {
                            // Step sessions exist but no consolidation yet
                            let response = ConsolidationStatusResponse {
                                execution_id: execution_id.clone(),
                                complete: false,
                                consolidated_session_id: None,
                                status: "in_progress".to_string(),
                                message: "Consolidation in progress (60s timeout)".to_string(),
                                updated_at: Utc::now().to_rfc3339(),
                                metadata: None,
                            };
                            return (StatusCode::OK, Json(response)).into_response();
                        }
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to check step sessions for execution: {}", execution_id);
                        return error_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to check consolidation status",
                            None,
                            Some(execution_id),
                        );
                    }
                }
            } else {
                // Consolidation exists
                let session = &sessions[0];

                let metadata = if query.include_metadata.unwrap_or(false) {
                    Some(ConsolidationMetadata {
                        avg_score: session.avg_score,
                        total_tools: session.total_tools,
                        success_rate: session.success_rate,
                        execution_duration_ms: session.execution_duration_ms,
                        session_count: session.original_session_ids.split(',').count(),
                    })
                } else {
                    None
                };

                let response = ConsolidationStatusResponse {
                    execution_id: execution_id.clone(),
                    complete: true,
                    consolidated_session_id: Some(session.consolidated_session_id.clone()),
                    status: "completed".to_string(),
                    message: "Consolidation completed successfully".to_string(),
                    updated_at: Utc::now().to_rfc3339(),
                    metadata,
                };

                (StatusCode::OK, Json(response)).into_response()
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to check consolidation status for execution: {}", execution_id);
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to check consolidation status",
                None,
                Some(execution_id),
            );
        }
    }
}

/// Extract consolidation metadata from consolidated content
fn extract_consolidation_metadata(content: &serde_json::Value) -> ConsolidationMetadata {
    let avg_score = content
        .get("avg_score")
        .or_else(|| content.get("metadata").and_then(|m| m.get("avg_score")))
        .and_then(|v| v.as_f64());

    let total_tools = content
        .get("total_tools")
        .or_else(|| content.get("metadata").and_then(|m| m.get("total_tools")))
        .and_then(|v| v.as_i64());

    let success_rate = content
        .get("success_rate")
        .or_else(|| content.get("metadata").and_then(|m| m.get("success_rate")))
        .and_then(|v| v.as_f64());

    let execution_duration_ms = content
        .get("execution_duration_ms")
        .or_else(|| {
            content
                .get("metadata")
                .and_then(|m| m.get("execution_duration_ms"))
        })
        .and_then(|v| v.as_i64());

    let session_count = content
        .get("steps")
        .and_then(|steps| steps.as_array())
        .map(|arr| arr.len())
        .unwrap_or(0);

    ConsolidationMetadata {
        avg_score,
        total_tools,
        success_rate,
        execution_duration_ms,
        session_count,
    }
}

/// Create error response for consolidation operations
fn error_response(
    status: StatusCode,
    message: &str,
    session_id: Option<String>,
    execution_id: Option<String>,
) -> axum::response::Response {
    let error = ConsolidationError {
        error: message.to_string(),
        session_id,
        execution_id,
        timestamp: Utc::now().to_rfc3339(),
    };

    (status, Json(error)).into_response()
}
