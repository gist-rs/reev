//! Flow handlers - provides flow information with stateDiagram visualization
use crate::handlers::flow_diagram::{FlowDiagramError, SessionParser, StateDiagramGenerator};
use crate::types::*;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tracing::{error, info, warn};

#[derive(Debug, Deserialize)]
pub struct FlowQuery {
    format: Option<String>,
}

/// Generate ETag from content
fn generate_etag(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!(r#""{}""#, hasher.finish())
}

/// Add caching headers to response
fn add_caching_headers(
    response: &mut axum::response::Response,
    last_modified: DateTime<Utc>,
    content: &str,
) {
    let headers = response.headers_mut();

    // Add Last-Modified header
    headers.insert(
        header::LAST_MODIFIED,
        last_modified.to_rfc2822().parse().unwrap(),
    );

    // Add ETag header
    let etag = generate_etag(content);
    headers.insert(header::ETAG, etag.parse().unwrap());

    // Add Cache-Control header for reasonable caching
    headers.insert(
        header::CACHE_CONTROL,
        "public, max-age=30, must-revalidate".parse().unwrap(),
    );

    // Add polling frequency recommendation header
    headers.insert(
        "X-Polling-Recommendation",
        "Recommended polling interval: 1-5 seconds for active flows, 30-60 seconds for completed flows".parse().unwrap(),
    );
}

/// Get flows for a benchmark with optional stateDiagram visualization
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
                "sessions": []
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

            // Add dynamic flow session support header
            response
                .headers_mut()
                .insert("X-Flow-Type", "dynamic-flow-capable".parse().unwrap());

            response
        }
        Err(e) => {
            warn!(
                "Failed to generate state diagram from session files for session {}: {}, falling back to session data",
                session_id, e
            );

            // Final fallback to existing session data
            get_session_fallback(state, session_id, query).await
        }
    }
}

/// Generate stateDiagram from database session data
async fn generate_state_diagram_from_db(
    state: &ApiState,
    session_id: &str,
) -> Result<crate::handlers::flow_diagram::FlowDiagram, FlowDiagramError> {
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
            let parsed_session = SessionParser::parse_session_content(&session_data.to_string())?;

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
            warn!(
                "Failed to get session log from database for session {}: {}",
                session_id, e
            );

            // For dynamic flows, provide a fallback diagram even if session not found
            if is_dynamic_flow {
                info!(
                    "Creating fallback diagram for dynamic flow session: {}",
                    session_id
                );
                let fallback_session = json!({
                    "session_id": session_id,
                    "tool_calls": [],
                    "start_time": 0,
                    "end_time": 1,
                    "is_dynamic_flow": true,
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

                let parsed_fallback =
                    SessionParser::parse_session_content(&fallback_session.to_string())?;
                return Ok(StateDiagramGenerator::generate_dynamic_flow_diagram(
                    &parsed_fallback,
                    session_id,
                ));
            }

            Err(FlowDiagramError::SessionNotFound(format!(
                "Session not found in database: {session_id}"
            )))
        }
    }
}

/// Fallback to existing session data if diagram generation fails
async fn get_session_fallback(
    state: ApiState,
    session_id: String,
    query: FlowQuery,
) -> axum::response::Response {
    info!("Using fallback session data for session: {}", session_id);

    // Database-only approach: get sessions instead of in-memory executions
    let filter = reev_db::types::SessionFilter {
        benchmark_id: None,
        agent_type: None,
        interface: Some("web".to_string()),
        status: None,
        limit: None,
    };

    let mut active_executions = Vec::new();

    match state.db.list_sessions(&filter).await {
        Ok(sessions) => {
            for session in sessions {
                let is_running = session.status == "running" || session.status == "queued";
                active_executions.push(json!({
                    "session_id": session.session_id,
                    "agent_type": session.agent_type,
                    "interface": session.interface,
                    "status": session.status,
                    "score": session.score,
                    "final_status": session.final_status,
                    "log_content": "", // Will be populated from session log if needed
                    "is_running": is_running
                }));
            }
        }
        Err(e) => {
            warn!("Failed to get sessions from database: {}", e);
        }
    }

    if !active_executions.is_empty() {
        let response_data = json!({
            "session_id": session_id,
            "sessions": active_executions,
            "diagram": null,
            "metadata": null
        });

        return match query.format.as_deref() {
            Some("html") => Html(generate_fallback_html(&response_data)).into_response(),
            _ => Json(response_data).into_response(),
        };
    }

    // Look for the specific session
    let filter = reev_db::types::SessionFilter {
        benchmark_id: None, // We're looking for specific session_id
        agent_type: None,
        interface: None,
        status: None,
        limit: None,
    };

    match state.db.list_sessions(&filter).await {
        Ok(sessions) => {
            let response_data = if sessions.is_empty() {
                json!({
                    "session_id": session_id,
                    "sessions": [],
                    "diagram": null,
                    "metadata": null
                })
            } else {
                let mut session_logs = Vec::new();
                for session in sessions {
                    // Only include the session that matches our session_id
                    if session.session_id == session_id {
                        match state.db.get_session_log(&session.session_id).await {
                            Ok(log_content) => {
                                session_logs.push(json!({
                                    "session_id": session.session_id,
                                    "agent_type": session.agent_type,
                                    "interface": session.interface,
                                    "status": session.status,
                                    "score": session.score,
                                    "final_status": session.final_status,
                                    "log_content": log_content,
                                    "is_running": false
                                }));
                            }
                            Err(e) => {
                                error!(
                                    "Failed to get log for session {}: {}",
                                    session.session_id, e
                                );
                            }
                        }
                        break; // Found our session, no need to continue
                    }
                }

                json!({
                    "session_id": session_id,
                    "sessions": session_logs,
                    "diagram": null,
                    "metadata": null
                })
            };

            match query.format.as_deref() {
                Some("html") => Html(generate_fallback_html(&response_data)).into_response(),
                _ => Json(response_data).into_response(),
            }
        }
        Err(e) => {
            error!("Failed to get sessions for session {}: {}", session_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

/// Generate fallback HTML for when diagram generation fails
fn generate_fallback_html(data: &serde_json::Value) -> String {
    let benchmark_id = data
        .get("benchmark_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Flow: {} (No Diagram Available)</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 800px;
            margin: 0 auto;
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .warning {{
            background-color: #fff3cd;
            border: 1px solid #ffeaa7;
            color: #856404;
            padding: 15px;
            border-radius: 4px;
            margin-bottom: 20px;
        }}
        .info {{
            background-color: #d1ecf1;
            border: 1px solid #bee5eb;
            color: #0c5460;
            padding: 15px;
            border-radius: 4px;
            margin-bottom: 20px;
        }}
        .sessions {{
            margin-top: 20px;
        }}
        .session {{
            border: 1px solid #ddd;
            margin: 10px 0;
            padding: 15px;
            border-radius: 5px;
            background: #fafafa;
        }}
        .dynamic-flow {{
            border-left: 4px solid #28a745;
            background-color: #f8fff9;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="info">
            <strong>Dynamic Flow Support</strong><br>
            This API supports dynamic flow execution with the following endpoints:<br>
            • <code>POST /api/v1/benchmarks/execute-direct</code> - Zero file I/O execution<br>
            • <code>POST /api/v1/benchmarks/execute-bridge</code> - Compatibility mode<br>
            • <code>POST /api/v1/benchmarks/execute-recovery</code> - Resilient execution<br>
            <br>
            <strong>API Usage Guidelines:</strong><br>
            • Polling frequency: 1-5 seconds for active flows, 30-60 seconds for completed<br>
            • Use ETag/Last-Modified headers for conditional requests<br>
            • Dynamic flow sessions are marked with <span class="dynamic-flow">green border</span>
        </div>

        <div class="warning">
            <strong>Flow Diagram Not Available</strong><br>
            Unable to generate state diagram for this benchmark. This could be because:
            <ul>
                <li>No session logs found with tool call information</li>
                <li>Session logs are in an older format without tool tracking</li>
                <li>The benchmark execution didn't involve any tool calls</li>
                <li>Dynamic flow session may still be initializing</li>
            </ul>
        </div>

        <div class="sessions">
            <h3>Available Sessions</h3>
            {}
        </div>
    </div>
</body>
</html>"#,
        benchmark_id,
        format_sessions_for_fallback(data)
    )
}

/// Format sessions for fallback HTML display
fn format_sessions_for_fallback(data: &serde_json::Value) -> String {
    if let Some(sessions) = data.get("sessions").and_then(|s| s.as_array()) {
        if sessions.is_empty() {
            "<p>No sessions found for this benchmark.</p><p><strong>Note:</strong> Dynamic flow sessions use execution IDs starting with 'direct-', 'bridge-', or 'recovery-' prefixes.</p>".to_string()
        } else {
            sessions
                .iter()
                .map(|session| {
                    let session_id = session
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let agent_type = session
                        .get("agent_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let status = session
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    let session_class = if session_id.starts_with("direct-")
                        || session_id.starts_with("bridge-")
                        || session_id.starts_with("recovery-")
                    {
                        "session dynamic-flow"
                    } else {
                        "session"
                    };

                    format!(
                        r#"<div class="{}">
                            <strong>Session:</strong> {}<br>
                            <strong>Agent:</strong> {}<br>
                            <strong>Status:</strong> {}<br>
                            <strong>Type:</strong> {}
                        </div>"#,
                        session_class,
                        &session_id[..8.min(session_id.len())],
                        agent_type,
                        status,
                        if session_id.starts_with("direct-") {
                            "Direct Dynamic Flow"
                        } else if session_id.starts_with("bridge-") {
                            "Bridge Dynamic Flow"
                        } else if session_id.starts_with("recovery-") {
                            "Recovery Dynamic Flow"
                        } else {
                            "Static Flow"
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        }
    } else {
        "<p>No session data available.</p>".to_string()
    }
}
