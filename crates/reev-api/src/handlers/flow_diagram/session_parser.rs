//! Session Log Parser
//!
//! This module handles parsing of enhanced session logs to extract tool calls
//! and execution information for flow diagram generation.

use crate::handlers::flow_diagram::{DiagramMetadata, FlowDiagramError};
use serde_json::Value;
use std::path::Path;
use tracing::{debug, info};

/// Session log parser for extracting tool calls and execution data
pub struct SessionParser;

/// Parsed session data suitable for diagram generation
#[derive(Debug, Clone)]
pub struct ParsedSession {
    /// Session identifier
    pub session_id: String,
    /// Benchmark identifier
    pub benchmark_id: String,
    /// Agent type
    pub agent_type: String,
    /// Extracted tool calls
    pub tool_calls: Vec<ParsedToolCall>,
    /// Session prompt
    pub prompt: Option<String>,
    /// Final execution result
    pub final_result: Option<Value>,
    /// Session start time
    pub start_time: u64,
    /// Session end time
    pub end_time: Option<u64>,
}

/// Parsed tool call information
#[derive(Debug, Clone)]
pub struct ParsedToolCall {
    /// Tool identifier
    pub tool_id: String,
    /// Tool start time (Unix timestamp)
    pub start_time: u64,
    /// Tool end time (Unix timestamp)
    pub end_time: u64,
    /// Tool parameters
    pub params: Value,
    /// Tool result
    pub result: Option<Value>,
    /// Tool execution status
    pub status: String,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

impl SessionParser {
    /// Parse a session log file and extract tool calls
    pub async fn parse_session_file(file_path: &Path) -> Result<ParsedSession, FlowDiagramError> {
        info!("Parsing session file: {}", file_path.display());

        let content = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| FlowDiagramError::ParseError(format!("Failed to read file: {e}")))?;

        Self::parse_session_content(&content)
    }

    /// Parse session log content and extract tool calls
    pub fn parse_session_content(content: &str) -> Result<ParsedSession, FlowDiagramError> {
        debug!("Parsing session content (length: {})", content.len());

        let session_log: Value = serde_json::from_str(content).map_err(|e| {
            FlowDiagramError::InvalidLogFormat(format!("JSON parsing failed: {e}"))
        })?;

        // Extract basic session information
        let session_id = session_log
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowDiagramError::InvalidLogFormat("Missing session_id".to_string()))?
            .to_string();

        let benchmark_id = session_log
            .get("benchmark_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let agent_type = session_log
            .get("agent_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let start_time = session_log
            .get("start_time")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let end_time = session_log.get("end_time").and_then(|v| v.as_u64());

        // Extract prompt from final_result data
        let prompt = session_log
            .get("final_result")
            .and_then(|fr| fr.get("data"))
            .and_then(|data| data.get("prompt"))
            .and_then(|p| p.as_str())
            .map(|s| s.to_string());

        // Extract tool calls from multiple sources
        let mut tool_calls = Vec::new();

        // First try: Enhanced session logs with tools array
        if let Some(tools) = session_log
            .get("final_result")
            .and_then(|fr| fr.get("data"))
            .and_then(|data| data.get("tools"))
            .and_then(|tools| tools.as_array())
        {
            debug!("Found {} tools in enhanced session log", tools.len());
            for tool in tools {
                if let Ok(parsed_tool) = Self::parse_enhanced_tool(tool) {
                    tool_calls.push(parsed_tool);
                }
            }
        } else {
            debug!("No tools array found, trying to extract from events");
            // Second try: Extract from events (backward compatibility)
            if let Some(events) = session_log.get("events").and_then(|e| e.as_array()) {
                Self::extract_tools_from_events(events, &mut tool_calls);
            }
        }

        // Sort tool calls by start time
        tool_calls.sort_by_key(|t| t.start_time);

        let final_result = session_log.get("final_result").cloned();

        info!(
            "Parsed session {} with {} tool calls",
            session_id,
            tool_calls.len()
        );

        Ok(ParsedSession {
            session_id,
            benchmark_id,
            agent_type,
            tool_calls,
            prompt,
            final_result,
            start_time,
            end_time,
        })
    }

    /// Parse enhanced tool call from tools array
    fn parse_enhanced_tool(tool: &Value) -> Result<ParsedToolCall, FlowDiagramError> {
        let tool_id = tool
            .get("tool_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowDiagramError::InvalidLogFormat("Missing tool_id".to_string()))?
            .to_string();

        let start_time = tool
            .get("start_time")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| FlowDiagramError::InvalidLogFormat("Missing start_time".to_string()))?;

        let end_time = tool
            .get("end_time")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| FlowDiagramError::InvalidLogFormat("Missing end_time".to_string()))?;

        let params = tool.get("params").cloned().unwrap_or(Value::Null);
        let result = tool.get("result").cloned();
        let status = tool
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let duration_ms = end_time.saturating_sub(start_time) * 1000;

        Ok(ParsedToolCall {
            tool_id,
            start_time,
            end_time,
            params,
            result,
            status,
            duration_ms,
        })
    }

    /// Extract tool calls from events array (backward compatibility)
    fn extract_tools_from_events(events: &[Value], tool_calls: &mut Vec<ParsedToolCall>) {
        let mut tool_starts = std::collections::HashMap::new();

        for event in events {
            if let Some(event_type) = event.get("event_type").and_then(|v| v.as_str()) {
                match event_type {
                    "ToolCall" => {
                        if let (Some(tool_id), Some(start_time), Some(params)) = (
                            event
                                .get("data")
                                .and_then(|d| d.get("tool_id"))
                                .and_then(|v| v.as_str()),
                            event
                                .get("data")
                                .and_then(|d| d.get("start_time"))
                                .and_then(|v| v.as_u64()),
                            event.get("data").and_then(|d| d.get("params")),
                        ) {
                            tool_starts.insert(tool_id.to_string(), (start_time, params.clone()));
                        }
                    }
                    "ToolResult" => {
                        if let (Some(tool_id), Some(end_time), Some(result), Some(status)) = (
                            event
                                .get("data")
                                .and_then(|d| d.get("tool_id"))
                                .and_then(|v| v.as_str()),
                            event
                                .get("data")
                                .and_then(|d| d.get("end_time"))
                                .and_then(|v| v.as_u64()),
                            event.get("data").and_then(|d| d.get("result")),
                            event
                                .get("data")
                                .and_then(|d| d.get("status"))
                                .and_then(|v| v.as_str()),
                        ) {
                            if let Some((start_time, params)) = tool_starts.remove(tool_id) {
                                let duration_ms = end_time.saturating_sub(start_time) * 1000;
                                tool_calls.push(ParsedToolCall {
                                    tool_id: tool_id.to_string(),
                                    start_time,
                                    end_time,
                                    params,
                                    result: Some(result.clone()),
                                    status: status.to_string(),
                                    duration_ms,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        debug!("Extracted {} tool calls from events", tool_calls.len());
    }

    /// Create diagram metadata from parsed session
    pub fn create_metadata(parsed: &ParsedSession) -> DiagramMetadata {
        let execution_time_ms = if let Some(end) = parsed.end_time {
            end.saturating_sub(parsed.start_time) * 1000
        } else {
            parsed.tool_calls.iter().map(|t| t.duration_ms).sum()
        };

        DiagramMetadata {
            state_count: 2 + parsed.tool_calls.len(), // Start + End + Tools
            tool_count: parsed.tool_calls.len(),
            execution_time_ms,
            benchmark_id: parsed.benchmark_id.clone(),
            session_id: Some(parsed.session_id.clone()),
        }
    }

    /// Find the latest session file for a benchmark
    pub async fn find_latest_session(
        benchmark_id: &str,
        sessions_dir: &Path,
    ) -> Result<String, FlowDiagramError> {
        let mut sessions = tokio::fs::read_dir(sessions_dir).await.map_err(|e| {
            FlowDiagramError::SessionNotFound(format!("Failed to read sessions directory: {e}"))
        })?;

        let mut latest_session = None;
        let mut latest_timestamp = 0u64;
        let mut session_count = 0;

        while let Some(entry) = sessions.next_entry().await.map_err(|e| {
            FlowDiagramError::SessionNotFound(format!("Failed to read directory entry: {e}"))
        })? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if filename.starts_with("session_") {
                    session_count += 1;
                    debug!("Checking session file #{}: {}", session_count, filename);
                    // Try to read the session to check its benchmark_id and timestamp
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        if let Ok(session) = serde_json::from_str::<Value>(&content) {
                            // Check if this session belongs to the requested benchmark
                            if let Some(session_benchmark_id) =
                                session.get("benchmark_id").and_then(|v| v.as_str())
                            {
                                debug!(
                                    "Session {} has benchmark_id: {} (looking for: {})",
                                    filename, session_benchmark_id, benchmark_id
                                );
                                if session_benchmark_id == benchmark_id {
                                    if let Some(start_time) =
                                        session.get("start_time").and_then(|v| v.as_u64())
                                    {
                                        debug!(
                                            "Found matching session: {} with start_time: {}",
                                            filename, start_time
                                        );
                                        if start_time > latest_timestamp {
                                            latest_timestamp = start_time;
                                            latest_session =
                                                Some(path.to_string_lossy().to_string());
                                            debug!(
                                                "New latest session: {} at time: {}",
                                                filename, start_time
                                            );
                                        }
                                    }
                                }
                            } else {
                                debug!("Session {} has no benchmark_id", filename);
                            }
                        } else {
                            debug!("Failed to parse session JSON: {}", filename);
                        }
                    } else {
                        debug!("Failed to read session file: {}", filename);
                    }
                }
            }
        }

        debug!("Checked {} session files total", session_count);
        if let Some(session) = latest_session {
            debug!("Returning latest session: {}", session);
            Ok(session)
        } else {
            Err(FlowDiagramError::SessionNotFound(format!(
                "No session found for benchmark: {benchmark_id}"
            )))
        }
    }
}
