use reev_flow::JsonlToYmlConverter;
use reev_db::{DatabaseWriter, DatabaseConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test enhanced_otel to YML conversion
    let jsonl_path = PathBuf::from("logs/sessions/enhanced_otel_0cd1d311-5de8-427d-a522-a1fe930258d6.jsonl");
    let temp_yml_path = PathBuf::from("test_conversion_output.yml");

    if !jsonl_path.exists() {
        println!("❌ enhanced_otel file not found: {:?}", jsonl_path);
        return Ok(());
    }

    println!("🔄 Converting enhanced_otel to YML...");

    // Convert JSONL to YML
    let session_data = JsonlToYmlConverter::convert_file(&jsonl_path, &temp_yml_path)?;

    println!("✅ Conversion successful!");
    println!("   Session ID: {}", session_data.session_id);
    println!("   Tool calls: {}", session_data.tool_calls.len());

    for (i, tool) in session_data.tool_calls.iter().enumerate() {
        println!("   Tool {}: {} ({}ms) - success: {}",
                 i+1, tool.tool_name, tool.duration_ms, tool.success);
    }

    // Read YML content
    let yml_content = tokio::fs::read_to_string(&temp_yml_path).await?;
    println!("📄 YML content (first 1000 chars):");
    println!("{}", &yml_content[..yml_content.len().min(1000)]);

    // Test database storage
    println!("\n🗄️ Testing database storage...");

    // Initialize database
    let config = DatabaseConfig::default();
    let db = DatabaseWriter::new(&config).await?;

    // Store session log
    if let Err(e) = db.store_complete_log(&session_data.session_id, &yml_content).await {
        println!("❌ Failed to store session log: {}", e);
    } else {
        println!("✅ Session log stored in database");
    }

    // Store tool calls
    for tool in &session_data.tool_calls {
        let tool_data = serde_json::json!({
            "tool_name": tool.tool_name,
            "start_time": tool.start_time,
            "end_time": tool.end_time,
            "duration_ms": tool.duration_ms,
            "input": tool.input,
            "output": tool.output,
            "success": tool.success,
            "error_message": tool.error_message
        });

        if let Err(e) = db.store_tool_call_consolidated(&reev_db::writer::sessions::ToolCallData {
            session_id: session_data.session_id.clone(),
            tool_name: tool.tool_name.clone(),
            start_time: tool.start_time,
            execution_time_ms: tool.duration_ms,
            input_params: tool.input.clone(),
            output_result: tool.output.clone(),
            status: if tool.success { "success".to_string() } else { "failed".to_string() },
            error_message: tool.error_message.clone(),
        }).await {
            println!("❌ Failed to store tool call {}: {}", tool.tool_name, e);
        } else {
            println!("✅ Tool call {} stored in database", tool.tool_name);
        }
    }

    // Test retrieval
    println!("\n🔍 Testing database retrieval...");

    if let Ok(log_content) = db.get_session_log(&session_data.session_id).await {
        println!("✅ Retrieved session log from database");
        println!("   Content length: {} chars", log_content.len());

        // Test if our parser can read it
        use reev_api::handlers::flow_diagram::SessionParser;
        match SessionParser::parse_session_content(&log_content) {
            Ok(parsed) => {
                println!("✅ Parser successfully read YML content");
                println!("   Found {} tool calls", parsed.tool_calls.len());
                for (i, tool) in parsed.tool_calls.iter().enumerate() {
                    println!("   Tool {}: {} ({}ms)", i+1, tool.tool_name, tool.duration_ms);
                }
            }
            Err(e) => {
                println!("❌ Parser failed to read YML content: {}", e);
            }
        }
    } else {
        println!("❌ Failed to retrieve session log from database");
    }

    // Clean up
    tokio::fs::remove_file(&temp_yml_path).await.ok();

    println!("\n🎉 Enhanced_otel conversion test completed!");
    Ok(())
}
