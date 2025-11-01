use reev_flow::JsonlToYmlConverter;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let test_files_dir = current_dir.join("crates/reev-api/tests");
    let jsonl_path =
        test_files_dir.join("enhanced_otel_057d2e4a-f687-469f-8885-ad57759817c0.jsonl");
    let temp_yml_path = PathBuf::from("debug_conversion_output.yml");

    if !jsonl_path.exists() {
        println!("❌ enhanced_otel test file not found: {jsonl_path:?}");
        return Ok(());
    }

    println!("🔄 Converting enhanced_otel to YML...");

    // Convert JSONL to YML
    let session_data = JsonlToYmlConverter::convert_file(&jsonl_path, &temp_yml_path)?;

    println!("✅ Conversion successful!");
    println!("   Session ID: {}", session_data.session_id);
    println!("   Tool calls: {}", session_data.tool_calls.len());

    // Read YML content
    let yml_content = std::fs::read_to_string(&temp_yml_path)?;
    println!("📄 Full YML content:");
    println!("{}", yml_content);

    Ok(())
}
