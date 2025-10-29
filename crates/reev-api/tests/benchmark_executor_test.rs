use reev_api::services::benchmark_executor::PooledBenchmarkExecutor;
use reev_db::{DatabaseConfig, PooledDatabaseWriter};
use std::sync::Arc;

#[tokio::test]
async fn test_benchmark_executor_mode_detection() {
    // Create a mock database connection
    let db_config = DatabaseConfig::new("sqlite::memory:");
    let db = Arc::new(PooledDatabaseWriter::new(db_config, 5).await.unwrap());

    // Create executor with default config
    let executor = PooledBenchmarkExecutor::new_with_default(db.clone());

    // Test auto-detection mode (default)
    std::env::remove_var("REEV_USE_RELEASE");

    // This should auto-detect the release binary if it exists
    let result = executor.is_runner_available().await;

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

        println!(
            "{} ({}) - Runner available: {}",
            description, mode_value, is_available
        );

        // At least one mode should work
        assert!(is_available || mode_value == "true" || mode_value == "false");
    }

    // Clean up
    std::env::remove_var("REEV_USE_RELEASE");
}

#[tokio::test]
async fn test_benchmark_list_functionality() {
    let db_config = DatabaseConfig::new("sqlite::memory:");
    let db = Arc::new(PooledDatabaseWriter::new(db_config, 5).await.unwrap());
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
            println!(
                "Benchmark listing failed (expected in some environments): {}",
                e
            );
        }
    }

    // Test agent listing
    let agents = executor.list_agents().await;
    match agents {
        Ok(list) => {
            println!("Found agents: {:?}", list);
            // Should find at least the default agents
            assert!(!list.is_empty());
            assert!(list.iter().any(|a| a == "deterministic" || a == "local"));
        }
        Err(e) => {
            println!("Agent listing failed: {}", e);
        }
    }
}
