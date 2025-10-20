use reev_lib::llm_agent::LlmAgent;
use std::time::SystemTime;

#[tokio::test]
async fn test_tool_call_extraction_from_response() {
    // Create a test agent
    let mut agent = LlmAgent::new("test-agent").expect("Failed to create agent");

    // Test 1: Jupiter swap response
    let jupiter_response = r#"
    {
        "transactions": [
            {
                "program_id": "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
                "accounts": [...],
                "data": "..."
            }
        ],
        "summary": "Successfully swapped 0.1 SOL for USDC using Jupiter"
    }
    "#;

    let response_start = SystemTime::now();
    let result = agent.extract_tool_calls_from_response(jupiter_response, response_start).await;
    assert!(result.is_ok(), "Tool call extraction should succeed");

    let tool_calls = agent.get_tool_calls();
    assert!(!tool_calls.is_empty(), "Should detect Jupiter swap tool call");
    assert_eq!(tool_calls[0].params["tool_name"], "jupiter_swap");
}

#[tokio::test]
async fn test_tool_call_extraction_from_text() {
    let mut agent = LlmAgent::new("test-agent").expect("Failed to create agent");

    // Test 2: Transfer operation in text
    let transfer_response = "Please send 0.1 SOL to the recipient. Transfer initiated successfully.";

    let response_start = SystemTime::now();
    let result = agent.extract_tool_calls_from_response(transfer_response, response_start).await;
    assert!(result.is_ok(), "Tool call extraction should succeed");

    let tool_calls = agent.get_tool_calls();
    assert!(!tool_calls.is_empty(), "Should detect transfer tool call");
    assert_eq!(tool_calls[0].params["tool_name"], "transfer_tokens");
}

#[tokio::test]
async fn test_tool_call_extraction_flow_response() {
    let mut agent = LlmAgent::new("test-agent").expect("Failed to create agent");

    // Test 3: Flow response
    let flow_response = r#"
    {
        "flow_completed": true,
        "steps": [
            {
                "step": 1,
                "action": "get_quote",
                "status": "success"
            }
        ]
    }
    "#;

    let response_start = SystemTime::now();
    let result = agent.extract_tool_calls_from_response(flow_response, response_start).await;
    assert!(result.is_ok(), "Tool call extraction should succeed");

    let tool_calls = agent.get_tool_calls();
    assert!(!tool_calls.is_empty(), "Should detect flow execution tool call");
    assert_eq!(tool_calls[0].params["tool_name"], "execute_flow");
}

#[tokio::test]
async fn test_tool_call_extraction_no_tools() {
    let mut agent = LlmAgent::new("test-agent").expect("Failed to create agent");

    // Test 4: Response with no tool calls
    let empty_response = "Hello, how can I help you today?";

    let response_start = SystemTime::now();
    let result = agent.extract_tool_calls_from_response(empty_response, response_start).await;
    assert!(result.is_ok(), "Tool call extraction should succeed");

    let tool_calls = agent.get_tool_calls();
    assert!(tool_calls.is_empty(), "Should not detect any tool calls");
}
