use anyhow::Result;
use reev_api::services::benchmark_executor::{BenchmarkExecutor, RunnerConfig, TimeoutConfig};
use reev_db::writer::{DatabaseWriterTrait, PooledDatabaseWriter};
use reev_db::config::DatabaseConfig;
use reev_types::{ExecutionRequest, ExecutionState, ExecutionStatus, RunnerConfig as ConfigType};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs;
use tracing::{info, debug};
use uuid::Uuid;

/// Mock test for API flow using real session data
/// This test uses existing successful execution data to verify API state management
/// without running the actual CLI runner, making tests much faster and more reliable.
#[tokio::test]
async fn test_api_flow_with_mock_session_data() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🧪 Starting API mock test with real session data");

    // Setup test database
    let db_config = DatabaseConfig::new("test_api_mock.db");
    let db = PooledDatabaseWriter::new(db_config, 5).await?;

    // Create benchmark executor with test config
    let config = RunnerConfig {
        runner_binary_path: "mock-runner".to_string(), // Won't actually run
        working_directory: ".".to_string(),
        environment: HashMap::new(),
        default_timeout_seconds: 300,
        max_concurrent_executions: 1,
    };

    let timeout_config = TimeoutConfig {
        session_creation_timeout_seconds: 30,
        session_completion_timeout_seconds: 300,
    };

    let executor = BenchmarkExecutor::new(Arc::new(db.clone()), config, timeout_config);

    // Test execution ID from our real data
    let execution_id = "057d2e4a-f687-469f-8885-ad57759817c0";
    let benchmark_id = "001-sol-transfer";
    let agent = "glm-4.6";

    info!("📋 Using real session data: execution_id={}, agent={}", execution_id, agent);

    // Verify the mock session file exists
    let session_file_path = format!("tests/session_{}.json", execution_id);
    assert!(fs::metadata(&session_file_path).await.is_ok(),
           "Mock session file should exist at {}", session_file_path);

    // Create execution state
    let execution_state = ExecutionState::new(
        execution_id.to_string(),
        benchmark_id.to_string(),
        agent.to_string(),
        HashMap::new(),
    );

    // Mock the session file reading process
    info!("📖 Reading mock session file...");
    let session_content = fs::read_to_string(&session_file_path).await?;
    debug!("Session content: {}", &session_content[..200.min(session_content.len())]);

    // Parse session data like the real executor would
    let session_data: serde_json::Value = serde_json::from_str(&session_content)?;

    // Verify the session data contains expected fields
    assert_eq!(session_data["session_id"], execution_id);
    assert_eq!(session_data["benchmark_id"], benchmark_id);
    assert_eq!(session_data["agent_type"], agent);

    // Verify successful execution result
    let final_result = &session_data["final_result"];
    assert_eq!(final_result["success"], true);
    assert_eq!(final_result["score"], 1.0); // Perfect score!
    assert_eq!(final_result["status"], "Succeeded");

    info!("✅ Session data validation passed");

    // Test database storage of execution state
    info!("💾 Testing database storage...");

    // Create execution request like the real API would
    let execution_request = ExecutionRequest {
        execution_id: execution_id.to_string(),
        benchmark_id: benchmark_id.to_string(),
        agent: agent.to_string(),
        config: None,
        interface: "web".to_string(),
        created_at: chrono::Utc::now(),
    };

    // Store in database like the real executor would
    db.store_execution_request(&execution_request).await?;

    // Update execution state with session results
    let updated_state = ExecutionState {
        execution_id: execution_id.to_string(),
        benchmark_id: benchmark_id.to_string(),
        agent: agent.to_string(),
        status: ExecutionStatus::Completed,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        progress: None,
        error_message: None,
        result_data: Some(session_data["final_result"].clone()),
        metadata: HashMap::new(),
    };

    db.update_execution_state(&updated_state).await?;

    info!("✅ Database storage successful");

    // Test retrieval to verify the data was stored correctly
    info!("🔍 Testing database retrieval...");

    let retrieved_state = db.get_execution_state(execution_id).await?;
    assert!(retrieved_state.is_some(), "Should be able to retrieve stored execution state");

    let state = retrieved_state.unwrap();
    assert_eq!(state.execution_id, execution_id);
    assert_eq!(state.benchmark_id, benchmark_id);
    assert_eq!(state.agent, agent);
    assert_eq!(state.status, ExecutionStatus::Completed);

    if let Some(result_data) = &state.result_data {
        assert_eq!(result_data["success"], true);
        assert_eq!(result_data["score"], 1.0);
        info!("✅ Result data verification passed");
    } else {
        anyhow::bail!("Result data should be present");
    }

    info!("🎉 API mock test completed successfully!");

    // Cleanup test database
    if let Err(e) = fs::remove_file("test_api_mock.db").await {
        debug!("Cleanup warning: {}", e);
    }

    Ok(())
}

/// Test the specific session file parsing logic that the benchmark executor uses
#[tokio::test]
async fn test_session_file_parsing() -> Result<()> {
    info!("🧪 Testing session file parsing logic");

    let execution_id = "057d2e4a-f687-469f-8885-ad57759817c0";
    let session_file_path = format!("tests/session_{}.json", execution_id);

    // Read and parse the session file
    let session_content = fs::read_to_string(&session_file_path).await?;
    let session_data: serde_json::Value = serde_json::from_str(&session_content)?;

    // Verify all expected fields are present and have correct values
    assert_eq!(session_data["session_id"], execution_id);
    assert_eq!(session_data["benchmark_id"], "001-sol-transfer");
    assert_eq!(session_data["agent_type"], "glm-4.6");

    let final_result = &session_data["final_result"];
    assert!(final_result["success"].as_bool().unwrap_or(false));
    assert_eq!(final_result["score"].as_f64().unwrap_or(0.0), 1.0);
    assert_eq!(final_result["status"].as_str().unwrap_or(""), "Succeeded");

    // Verify execution time is reasonable
    if let Some(execution_time_ms) = final_result["execution_time_ms"].as_u64() {
        assert!(execution_time_ms > 0, "Execution time should be positive");
        assert!(execution_time_ms < 300_000, "Execution time should be under 5 minutes");
    }

    // Verify there are steps in the result
    let steps = final_result["data"]["steps"].as_array().unwrap_or(&vec![]);
    assert!(!steps.is_empty(), "Should have at least one execution step");

    info!("✅ Session file parsing verification passed");

    Ok(())
}

/// Test OTEL file verification
#[tokio::test]
async fn test_otel_file_verification() -> Result<()> {
    info!("🧪 Testing OTEL file verification");

    let execution_id = "057d2e4a-f687-469f-8885-ad57759817c0";
    let otel_file_path = format!("tests/enhanced_otel_{}.jsonl", execution_id);

    // Verify OTEL file exists
    assert!(fs::metadata(&otel_file_path).await.is_ok(),
           "Enhanced OTEL file should exist at {}", otel_file_path);

    // Read and verify OTEL content
    let otel_content = fs::read_to_string(&otel_file_path).await?;
    assert!(!otel_content.is_empty(), "OTEL file should not be empty");

    // Count number of OTEL events
    let otel_lines: Vec<&str> = otel_content.lines().collect();
    assert!(otel_lines.len() > 0, "Should have OTEL events");

    info!("✅ OTEL file verification passed - found {} events", otel_lines.len());

    Ok(())
}
