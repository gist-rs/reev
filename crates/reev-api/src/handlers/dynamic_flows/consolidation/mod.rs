//! Consolidation module for dynamic flow JSONL→YML pipeline
//!
//! This module provides the same JSONL→YML consolidation process
//! that static flows use, ensuring dynamic flows have proper
//! flow visualization with complete tool call tracking.

use anyhow::{anyhow, Result};
use glob::glob;
use reev_db::writer::DatabaseWriterTrait;
use reev_flow::{get_enhanced_otel_logger, JsonlToYmlConverter};
use std::path::PathBuf;
use tracing::{error, info, warn};

/// Consolidate enhanced OTEL data for dynamic flows
///
/// This function ensures dynamic flows use the same JSONL→YML consolidation
/// pipeline as static flows, fixing Issue #41 where dynamic flows bypass
/// consolidation causing empty flow diagrams.
///
/// # Arguments
/// * `state` - API state with database access
/// * `flow_id` - Dynamic flow identifier
/// * `execution_id` - Execution identifier for session tracking
///
/// # Returns
/// * `Option<usize>` - Number of tool calls consolidated, or None if failed
pub async fn consolidate_otel_data(
    state: &crate::types::ApiState,
    flow_id: &str,
    execution_id: &str,
) -> Option<usize> {
    info!(
        "[Consolidation] Starting JSONL→YML consolidation for dynamic flow: {} (execution: {})",
        flow_id, execution_id
    );

    // Get enhanced OTEL logger and trigger summary writing
    let session_id = if let Ok(logger) = get_enhanced_otel_logger() {
        // Write summary to ensure JSONL file is complete
        if let Err(e) = logger.write_summary() {
            warn!(
                "[Consolidation] Failed to write enhanced OTEL summary: {}",
                e
            );
        }
        logger.session_id().to_string()
    } else {
        error!("[Consolidation] Failed to get enhanced OTEL logger");
        return None;
    };

    // Try global enhanced OTEL file first (tool calls are written here by log_tool_call! macro)
    let global_jsonl_path =
        PathBuf::from(format!("logs/sessions/enhanced_otel_{session_id}.jsonl"));
    if global_jsonl_path.exists() {
        info!(
            "[Consolidation] Found global enhanced OTEL file: {:?}",
            global_jsonl_path
        );
        return match perform_consolidation(state, &global_jsonl_path, execution_id).await {
            Ok(count) => Some(count),
            Err(e) => {
                error!(
                    "[Consolidation] Failed to consolidate global OTEL file: {}",
                    e
                );
                None
            }
        };
    }

    // Fallback: try orchestrator session format (used by ping-pong executor initialization)
    let orchestrator_glob_pattern = format!(
        "logs/sessions/enhanced_otel_orchestrator-flow-{flow_id}-*.jsonl"
    );

    info!(
        "[Consolidation] Looking for orchestrator files with pattern: {}",
        orchestrator_glob_pattern
    );

    if let Ok(glob_pattern) = glob(&orchestrator_glob_pattern) {
        if let Some(found_path) = glob_pattern.filter_map(Result::ok).next() {
            info!(
                "[Consolidation] Found orchestrator JSONL file: {:?}",
                found_path
            );
            return match perform_consolidation(state, &found_path, execution_id).await {
                Ok(count) => Some(count),
                Err(e) => {
                    error!(
                        "[Consolidation] Failed to consolidate orchestrator file: {}",
                        e
                    );
                    None
                }
            };
        } else {
            warn!(
                "[Consolidation] No orchestrator files found for pattern: {}",
                orchestrator_glob_pattern
            );
        }
    } else {
        error!(
            "[Consolidation] Failed to create glob pattern: {}",
            orchestrator_glob_pattern
        );
    }

    // Try direct flow_id as session identifier
    let direct_jsonl_path = PathBuf::from(format!("logs/sessions/enhanced_otel_{flow_id}.jsonl"));
    if direct_jsonl_path.exists() {
        info!(
            "[Consolidation] Found direct JSONL file: {:?}",
            direct_jsonl_path
        );
        return match perform_consolidation(state, &direct_jsonl_path, execution_id).await {
            Ok(count) => Some(count),
            Err(e) => {
                error!("[Consolidation] Failed to consolidate direct file: {}", e);
                None
            }
        };
    }

    // Try execution_id as fallback
    let fallback_jsonl_path = PathBuf::from(format!(
        "logs/sessions/enhanced_otel_{execution_id}.jsonl"
    ));

    if fallback_jsonl_path.exists() {
        info!(
            "[Consolidation] Found fallback JSONL file: {:?}",
            fallback_jsonl_path
        );
        return match perform_consolidation(state, &fallback_jsonl_path, execution_id).await {
            Ok(count) => Some(count),
            Err(e) => {
                error!("[Consolidation] Failed to consolidate fallback file: {}", e);
                None
            }
        };
    }

    error!(
        "[Consolidation] No JSONL files found for flow_id: {}, execution_id: {}",
        flow_id, execution_id
    );
    None
}

/// Validate consolidation pipeline integrity
///
/// This function checks that the consolidation pipeline is working
/// correctly by verifying file creation and content.
pub async fn validate_consolidation(flow_id: &str) -> Result<bool> {
    let jsonl_path = PathBuf::from(format!("logs/sessions/enhanced_otel_{flow_id}.jsonl"));

    // Check JSONL file exists
    if !jsonl_path.exists() {
        return Ok(false);
    }

    // Try to parse JSONL content
    let temp_yml_path = jsonl_path.with_extension("yml");
    match JsonlToYmlConverter::convert_file(&jsonl_path, &temp_yml_path) {
        Ok(session_data) => {
            // Clean up temp file
            let _ = tokio::fs::remove_file(&temp_yml_path).await;
            Ok(!session_data.tool_calls.is_empty())
        }
        Err(_) => {
            // Clean up temp file if it was created
            let _ = tokio::fs::remove_file(&temp_yml_path).await;
            Ok(false)
        }
    }
}

/// Perform the actual JSONL→YML consolidation
async fn perform_consolidation(
    state: &crate::types::ApiState,
    jsonl_path: &PathBuf,
    execution_id: &str,
) -> Result<usize> {
    // Convert JSONL to YML format using temporary file
    let temp_yml_path = jsonl_path.with_extension("yml");
    let session_data = JsonlToYmlConverter::convert_file(jsonl_path, &temp_yml_path)
        .map_err(|e| anyhow!("Failed to convert enhanced OTEL JSONL to YML: {e}"))?;

    info!(
        "[Consolidation] ✅ JSONL→YML conversion successful: {} tool calls",
        session_data.tool_calls.len()
    );

    // Read YML content for database storage
    let yml_content = tokio::fs::read_to_string(&temp_yml_path)
        .await
        .map_err(|e| anyhow!("Failed to read temporary YML file: {e}"))?;

    info!(
        "[Consolidation] ✅ Read YML content ({} bytes)",
        yml_content.len()
    );

    // Debug: let's also log the problematic line content
    let file_content = tokio::fs::read_to_string(jsonl_path).await?;
    warn!(
        "[Consolidation] DEBUG: JSONL file content preview:\n{}",
        file_content.chars().take(500).collect::<String>()
    );

    // Clean up temporary file
    if let Err(e) = tokio::fs::remove_file(&temp_yml_path).await {
        warn!(
            "[Consolidation] Failed to clean up temporary YML file: {}",
            e
        );
    }

    // Store consolidated YML in database
    let session_id = execution_id; // Use execution_id as session identifier
    state
        .db
        .store_session_log(session_id, &yml_content)
        .await
        .map_err(|e| anyhow!("Failed to store session log in database: {e}"))?;

    info!(
        "[Consolidation] ✅ Stored consolidated session log in database: {}",
        session_id
    );

    Ok(session_data.tool_calls.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reev_flow::enhanced_otel::{
        EnhancedToolCall, EventType, TimingInfo, ToolInputInfo, ToolOutputInfo,
    };
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_consolidation_validation() {
        let temp_dir = tempdir().unwrap();
        let jsonl_path = temp_dir.path().join("test.jsonl");
        let flow_id = "test_flow";

        // Create test JSONL content
        let test_tool_call = EnhancedToolCall {
            timestamp: chrono::Utc::now(),
            session_id: flow_id.to_string(),
            reev_runner_version: "1.0.0".to_string(),
            reev_agent_version: "1.0.0".to_string(),
            event_type: EventType::ToolInput,
            prompt: None,
            tool_input: Some(ToolInputInfo {
                tool_name: "jupiter_swap".to_string(),
                tool_args: serde_json::json!({"amount": 1000000}),
            }),
            tool_output: Some(ToolOutputInfo {
                success: true,
                results: serde_json::json!({"signature": "test_sig"}),
                error_message: None,
            }),
            timing: TimingInfo {
                flow_timeuse_ms: 1000,
                step_timeuse_ms: 500,
            },
            metadata: serde_json::json!({}),
        };

        // Write test JSONL file
        use std::fs::File;
        use std::io::Write;
        let mut file = File::create(&jsonl_path).unwrap();
        writeln!(file, "{}", serde_json::to_string(&test_tool_call).unwrap()).unwrap();

        // Test validation
        let result = validate_consolidation(flow_id).await.unwrap();
        assert!(
            result,
            "Consolidation validation should succeed with valid JSONL"
        );

        // Test with non-existent flow
        let invalid_result = validate_consolidation("invalid_flow").await.unwrap();
        assert!(
            !invalid_result,
            "Consolidation validation should fail with non-existent flow"
        );
    }
}
