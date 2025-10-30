use anyhow::Result;
use reev_api::services::benchmark_executor::PooledBenchmarkExecutor;
// DatabaseWriterTrait not needed
use reev_db::{DatabaseConfig, PooledDatabaseWriter};
use reev_types::RunnerConfig;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Get the runner binary path, building if necessary
fn get_runner_binary() -> Result<String> {
    // Test runs from crates/reev-api, so we need to go to project root
    let binary_path = "../../target/debug/reev-runner";

    if !std::path::Path::new(binary_path).exists() {
        info!("🔨 Runner binary not found, building it...");
        let output = Command::new("cargo")
            .args(["build", "-p", "reev-runner"])
            .current_dir("../../")
            .output()?;

        if !output.status.success() {
            error!("Failed to build runner binary");
            error!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            anyhow::bail!("Binary build failed");
        }
    } else {
        info!("✅ Using existing runner binary: {}", binary_path);
    }

    if !std::path::Path::new(binary_path).exists() {
        anyhow::bail!("Binary not found at {}", binary_path);
    }

    Ok(binary_path.to_string())
}

/// Simple test to debug execute_cli_command function behavior
/// This test builds the binary first to avoid cargo watch hanging
#[tokio::test]
async fn test_simple_cli_command() -> Result<()> {
    // Only init tracing subscriber if not already set
    let _ = tracing_subscriber::fmt::try_init();

    info!("🔍 Testing execute_cli_command function with built binary");

    // Get the binary path (build if needed)
    let binary_path = get_runner_binary()?;

    // Setup database with unique path to avoid locking
    let test_db_path = format!(
        "test_db_simple_cli_{}.db",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let db_config = DatabaseConfig::new(&test_db_path);
    let db = PooledDatabaseWriter::new(db_config, 5).await?;

    // Create config with the built binary path
    let config = RunnerConfig {
        runner_binary_path: binary_path,
        working_directory: ".".to_string(),
        environment: HashMap::new(),
        default_timeout_seconds: 300,
        max_concurrent_executions: 1,
    };

    // Get executor with custom config
    let executor = PooledBenchmarkExecutor::new(
        Arc::new(db),
        config,
        reev_types::TimeoutConfig {
            default_timeout_seconds: 300,
            max_timeout_seconds: 600,
            status_check_timeout_seconds: 30,
        },
    );

    // Test the exact command that should work
    let args = vec![
        "benchmarks/001-sol-transfer.yml".to_string(),
        "--agent=glm-4.6-coding".to_string(),
        "--execution-id=debug-cli-test".to_string(),
    ];

    info!("🚀 Calling execute_cli_command with: {:?}", args);

    // Call execute_cli_command directly
    let result = executor.execute_cli_command(args, "debug-cli-test").await?;

    info!("📊 Result:");
    info!("  Exit Code: {:?}", result.exit_code);
    info!("  Timed Out: {}", result.timed_out);
    info!("  Duration: {}ms", result.duration_ms);
    info!("  Stdout length: {} chars", result.stdout.len());
    info!("  Stderr length: {} chars", result.stderr.len());

    if !result.stdout.is_empty() {
        info!(
            "  Stdout preview: {}",
            &result.stdout[..result.stdout.len().min(200)]
        );
    }

    if !result.stderr.is_empty() {
        info!(
            "  Stderr preview: {}",
            &result.stderr[..result.stderr.len().min(200)]
        );
    }

    // Check if process succeeded
    match result.exit_code {
        Some(0) => info!("✅ Process completed successfully"),
        Some(code) => warn!("⚠️ Process exited with code: {}", code),
        None => warn!("⚠️ Process exit code unknown"),
    }

    // Wait a bit for session file to be written
    sleep(Duration::from_millis(500)).await;

    // Check if session file was created
    let session_file = "logs/sessions/session_debug-cli-test.json";
    if std::path::Path::new(&session_file).exists() {
        info!("✅ Session file exists: {}", session_file);
        let content = std::fs::read_to_string(&session_file)?;
        info!("📁 Session file size: {} bytes", content.len());

        // Parse and verify basic structure
        if let Ok(session_data) = serde_json::from_str::<serde_json::Value>(&content) {
            info!("✅ Session file is valid JSON");
            if let Some(session_id) = session_data.get("session_id").and_then(|v| v.as_str()) {
                info!("📋 Session ID: {}", session_id);
            }
            if let Some(benchmark_id) = session_data.get("benchmark_id").and_then(|v| v.as_str()) {
                info!("📋 Benchmark ID: {}", benchmark_id);
            }
            if let Some(final_result) = session_data.get("final_result") {
                info!("📋 Final result present: {}", final_result);
            }
        } else {
            warn!("⚠️ Session file is not valid JSON");
        }
    } else {
        warn!("❌ Session file NOT found: {}", session_file);

        // Check if logs directory exists
        if std::path::Path::new("logs/sessions").exists() {
            info!("📁 logs/sessions directory exists");
            // List files in sessions directory
            if let Ok(entries) = std::fs::read_dir("logs/sessions") {
                for entry in entries.flatten() {
                    info!("📄 Found file: {}", entry.file_name().to_string_lossy());
                }
            }
        } else {
            warn!("📁 logs/sessions directory does NOT exist");
        }
    }

    // Check OTEL file as well
    let otel_file = "logs/sessions/enhanced_otel_debug-cli-test.jsonl";
    if std::path::Path::new(&otel_file).exists() {
        info!("✅ OTEL file exists: {}", otel_file);
        let content = std::fs::read_to_string(&otel_file)?;
        let line_count = content.lines().count();
        info!("📁 OTEL file lines: {}", line_count);
    } else {
        warn!("❌ OTEL file NOT found: {}", otel_file);
    }

    // The test passes if we got results and didn't timeout
    if result.timed_out {
        anyhow::bail!("Test failed: Process timed out");
    }

    // Cleanup test database file
    if std::path::Path::new(&test_db_path).exists() {
        std::fs::remove_file(&test_db_path)?;
        info!("🧹 Cleaned up test database file: {}", test_db_path);
    }

    info!("✅ CLI command test completed successfully");
    Ok(())
}

/// Test with a very simple command to verify basic process execution
#[tokio::test]
async fn test_simple_process_execution() -> Result<()> {
    // Only init tracing subscriber if not already set
    let _ = tracing_subscriber::fmt::try_init();

    info!("🔍 Testing basic process execution");

    // Get the binary path (build if needed)
    let binary_path = get_runner_binary()?;

    // Setup database with unique path to avoid locking
    let test_db_path = format!(
        "test_db_process_exec_{}.db",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let db_config = DatabaseConfig::new(&test_db_path);
    let db = PooledDatabaseWriter::new(db_config, 5).await?;

    // Create config with the built binary path
    let config = RunnerConfig {
        runner_binary_path: binary_path,
        working_directory: ".".to_string(),
        environment: HashMap::new(),
        default_timeout_seconds: 10, // Short timeout for this test
        max_concurrent_executions: 1,
    };

    let executor = PooledBenchmarkExecutor::new(
        Arc::new(db),
        config,
        reev_types::TimeoutConfig {
            default_timeout_seconds: 10, // Override for this test
            max_timeout_seconds: 30,
            status_check_timeout_seconds: 10,
        },
    );

    // Test with --help which should be quick
    let args = vec!["--help".to_string()];

    info!("🚀 Testing --help command");
    let result = executor.execute_cli_command(args, "help-test").await?;

    info!("📊 Help command result:");
    info!("  Exit Code: {:?}", result.exit_code);
    info!("  Timed Out: {}", result.timed_out);
    info!("  Duration: {}ms", result.duration_ms);

    if !result.stdout.is_empty() {
        info!("✅ Help output received ({} chars)", result.stdout.len());
        // Should contain usage information
        if result.stdout.contains("Usage:") || result.stdout.contains("USAGE") {
            info!("✅ Help output contains usage information");
        } else {
            warn!("⚠️ Help output doesn't contain expected usage information");
        }
    }

    if result.timed_out {
        anyhow::bail!("Help command timed out");
    }

    // Cleanup test database file
    if std::path::Path::new(&test_db_path).exists() {
        std::fs::remove_file(&test_db_path)?;
        info!("🧹 Cleaned up test database file: {}", test_db_path);
    }

    info!("✅ Basic process execution test passed");
    Ok(())
}
