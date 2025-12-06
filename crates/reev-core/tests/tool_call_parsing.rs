//! Tests for parsing tool calls from structured JSON responses

use reev_core::execution::rig_agent::types::StructuredLLMResponse;

#[test]
fn test_parse_tool_calls_from_structured_response() {
    // Test data with tool calls
    let response_json = r#"
    {
        "content": "I'll execute these swaps for you",
        "tool_calls": [
            {
                "name": "jupiter_swap",
                "parameters": {
                    "input_mint": "So11111111111111111111111111111111111111112",
                    "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    "input_amount": 1000000,
                    "slippage": 0.5
                }
            },
            {
                "name": "sol_transfer",
                "parameters": {
                    "recipient": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
                    "amount": 10000000
                }
            }
        ]
    }
    "#;

    // Parse the response using the structured type
    let parsed_response: Result<StructuredLLMResponse, _> = serde_json::from_str(response_json);
    assert!(
        parsed_response.is_ok(),
        "Failed to parse structured response"
    );

    let response = parsed_response.unwrap();
    assert_eq!(response.content, "I'll execute these swaps for you");

    // Verify tool calls were parsed correctly
    assert!(
        response.tool_calls.is_some(),
        "Expected tool_calls to be present"
    );
    let tool_calls = response.tool_calls.unwrap();
    assert_eq!(tool_calls.len(), 2, "Expected 2 tool calls");

    // Verify first tool call
    let first_call = &tool_calls[0];
    assert_eq!(first_call.name, "jupiter_swap");
    assert!(first_call.parameters.get("input_mint").is_some());
    assert!(first_call.parameters.get("output_mint").is_some());
    assert!(first_call.parameters.get("input_amount").is_some());
    assert!(first_call.parameters.get("slippage").is_some());

    // Verify second tool call
    let second_call = &tool_calls[1];
    assert_eq!(second_call.name, "sol_transfer");
    assert!(second_call.parameters.get("recipient").is_some());
    assert!(second_call.parameters.get("amount").is_some());
}

#[test]
fn test_parse_response_without_tool_calls() {
    // Test data without tool calls
    let response_json = r#"
    {
        "content": "I understand your request, but I cannot perform any operations at this time."
    }
    "#;

    // Parse the response using the structured type
    let parsed_response: Result<StructuredLLMResponse, _> = serde_json::from_str(response_json);
    assert!(
        parsed_response.is_ok(),
        "Failed to parse structured response"
    );

    let response = parsed_response.unwrap();
    assert_eq!(
        response.content,
        "I understand your request, but I cannot perform any operations at this time."
    );

    // Verify no tool calls
    assert!(
        response.tool_calls.is_none(),
        "Expected tool_calls to be None"
    );
}

#[test]
fn test_parse_response_with_empty_tool_calls() {
    // Test data with empty tool calls array
    let response_json = r#"
    {
        "content": "No operations needed",
        "tool_calls": []
    }
    "#;

    // Parse the response using the structured type
    let parsed_response: Result<StructuredLLMResponse, _> = serde_json::from_str(response_json);
    assert!(
        parsed_response.is_ok(),
        "Failed to parse structured response"
    );

    let response = parsed_response.unwrap();
    assert_eq!(response.content, "No operations needed");

    // Verify empty tool calls
    assert!(
        response.tool_calls.is_some(),
        "Expected tool_calls to be Some"
    );
    let tool_calls = response.tool_calls.unwrap();
    assert_eq!(tool_calls.len(), 0, "Expected 0 tool calls");
}
