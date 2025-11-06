use chrono::Utc;
use reev_flow::enhanced_otel::{
    EnhancedToolCall, EventType, PromptInfo, TimingInfo, ToolInputInfo, ToolOutputInfo,
};
use reev_flow::jsonl_converter::JsonlToYmlConverter;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_jsonl_to_yml_conversion() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let jsonl_path = temp_dir.path().join("test.jsonl");
    let yml_path = temp_dir.path().join("test.yml");

    // Create test JSONL content
    let test_events = vec![
        EnhancedToolCall {
            timestamp: Utc::now(),
            session_id: "test_session".to_string(),
            reev_runner_version: "0.1.0".to_string(),
            reev_agent_version: "0.1.0".to_string(),
            event_type: EventType::Prompt,
            prompt: Some(PromptInfo {
                tool_name_list: vec![
                    reev_constants::SOL_TRANSFER.to_string(),
                    reev_constants::JUPITER_SWAP.to_string(),
                ],
                user_prompt: "Transfer SOL to user".to_string(),
                final_prompt: format!(
                    "Available tools: {}, {}. Transfer SOL to user",
                    reev_constants::SOL_TRANSFER,
                    reev_constants::JUPITER_SWAP
                ),
            }),
            tool_input: None,
            tool_output: None,
            timing: TimingInfo {
                flow_timeuse_ms: 0,
                step_timeuse_ms: 0,
            },
            metadata: serde_json::json!({}),
        },
        EnhancedToolCall {
            timestamp: Utc::now(),
            session_id: "test_session".to_string(),
            reev_runner_version: "0.1.0".to_string(),
            reev_agent_version: "0.1.0".to_string(),
            event_type: EventType::ToolInput,
            prompt: None,
            tool_input: Some(ToolInputInfo {
                tool_name: reev_constants::SOL_TRANSFER.to_string(),
                tool_args: serde_json::json!({"user_pubkey": "abc123", "amount": 1000}),
            }),
            tool_output: None,
            timing: TimingInfo {
                flow_timeuse_ms: 0,
                step_timeuse_ms: 0,
            },
            metadata: serde_json::json!({}),
        },
        EnhancedToolCall {
            timestamp: Utc::now(),
            session_id: "test_session".to_string(),
            reev_runner_version: "0.1.0".to_string(),
            reev_agent_version: "0.1.0".to_string(),
            event_type: EventType::ToolOutput,
            prompt: None,
            tool_input: None,
            tool_output: Some(ToolOutputInfo {
                success: true,
                results: serde_json::json!({
                    "transaction": "tx123",
                    "tool_name": reev_constants::SOL_TRANSFER
                }),
                error_message: None,
            }),
            timing: TimingInfo {
                flow_timeuse_ms: 100,
                step_timeuse_ms: 100,
            },
            metadata: serde_json::json!({}),
        },
    ];

    // Write test JSONL file
    let mut jsonl_content = String::new();
    for event in test_events {
        jsonl_content.push_str(&serde_json::to_string(&event)?);
        jsonl_content.push('\n');
    }
    fs::write(&jsonl_path, jsonl_content)?;

    // Convert to YML
    let session_data = JsonlToYmlConverter::convert_file(&jsonl_path, &yml_path)?;

    // Verify conversion
    assert_eq!(session_data.session_id, "test_session");
    assert_eq!(session_data.tool_calls.len(), 1);
    assert_eq!(
        session_data.tool_calls[0].tool_name,
        reev_constants::SOL_TRANSFER
    );
    assert_eq!(session_data.summary.total_tool_calls, 1);
    assert_eq!(session_data.summary.successful_tool_calls, 1);

    // Verify YML file was created
    assert!(yml_path.exists());
    let yml_content = fs::read_to_string(&yml_path)?;
    assert!(yml_content.contains("session_id: test_session"));
    assert!(yml_content.contains("tool_name: sol_transfer"));

    Ok(())
}
