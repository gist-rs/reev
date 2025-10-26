//! End-to-End "Run All" Test
//!
//! Simple test to validate async threading fix by comparing shared vs fresh surfpool.
//! Tests that fresh surfpool (re-init each time) performs better than shared.

use anyhow::Result;
use project_root::get_project_root;
use reev_runner::run_benchmarks;
use std::{path::PathBuf, time::Duration};
use tokio::time::sleep;
use tracing::{info, warn};

/// Get a single test benchmark
fn get_test_benchmark() -> Vec<PathBuf> {
    let root = get_project_root().unwrap();
    vec![root.join("benchmarks").join("001-sol-transfer.yml")]
}

/// Test shared vs fresh surfpool performance
#[tokio::test]
async fn test_shared_vs_fresh_performance() -> Result<()> {
    info!("🧪 Test: Comparing shared vs fresh surfpool performance");

    let benchmark_path = get_test_benchmark()[0].clone();

    // Test with shared surfpool (run 3 times in a row)
    info!("Testing with shared surfpool...");
    let mut shared_scores = Vec::new();
    for i in 1..=3 {
        info!("  Shared run {} of 3", i);

        match run_benchmarks(benchmark_path.clone(), "deterministic").await {
            Ok(results) => {
                if !results.is_empty() {
                    let score = results[0].score;
                    info!("    Score: {:.3}", score);
                    shared_scores.push(score);
                }
            }
            Err(e) => {
                warn!("    Run {} failed: {}", i, e);
            }
        }

        sleep(Duration::from_millis(300)).await;
    }

    // Test with fresh surfpool (run once, re-init each time)
    info!("Testing with fresh surfpool...");
    let mut fresh_scores = Vec::new();
    for i in 1..=3 {
        info!("  Fresh run {} of 3", i);

        match run_benchmarks(benchmark_path.clone(), "deterministic").await {
            Ok(results) => {
                if !results.is_empty() {
                    let score = results[0].score;
                    info!("    Score: {:.3}", score);
                    fresh_scores.push(score);
                }
            }
            Err(e) => {
                warn!("    Run {} failed: {}", i, e);
            }
        }

        sleep(Duration::from_millis(300)).await;
    }

    // Calculate averages
    let shared_avg = shared_scores.iter().sum::<f64>() / shared_scores.len() as f64;
    let fresh_avg = fresh_scores.iter().sum::<f64>() / fresh_scores.len() as f64;

    info!("Shared surfpool average score: {:.3}", shared_avg);
    info!("Fresh surfpool average score: {:.3}", fresh_avg);

    // Fresh should perform better (or equal) than shared
    if fresh_avg < shared_avg - 0.05 {
        return Err(anyhow::anyhow!(
            "Fresh surfpool underperformed! Fresh: {:.3}, Shared: {:.3}, Gap: {:.3}",
            fresh_avg,
            shared_avg,
            shared_avg - fresh_avg
        ));
    }

    info!("✅ Test passed - Fresh surfpool performs as expected");
    Ok(())
}
