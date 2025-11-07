//! Flow diagram handler for generating Mermaid diagrams from session logs
//!
//! This module provides endpoints for generating flow visualizations from
//! agent execution sessions, supporting both individual sessions and consolidated
//! ping-pong sessions with metadata.

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json},
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use tracing::{error, info, warn};

use crate::handlers::flow_diagram::{FlowDiagramError, SessionParser, StateDiagramGenerator};
use crate::types::ApiState;
use reev_db::writer::DatabaseWriterTrait;

/// Query parameters for flow generation
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields reserved for future API enhancements
pub struct FlowQuery {
    /// Response format: json or html
    pub format: Option<String>,
    /// Include detailed metadata
    pub include_metadata: Option<bool>,
    /// Cache timeout in seconds
    pub cache_timeout: Option<u64>,
}

/// Generate ETag for caching
fn generate_etag(content: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    format!("\"{:x}\"", hasher.finish())
}

/// Add caching headers to response
fn add_caching_headers(
    response: &mut axum::response::Response,
    current_time: chrono::DateTime<chrono::Utc>,
    content: &str,
) {
    let etag = generate_etag(content);
    let cache_control = "public, max-age=300"; // 5 minutes cache

    response
        .headers_mut()
        .insert(header::ETAG, etag.parse().unwrap());
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, cache_control.parse().unwrap());
    response.headers_mut().insert(
        header::LAST_MODIFIED,
        current_time.to_rfc2822().parse().unwrap(),
    );
}

/// Get flow diagram for session
pub async fn get_flow(
    State(state): State<ApiState>,
    Path(session_id): Path<String>,
    Query(query): Query<FlowQuery>,
) -> axum::response::Response {
    info!(
        "Getting flow diagram for session: {} with format: {:?}",
        session_id, query.format
    );

    let current_time = Utc::now();

    // Try to generate stateDiagram from database session data first
    match generate_state_diagram_from_db(&state, &session_id).await {
        Ok(flow_diagram) => {
            let response_data = json!({
                "session_id": session_id,
                "diagram": flow_diagram.diagram,
                "metadata": flow_diagram.metadata,
                "sessions": [],
                "tool_calls": flow_diagram.tool_calls
            });

            let mut response = match query.format.as_deref() {
                Some("html") => {
                    let html_content = StateDiagramGenerator::generate_html(&flow_diagram);
                    let mut resp = Html(html_content).into_response();
                    add_caching_headers(&mut resp, current_time, &flow_diagram.diagram);
                    resp
                }
                _ => {
                    let json_content = response_data.to_string();
                    let mut resp = Json(response_data).into_response();
                    add_caching_headers(&mut resp, current_time, &json_content);
                    resp
                }
            };

            // Add consolidated session support header
            response.headers_mut().insert(
                "X-Flow-Type",
                "consolidated-session-capable".parse().unwrap(),
            );

            response
        }
        Err(e) => {
            warn!(
                "Failed to generate state diagram from session data for session {}: {}, falling back",
                session_id, e
            );

            // Fallback to session data
            get_session_fallback(state, session_id, query).await
        }
    }
}

/// Generate stateDiagram from database session data
async fn generate_state_diagram_from_db(
    state: &ApiState,
    session_id: &str,
) -> Result<crate::handlers::flow_diagram::FlowDiagram, FlowDiagramError> {
    // First check if this is a consolidated session
    info!("Checking for consolidated session: {}", session_id);
    match state.db.get_consolidated_session(session_id).await {
        Ok(Some(consolidated_content)) => {
            info!("Found consolidated session, generating ping-pong diagram");

            // Parse consolidated content as session
            let _consolidated_data: serde_json::Value =
                match serde_json::from_str(&consolidated_content) {
                    Ok(data) => data,
                    Err(e) => {
                        warn!("Failed to parse consolidated content as JSON: {}", e);
                        return Err(FlowDiagramError::InvalidLogFormat(format!(
                            "Consolidated content parsing failed: {e}"
                        )));
                    }
                };

            // Create parsed session from consolidated content
            let session_data = json!({
                "session_id": session_id,
                "log_content": consolidated_content,
                "start_time": 0,
                "end_time": 1,
                "is_dynamic_flow": true,
                "flow_type": "consolidated",
                "is_consolidated": true
            });

            // Parse the consolidated session content
            match SessionParser::parse_session_content(&session_data.to_string()) {
                Ok(parsed_session) => {
                    // Generate diagram for consolidated session with special handling
                    let flow_diagram = if parsed_session.tool_calls.is_empty() {
                        info!("No tool calls in consolidated session, generating enhanced diagram");
                        StateDiagramGenerator::generate_dynamic_flow_diagram(
                            &parsed_session,
                            session_id,
                        )
                    } else {
                        info!(
                            "Generating consolidated diagram with {} tool calls",
                            parsed_session.tool_calls.len()
                        );
                        StateDiagramGenerator::generate_dynamic_flow_diagram(
                            &parsed_session,
                            session_id,
                        )
                    };

                    Ok(flow_diagram)
                }
                Err(e) => {
                    warn!("Failed to parse consolidated session content: {}", e);
                    Err(FlowDiagramError::InvalidLogFormat(format!(
                        "Consolidated session parsing failed: {e}"
                    )))
                }
            }
        }
        Ok(None) => {
            // Not a consolidated session, continue with normal session processing
            info!("No consolidated session found, checking regular session logs");

            // Check if this is a dynamic flow session
            let is_dynamic_flow = session_id.starts_with("direct-")
                || session_id.starts_with("bridge-")
                || session_id.starts_with("recovery-");

            if is_dynamic_flow {
                info!("Processing dynamic flow session: {}", session_id);
            }

            // Get session log from database
            match state.db.get_session_log(session_id).await {
                Ok(log_content) => {
                    info!("Found session log in database for session: {}", session_id);

                    // Create a mock session structure for parsing
                    let session_data = json!({
                        "session_id": session_id,
                        "log_content": log_content,
                        "start_time": 0,
                        "end_time": 1,
                        "is_dynamic_flow": is_dynamic_flow,
                        "flow_type": if session_id.starts_with("direct-") {
                            "direct"
                        } else if session_id.starts_with("bridge-") {
                            "bridge"
                        } else if session_id.starts_with("recovery-") {
                            "recovery"
                        } else {
                            "static"
                        }
                    });

                    // Parse the session content
                    let parsed_session =
                        SessionParser::parse_session_content(&session_data.to_string())?;

                    // Generate the diagram
                    if parsed_session.tool_calls.is_empty() {
                        if is_dynamic_flow {
                            info!("No tool calls found in dynamic flow log, generating enhanced diagram");
                            Ok(StateDiagramGenerator::generate_dynamic_flow_diagram(
                                &parsed_session,
                                session_id,
                            ))
                        } else {
                            info!("No tool calls found in database log, generating simple diagram");
                            Ok(StateDiagramGenerator::generate_simple_diagram(
                                &parsed_session,
                            ))
                        }
                    } else {
                        info!(
                            "Generating diagram with {} tool calls from database",
                            parsed_session.tool_calls.len()
                        );
                        if is_dynamic_flow {
                            Ok(StateDiagramGenerator::generate_dynamic_flow_diagram(
                                &parsed_session,
                                session_id,
                            ))
                        } else {
                            StateDiagramGenerator::generate_diagram(&parsed_session)
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to retrieve session log from database for session {}: {}",
                        session_id, e
                    );
                    Err(FlowDiagramError::SessionNotFound(format!(
                        "Database session log not found: {session_id}"
                    )))
                }
            }
        }
        Err(e) => {
            error!(
                "Failed to check consolidated session for {}: {}",
                session_id, e
            );
            Err(FlowDiagramError::SessionNotFound(format!(
                "Failed to retrieve session: {session_id}"
            )))
        }
    }
}

/// Fallback to session data when diagram generation fails
async fn get_session_fallback(
    state: ApiState,
    session_id: String,
    query: FlowQuery,
) -> axum::response::Response {
    info!("Using session fallback for: {}", session_id);

    // Try to get session from database as fallback
    match state.db.get_session_log(&session_id).await {
        Ok(log_content) => {
            info!("Found session log fallback for: {}", session_id);

            let session_data = json!({
                "session_id": session_id,
                "content": log_content,
                "timestamp": Utc::now().to_rfc3339()
            });

            let response_data = json!({
                "session_id": session_id,
                "fallback": true,
                "session_data": session_data,
                "message": "Using session data fallback"
            });

            match query.format.as_deref() {
                Some("html") => {
                    let html_content = generate_fallback_html(&session_id, &response_data);
                    Html(html_content).into_response()
                }
                _ => Json(response_data).into_response(),
            }
        }
        Err(e) => {
            error!("Session fallback failed for {}: {}", session_id, e);

            let error_response = json!({
                "error": "Session not found",
                "session_id": session_id,
                "message": "No session data available",
                "details": format!("{}", e)
            });

            (StatusCode::NOT_FOUND, Json(error_response)).into_response()
        }
    }
}

/// Generate HTML fallback for session data
fn generate_fallback_html(session_id: &str, data: &serde_json::Value) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Session Fallback - {}</title>
    <script src="https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js"></script>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .session-info {{ background: #f5f5f5; padding: 15px; border-radius: 5px; margin-bottom: 20px; }}
        .error {{ background: #ffebee; padding: 15px; border-radius: 5px; border-left: 4px solid #f44336; }}
        .diagram {{ background: white; padding: 20px; border: 1px solid #ddd; border-radius: 5px; }}
    </style>
</head>
<body>
    <h1>Session: {}</h1>
    <div class="session-info">
        <h3>Session Information</h3>
        <p><strong>Status:</strong> Fallback Mode</p>
        <p><strong>Reason:</strong> Session data available but diagram generation failed</p>
        <pre>{}</pre>
    </div>
    <div class="diagram">
        <h3>Session Data</h3>
        <pre>{}</pre>
    </div>
</body>
</html>
        "#,
        session_id,
        session_id,
        "Session data available but cannot generate Mermaid diagram",
        serde_json::to_string_pretty(data).unwrap_or_default()
    )
}
