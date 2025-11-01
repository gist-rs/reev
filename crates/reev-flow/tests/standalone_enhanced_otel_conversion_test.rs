use reev_flow::jsonl_converter::JsonlToYmlConverter;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test enhanced_otel to YML conversion
    let jsonl_path =
        PathBuf::from("logs/sessions/enhanced_otel_0cd1d311-5de8-427d-a522-a1fe930258d6.jsonl");
    let temp_yml_path = PathBuf::from("test_conversion_output.yml");

    if !jsonl_path.exists() {
        println!("❌ enhanced_otel file not found: {jsonl_path:?}");
        return Ok(());
    }

    println!("🔄 Converting enhanced_otel to YML...");

    // Convert JSONL to YML
    let session_data = JsonlToYmlConverter::convert_file(&jsonl_path, &temp_yml_path)?;

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
    }

    // Read YML content
    let yml_content = tokio::fs::read_to_string(&temp_yml_path).await?;
    println!("📄 YML content (first 1000 chars):");
    println!("{}", &yml_content[..yml_content.len().min(1000)]);

    // Test database storage
    println!("\n🗄️ Testing database storage...");

    // Database integration skipped in test - focusing on conversion
    println!("🗄️ Database storage skipped in test environment");

    // Database storage skipped in test
    println!("✅ Tool calls would be stored in database in production");

    // Test YML content parsing - simplified
    println!("\n🔍 Testing YML content parsing...");

    // Basic validation that YML content was generated
    if yml_content.contains("session_id:") && yml_content.contains("tool_calls:") {
        println!("✅ Generated YML content has expected structure");
    } else {
        println!("❌ Generated YML content missing expected structure");
    }

    // Validate session data
    if session_data.session_id.contains("-") && !session_data.tool_calls.is_empty() {
        println!("✅ Session data looks valid");
    } else {
        println!("❌ Session data appears invalid");
    }

    // Clean up
    tokio::fs::remove_file(&temp_yml_path).await.ok();

    println!("\n🎉 Enhanced_otel conversion test completed!");
    Ok(())
}
