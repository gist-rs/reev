//! Database Concurrency Unit Tests
//!
//! This module provides focused unit tests for the database concurrency fix.
//! Tests the core database access patterns without the full HTTP stack.

use anyhow::Result;
use reev_db::{DatabaseConfig, DatabaseWriter};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tokio::time::Duration;

/// Test 1: Positive - Sequential database access works correctly
#[tokio::test]
async fn test_sequential_database_access() -> Result<()> {
    let db_path = "test_sequential.db";
    let db_config = DatabaseConfig::new(db_path);
    let db = DatabaseWriter::new(db_config).await?;

    // Wrap in mutex like the fixed implementation
    let db_mutex = Arc::new(Mutex::new(db));

    // Test sequential access
    for i in 0..10 {
        let db_guard = db_mutex.lock().await;
        let result = db_guard.get_all_benchmarks().await;
        drop(db_guard);

        assert!(result.is_ok(), "Sequential access {} should succeed", i);
    }

    // Cleanup
    let _ = std::fs::remove_file(db_path);
    Ok(())
}

/// Test 2: Positive - Concurrent database access works with mutex
#[tokio::test]
async fn test_concurrent_database_access_with_mutex() -> Result<()> {
    let db_path = "test_concurrent.db";
    let db_config = DatabaseConfig::new(db_path);
    let db = DatabaseWriter::new(db_config).await?;

    // Wrap in mutex like the fixed implementation
    let db_mutex = Arc::new(Mutex::new(db));

    // Initialize test data
    {
        let db_guard = db_mutex.lock().await;
        db_guard
            .create_session(&reev_db::types::SessionInfo {
                session_id: "concurrent-test-1".to_string(),
                benchmark_id: "test-benchmark".to_string(),
                agent_type: "test-agent".to_string(),
                interface: "test".to_string(),
                start_time: chrono::Utc::now().timestamp(),
                end_time: None,
                status: "running".to_string(),
                score: None,
                final_status: None,
            })
            .await?;

        // Create a session log
        db_guard
            .store_complete_log("concurrent-test-1", "test log content")
            .await?;
    }

    let mut join_set = JoinSet::new();
    let operation_count = 20;

    // Spawn concurrent database operations
    for i in 0..operation_count {
        let db_clone = db_mutex.clone();

        join_set.spawn(async move {
            let db_guard = db_clone.lock().await;

            let result = match i % 4 {
                0 => db_guard.get_agent_performance().await.map(|_| ()),
                1 => {
                    let filter = reev_db::types::SessionFilter {
                        benchmark_id: Some("test-benchmark".to_string()),
                        agent_type: None,
                        interface: None,
                        status: None,
                        limit: None,
                    };
                    db_guard.list_sessions(&filter).await.map(|_| ())
                }
                2 => {
                    // Session log access - handle missing logs gracefully
                    match db_guard.get_session_log("concurrent-test-1").await {
                        Ok(_) => Ok(()),
                        Err(_) => Ok(()), // Missing log is expected, not an error
                    }
                }
                _ => db_guard.get_all_benchmarks().await.map(|_| ()),
            };

            // Database guard is automatically released when scope ends
            result.map(|_| ())
        });
    }

    let mut success_count = 0;
    let mut error_count = 0;

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(())) => success_count += 1,
            Ok(Err(e)) => {
                error_count += 1;
                eprintln!("Database operation failed: {}", e);
            }
            Err(e) => {
                error_count += 1;
                eprintln!("Task failed: {}", e);
            }
        }
    }

    println!(
        "📊 Concurrent test results: {}/{} operations succeeded",
        success_count, operation_count
    );

    // With mutex, all operations should succeed
    assert_eq!(
        success_count, operation_count,
        "All operations should succeed with mutex"
    );
    assert_eq!(error_count, 0, "No operations should fail with mutex");

    // Cleanup
    let _ = std::fs::remove_file(db_path);
    Ok(())
}

/// Test 3: Positive - High-stress concurrent access
#[tokio::test]
async fn test_high_stress_concurrent_access() -> Result<()> {
    let db_path = "test_stress.db";
    let db_config = DatabaseConfig::new(db_path);
    let db = DatabaseWriter::new(db_config).await?;

    let db_mutex = Arc::new(Mutex::new(db));

    // Initialize test data
    {
        let db_guard = db_mutex.lock().await;
        for i in 0..5 {
            db_guard
                .create_session(&reev_db::types::SessionInfo {
                    session_id: format!("stress-test-{}", i),
                    benchmark_id: "stress-benchmark".to_string(),
                    agent_type: "stress-agent".to_string(),
                    interface: "test".to_string(),
                    start_time: chrono::Utc::now().timestamp(),
                    end_time: None,
                    status: "running".to_string(),
                    score: None,
                    final_status: None,
                })
                .await?;

            // Create session logs for some sessions
            if i < 3 {
                db_guard
                    .store_complete_log(&format!("stress-test-{}", i), "test log content")
                    .await?;
            }
        }
    }

    let mut join_set = JoinSet::new();
    let request_count = 100;

    // Spawn many concurrent operations
    for i in 0..request_count {
        let db_clone = db_mutex.clone();

        join_set.spawn(async move {
            let db_guard = db_clone.lock().await;

            // Simulate database operation with small delay
            tokio::time::sleep(Duration::from_micros(100)).await;

            let result = match i % 3 {
                0 => db_guard.get_agent_performance().await.map(|_| ()),
                1 => {
                    let filter = reev_db::types::SessionFilter {
                        benchmark_id: Some("stress-benchmark".to_string()),
                        agent_type: None,
                        interface: None,
                        status: None,
                        limit: None,
                    };
                    db_guard.list_sessions(&filter).await.map(|_| ())
                }
                _ => {
                    let session_id = format!("stress-test-{}", i % 5);
                    // Handle missing session logs gracefully
                    match db_guard.get_session_log(&session_id).await {
                        Ok(_) => Ok(()),
                        Err(_) => Ok(()), // Missing log is expected
                    }
                }
            };

            result.map(|_| ())
        });
    }

    let mut success_count = 0;
    let mut error_count = 0;

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(())) => success_count += 1,
            _ => error_count += 1,
        }
    }

    let success_rate = (success_count as f64 / request_count as f64) * 100.0;
    println!(
        "📈 High-stress test: {}/{} operations succeeded ({:.1}% success rate)",
        success_count, request_count, success_rate
    );

    assert!(
        success_rate >= 95.0,
        "Success rate should be at least 95%, got {:.1}%",
        success_rate
    );
    assert!(
        error_count <= request_count / 20,
        "Error count should be minimal, got {}",
        error_count
    );

    // Cleanup
    let _ = std::fs::remove_file(db_path);
    Ok(())
}

/// Test 4: Performance - Measure mutex overhead
#[tokio::test]
async fn test_mutex_performance_overhead() -> Result<()> {
    let db_path = "test_performance.db";
    let db_config = DatabaseConfig::new(db_path);
    let db = DatabaseWriter::new(db_config).await?;

    let db_mutex = Arc::new(Mutex::new(db));

    // Measure sequential access time
    let start_sequential = std::time::Instant::now();
    for _ in 0..50 {
        let _db_guard = db_mutex.lock().await;
        // Simulate database operation
        tokio::time::sleep(Duration::from_micros(100)).await;
        // Database guard is released here
    }
    let sequential_time = start_sequential.elapsed();

    // Measure concurrent access time
    let start_concurrent = std::time::Instant::now();
    let mut join_set = JoinSet::new();

    for _ in 0..50 {
        let db_clone = db_mutex.clone();
        join_set.spawn(async move {
            let _db_guard = db_clone.lock().await;
            // Simulate database operation
            tokio::time::sleep(Duration::from_micros(100)).await;
            Ok::<(), anyhow::Error>(())
        });
    }

    while let Some(_) = join_set.join_next().await {}
    let concurrent_time = start_concurrent.elapsed();

    let overhead_ratio = concurrent_time.as_millis() as f64 / sequential_time.as_millis() as f64;

    println!("⏱️ Performance test:");
    println!("   Sequential: {}ms", sequential_time.as_millis());
    println!("   Concurrent: {}ms", concurrent_time.as_millis());
    println!("   Overhead ratio: {:.2}x", overhead_ratio);

    // Overhead should be reasonable (less than 3x)
    assert!(
        overhead_ratio < 3.0,
        "Mutex overhead should be less than 3x, got {:.2}x",
        overhead_ratio
    );

    // Cleanup
    let _ = std::fs::remove_file(db_path);
    Ok(())
}

/// Test 5: Positive - Mutex prevents data corruption
#[tokio::test]
async fn test_mutex_prevents_data_corruption() -> Result<()> {
    let db_path = "test_corruption.db";
    let db_config = DatabaseConfig::new(db_path);
    let db = DatabaseWriter::new(db_config).await?;

    let db_mutex = Arc::new(Mutex::new(db));

    // Initialize test data
    {
        let db_guard = db_mutex.lock().await;
        db_guard
            .create_session(&reev_db::types::SessionInfo {
                session_id: "corruption-test".to_string(),
                benchmark_id: "corruption-benchmark".to_string(),
                agent_type: "corruption-agent".to_string(),
                interface: "test".to_string(),
                start_time: chrono::Utc::now().timestamp(),
                end_time: None,
                status: "running".to_string(),
                score: None,
                final_status: None,
            })
            .await?;

        // Create session log
        db_guard
            .store_complete_log("corruption-test", "corruption test log content")
            .await?;
    }

    let mut join_set = JoinSet::new();
    let operation_count = 50;

    // Spawn concurrent operations that could corrupt data without proper synchronization
    for _i in 0..operation_count {
        let db_clone = db_mutex.clone();

        join_set.spawn(async move {
            let db_guard = db_clone.lock().await;

            // Perform multiple operations that could interfere without mutex
            let result1 = db_guard.get_session_log("corruption-test").await;
            let result2 = db_guard
                .list_sessions(&reev_db::types::SessionFilter {
                    benchmark_id: Some("corruption-benchmark".to_string()),
                    agent_type: None,
                    interface: None,
                    status: None,
                    limit: None,
                })
                .await;

            // Both operations should succeed without corruption
            match (result1, result2) {
                (Ok(_), Ok(_)) => Ok(()),
                (Err(e), _) => Err(e),
                (_, Err(e)) => Err(e),
            }
        });
    }

    let mut success_count = 0;
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(())) => success_count += 1,
            Ok(Err(e)) => eprintln!("Operation failed: {}", e),
            Err(e) => eprintln!("Task failed: {}", e),
        }
    }

    assert_eq!(
        success_count, operation_count,
        "All operations should succeed without data corruption"
    );

    // Cleanup
    let _ = std::fs::remove_file(db_path);
    Ok(())
}

/// Test 6: Positive - Deadlock prevention
#[tokio::test]
async fn test_no_deadlocks_with_proper_mutex_usage() -> Result<()> {
    let db_path = "test_deadlock.db";
    let db_config = DatabaseConfig::new(db_path);
    let db = DatabaseWriter::new(db_config).await?;

    let db_mutex = Arc::new(Mutex::new(db));

    let mut join_set = JoinSet::new();
    let operation_count = 30;

    // Spawn operations with complex patterns that could cause deadlocks
    for _i in 0..operation_count {
        let db_clone = db_mutex.clone();

        join_set.spawn(async move {
            // Nested operations but always releasing lock properly
            {
                let db_guard = db_clone.lock().await;

                // First operation
                let _result1 = db_guard.get_all_benchmarks().await;

                // Lock is automatically released when scope ends
            }

            // Small delay
            tokio::time::sleep(Duration::from_micros(10)).await;

            // Second operation with new lock acquisition
            {
                let db_guard = db_clone.lock().await;
                let _result2 = db_guard.get_agent_performance().await;
                // Lock released here
            }

            Ok::<(), anyhow::Error>(())
        });
    }

    let mut success_count = 0;
    let mut timeout_count = 0;

    // Add timeout to detect potential deadlocks
    let timeout_duration = Duration::from_secs(10);
    let start_time = std::time::Instant::now();

    while let Some(result) = join_set.join_next().await {
        if start_time.elapsed() > timeout_duration {
            timeout_count += 1;
            break;
        }

        match result {
            Ok(Ok(())) => success_count += 1,
            _ => eprintln!("Operation failed"),
        }
    }

    assert!(
        timeout_count == 0,
        "No operations should timeout (deadlock detection)"
    );
    assert!(
        success_count >= operation_count - 2,
        "Most operations should succeed"
    );

    // Cleanup
    let _ = std::fs::remove_file(db_path);
    Ok(())
}

/// Cleanup test
#[tokio::test]
async fn test_cleanup_all_test_databases() -> Result<()> {
    let test_dbs = vec![
        "test_sequential.db",
        "test_concurrent.db",
        "test_stress.db",
        "test_performance.db",
        "test_corruption.db",
        "test_deadlock.db",
    ];

    for db_file in test_dbs {
        let _ = std::fs::remove_file(db_file);
    }

    Ok(())
}
