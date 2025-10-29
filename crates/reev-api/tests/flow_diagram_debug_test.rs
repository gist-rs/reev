//! Focused test to debug flow diagram rendering issues with mock data

use chrono::DateTime;
use std::fs;

#[test]
fn test_jsonl_parsing_safety() {
    // Load the mock JSONL data
    let mock_data = include_str!("enhanced_otel_93aebfa7-cf08-4793-bc0e-7a8ef4cdddaa.jsonl");

    println!("🔍 Mock data loaded: {} lines", mock_data.lines().count());

    // Parse the mock data line by line to identify any issues
    for (i, line) in mock_data.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        println!("🔍 Testing line {}: {} chars", i + 1, line.len());

        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(event) => {
                let event_type = event["event_type"].as_str().unwrap_or("unknown");
                println!("✅ Line {}: event_type = {}", i + 1, event_type);

                // Test timestamp extraction specifically
                if let Some(timestamp) = event.get("timestamp").and_then(|v| v.as_str()) {
                    println!("  🕐 Timestamp: {}", timestamp);

                    // Test our timestamp parsing logic from flows.rs
                    match DateTime::parse_from_rfc3339(timestamp) {
                        Ok(dt) => {
                            let unix_ts = dt.timestamp() as u64;
                            println!("  ✅ Parsed to Unix: {}", unix_ts);
                        }
                        Err(e) => {
                            println!("  ❌ Timestamp parse failed: {}", e);
                        }
                    }
                }

                // Check for large payloads that might cause memory issues
                let line_size = line.len();
                if line_size > 100_000 {
                    println!("  ⚠️  Large line detected: {} bytes", line_size);
                }
            }
            Err(e) => {
                println!("❌ Line {} failed: {}", i + 1, e);
                panic!("Failed to parse line {} individually", i + 1);
            }
        }
    }
}

#[test]
fn test_memory_usage_analysis() {
    // Analyze the mock data for memory issues
    let mock_data = include_str!("enhanced_otel_93aebfa7-cf08-4793-bc0e-7a8ef4cdddaa.jsonl");

    println!("📊 Memory Analysis:");
    println!("  Total data size: {} bytes", mock_data.len());
    println!("  Total lines: {}", mock_data.lines().count());

    let mut max_line_size = 0;
    let mut total_size = 0;
    let mut large_lines = Vec::new();

    for (i, line) in mock_data.lines().enumerate() {
        let line_size = line.len();
        total_size += line_size;

        if line_size > max_line_size {
            max_line_size = line_size;
        }

        if line_size > 5000 {
            large_lines.push((i + 1, line_size));
        }
    }

    println!("  Max line size: {} bytes", max_line_size);
    println!(
        "  Average line size: {} bytes",
        total_size / mock_data.lines().count()
    );

    if !large_lines.is_empty() {
        println!("  ⚠️  Large lines detected:");
        for (line_num, size) in large_lines {
            println!("    Line {}: {} bytes", line_num, size);
        }
    }

    // Estimate memory usage for multiple sessions
    let estimated_per_session = total_size * 2; // Rough estimate
    let max_concurrent_sessions = 100_000_000 / estimated_per_session; // 100MB limit

    println!(
        "  Estimated memory per session: ~{} bytes",
        estimated_per_session
    );
    println!(
        "  Max concurrent sessions (100MB limit): ~{}",
        max_concurrent_sessions
    );

    if estimated_per_session > 10_000_000 {
        println!("  🚨 CRITICAL: Single session uses >10MB - this will cause crashes!");
    }
}
