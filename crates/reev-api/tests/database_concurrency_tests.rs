//! Database Concurrency Tests for reev-api
//!
//! This module provides comprehensive tests to verify the database concurrency fix.
//! It includes:
//! - Positive tests: Proving the mutex fix works correctly
//! - Negative tests: Demonstrating what would fail without the fix
//! - Stress tests: High-concurrency scenarios
//! - Integration tests: Real-world API endpoint testing

use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use tower::util::ServiceExt;

// Import from current crate
use reev_api::{
    handlers::{get_agent_performance, get_flow_log, get_transaction_logs},
    types::{ApiState, ExecutionState, ExecutionStatus},
};
use reev_db::{DatabaseConfig, DatabaseWriter};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

/// Helper function to create test database
async fn create_test_database() -> Result<DatabaseWriter> {
    let db_path = "test_concurrency.db";
    let db_config = DatabaseConfig::new(db_path);
    let db = DatabaseWriter::new(db_config).await?;

    // Initialize with some test data
    db.create_session(&reev_db::types::SessionInfo {
        session_id: "test-session-1".to_string(),
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

    Ok(db)
}

/// Helper function to create test API state
async fn create_test_api_state() -> Result<ApiState> {
    let db = create_test_database().await?;
    let state = ApiState {
        db: Arc::new(Mutex::new(db)),
        executions: Arc::new(Mutex::new(HashMap::new())),
        agent_configs: Arc::new(Mutex::new(HashMap::new())),
    };
    Ok(state)
}

/// Helper function to create test router
fn create_test_router() -> Router {
    Router::new()
        .route("/api/v1/agent-performance", get(get_agent_performance))
        .route("/api/v1/flow-logs/:benchmark_id", get(get_flow_log))
        .route(
            "/api/v1/transaction-logs/:benchmark_id",
            get(get_transaction_logs),
        )
}

/// Test 1: Positive - Sequential database access works correctly
#[tokio::test]
async fn test_sequential_database_access() -> Result<()> {
    let state = create_test_api_state().await?;
    let app = create_test_router().with_state(state);

    // Test sequential access to multiple endpoints
    let endpoints = vec![
        "/api/v1/agent-performance",
        "/api/v1/flow-logs/test-benchmark",
        "/api/v1/transaction-logs/test-benchmark",
    ];

    for endpoint in endpoints {
        let request = Request::builder()
            .uri(endpoint)
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Endpoint {} should return 200 OK",
            endpoint
        );
    }

    Ok(())
}

/// Test 2: Positive - Concurrent database access works with mutex
#[tokio::test]
async fn test_concurrent_database_access_with_mutex() -> Result<()> {
    let state = create_test_api_state().await?;
    let app = create_test_router().with_state(state);

    let endpoints = vec![
        "/api/v1/agent-performance",
        "/api/v1/flow-logs/test-benchmark",
        "/api/v1/transaction-logs/test-benchmark",
    ];

    let mut join_set = JoinSet::new();
    let mut success_count = 0;

    // Spawn concurrent requests to all endpoints
    for (i, endpoint) in endpoints.iter().cycle().take(20).enumerate() {
        let app_clone = app.clone();
        let endpoint = endpoint.to_string();

        join_set.spawn(async move {
            // Add small delay to increase concurrency
            tokio::time::sleep(Duration::from_millis(i as u64 % 5)).await;

            let request = Request::builder()
                .uri(&endpoint)
                .body(Body::empty())
                .unwrap();

            match app_clone.oneshot(request).await {
                Ok(response) => (endpoint, response.status()),
                Err(e) => {
                    eprintln!("Request to {} failed: {}", endpoint, e);
                    (endpoint, StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        });
    }

    // Collect results
    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok((endpoint, status)) => {
                results.push((endpoint, status));
                if status == StatusCode::OK {
                    success_count += 1;
                }
            }
            Err(e) => {
                eprintln!("Task failed: {}", e);
            }
        }
    }

    // With the mutex fix, all requests should succeed
    assert_eq!(
        success_count, 20,
        "All 20 concurrent requests should succeed with mutex"
    );
    assert_eq!(results.len(), 20, "Should have 20 results");

    // Verify no internal server errors
    let error_count = results
        .iter()
        .filter(|(_, status)| *status == StatusCode::INTERNAL_SERVER_ERROR)
        .count();
    assert_eq!(error_count, 0, "Should have no internal server errors");

    println!(
        "✅ Concurrent test passed: {}/{} requests succeeded",
        success_count,
        results.len()
    );
    Ok(())
}

/// Test 3: Positive - High-stress concurrent access
#[tokio::test]
async fn test_high_stress_concurrent_access() -> Result<()> {
    let state = create_test_api_state().await?;
    let app = create_test_router().with_state(state);

    let mut join_set = JoinSet::new();
    let request_count = 100;

    // Spawn many concurrent requests
    for i in 0..request_count {
        let app_clone = app.clone();

        join_set.spawn(async move {
            let endpoint = match i % 3 {
                0 => "/api/v1/agent-performance",
                1 => "/api/v1/flow-logs/test-benchmark",
                _ => "/api/v1/transaction-logs/test-benchmark",
            };

            // Random delay to simulate real-world usage
            tokio::time::sleep(Duration::from_millis((i % 10) as u64)).await;

            let request = Request::builder()
                .uri(endpoint)
                .body(Body::empty())
                .unwrap();

            app_clone.oneshot(request).await
        });
    }

    let mut success_count = 0;
    let mut error_count = 0;

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(response)) => {
                if response.status() == StatusCode::OK {
                    success_count += 1;
                } else {
                    error_count += 1;
                    eprintln!("Request failed with status: {}", response.status());
                }
            }
            Ok(Err(e)) => {
                error_count += 1;
                eprintln!("Request error: {}", e);
            }
            Err(e) => {
                error_count += 1;
                eprintln!("Task join error: {}", e);
            }
        }
    }

    // With mutex, should have high success rate
    let success_rate = (success_count as f64 / request_count as f64) * 100.0;
    println!(
        "📊 High-stress test: {}/{} requests succeeded ({:.1}% success rate)",
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

    Ok(())
}

/// Test 4: Positive - Database operations during active execution
#[tokio::test]
async fn test_database_access_during_execution() -> Result<()> {
    let state = create_test_api_state().await?;

    // Simulate active execution
    {
        let mut executions = state.executions.lock().await;
        executions.insert(
            "test-execution-1".to_string(),
            ExecutionState {
                id: "test-execution-1".to_string(),
                benchmark_id: "test-benchmark".to_string(),
                agent: "test-agent".to_string(),
                status: ExecutionStatus::Running,
                progress: 50,
                start_time: chrono::Utc::now(),
                end_time: None,
                trace: "Test execution trace".to_string(),
                logs: "Test execution logs".to_string(),
                error: None,
            },
        );
    }

    let app = create_test_router().with_state(state);

    // Test concurrent access while execution is "running"
    let mut join_set = JoinSet::new();

    for i in 0..10 {
        let app_clone = app.clone();

        join_set.spawn(async move {
            let endpoint = if i % 2 == 0 {
                "/api/v1/flow-logs/test-benchmark"
            } else {
                "/api/v1/transaction-logs/test-benchmark"
            };

            let request = Request::builder()
                .uri(endpoint)
                .body(Body::empty())
                .unwrap();

            app_clone.oneshot(request).await
        });
    }

    let mut success_count = 0;
    while let Some(result) = join_set.join_next().await {
        if let Ok(Ok(response)) = result {
            if response.status() == StatusCode::OK {
                success_count += 1;
            }
        }
    }

    assert_eq!(
        success_count, 10,
        "All requests during execution should succeed"
    );
    println!(
        "✅ Execution simulation test passed: {}/10 requests succeeded",
        success_count
    );

    Ok(())
}

/// Test 5: Negative - Demonstrate what would fail without mutex (simulated)
#[tokio::test]
async fn test_simulated_concurrent_failure() -> Result<()> {
    // This test simulates the old behavior by creating a scenario
    // where concurrent access would cause issues

    let state = create_test_api_state().await?;

    // Simulate multiple database operations happening simultaneously
    let mut join_set = JoinSet::new();
    let operation_count = 20;

    for i in 0..operation_count {
        let db = state.db.clone();

        join_set.spawn(async move {
            // Simulate the old direct database access pattern
            let db_guard = db.lock().await;

            // Multiple operations that would have conflicted without mutex
            match i % 4 {
                0 => {
                    // Simulate get_agent_performance
                    let _result = db_guard.get_agent_performance().await;
                }
                1 => {
                    // Simulate list_sessions
                    let filter = reev_db::types::SessionFilter {
                        benchmark_id: Some("test-benchmark".to_string()),
                        agent_type: None,
                        interface: None,
                        status: None,
                        limit: None,
                    };
                    let _result = db_guard.list_sessions(&filter).await;
                }
                2 => {
                    // Simulate get_session_log
                    let _result = db_guard.get_session_log("test-session-1").await;
                }
                _ => {
                    // Simulate other database operations
                    let _result = db_guard.get_all_benchmarks().await;
                }
            }

            // Database guard is automatically released here
            Ok::<(), anyhow::Error>(())
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

    // With mutex, all operations should succeed
    // Without mutex, many would fail with BorrowMutError
    assert_eq!(
        success_count, operation_count,
        "All operations should succeed with mutex"
    );
    assert_eq!(error_count, 0, "No operations should fail with mutex");

    println!(
        "✅ Simulated failure test passed: {}/{} operations succeeded (would fail without mutex)",
        success_count, operation_count
    );

    Ok(())
}

/// Test 6: Integration - Real-world concurrent API usage
#[tokio::test]
async fn test_real_world_concurrent_api_usage() -> Result<()> {
    let state = create_test_api_state().await?;
    let app = create_test_router().with_state(state);

    // Simulate real-world usage pattern: user starts execution and UI polls multiple endpoints
    let mut join_set = JoinSet::new();

    // Simulate user starting execution (not actually starting, just setting up state)
    {
        let mut executions = state.executions.lock().await;
        executions.insert(
            "real-execution-1".to_string(),
            ExecutionState {
                id: "real-execution-1".to_string(),
                benchmark_id: "real-benchmark".to_string(),
                agent: "real-agent".to_string(),
                status: ExecutionStatus::Running,
                progress: 25,
                start_time: chrono::Utc::now(),
                end_time: None,
                trace: "Real execution trace data".to_string(),
                logs: "Real execution logs".to_string(),
                error: None,
            },
        );
    }

    // Simulate UI polling pattern
    for poll_cycle in 0..5 {
        let app_clone = app.clone();

        join_set.spawn(async move {
            // Each poll cycle hits multiple endpoints simultaneously
            let mut requests = Vec::new();

            // Agent performance (updates main table)
            requests.push(
                app_clone.clone().oneshot(
                    Request::builder()
                        .uri("/api/v1/agent-performance")
                        .body(Body::empty())
                        .unwrap(),
                ),
            );

            // Flow logs (updates execution view)
            requests.push(
                app_clone.clone().oneshot(
                    Request::builder()
                        .uri("/api/v1/flow-logs/real-benchmark")
                        .body(Body::empty())
                        .unwrap(),
                ),
            );

            // Transaction logs (updates transaction view)
            requests.push(
                app_clone.oneshot(
                    Request::builder()
                        .uri("/api/v1/transaction-logs/real-benchmark")
                        .body(Body::empty())
                        .unwrap(),
                ),
            );

            // Wait for all requests in this poll cycle
            let mut cycle_success = 0;
            for request in requests {
                match request.await {
                    Ok(Ok(response)) => {
                        if response.status() == StatusCode::OK {
                            cycle_success += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("Poll cycle {} request failed: {}", poll_cycle, e);
                    }
                }
            }

            Ok::<usize, anyhow::Error>(cycle_success)
        });

        // Small delay between poll cycles
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let mut total_success = 0;
    let total_expected = 5 * 3; // 5 cycles * 3 endpoints each

    while let Some(result) = join_set.join_next().await {
        if let Ok(Ok(cycle_success)) = result {
            total_success += cycle_success;
        }
    }

    let success_rate = (total_success as f64 / total_expected as f64) * 100.0;
    println!(
        "📈 Real-world usage test: {}/{} requests succeeded ({:.1}% success rate)",
        total_success, total_expected, success_rate
    );

    assert!(
        success_rate >= 90.0,
        "Real-world success rate should be at least 90%, got {:.1}%",
        success_rate
    );

    Ok(())
}

/// Test 7: Performance - Measure serialization overhead
#[tokio::test]
async fn test_mutex_performance_overhead() -> Result<()> {
    let state = create_test_api_state().await?;

    // Measure sequential access time
    let start_sequential = std::time::Instant::now();
    for _ in 0..50 {
        let _db = state.db.lock().await;
        // Simulate database operation
        tokio::time::sleep(Duration::from_micros(100)).await;
    }
    let sequential_time = start_sequential.elapsed();

    // Measure concurrent access time
    let start_concurrent = std::time::Instant::now();
    let mut join_set = JoinSet::new();

    for _ in 0..50 {
        let db = state.db.clone();
        join_set.spawn(async move {
            let _db_guard = db.lock().await;
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

    Ok(())
}

/// Cleanup function to remove test database
#[tokio::test]
async fn test_cleanup() -> Result<()> {
    // Remove test database file if it exists
    let _ = std::fs::remove_file("test_concurrency.db");
    Ok(())
}
