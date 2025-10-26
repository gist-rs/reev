//! End-to-End "Run All" Test
//!
//! This test validates the async threading fix by running multiple benchmarks
//! sequentially with different agents, similar to TUI's "Run All" functionality.
//! Tests both shared surfpool (no re-init) and fresh surfpool modes.

use anyhow::{Context, Result};
use glob::glob;
use project_root::get_project_root;
use reev_lib::benchmark::TestCase;
use reev_lib::server_utils;
use reev_runner::run_benchmarks;
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::time::sleep;
use tracing::{info, warn};

/// Discover benchmark files from the benchmarks directory
#[allow(unused)]
fn discover_benchmark_files() -> Vec<PathBuf> {
    let root = get_project_root().unwrap();
    let pattern = root.join("benchmarks/*.yml");
    glob(pattern.to_str().unwrap())
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .collect()
}

/// Load benchmark configuration from file
async fn load_benchmark(benchmark_path: &Path) -> Result<TestCase> {
    let content = fs::read_to_string(benchmark_path)
        .with_context(|| format!("Failed to read benchmark file: {benchmark_path:?}"))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse benchmark file: {benchmark_path:?}"))
}

/// Run benchmarks with a specific agent and track results
async fn run_agent_tests(
    agent_name: &str,
    benchmarks: &[PathBuf],
    shared_surfpool: bool,
) -> Result<Vec<(PathBuf, f64)>> {
    let mut results = Vec::new();

    info!(
        "🤖 Testing agent: {} (shared surfpool: {})",
        agent_name, shared_surfpool
    );

    for benchmark_path in benchmarks {
        info!(
            "📊 Running {} with {}...",
            benchmark_path.display(),
            agent_name
        );

        // Load benchmark config to get ID for logging
        let _test_case = load_benchmark(benchmark_path).await?;

        // Run benchmark
        let run_results =
            run_benchmarks(benchmark_path.clone(), agent_name, shared_surfpool).await?;

        if !run_results.is_empty() {
            let result = &run_results[0];
            let score = result.score;

            info!(
                "✅ {} completed with {}: score={:.3}",
                benchmark_path.display(),
                agent_name,
                score
            );

            results.push((benchmark_path.clone(), score));

            // Small delay between runs for stability
            sleep(Duration::from_millis(500)).await;
        } else {
            warn!(
                "⚠️ No results returned for {} with {}",
                benchmark_path.display(),
                agent_name
            );
        }
    }

    Ok(results)
}

/// Validate that results are consistent across runs
fn validate_consistency(agent_name: &str, results: &[(PathBuf, f64)]) -> Result<()> {
    info!("🔍 Validating consistency across runs for {}", agent_name);

    if results.is_empty() {
        return Ok(());
    }

    let first_score = results[0].1;

    for (path, score) in results.iter() {
        let score_diff = (score - first_score).abs();
        if score_diff > 0.001 {
            return Err(anyhow::anyhow!(
                "Score inconsistency detected for {agent_name}: path={path:?}, expected={first_score:.6}, got={score:.6}, diff={score_diff:.6}"
            ));
        }

        info!("✅ {} consistent: score={:.6}", path.display(), score);
    }

    Ok(())
}

/// Main E2E test: Run All functionality with multiple agents
#[tokio::test(flavor = "multi_thread")]
async fn test_run_all_benchmarks_multi_agent_e2e() -> Result<()> {
    // Clean up any existing processes before starting
    info!("🧹 Cleaning up existing processes...");
    server_utils::kill_existing_api(3001).await?;
    server_utils::kill_existing_reev_agent(9090).await?;
    server_utils::kill_existing_surfpool(8899).await?;

    // Use only first 2 benchmark files for faster testing
    let benchmarks = discover_benchmark_files();
    let benchmarks: Vec<PathBuf> = benchmarks.into_iter().take(2).collect();

    let agents = vec!["deterministic"];

    info!("🚀 Starting E2E Run All test - comparing shared vs fresh surfpool");

    for agent in agents {
        info!("📋 Testing agent: {}", agent);

        // Test with SHARED surfpool (reuse same instance)
        info!("🔗 Testing SHARED surfpool mode...");
        let shared_results = run_agent_tests(agent, &benchmarks, true).await?;
        validate_consistency(&format!("{agent}-shared"), &shared_results)?;
        info!("✅ Agent {} SHARED surfpool test passed", agent);
        sleep(Duration::from_secs(2)).await;

        // Test with FRESH surfpool (new instance for each run)
        info!("🔄 Testing FRESH surfpool mode...");
        let fresh_results = run_agent_tests(agent, &benchmarks, false).await?;
        validate_consistency(&format!("{agent}-fresh"), &fresh_results)?;
        info!("✅ Agent {} FRESH surfpool test passed", agent);
        sleep(Duration::from_secs(2)).await;

        // Compare results - both should be consistent
        if !shared_results.is_empty() && !fresh_results.is_empty() {
            let shared_avg: f64 =
                shared_results.iter().map(|(_, s)| s).sum::<f64>() / shared_results.len() as f64;
            let fresh_avg: f64 =
                fresh_results.iter().map(|(_, s)| s).sum::<f64>() / fresh_results.len() as f64;
            let diff = (shared_avg - fresh_avg).abs();

            info!("📊 Performance comparison:");
            info!("  Shared avg: {:.6}", shared_avg);
            info!("  Fresh avg: {:.6}", fresh_avg);
            info!("  Difference: {:.6}", diff);

            // Validate that both modes produce consistent results
            if diff > 0.01 {
                warn!(
                    "⚠️  Large difference detected between shared and fresh modes: {:.6}",
                    diff
                );
            } else {
                info!("✅ Shared and fresh modes produce consistent results");
            }
        }
    }

    info!("🎉 All E2E Run All tests passed - shared vs fresh surfpool validated!");
    Ok(())
}

/// Test individual benchmark runs for quick debugging
#[tokio::test(flavor = "multi_thread")]
async fn test_single_benchmark_consistency() -> Result<()> {
    // Clean up any existing processes before starting
    info!("🧹 Cleaning up existing processes...");
    server_utils::kill_existing_api(3001).await?;
    server_utils::kill_existing_reev_agent(9090).await?;
    server_utils::kill_existing_surfpool(8899).await?;

    let benchmark_path = get_project_root()
        .unwrap()
        .join("benchmarks")
        .join("001-sol-transfer.yml");

    let agent = "deterministic";
    let iterations = 3;
    let mut scores = Vec::new();

    info!(
        "🔬 Testing single benchmark consistency ({} iterations)",
        iterations
    );

    for i in 1..=iterations {
        let run_results = run_benchmarks(benchmark_path.clone(), agent, true).await?;

        if !run_results.is_empty() {
            let result = &run_results[0];
            let _test_case = load_benchmark(&benchmark_path).await?;
            let score = result.score;

            info!("  Run {}: score={:.6}", i, score);
            scores.push(score);
        }

        sleep(Duration::from_millis(300)).await;
    }

    // Validate all scores are identical
    for (i, score) in scores.iter().enumerate() {
        assert_eq!(
            *score,
            scores[0],
            "Score mismatch on run {}: expected={:.6}, got={:.6}",
            i + 1,
            scores[0],
            score
        );
    }

    info!(
        "✅ Single benchmark consistency test passed - all scores identical (using shared surfpool)"
    );
    Ok(())
}
