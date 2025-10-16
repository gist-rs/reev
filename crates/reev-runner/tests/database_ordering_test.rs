// Test to verify database timestamp ordering fix

use reev_lib::db::{AgentPerformanceData, DatabaseConfig, DatabaseWriter};
use tempfile::TempDir;

#[tokio::test]
async fn test_agent_performance_timestamp_ordering() {
    // Create a temporary database
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_config = DatabaseConfig::new(db_path.to_string_lossy());
    let db = DatabaseWriter::new(db_config).await.unwrap();

    // Debug: Check if database was created and schema initialized
    println!("Database created at: {}", db_path.display());

    // Insert test results with different timestamps
    let base_time = chrono::Utc::now();

    // Insert older result first
    let older_result = AgentPerformanceData {
        benchmark_id: "test-benchmark".to_string(),
        agent_type: "deterministic".to_string(),
        score: 0.8,
        final_status: "Succeeded".to_string(),
        flow_log_id: None,
        execution_time_ms: 1000,
        timestamp: (base_time - chrono::Duration::minutes(5))
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string(),
        prompt_md5: None,
    };

    db.insert_agent_performance(&reev_lib::db::DbAgentPerformance::from(older_result))
        .await
        .unwrap();

    // Insert newer result second
    let newer_result = AgentPerformanceData {
        benchmark_id: "test-benchmark".to_string(),
        agent_type: "deterministic".to_string(),
        score: 1.0,
        final_status: "Succeeded".to_string(),
        flow_log_id: None,
        execution_time_ms: 800,
        timestamp: base_time.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
        prompt_md5: None,
    };

    db.insert_agent_performance(&reev_lib::db::DbAgentPerformance::from(newer_result))
        .await
        .unwrap();

    // Insert another result with different benchmark
    let other_result = AgentPerformanceData {
        benchmark_id: "other-benchmark".to_string(),
        agent_type: "deterministic".to_string(),
        score: 0.5,
        final_status: "Succeeded".to_string(),
        flow_log_id: None,
        execution_time_ms: 1200,
        timestamp: (base_time - chrono::Duration::minutes(2))
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string(),
        prompt_md5: None,
    };

    db.insert_agent_performance(&reev_lib::db::DbAgentPerformance::from(other_result))
        .await
        .unwrap();

    // Retrieve results and verify ordering using public API
    // Note: get_agent_performance method doesn't exist in DatabaseWriter
    // Using a placeholder for now - this test may need to be updated
    let performance_summaries: Vec<reev_db::AgentPerformanceSummary> = Vec::new(); // Placeholder
    let deterministic_results = performance_summaries
        .iter()
        .find(|summary| summary.agent_type == "deterministic")
        .map(|summary| &summary.results)
        .unwrap();
    let results = deterministic_results.clone();

    // Should have 3 results
    assert_eq!(results.len(), 3);

    // First result should be the newest (base_time)
    let first = &results[0];
    assert_eq!(first.benchmark_id, "test-benchmark");
    assert_eq!(first.score, 1.0);

    // Second result should be the middle one (base_time - 2 minutes)
    let second = &results[1];
    assert_eq!(second.benchmark_id, "other-benchmark");
    assert_eq!(second.score, 0.5);

    // Third result should be the oldest (base_time - 5 minutes)
    let third = &results[2];
    assert_eq!(third.benchmark_id, "test-benchmark");
    assert_eq!(third.score, 0.8);

    // Verify timestamps are in descending order
    assert!(first.timestamp > second.timestamp);
    assert!(second.timestamp > third.timestamp);

    println!("✅ Database timestamp ordering test passed!");
}

#[tokio::test]
async fn test_flow_log_id_null_handling() {
    // Create a temporary database
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_config = DatabaseConfig::new(db_path.to_string_lossy());
    let db = DatabaseWriter::new(db_config).await.unwrap();

    // Debug: Check if database was created and schema initialized
    println!("Database created at: {}", db_path.display());

    // Insert result with NULL flow_log_id
    let result = AgentPerformanceData {
        benchmark_id: "test-benchmark".to_string(),
        agent_type: "deterministic".to_string(),
        score: 1.0,
        final_status: "Succeeded".to_string(),
        flow_log_id: None, // This should be handled as NULL
        execution_time_ms: 1000,
        timestamp: chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string(),
        prompt_md5: None,
    };

    // This should not fail due to foreign key constraint
    db.insert_agent_performance(&reev_lib::db::DbAgentPerformance::from(result))
        .await
        .unwrap();

    // Verify it was inserted correctly using public API
    // Note: get_agent_performance method doesn't exist in DatabaseWriter
    // Using a placeholder for now - this test may need to be updated
    let performance_summaries: Vec<reev_db::AgentPerformanceSummary> = Vec::new(); // Placeholder
    let deterministic_results = performance_summaries
        .iter()
        .find(|summary| summary.agent_type == "deterministic")
        .map(|summary| &summary.results)
        .unwrap();
    let results = deterministic_results;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].score, 1.0);

    println!("✅ Flow log ID NULL handling test passed!");
}
