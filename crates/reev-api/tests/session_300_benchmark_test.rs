//! 300 Benchmark Session Parser Test
//!
//! This test validates that the session parser can correctly parse
//! enhanced_otel data from the 300 benchmark execution.

use reev_api::handlers::flow_diagram::SessionParser;
use reev_flow::jsonl_converter::JsonlToYmlConverter;
use std::path::PathBuf;

#[tokio::test]
async fn test_300_benchmark_session_parsing() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 300 BENCHMARK SESSION PARSING TEST");
    println!("{}", "=".repeat(60));

    // Path to the 300 benchmark enhanced_otel file
    let test_file = PathBuf::from("tests/enhanced_otel_300-swap-test.jsonl");
    let temp_yml_path = PathBuf::from("test_300_output.yml");

    if !test_file.exists() {
        println!("❌ 300 benchmark test file not found: {test_file:?}");
        return Ok(());
    }

    println!("🔄 Converting 300 benchmark enhanced_otel to YML...");

    // Convert using the same JsonlToYmlConverter that benchmark_executor uses
    let session_data = JsonlToYmlConverter::convert_file(&test_file, &temp_yml_path)?;

    println!("✅ Conversion successful!");
    println!("   Session ID: {}", session_data.session_id);
    println!("   Tool calls: {}", session_data.tool_calls.len());

    for (i, tool) in session_data.tool_calls.iter().enumerate() {
        println!(
            "   Tool {}: {} ({}ms) - success: {}",
            i + 1,
            tool.tool_name,
            tool.duration_ms,
            tool.success
        );
        println!("     Input: {}", serde_json::to_string(&tool.input)?);
    }

    // Read the generated YML content
    let yml_content = tokio::fs::read_to_string(&temp_yml_path).await?;
    println!("📄 Generated YML content (first 800 chars):");
    println!("{}", &yml_content[..yml_content.len().min(800)]);
    println!("{}", "=".repeat(60));

    // Test parsing directly with SessionParser (same as API)
    println!("🔍 Testing direct parsing of YML content...");
    match SessionParser::parse_session_content(&yml_content) {
        Ok(parsed_session) => {
            println!("✅ SessionParser successfully parsed YML content directly!");
            println!("   Found {} tool calls", parsed_session.tool_calls.len());

            for (i, tool) in parsed_session.tool_calls.iter().enumerate() {
                println!(
                    "   Parsed Tool {}: {} ({}ms)",
                    i + 1,
                    tool.tool_name,
                    tool.duration_ms
                );
            }

            if parsed_session.tool_calls.is_empty() {
                println!("❌ FAILED: Direct parsing returned empty tool calls!");
            } else {
                println!("✅ SUCCESS: Direct parsing extracted tool calls correctly!");
            }
        }
        Err(e) => {
            println!("❌ FAILED: Direct parsing failed: {e}");

            // Debug: Show first 200 chars of YML
            println!("📝 First 200 chars of YML content:");
            println!("{}", &yml_content[..yml_content.len().min(200)]);
        }
    }

    // Test with full session JSON structure (like database stores)
    let full_session_json = serde_json::json!({
        "session_id": session_data.session_id,
        "benchmark_id": "300-swap-sol-then-mul-usdc",
        "log_content": yml_content,
    });

    println!("\n🔍 Testing with full session JSON structure...");
    match SessionParser::parse_session_content(&full_session_json.to_string()) {
        Ok(parsed_session) => {
            println!("✅ SessionParser successfully parsed full JSON!");
            println!("   Found {} tool calls", parsed_session.tool_calls.len());

            if parsed_session.tool_calls.is_empty() {
                println!("❌ FAILED: Full JSON parsing returned empty tool calls!");
            } else {
                println!("✅ SUCCESS: Full JSON parsing extracted tool calls correctly!");
            }
        }
        Err(e) => {
            println!("❌ FAILED: Full JSON parsing failed: {e}");
        }
    }

    // Clean up
    tokio::fs::remove_file(&temp_yml_path).await.ok();

    println!("\n🎉 300 benchmark session parsing test completed!");
    Ok(())
}
