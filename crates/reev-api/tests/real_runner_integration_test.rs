use anyhow::Result;
use reev_api::services::benchmark_executor::PooledBenchmarkExecutor;
use reev_db::writer::DatabaseWriterTrait;
use reev_db::{DatabaseConfig, PooledDatabaseWriter};
use reev_types::{ExecutionRequest, ExecutionStatus, RunnerConfig};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

/// Real integration test for API → Runner → Database flow
///
/// This test verifies that:
/// 1. API can call real reev-runner binary
/// 2. Runner executes successfully and creates session files
/// 3. API reads session files and stores results in database
/// 4. Database operations work correctly (UPSERT fix validation)
/// 5. End-to-end status transitions work: Queued → Running → Completed
///
/// Note: This test requires reev-runner binary to exist. It will be skipped
/// in development environments where the binary hasn't been built yet.
#[tokio::test]
async fn test_real_runner_integration() -> Result<()> {
    // Skip test if runner binary doesn't exist (development environment)
    let runner_path = if cfg!(target_os = "windows") {
        "target/release/reev-runner.exe"
    } else {
        "target/release/reev-runner"
    };

    if !std::path::Path::new(runner_path).exists() {
        println!(
            "⚠️  Skipping real runner integration test - binary not found at: {}",
            runner_path
        );
        println!("💡 Build with: cargo build --release -p reev-runner");
        return Ok(());
    }
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting real runner integration test");

    // Verify release binary exists - use project root path for reliability
    let current_dir = std::env::current_dir()?;
    let project_root = if current_dir.ends_with("crates/reev-api") {
        current_dir.join("../..")
    } else {
        current_dir
    };
    let runner_path = project_root.join("target/release/reev-runner");
    assert!(
        runner_path.exists(),
        "Release binary should exist at {}",
        runner_path.display()
    );

    // Setup file-based database for test
    let test_db_path = format!(
        "test_db_integration_{}.db",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let db_config = DatabaseConfig::new(&test_db_path);
    let db = PooledDatabaseWriter::new(db_config, 5).await?;

    // Create benchmark executor with real runner config
    let _config = RunnerConfig {
        runner_binary_path: runner_path.to_string_lossy().to_string(), // Real binary!
        working_directory: project_root.to_string_lossy().to_string(),
        environment: HashMap::new(),
        default_timeout_seconds: 300,
        max_concurrent_executions: 1,
    };

    let executor = PooledBenchmarkExecutor::new_with_default(Arc::new(db.clone()));

    info!("📋 Creating real execution request");

    // Create execution request
    let execution_id = format!("integration-test-{}", uuid::Uuid::new_v4());
    let benchmark_id = "001-sol-transfer";
    let agent = "glm-4.6-coding"; // Use deterministic agent for reliable results

    let execution_request = ExecutionRequest {
        request_id: format!("req-{execution_id}"),
        execution_id: Some(execution_id.clone()),
        benchmark_path: project_root
            .join(format!("benchmarks/{benchmark_id}.yml"))
            .to_string_lossy()
            .to_string(),
        agent: agent.to_string(),
        priority: 1,
        timeout_seconds: 300,
        shared_surfpool: false,
        metadata: HashMap::new(),
    };

    info!(
        "🎯 Executing real benchmark: {} (execution_id: {})",
        benchmark_id, execution_id
    );

    // Verify benchmark file exists
    let benchmark_file_path = &execution_request.benchmark_path;
    assert!(
        std::path::Path::new(benchmark_file_path).exists(),
        "Benchmark file should exist at {benchmark_file_path}"
    );

    // QUICK DEBUG: Test session file reading logic directly
    info!("🔍 QUICK DEBUG: Testing session file reading logic");
    let session_id = &execution_id;
    let session_file = PathBuf::from(format!("logs/sessions/session_{session_id}.json"));
    info!("📁 Session file path: {:?}", session_file);
    info!("📁 Session file exists: {}", session_file.exists());

    // Check if session file exists and read it
    if session_file.exists() {
        let session_content = std::fs::read_to_string(&session_file)?;
        info!(
            "✅ Session file read successfully, length: {} bytes",
            session_content.len()
        );

        // Parse and verify session content
        let session_data: serde_json::Value = serde_json::from_str(&session_content)?;
        if let Some(success) = session_data["final_result"]["success"].as_bool() {
            info!("🎉 Session shows success: {}", success);
        }
        if let Some(score) = session_data["final_result"]["score"].as_f64() {
            info!("📊 Session shows score: {}", score);
        }
        return Ok(());
    } else {
        info!("❌ Session file does NOT exist yet");
    }

    // Store initial execution state (Queued)
    let execution_state = reev_types::ExecutionState::new(
        execution_id.clone(),
        benchmark_id.to_string(),
        agent.to_string(),
    );

    db.store_execution_state(&execution_state).await?;
    info!("✅ Stored initial execution state (Queued)");

    // Execute benchmark using real runner
    let result_execution_id = executor.execute_benchmark(execution_request).await?;
    assert_eq!(result_execution_id, execution_id);

    info!("✅ Real runner execution completed");

    // Give a moment for session file creation and database storage
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Verify final execution state in database
    let final_state = db.get_execution_state(&execution_id).await?;
    assert!(
        final_state.is_some(),
        "Should be able to retrieve final execution state"
    );

    let state = final_state.unwrap();
    info!("📊 Final execution state: {:?}", state);

    // Verify status transitions
    assert_eq!(state.status, ExecutionStatus::Completed);
    assert_eq!(state.execution_id, execution_id);
    assert_eq!(state.benchmark_id, benchmark_id);
    assert_eq!(state.agent, agent);

    // Verify execution results are stored
    assert!(
        state.result_data.is_some(),
        "Result data should be present after real execution"
    );

    if let Some(result_data) = &state.result_data {
        info!(
            "📋 Execution result: {}",
            serde_json::to_string_pretty(result_data)?
        );

        // Verify expected result structure
        assert!(
            result_data.get("success").is_some(),
            "Should have success field"
        );
        assert!(
            result_data.get("score").is_some(),
            "Should have score field"
        );
        assert!(
            result_data.get("status").is_some(),
            "Should have status field"
        );

        // Verify execution was successful
        if let Some(success) = result_data.get("success").and_then(|v| v.as_bool()) {
            assert!(success, "Real execution should be successful");
            info!(
                "✅ Real execution completed successfully with score: {:?}",
                result_data.get("score")
            );
        } else {
            anyhow::bail!("Real execution should have success=true");
        }
    }

    // Verify session file was created
    let session_file_path = format!("logs/sessions/session_{execution_id}.json");
    assert!(
        std::path::Path::new(&session_file_path).exists(),
        "Session file should be created at {session_file_path}"
    );
    info!("✅ Session file created: {}", session_file_path);

    // Verify OTEL file was created
    let otel_file_path = format!("logs/sessions/enhanced_otel_{execution_id}.jsonl");
    assert!(
        std::path::Path::new(&otel_file_path).exists(),
        "OTEL file should be created at {otel_file_path}"
    );
    info!("✅ OTEL file created: {}", otel_file_path);

    info!("🎉 Real runner integration test completed successfully!");

    // Cleanup test database file
    if std::path::Path::new(&test_db_path).exists() {
        std::fs::remove_file(&test_db_path)?;
        info!("🧹 Cleaned up test database file: {}", test_db_path);
    }

    Ok(())
}

// Test runner availability check
// #[tokio::test]
// async fn test_runner_availability() -> Result<()> {
//     info!("🔍 Testing runner availability");

//     let db_config = DatabaseConfig::new("sqlite::memory:");
//     let db = PooledDatabaseWriter::new(db_config, 5).await?;
//     let executor = PooledBenchmarkExecutor::new_with_default(Arc::new(db));

//     let is_available = executor.is_runner_available().await;
//     info!("Runner available: {}", is_available);

//     // Should be available since release binary exists
//     assert!(is_available, "Runner should be available");

//     info!("✅ Runner availability test passed");
//     Ok(())
// }
