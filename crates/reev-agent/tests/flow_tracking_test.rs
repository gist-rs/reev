//! Integration test for flow tracking functionality
//!
//! This test demonstrates that the flow tracking system captures tool calls
//! and execution order when REEV_ENABLE_FLOW_LOGGING is enabled.

use std::env;

#[tokio::test]
#[serial_test::serial]
async fn test_flow_tracking_integration() {
    // Clean up any existing state first
    env::remove_var("REEV_ENABLE_FLOW_LOGGING");
    reev_agent::flow::GlobalFlowTracker::reset();

    // Enable flow logging for this test
    env::set_var("REEV_ENABLE_FLOW_LOGGING", "1");
    env::set_var("RUST_LOG", "info");

    // Simulate some tool calls
    reev_agent::flow::GlobalFlowTracker::record_tool_call(
        reev_agent::flow::tracker::tool_wrapper::ToolCallParams {
            tool_name: "test_tool_1".to_string(),
            tool_args: r#"{"param1": "value1", "param2": 42}"#.to_string(),
            execution_time_ms: 150,
            result_status: reev_lib::agent::ToolResultStatus::Success,
            result_data: Some(serde_json::json!({"result": "success"})),
            error_message: None,
            depth: 1,
        },
    );

    reev_agent::flow::GlobalFlowTracker::record_tool_call(
        reev_agent::flow::tracker::tool_wrapper::ToolCallParams {
            tool_name: "test_tool_2".to_string(),
            tool_args: r#"{"input": "data"}"#.to_string(),
            execution_time_ms: 200,
            result_status: reev_lib::agent::ToolResultStatus::Success,
            result_data: Some(serde_json::json!({"output": "processed"})),
            error_message: None,
            depth: 2,
        },
    );

    reev_agent::flow::GlobalFlowTracker::record_tool_call(
        reev_agent::flow::tracker::tool_wrapper::ToolCallParams {
            tool_name: "test_tool_3".to_string(),
            tool_args: r#"{"query": "search"}"#.to_string(),
            execution_time_ms: 75,
            result_status: reev_lib::agent::ToolResultStatus::Error,
            result_data: None,
            error_message: Some("Network timeout".to_string()),
            depth: 3,
        },
    );

    // Extract flow data
    let flow_data = reev_agent::flow::GlobalFlowTracker::get_flow_data();

    assert!(flow_data.is_some(), "Flow data should be available");

    let data = flow_data.unwrap();
    assert_eq!(
        data.total_tool_calls, 3,
        "Should have 3 tool calls recorded"
    );
    assert_eq!(data.tool_calls.len(), 3, "Should have 3 tool call details");

    // Verify first tool call
    let first_call = &data.tool_calls[0];
    assert_eq!(first_call.tool_name, "test_tool_1");
    assert_eq!(first_call.execution_time_ms, 150);
    assert!(matches!(
        first_call.result_status,
        reev_lib::agent::ToolResultStatus::Success
    ));

    // Verify second tool call
    let second_call = &data.tool_calls[1];
    assert_eq!(second_call.tool_name, "test_tool_2");
    assert_eq!(second_call.execution_time_ms, 200);
    assert!(matches!(
        second_call.result_status,
        reev_lib::agent::ToolResultStatus::Success
    ));

    // Verify third tool call (error case)
    let third_call = &data.tool_calls[2];
    assert_eq!(third_call.tool_name, "test_tool_3");
    assert_eq!(third_call.execution_time_ms, 75);
    assert!(matches!(
        third_call.result_status,
        reev_lib::agent::ToolResultStatus::Error
    ));
    assert_eq!(
        third_call.error_message,
        Some("Network timeout".to_string())
    );

    // Verify tool usage statistics
    assert_eq!(data.tool_usage.len(), 3);
    assert_eq!(data.tool_usage.get("test_tool_1"), Some(&1));
    assert_eq!(data.tool_usage.get("test_tool_2"), Some(&1));
    assert_eq!(data.tool_usage.get("test_tool_3"), Some(&1));

    println!("✅ Flow tracking integration test passed!");
    println!("📊 Total tool calls: {}", data.total_tool_calls);
    println!("🔧 Tool usage: {:?}", data.tool_usage);

    // Clean up
    env::remove_var("REEV_ENABLE_FLOW_LOGGING");
    env::remove_var("RUST_LOG");
}

#[tokio::test]
#[serial_test::serial]
async fn test_flow_tracking_disabled() {
    // Clean up any existing state first
    env::remove_var("REEV_ENABLE_FLOW_LOGGING");
    reev_agent::flow::GlobalFlowTracker::reset();

    // Ensure flow logging is disabled for this test
    env::set_var("REEV_ENABLE_FLOW_LOGGING", "false");

    // Record a tool call (should be ignored when logging is disabled)
    reev_agent::flow::GlobalFlowTracker::record_tool_call(
        reev_agent::flow::tracker::tool_wrapper::ToolCallParams {
            tool_name: "disabled_test_tool".to_string(),
            tool_args: r#"{"test": "data"}"#.to_string(),
            execution_time_ms: 100,
            result_status: reev_lib::agent::ToolResultStatus::Success,
            result_data: None,
            error_message: None,
            depth: 1,
        },
    );

    // Extract flow data
    let flow_data = reev_agent::flow::GlobalFlowTracker::get_flow_data();

    // When logging is disabled, the tracker should return None
    match flow_data {
        None => println!("✅ Flow tracking disabled test passed!"),
        Some(data) => {
            // If data exists, it should be empty when logging is disabled
            if data.total_tool_calls == 0 {
                println!("✅ Flow tracking disabled test passed! (empty data)");
            } else {
                panic!(
                    "Flow data should be empty when logging is disabled, but got {} tool calls",
                    data.total_tool_calls
                );
            }
        }
    }
}

#[tokio::test]
#[serial_test::serial]
async fn test_llm_response_with_flows() {
    // Clean up any existing state first
    env::remove_var("REEV_ENABLE_FLOW_LOGGING");
    reev_agent::flow::GlobalFlowTracker::reset();

    // Enable flow logging for this test
    env::set_var("REEV_ENABLE_FLOW_LOGGING", "1");

    // Simulate tool calls that might occur during LLM execution
    reev_agent::flow::GlobalFlowTracker::record_tool_call(reev_agent::flow::tracker::tool_wrapper::ToolCallParams {
        tool_name: "jupiter_swap".to_string(),
        tool_args: r#"{"user_pubkey": "USER_WALLET_PUBKEY", "input_mint": "So11111111111111111111111111111111111112", "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "amount": 10000000}"#.to_string(),
        execution_time_ms: 1250,
        result_status: reev_lib::agent::ToolResultStatus::Success,
        result_data: Some(serde_json::json!({
            "instruction_count": 3,
            "estimated_signatures": ["swap_tx_1_1760115000123456789"]
        })),
        error_message: None,
        depth: 1,
    });

    // Extract flow data and verify it matches expected format
    let flow_data = reev_agent::flow::GlobalFlowTracker::get_flow_data();
    assert!(
        flow_data.is_some(),
        "Flow data should be available for LLM test"
    );

    let data = flow_data.unwrap();

    // Verify the format matches what would be in LlmResponse.flows
    let serialized = serde_json::to_string(&data).unwrap();
    let parsed: reev_lib::agent::FlowData = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed.total_tool_calls, 1);
    assert_eq!(parsed.tool_calls[0].tool_name, "jupiter_swap");
    assert_eq!(parsed.tool_calls[0].execution_time_ms, 1250);

    println!("✅ LLM response with flows test passed!");
    println!("📄 Serialized flow data: {serialized}");

    // Clean up
    env::remove_var("REEV_ENABLE_FLOW_LOGGING");
}
