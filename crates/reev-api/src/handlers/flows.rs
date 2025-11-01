//! Flow handlers - provides flow information with stateDiagram visualization
use crate::handlers::flow_diagram::{FlowDiagramError, SessionParser, StateDiagramGenerator};
use crate::types::*;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use serde::Deserialize;
use serde_json::json;
use tracing::{error, info, warn};

#[derive(Debug, Deserialize)]
pub struct FlowQuery {
    format: Option<String>,
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

    // Try to generate stateDiagram from database session data first
    match generate_state_diagram_from_db(&state, &session_id).await {
        Ok(flow_diagram) => {
            let response_data = json!({
                "session_id": session_id,
                "diagram": flow_diagram.diagram,
                "metadata": flow_diagram.metadata,
                "sessions": []
            });

            match query.format.as_deref() {
                Some("html") => {
                    Html(StateDiagramGenerator::generate_html(&flow_diagram)).into_response()
                }
                _ => Json(response_data).into_response(),
            }
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
    // Get session log from database
    match state.db.get_session_log(session_id).await {
        Ok(log_content) => {
            info!("Found session log in database for session: {}", session_id);

            // Create a mock session structure for parsing
            let session_data = json!({
                "session_id": session_id,
                "log_content": log_content,
                "start_time": 0,
                "end_time": 1
            });

            // Parse the session content
            let parsed_session = SessionParser::parse_session_content(&session_data.to_string())?;

            // Generate the diagram
            if parsed_session.tool_calls.is_empty() {
                info!("No tool calls found in database log, generating simple diagram");
                Ok(StateDiagramGenerator::generate_simple_diagram(
                    &parsed_session,
                ))
            } else {
                info!(
                    "Generating diagram with {} tool calls from database",
                    parsed_session.tool_calls.len()
                );
                StateDiagramGenerator::generate_diagram(&parsed_session)
            }
        }
        Err(e) => {
            warn!(
                "Failed to get session log from database for session {}: {}",
                session_id, e
            );
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
    </style>
</head>
<body>
    <div class="container">
        <div class="warning">
            <strong>Flow Diagram Not Available</strong><br>
            Unable to generate state diagram for this benchmark. This could be because:
            <ul>
                <li>No session logs found with tool call information</li>
                <li>Session logs are in an older format without tool tracking</li>
                <li>The benchmark execution didn't involve any tool calls</li>
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
            "<p>No sessions found for this benchmark.</p>".to_string()
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

                    format!(
                        r#"<div class="session">
                            <strong>Session:</strong> {}<br>
                            <strong>Agent:</strong> {}<br>
                            <strong>Status:</strong> {}
                        </div>"#,
                        &session_id[..8.min(session_id.len())],
                        agent_type,
                        status
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        }
    } else {
        "<p>No session data available.</p>".to_string()
    }
}
