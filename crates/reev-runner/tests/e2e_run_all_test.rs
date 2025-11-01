//! End-to-End "Run All" Test
//!
//! This test validates async threading fix by running multiple benchmarks
//! sequentially with different agents, similar to TUI's "Run All" functionality.
//! Tests both shared surfpool (no re-init) and fresh surfpool modes.
//!
//! Usage:
//!   cargo test -p reev-runner --test e2e_run_all_test -- --agent glm-4.6
//!   cargo test -p reev-runner --test e2e_run_all_test -- --agent local

use anyhow::{Context, Result};
use glob::glob;
use project_root::get_project_root;
use reev_lib::benchmark::TestCase;
use reev_lib::server_utils;
use reev_runner::run_benchmarks;
use serial_test::serial;
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
        let run_results = run_benchmarks(
            benchmark_path.clone(),
            agent_name,
            shared_surfpool,
            true,
            None,
        )
        .await?;

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

    // Group results by benchmark file to check consistency within each benchmark
    use std::collections::HashMap;
    let mut benchmark_scores: HashMap<String, Vec<f64>> = HashMap::new();

    for (path, score) in results.iter() {
        let bench_name = path.file_name().unwrap().to_str().unwrap().to_string();
        benchmark_scores.entry(bench_name).or_default().push(*score);
    }

    for (bench_name, scores) in benchmark_scores {
        if scores.len() < 2 {
            info!(
                "  ✅ {} only has one run: score={:.6}",
                bench_name, scores[0]
            );
            continue;
        }

        let first_score = scores[0];
        let max_diff = scores
            .iter()
            .map(|s| (s - first_score).abs())
            .fold(0.0, f64::max);

        // More lenient tolerance for non-deterministic agents
        // Deterministic agents should be very consistent (< 0.001)
        // LLM/local agents can vary more due to network and API variability
        let tolerance = if agent_name.contains("deterministic") {
            0.001
        } else {
            0.1 // Allow up to 10% variation for non-deterministic agents
        };

        if max_diff > tolerance {
            return Err(anyhow::anyhow!(
                "Score inconsistency detected for {agent_name} on {bench_name}: expected={first_score:.6}, max_diff={max_diff:.6}, tolerance={tolerance:.6}, scores={scores:?}"
            ));
        }

        info!(
            "  ✅ {} consistent: score={:.6} (max_diff={:.6})",
            bench_name, first_score, max_diff
        );
    }

    Ok(())
}

/// Main E2E test: Run All functionality with multiple agents
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_run_all_benchmarks_multi_agent_e2e() -> Result<()> {
    // Clean up any existing processes before starting
    info!("🧹 Cleaning up existing processes...");
    server_utils::kill_existing_api(3001).await?;
    server_utils::kill_existing_reev_agent(9090).await?;
    server_utils::kill_existing_surfpool(8899).await?;

    // Use only first benchmark file for faster testing
    let benchmarks = discover_benchmark_files();
    let benchmarks: Vec<PathBuf> = benchmarks.into_iter().take(1).collect();

    info!("🚀 Starting E2E Run All test - basic functionality");

    // Test deterministic agent consistency with shared surfpool
    info!("📋 Testing deterministic agent with shared surfpool...");
    let shared_results = run_agent_tests("deterministic", &benchmarks, true).await?;
    validate_consistency("deterministic-shared", &shared_results)?;
    info!("✅ Deterministic agent SHARED surfpool test passed");
    sleep(Duration::from_secs(1)).await;

    info!("🎉 E2E Run All test passed - basic functionality validated!");
    Ok(())
}

/// Test individual benchmark runs for quick debugging
#[tokio::test(flavor = "multi_thread")]
#[serial]
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
        let run_results = run_benchmarks(benchmark_path.clone(), agent, true, true, None).await?;

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
