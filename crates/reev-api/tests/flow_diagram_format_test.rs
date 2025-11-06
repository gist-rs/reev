//! Test to verify flow diagram format matches expected output for SOL transfer

use reev_api::handlers::flow_diagram::{
    session_parser::{ParsedSession, ParsedToolCall},
    state_diagram_generator::StateDiagramGenerator,
};
use serde_json::json;

#[tokio::test]
async fn test_sol_transfer_diagram_format() {
    // Create test session data matching 001-sol-transfer.yml execution
    let mut session = ParsedSession {
        session_id: "test-session-001".to_string(),
        benchmark_id: "001-sol-transfer".to_string(),
        prompt: Some("Please send 0.1 SOL to the recipient (RECIPIENT_WALLET_PUBKEY).".to_string()),
        tool_calls: vec![],
        start_time: 0,
        end_time: Some(0),
    };

    // Add sol_transfer tool call with real parameters
    let tool_call = ParsedToolCall {
        tool_name: reev_constants::SOL_TRANSFER.to_string(),
        start_time: 0,
        params: json!({
            "amount": 100000000, // 0.1 SOL in lamports
            "mint_address": null,
            "operation": "sol",
            "recipient_pubkey": "RECIPIENT_WALLET_PUBKEY",
            "user_pubkey": "6Mkfyk5CktTDvW1KrFkphXYUi55Foh1LxbRYT7UAxvx5"
        }),
        duration_ms: 1000,
        result_data: Some(json!({
            "success": true,
            "results": "[{\"program_id\":\"11111111111111111111111111111111\",\"accounts\":[{\"pubkey\":\"6Mkfyk5CktTDvW1KrFkphXYUi55Foh1LxbRYT7UAxvx5\",\"is_signer\":true,\"is_writable\":true},{\"pubkey\":\"896ux5MTCkgyarNo3n2YixQSRoLGzLNfpnNVManXrcgB\",\"is_signer\":false,\"is_writable\":true}],\"data\":\"3Bxs411Dtc7pkFQj\"}]",
            "instruction_count": 1
        })),
        tool_args: Some("{\"amount\":100000000,\"mint_address\":null,\"operation\":\"sol\",\"recipient_pubkey\":\"RECIPIENT_WALLET_PUBKEY\",\"user_pubkey\":\"6Mkfyk5CktTDvW1KrFkphXYUi55Foh1LxbRYT7UAxvx5\"}".to_string()),
    };

    session.tool_calls.push(tool_call);

    // Generate diagram
    let result =
        StateDiagramGenerator::generate_diagram(&session).expect("Failed to generate diagram");

    println!("🎯 Generated diagram:\n{}", result.diagram);

    // Check if diagram contains expected structure
    let _diagram_lines: Vec<&str> = result.diagram.lines().collect();

    // Verify basic structure
    assert!(
        result.diagram.contains("stateDiagram"),
        "Should contain stateDiagram declaration"
    );
    assert!(
        result.diagram.contains("[*] --> Prompt"),
        "Should contain initial transition"
    );
    assert!(
        result.diagram.contains("Prompt --> Agent"),
        "Should contain prompt transition"
    );
    assert!(
        result.diagram.contains("Agent --> sol_transfer"),
        "Should contain tool transition"
    );
    assert!(
        result.diagram.contains("sol_transfer --> [*]"),
        "Should contain final transition"
    );

    // Verify tool state is created properly
    assert!(
        result.diagram.contains("state sol_transfer {"),
        "Should contain nested tool state"
    );
    assert!(result.diagram.contains("}"), "Should close nested state");

    // Verify CSS classes
    assert!(
        result.diagram.contains("classDef tools fill:grey"),
        "Should contain tools CSS class definition"
    );
    assert!(
        result.diagram.contains("class sol_transfer tools"),
        "Should apply tools class to sol_transfer"
    );

    // Verify instruction count display
    assert!(
        result.diagram.contains("1 ix"),
        "Should show 1 ix for single instruction transfer"
    );

    // Check that program IDs are NOT shown in transition (current issue)
    assert!(
        !result
            .diagram
            .contains("program_11111111111111111111111111111111"),
        "Should NOT show program ID in transition, should use tool name instead"
    );

    println!("✅ Diagram format test passed!");
}

#[tokio::test]
async fn test_extract_sol_transfer_details() {
    // Test the specific extraction logic for SOL transfer
    let tool_call = ParsedToolCall {
        tool_name: reev_constants::SOL_TRANSFER.to_string(),
        start_time: 0,
        params: json!({
            "amount": 100000000,
            "user_pubkey": "GVKYhnPTY4JRQSCM7NjbHNb3VJduWfHFRroWhUSMTYg1",
            "recipient_pubkey": "MXnpbf2eNu8WGt4sGzKX7asFAtkBdnuLXaGCGT1SwKx",
            "operation": "sol"
        }),
        duration_ms: 1000,
        result_data: None,
        tool_args: None,
    };

    // Test that we can extract the transfer details properly
    // This will help debug why the current implementation shows program IDs instead of tool names
    println!(
        "🔍 Tool params: {}",
        serde_json::to_string_pretty(&tool_call.params).unwrap()
    );

    // The issue is likely in extract_tool_details function - it should look for user_pubkey/recipient_pubkey
    // instead of just 'from' and 'to' fields for sol_transfer tool

    assert_eq!(tool_call.tool_name, reev_constants::SOL_TRANSFER);
    assert!(
        tool_call.params.get("user_pubkey").is_some(),
        "Should have user_pubkey"
    );
    assert!(
        tool_call.params.get("recipient_pubkey").is_some(),
        "Should have recipient_pubkey"
    );
    assert!(
        tool_call.params.get("amount").is_some(),
        "Should have amount"
    );
}

#[test]
fn test_sol_transfer_parameter_extraction() {
    // Test the parameter extraction for SOL transfer specifically
    let params = json!({
        "amount": 100000000,
        "user_pubkey": "GVKYhnPTY4JRQSCM7NjbHNb3VJduWfHFRroWhUSMTYg1",
        "recipient_pubkey": "MXnpbf2eNu8WGt4sGzKX7asFAtkBdnuLXaGCGT1SwKx",
        "operation": "sol"
    });

    // Extract fields that should be used for transfer details
    let user_pubkey = params
        .get("user_pubkey")
        .and_then(|v| v.as_str())
        .unwrap_or("NOT_FOUND");
    let recipient_pubkey = params
        .get("recipient_pubkey")
        .and_then(|v| v.as_str())
        .unwrap_or("NOT_FOUND");
    let amount = params.get("amount").and_then(|v| v.as_u64()).unwrap_or(0);

    // Convert lamports to SOL
    let sol_amount = amount as f64 / 1_000_000_000.0;

    println!("🔍 Extracted SOL transfer details:");
    println!("  From: {user_pubkey}");
    println!("  To: {recipient_pubkey}");
    println!("  Amount: {sol_amount} SOL");

    assert_eq!(user_pubkey, "GVKYhnPTY4JRQSCM7NjbHNb3VJduWfHFRroWhUSMTYg1");
    assert_eq!(
        recipient_pubkey,
        "MXnpbf2eNu8WGt4sGzKX7asFAtkBdnuLXaGCGT1SwKx"
    );
    assert_eq!(amount, 100000000);
    assert_eq!(sol_amount, 0.1);
}
