//! Tool Call Consolidation Test
//!
//! Test the database consolidation logic for handling duplicate tool call entries.

use reev_db::writer::sessions::ToolCallData;
use reev_db::{DatabaseConfig, DatabaseWriter};
use serde_json::json;

/// Test consolidation of duplicate sol_transfer tool calls
#[tokio::test]
async fn test_sol_transfer_consolidation() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize test database
    let config = DatabaseConfig::new(":memory:");
    let db = DatabaseWriter::new(config).await?;

    let session_id = "test-session-123";
    let tool_name = "sol_transfer";
    let start_time = 1761359959;

    // Create first tool call entry (initial call with input params, empty output)
    let initial_call = ToolCallData {
        session_id: session_id.to_string(),
        tool_name: tool_name.to_string(),
        start_time,
        execution_time_ms: 0,
        input_params: json!({
            "amount": 100000000,
            "mint_address": None::<String>,
            "operation": "sol",
            "recipient_pubkey": "Df2x6LiqRXEgQvXxiWNE668tBiFfhSecYHNFuC4oB265",
            "user_pubkey": "5Q1NPFhnupvj52dAZjkWkMr4uvRMu7heuq9GBmoLJm6C"
        }),
        output_result: json!({}),
        status: "success".to_string(),
        error_message: None,
        metadata: Some(json!({})),
    };

    // Create second tool call entry (completion with empty input, actual output)
    let completion_call = ToolCallData {
        session_id: session_id.to_string(),
        tool_name: tool_name.to_string(),
        start_time,
        execution_time_ms: 0,
        input_params: json!({}),
        output_result: json!([{
            "program_id": "11111111111111111111111111111111",
            "accounts": [
                {
                    "pubkey": "5Q1NPFhnupvj52dAZjkWkMr4uvRMu7heuq9GBmoLJm6C",
                    "is_signer": true,
                    "is_writable": true
                },
                {
                    "pubkey": "Df2x6LiqRXEgQvXxiWNE668tBiFfhSecYHNFuC4oB265",
                    "is_signer": false,
                    "is_writable": true
                }
            ],
            "data": "3Bxs411Dtc7pkFQj"
        }]),
        status: "success".to_string(),
        error_message: None,
        metadata: Some(json!({})),
    };

    // Store both calls using consolidation logic
    db.store_tool_call_consolidated(&initial_call).await?;
    db.store_tool_call_consolidated(&completion_call).await?;

    // Verify we only have one consolidated entry
    let tool_calls = db.get_session_tool_calls(session_id).await?;
    assert_eq!(
        tool_calls.len(),
        1,
        "Expected 1 consolidated tool call, got {}",
        tool_calls.len()
    );

    let consolidated = &tool_calls[0];
    assert_eq!(consolidated.session_id, session_id);
    assert_eq!(consolidated.tool_name, tool_name);
    assert_eq!(consolidated.start_time, start_time);

    // Verify input params are from the first call
    assert!(!consolidated.input_params.as_object().unwrap().is_empty());
    assert_eq!(consolidated.input_params["amount"], json!(100000000));
    assert_eq!(
        consolidated.input_params["recipient_pubkey"],
        json!("Df2x6LiqRXEgQvXxiWNE668tBiFfhSecYHNFuC4oB265")
    );

    // Verify output result is from the second call
    if let Some(output_array) = consolidated.output_result.as_array() {
        assert!(!output_array.is_empty());
        assert_eq!(output_array.len(), 1);
        assert_eq!(
            output_array[0]["program_id"],
            json!("11111111111111111111111111111111")
        );
    } else {
        panic!(
            "Expected output_result to be an array, got: {}",
            consolidated.output_result
        );
    }

    println!("✅ Sol transfer consolidation test passed");
    Ok(())
}

/// Test consolidation with different execution times (should prefer non-zero)
#[tokio::test]
async fn test_consolidation_with_execution_time() -> Result<(), Box<dyn std::error::Error>> {
    let config = DatabaseConfig::new(":memory:");
    let db = DatabaseWriter::new(config).await?;

    let session_id = "test-execution-time";
    let tool_name = "test_tool";
    let start_time = 1761359959;

    // First call with 0ms execution time
    let zero_time_call = ToolCallData {
        session_id: session_id.to_string(),
        tool_name: tool_name.to_string(),
        start_time,
        execution_time_ms: 0,
        input_params: json!({"initial": "params"}),
        output_result: json!({}),
        status: "success".to_string(),
        error_message: None,
        metadata: Some(json!({})),
    };

    // Second call with actual execution time
    let real_time_call = ToolCallData {
        session_id: session_id.to_string(),
        tool_name: tool_name.to_string(),
        start_time,
        execution_time_ms: 150,
        input_params: json!({}),
        output_result: json!({"result": "success"}),
        status: "success".to_string(),
        error_message: None,
        metadata: Some(json!({})),
    };

    db.store_tool_call_consolidated(&zero_time_call).await?;
    db.store_tool_call_consolidated(&real_time_call).await?;

    let tool_calls = db.get_session_tool_calls(session_id).await?;
    assert_eq!(tool_calls.len(), 1);

    let consolidated = &tool_calls[0];
    assert_eq!(
        consolidated.execution_time_ms, 150,
        "Should prefer non-zero execution time"
    );
    assert_eq!(consolidated.input_params["initial"], json!("params"));
    assert_eq!(consolidated.output_result["result"], json!("success"));

    println!("✅ Execution time consolidation test passed");
    Ok(())
}

/// Test that different tools or times are not consolidated
#[tokio::test]
async fn test_no_consolidation_different_tools() -> Result<(), Box<dyn std::error::Error>> {
    let config = DatabaseConfig::new(":memory:");
    let db = DatabaseWriter::new(config).await?;

    let session_id = "test-different-tools";

    // Two different tool calls should not be consolidated
    let sol_call = ToolCallData {
        session_id: session_id.to_string(),
        tool_name: "sol_transfer".to_string(),
        start_time: 1761359959,
        execution_time_ms: 100,
        input_params: json!({"amount": 100}),
        output_result: json!({"result": "sol_success"}),
        status: "success".to_string(),
        error_message: None,
        metadata: Some(json!({})),
    };

    let swap_call = ToolCallData {
        session_id: session_id.to_string(),
        tool_name: "jupiter_swap".to_string(),
        start_time: 1761359959,
        execution_time_ms: 200,
        input_params: json!({"input_mint": "So11111111111111111111111111111111111111112"}),
        output_result: json!({"result": "swap_success"}),
        status: "success".to_string(),
        error_message: None,
        metadata: Some(json!({})),
    };

    db.store_tool_call_consolidated(&sol_call).await?;
    db.store_tool_call_consolidated(&swap_call).await?;

    let tool_calls = db.get_session_tool_calls(session_id).await?;
    assert_eq!(
        tool_calls.len(),
        2,
        "Different tools should not be consolidated"
    );

    // Verify both calls are present
    let tool_names: Vec<&str> = tool_calls.iter().map(|t| t.tool_name.as_str()).collect();
    assert!(tool_names.contains(&"sol_transfer"));
    assert!(tool_names.contains(&"jupiter_swap"));

    println!("✅ Different tools no consolidation test passed");
    Ok(())
}

/// Test time window consolidation (within 1 second)
#[tokio::test]
async fn test_consolidation_time_window() -> Result<(), Box<dyn std::error::Error>> {
    let config = DatabaseConfig::new(":memory:");
    let db = DatabaseWriter::new(config).await?;

    let session_id = "test-time-window";
    let tool_name = "timed_tool";

    // First call at timestamp 1000
    let first_call = ToolCallData {
        session_id: session_id.to_string(),
        tool_name: tool_name.to_string(),
        start_time: 1000,
        execution_time_ms: 0,
        input_params: json!({"step": 1}),
        output_result: json!({}),
        status: "success".to_string(),
        error_message: None,
        metadata: Some(json!({})),
    };

    // Second call at timestamp 1000.5 (within 1 second window)
    let second_call = ToolCallData {
        session_id: session_id.to_string(),
        tool_name: tool_name.to_string(),
        start_time: 1000, // Same second as first call
        execution_time_ms: 50,
        input_params: json!({}),
        output_result: json!({"final": "result"}),
        status: "success".to_string(),
        error_message: None,
        metadata: Some(json!({})),
    };

    db.store_tool_call_consolidated(&first_call).await?;
    db.store_tool_call_consolidated(&second_call).await?;

    let tool_calls = db.get_session_tool_calls(session_id).await?;
    assert_eq!(
        tool_calls.len(),
        1,
        "Calls within 1-second window should be consolidated"
    );

    let consolidated = &tool_calls[0];
    assert_eq!(consolidated.execution_time_ms, 50);
    assert_eq!(consolidated.input_params["step"], json!(1));
    assert_eq!(consolidated.output_result["final"], json!("result"));

    println!("✅ Time window consolidation test passed");
    Ok(())
}

/// Test that calls outside time window are not consolidated
#[tokio::test]
async fn test_no_consolidation_outside_time_window() -> Result<(), Box<dyn std::error::Error>> {
    let config = DatabaseConfig::new(":memory:");
    let db = DatabaseWriter::new(config).await?;

    let session_id = "test-outside-window";
    let tool_name = "separated_tool";

    // First call at timestamp 1000
    let early_call = ToolCallData {
        session_id: session_id.to_string(),
        tool_name: tool_name.to_string(),
        start_time: 1000,
        execution_time_ms: 25,
        input_params: json!({"call": 1}),
        output_result: json!({"result": "early"}),
        status: "success".to_string(),
        error_message: None,
        metadata: Some(json!({})),
    };

    // Second call at timestamp 1002 (outside 1-second window)
    let late_call = ToolCallData {
        session_id: session_id.to_string(),
        tool_name: tool_name.to_string(),
        start_time: 1002, // More than 1 second difference
        execution_time_ms: 75,
        input_params: json!({"call": 2}),
        output_result: json!({"result": "late"}),
        status: "success".to_string(),
        error_message: None,
        metadata: Some(json!({})),
    };

    db.store_tool_call_consolidated(&early_call).await?;
    db.store_tool_call_consolidated(&late_call).await?;

    let tool_calls = db.get_session_tool_calls(session_id).await?;
    assert_eq!(
        tool_calls.len(),
        2,
        "Calls outside 1-second window should not be consolidated"
    );

    println!("✅ Outside time window no consolidation test passed");
    Ok(())
}
