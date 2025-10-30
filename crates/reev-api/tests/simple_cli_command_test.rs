use anyhow::Result;
use reev_api::services::benchmark_executor::PooledBenchmarkExecutor;
use reev_db::writer::DatabaseWriterTrait;
use reev_db::{DatabaseConfig, PooledDatabaseWriter};
use reev_types::{ExecutionRequest, RunnerConfig};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// Simple test to debug execute_cli_command function behavior
#[tokio::test]
async fn test_simple_cli_command() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("🔍 Testing execute_cli_command function directly");

    // Setup database
    let db_config = DatabaseConfig::new("sqlite::memory:");
    let db = PooledDatabaseWriter::new(db_config, 5).await?;

    // Get executor (this has access to execute_cli_command)
    let executor = PooledBenchmarkExecutor::new_with_default(Arc::new(db));

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
    info!(
        "  Stdout preview: {}",
        &result.stdout[..result.stdout.len().min(200)]
    );
    info!(
        "  Stderr preview: {}",
        &result.stderr[..result.stderr.len().min(200)]
    );

    // Check if session file was created after this
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    let session_file = format!("logs/sessions/session_debug-cli-test.json");
    if std::path::Path::new(&session_file).exists() {
        info!("✅ Session file exists: {}", session_file);
        let content = std::fs::read_to_string(&session_file)?;
        info!("📁 Session file size: {} bytes", content.len());
    } else {
        info!("❌ Session file NOT found: {}", session_file);
    }

    Ok(())
}
