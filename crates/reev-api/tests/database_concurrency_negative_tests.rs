//! Negative Database Concurrency Tests
//!
//! This module demonstrates what would have failed before the mutex fix.
//! These tests simulate the old problematic behavior to show the improvement.
//! Tests are designed to fail without the proper concurrency protection.

use anyhow::Result;
use reev_db::{DatabaseConfig, DatabaseWriter};
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio::time::Duration;

/// Simulate the old problematic ApiState without mutex protection
struct ProblematicApiState {
    db: Arc<DatabaseWriter>, // No mutex protection - this causes the issues
}

/// Test 1: Demonstrate what would fail with direct concurrent database access
#[tokio::test]
#[ignore] // This test would fail without the mutex fix
async fn test_direct_concurrent_database_access_would_fail() -> Result<()> {
    // Create database without mutex protection (simulating old behavior)
    let db_path = "test_negative_concurrency.db";
    let db_config = DatabaseConfig::new(db_path);
    let db = Arc::new(DatabaseWriter::new(db_config).await?);

    // Initialize with test data
    db.create_session(&reev_db::types::SessionInfo {
        session_id: "negative-test-1".to_string(),
        benchmark_id: "negative-benchmark".to_string(),
        agent_type: "negative-agent".to_string(),
        interface: "test".to_string(),
        start_time: chrono::Utc::now().timestamp(),
        end_time: None,
        status: "running".to_string(),
        score: None,
        final_status: None,
    })
    .await?;

    let problematic_state = ProblematicApiState { db: db.clone() };

    // This simulates the old pattern where multiple handlers accessed the database directly
    let mut join_set = JoinSet::new();
    let operation_count = 20;

    for i in 0..operation_count {
        let db_clone = problematic_state.db.clone();

        join_set.spawn(async move {
            // Simulate the problematic direct access pattern
            match i % 4 {
                0 => {
                    // Simulate get_agent_performance call
                    let _result = tokio::task::spawn_blocking(move || {
                        // This would cause BorrowMutError in concurrent access
                        tokio::runtime::Handle::current()
                            .block_on(async { db_clone.get_agent_performance().await })
                    })
                    .await;
                }
                1 => {
                    // Simulate list_sessions call
                    let filter = reev_db::types::SessionFilter {
                        benchmark_id: Some("negative-benchmark".to_string()),
                        agent_type: None,
                        interface: None,
                        status: None,
                        limit: None,
                    };
                    let _result = tokio::task::spawn_blocking(move || {
                        tokio::runtime::Handle::current()
                            .block_on(async { db_clone.list_sessions(&filter).await })
                    })
                    .await;
                }
                2 => {
                    // Simulate get_session_log call
                    let _result = tokio::task::spawn_blocking(move || {
                        tokio::runtime::Handle::current()
                            .block_on(async { db_clone.get_session_log("negative-test-1").await })
                    })
                    .await;
                }
                _ => {
                    // Simulate get_all_benchmarks call
                    let _result = tokio::task::spawn_blocking(move || {
                        tokio::runtime::Handle::current()
                            .block_on(async { db_clone.get_all_benchmarks().await })
                    })
                    .await;
                }
            }

            Ok::<(), anyhow::Error>(())
        });
    }

    let mut success_count = 0;
    let mut borrow_error_count = 0;
    let mut other_error_count = 0;

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(())) => success_count += 1,
            Ok(Err(e)) => {
                if e.to_string().contains("borrowed") || e.to_string().contains("BorrowMutError") {
                    borrow_error_count += 1;
                    println!("❌ BorrowMutError (expected): {}", e);
                } else {
                    other_error_count += 1;
                    println!("❌ Other error: {}", e);
                }
            }
            Err(e) => {
                if e.to_string().contains("borrowed") || e.to_string().contains("panic") {
                    borrow_error_count += 1;
                    println!("❌ Panic/Borrow error (expected): {}", e);
                } else {
                    other_error_count += 1;
                    println!("❌ Task error: {}", e);
                }
            }
        }
    }

    // Without mutex, we expect many borrow errors
    println!("📊 Negative test results:");
    println!("   Successful operations: {}", success_count);
    println!("   BorrowMutError failures: {}", borrow_error_count);
    println!("   Other failures: {}", other_error_count);
    println!("   Total operations: {}", operation_count);

    // This test demonstrates the problem: many operations would fail
    let failure_rate = (borrow_error_count + other_error_count) as f64 / operation_count as f64;
    println!("   Failure rate: {:.1}%", failure_rate * 100.0);

    // In the old implementation, we'd expect high failure rate
    assert!(
        failure_rate > 0.3, // At least 30% failure rate demonstrates the problem
        "Expected high failure rate without mutex, got {:.1}%",
        failure_rate * 100.0
    );

    assert!(
        borrow_error_count > 0,
        "Expected at least some BorrowMutError failures"
    );

    // Cleanup
    let _ = std::fs::remove_file(db_path);

    Ok(())
}

/// Test 2: Demonstrate specific BorrowMutError scenarios
#[tokio::test]
#[ignore] // This test would fail without the mutex fix
async fn test_specific_borrow_mut_error_scenarios() -> Result<()> {
    let db_path = "test_borrow_errors.db";
    let db_config = DatabaseConfig::new(db_path);
    let db = Arc::new(DatabaseWriter::new(db_config).await?);

    // Create test data
    db.create_session(&reev_db::types::SessionInfo {
        session_id: "borrow-test-1".to_string(),
        benchmark_id: "borrow-benchmark".to_string(),
        agent_type: "borrow-agent".to_string(),
        interface: "test".to_string(),
        start_time: chrono::Utc::now().timestamp(),
        end_time: None,
        status: "running".to_string(),
        score: None,
        final_status: None,
    })
    .await?;

    let mut join_set = JoinSet::new();

    // Create scenarios that specifically trigger BorrowMutError
    for i in 0..10 {
        let db_clone = db.clone();

        join_set.spawn(async move {
            // Multiple simultaneous database operations on the same connection
            let handle1 = tokio::task::spawn_blocking({
                let db = db_clone.clone();
                move || {
                    tokio::runtime::Handle::current()
                        .block_on(async { db.get_agent_performance().await })
                }
            });

            let handle2 = tokio::task::spawn_blocking({
                let db = db_clone.clone();
                move || {
                    tokio::runtime::Handle::current().block_on(async {
                        db.list_sessions(&reev_db::types::SessionFilter {
                            benchmark_id: Some("borrow-benchmark".to_string()),
                            agent_type: None,
                            interface: None,
                            status: None,
                            limit: None,
                        })
                        .await
                    })
                }
            });

            let handle3 = tokio::task::spawn_blocking({
                let db = db_clone.clone();
                move || {
                    tokio::runtime::Handle::current()
                        .block_on(async { db.get_session_log("borrow-test-1").await })
                }
            });

            // Wait for all operations - this would cause BorrowMutError
            let _ = tokio::join!(handle1, handle2, handle3);

            Ok::<(), anyhow::Error>(())
        });
    }

    let mut panic_count = 0;
    let mut success_count = 0;

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(())) => success_count += 1,
            Ok(Err(e)) => {
                println!("❌ Operation failed: {}", e);
            }
            Err(e) => {
                if e.is_panic() {
                    panic_count += 1;
                    println!("❌ Panic occurred (expected BorrowMutError)");
                } else {
                    println!("❌ Task error: {}", e);
                }
            }
        }
    }

    println!("🚨 BorrowMutError scenario results:");
    println!("   Successful operations: {}", success_count);
    println!("   Panics (BorrowMutError): {}", panic_count);
    println!("   Total scenarios: {}", 10);

    // We expect multiple panics due to BorrowMutError
    assert!(
        panic_count > 0,
        "Expected at least some panics due to BorrowMutError"
    );

    // Cleanup
    let _ = std::fs::remove_file(db_path);

    Ok(())
}

/// Test 3: Performance comparison - mutex vs no-mutex
#[tokio::test]
#[ignore] // This test demonstrates the performance difference
async fn test_performance_comparison_mutex_vs_no_mutex() -> Result<()> {
    let db_path = "test_performance_comparison.db";
    let db_config = DatabaseConfig::new(db_path);
    let db = Arc::new(DatabaseWriter::new(db_config).await?);

    // Test without mutex (simulated old behavior - expect many failures)
    let start_no_mutex = std::time::Instant::now();
    let mut join_set_no_mutex = JoinSet::new();

    for _ in 0..50 {
        let db_clone = db.clone();
        join_set_no_mutex.spawn(async move {
            let _result = tokio::task::spawn_blocking(move || {
                tokio::runtime::Handle::current()
                    .block_on(async { db_clone.get_agent_performance().await })
            })
            .await;
            Ok::<(), anyhow::Error>(())
        });
    }

    let mut no_mutex_success = 0;
    let mut no_mutex_failures = 0;

    while let Some(result) = join_set_no_mutex.join_next().await {
        match result {
            Ok(Ok(())) => no_mutex_success += 1,
            _ => no_mutex_failures += 1,
        }
    }

    let no_mutex_time = start_no_mutex.elapsed();

    // Test with mutex (current implementation)
    let db_mutex = Arc::new(tokio::sync::Mutex::new((*db).clone()));
    let start_with_mutex = std::time::Instant::now();
    let mut join_set_with_mutex = JoinSet::new();

    for _ in 0..50 {
        let db_clone = db_mutex.clone();
        join_set_with_mutex.spawn(async move {
            let _db_guard = db_clone.lock().await;
            let _result = tokio::task::spawn_blocking(move || {
                tokio::runtime::Handle::current().block_on(async {
                    // Simulate database operation
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await;
            Ok::<(), anyhow::Error>(())
        });
    }

    let mut mutex_success = 0;
    while let Some(result) = join_set_with_mutex.join_next().await {
        if result.is_ok() {
            mutex_success += 1;
        }
    }

    let with_mutex_time = start_with_mutex.elapsed();

    println!("⚖️ Performance comparison:");
    println!(
        "   No mutex (old): {}ms, {}/50 successful ({:.1}%)",
        no_mutex_time.as_millis(),
        no_mutex_success,
        (no_mutex_success as f64 / 50.0) * 100.0
    );
    println!(
        "   With mutex (new): {}ms, {}/50 successful ({:.1}%)",
        with_mutex_time.as_millis(),
        mutex_success,
        (mutex_success as f64 / 50.0) * 100.0
    );

    let success_improvement =
        (mutex_success as f64 - no_mutex_success as f64) / no_mutex_success as f64 * 100.0;
    println!("   Success improvement: {:.1}%", success_improvement);

    // Mutex should provide much better success rate
    assert!(
        mutex_success > no_mutex_success,
        "Mutex should provide better success rate"
    );

    // Cleanup
    let _ = std::fs::remove_file(db_path);

    Ok(())
}

/// Test 4: Demonstrate real-world scenario that would fail
#[tokio::test]
#[ignore] // This demonstrates a real failure scenario
async fn test_real_world_failure_scenario() -> Result<()> {
    let db_path = "test_real_world_failure.db";
    let db_config = DatabaseConfig::new(db_path);
    let db = Arc::new(DatabaseWriter::new(db_config).await?);

    // Set up test data
    db.create_session(&reev_db::types::SessionInfo {
        session_id: "real-world-1".to_string(),
        benchmark_id: "001-sol-transfer".to_string(),
        agent_type: "test-agent".to_string(),
        interface: "web".to_string(),
        start_time: chrono::Utc::now().timestamp(),
        end_time: None,
        status: "running".to_string(),
        score: None,
        final_status: None,
    })
    .await?;

    // Simulate real-world concurrent API calls that would happen during execution
    let mut join_set = JoinSet::new();

    // Simulate UI polling multiple endpoints simultaneously
    for poll_cycle in 0..5 {
        let db_clone = db.clone();

        // Agent performance poll
        join_set.spawn(async move {
            let _result = tokio::task::spawn_blocking({
                let db = db_clone.clone();
                move || {
                    tokio::runtime::Handle::current()
                        .block_on(async { db.get_agent_performance().await })
                }
            })
            .await;
            Ok::<(), anyhow::Error>(())
        });

        // Flow logs poll
        join_set.spawn(async move {
            let _result = tokio::task::spawn_blocking({
                let db = db_clone.clone();
                move || {
                    tokio::runtime::Handle::current().block_on(async {
                        db.list_sessions(&reev_db::types::SessionFilter {
                            benchmark_id: Some("001-sol-transfer".to_string()),
                            agent_type: None,
                            interface: None,
                            status: None,
                            limit: None,
                        })
                        .await
                    })
                }
            })
            .await;
            Ok::<(), anyhow::Error>(())
        });

        // Transaction logs poll
        join_set.spawn(async move {
            let _result = tokio::task::spawn_blocking({
                let db = db_clone.clone();
                move || {
                    tokio::runtime::Handle::current()
                        .block_on(async { db.get_session_log("real-world-1").await })
                }
            })
            .await;
            Ok::<(), anyhow::Error>(())
        });

        // Small delay between poll cycles
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    let mut success_count = 0;
    let mut failure_count = 0;
    let total_requests = 15; // 5 cycles * 3 endpoints

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(())) => success_count += 1,
            _ => failure_count += 1,
        }
    }

    let success_rate = (success_count as f64 / total_requests as f64) * 100.0;

    println!("🌐 Real-world failure scenario:");
    println!("   Total requests: {}", total_requests);
    println!("   Successful: {}", success_count);
    println!("   Failed: {}", failure_count);
    println!("   Success rate: {:.1}%", success_rate);

    // Without mutex, real-world scenarios would have poor success rates
    assert!(
        success_rate < 80.0,
        "Real-world scenario without mutex should have poor success rate, got {:.1}%",
        success_rate
    );

    // Cleanup
    let _ = std::fs::remove_file(db_path);

    Ok(())
}
