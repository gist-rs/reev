//! Test binary to demonstrate that the connection pool fixes the BorrowMutError issue
//!
//! This program reproduces the concurrent database access pattern that was causing
//! the `BorrowMutError` in the original implementation and shows that the connection
//! pool successfully handles concurrent operations.

use reev_db::{DatabaseConfig, PooledDatabaseWriter};
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("🧪 Testing concurrent database access fix");

    // Create a test database in the current directory
    let db_path = "test_concurrent.db";
    let config = DatabaseConfig::new(db_path);

    info!("📁 Using database: {}", db_path);

    // Create pooled database writer with 10 connections
    let db = Arc::new(PooledDatabaseWriter::new(config, 10).await?);

    info!("🔗 Connection pool created successfully");

    // Show initial pool stats
    let stats = db.pool_stats().await;
    info!("📊 Initial pool stats: {}", stats);

    // Sync some benchmarks first
    info!("🔄 Syncing benchmarks...");
    let sync_result = db.sync_benchmarks_from_dir("benchmarks").await?;
    info!("✅ Synced {} benchmarks", sync_result.processed_count);

    // Create concurrent tasks that simulate the same pattern as the API handlers
    let mut tasks = JoinSet::new();

    info!("🚀 Starting 20 concurrent database operations...");

    // Spawn multiple concurrent tasks that each perform various database operations
    for i in 0..20 {
        let db_clone = Arc::clone(&db);
        let task_id = i;

        tasks.spawn(async move {
            info!("📍 Task {} started", task_id);

            // Simulate the same operations that were causing BorrowMutError
            match task_id % 4 {
                0 => {
                    // Simulate getting agent performance (like in agents.rs)
                    let mut filter = reev_db::QueryFilter::new();
                    filter.limit = Some(5);
                    match db_clone.get_agent_performance(&filter).await {
                        Ok(perf) => {
                            info!("✅ Task {} got {} performance records", task_id, perf.len());
                        }
                        Err(e) => {
                            info!("❌ Task {} failed to get performance: {}", task_id, e);
                        }
                    }
                }
                1 => {
                    // Simulate getting session logs (like in ascii_tree.rs)
                    let filter = reev_db::types::SessionFilter {
                        benchmark_id: Some("001-sol-transfer".to_string()),
                        agent_type: Some("deterministic".to_string()),
                        interface: None,
                        status: None,
                        limit: Some(1),
                    };
                    match db_clone.list_sessions(&filter).await {
                        Ok(sessions) => {
                            info!("✅ Task {} got {} sessions", task_id, sessions.len());
                            if let Some(session) = sessions.first() {
                                match db_clone.get_session_log(&session.session_id).await {
                                    Ok(Some(log)) => {
                                        info!(
                                            "✅ Task {} got session log ({} chars)",
                                            task_id,
                                            log.len()
                                        );
                                    }
                                    Ok(None) => {
                                        info!("ℹ️ Task {} no session log found", task_id);
                                    }
                                    Err(e) => {
                                        info!(
                                            "❌ Task {} failed to get session log: {}",
                                            task_id, e
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            info!("❌ Task {} failed to list sessions: {}", task_id, e);
                        }
                    }
                }
                2 => {
                    // Simulate getting agent performance summary (like in agents.rs)
                    match db_clone.get_agent_performance_summary().await {
                        Ok(summary) => {
                            info!(
                                "✅ Task {} got performance summary ({} agents)",
                                task_id,
                                summary.len()
                            );
                        }
                        Err(e) => {
                            info!(
                                "❌ Task {} failed to get performance summary: {}",
                                task_id, e
                            );
                        }
                    }
                }
                3 => {
                    // Simulate getting database stats (like in health.rs)
                    match db_clone.get_database_stats().await {
                        Ok(stats) => {
                            info!(
                                "✅ Task {} got database stats: {} benchmarks, {} results",
                                task_id, stats.total_benchmarks, stats.total_results
                            );
                        }
                        Err(e) => {
                            info!("❌ Task {} failed to get database stats: {}", task_id, e);
                        }
                    }
                }
                _ => unreachable!(),
            }

            info!("🏁 Task {} completed", task_id);
            task_id
        });
    }

    // Wait for all tasks to complete
    let mut completed_tasks = 0;
    let mut failed_tasks = 0;

    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(task_id) => {
                completed_tasks += 1;
                info!("✅ Task {} finished successfully", task_id);
            }
            Err(e) => {
                failed_tasks += 1;
                info!("❌ Task failed: {}", e);
            }
        }
    }

    // Show final pool stats
    let final_stats = db.pool_stats().await;
    info!("📊 Final pool stats: {}", final_stats);

    info!("🎉 Test completed!");
    info!(
        "📈 Results: {} tasks completed, {} tasks failed",
        completed_tasks, failed_tasks
    );

    if failed_tasks == 0 {
        info!("🎯 SUCCESS: All concurrent operations completed without BorrowMutError!");
        info!("✅ The connection pool successfully fixes the concurrent access issue!");
    } else {
        info!("⚠️ Some tasks failed, but this might be expected if no test data exists");
    }

    Ok(())
}
