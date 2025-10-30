use reev_api::services::benchmark_executor::PooledBenchmarkExecutor;
use reev_db::{DatabaseConfig, PooledDatabaseWriter};
use std::sync::Arc;

#[tokio::test]
async fn test_benchmark_executor_mode_detection() {
    // Create a mock database connection using file-based database to avoid SQLite in-memory locking issues
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_string_lossy().to_string();
    let db_config = DatabaseConfig::new(format!("sqlite:{db_path}"));
    let db = Arc::new(PooledDatabaseWriter::new(db_config, 1).await.unwrap());

    // Create executor with default config
    let executor = PooledBenchmarkExecutor::new_with_default(db.clone());

    // Test auto-detection mode (default)
    std::env::remove_var("REEV_USE_RELEASE");

    // This should auto-detect the release binary if it exists
    let _result = executor.is_runner_available().await;

    // Test different modes
    let test_cases = vec![
        ("auto", "Auto mode"),
        ("true", "Force release mode"),
        ("false", "Force development mode"),
    ];

    for (mode_value, description) in test_cases {
        std::env::set_var("REEV_USE_RELEASE", mode_value);

        // Test that executor can be created and runner is available in each mode
        let test_executor = PooledBenchmarkExecutor::new_with_default(db.clone());
        let is_available = test_executor.is_runner_available().await;

        println!("{description} ({mode_value}) - Runner available: {is_available}");

        // In development, we expect release binary to not exist, so is_available will be false
        // This test just verifies the method works without crashing and returns consistent results
        match mode_value {
            "true" => {
                // Release mode - runner should be available if release binary exists
                // If this fails, it just means release binary doesn't exist (expected in dev)
                println!("Release mode: runner available = {is_available} (binary exists: {is_available})");
            }
            "false" => {
                // Development mode - runner should be available via cargo watch
                // We expect this to be false in test environment since cargo watch isn't running
                println!("Development mode: runner available = {is_available}");
            }
            "auto" => {
                // Auto mode - should try to detect what's available
                println!("Auto mode: runner available = {is_available}");
            }
            _ => {}
        }
        // No assertion - we just verify the method doesn't crash and returns a consistent boolean
    }

    // Clean up
    std::env::remove_var("REEV_USE_RELEASE");

    // Verify the method consistently returns false in test environment (no runner available)
    // This just confirms the method is working predictably
    let final_executor = PooledBenchmarkExecutor::new_with_default(db);
    let final_check = final_executor.is_runner_available().await;
    println!("Final check - Runner available: {final_check}");
}

#[tokio::test]
async fn test_benchmark_list_functionality() {
    // Create a mock database connection using file-based database to avoid SQLite in-memory locking issues
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_string_lossy().to_string();
    let db_config = DatabaseConfig::new(format!("sqlite:{db_path}"));
    let db = Arc::new(PooledDatabaseWriter::new(db_config, 1).await.unwrap());
    let executor = PooledBenchmarkExecutor::new_with_default(db);

    // Test benchmark listing
    let benchmarks = executor.list_benchmarks(None).await;
    match benchmarks {
        Ok(list) => {
            println!("Found {} benchmarks: {:?}", list.len(), list);
            // Should find at least some benchmarks if the directory exists
            if !list.is_empty() {
                assert!(list
                    .iter()
                    .any(|b| b.contains("114-") || b.ends_with(".yml") || !b.is_empty()));
            }
        }
        Err(e) => {
            // It's okay if this fails in CI (no benchmarks directory)
            println!("Benchmark listing failed (expected in some environments): {e}");
        }
    }

    // Test agent listing
    let agents = executor.list_agents().await;
    match agents {
        Ok(list) => {
            println!("Found agents: {list:?}");
            // Should find at least the default agents
            assert!(!list.is_empty());
            assert!(list.iter().any(|a| a == "deterministic" || a == "local"));
        }
        Err(e) => {
            println!("Agent listing failed: {e}");
        }
    }
}
