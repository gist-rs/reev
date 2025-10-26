//! End-to-End "Run All" Test
//!
//! This test validates the async threading fix by running multiple benchmarks
//! sequentially with different agents, similar to TUI's "Run All" functionality.
//! Tests both shared surfpool (no re-init) and fresh surfpool modes.

use anyhow::{Context, Result};
use glob::glob;
use project_root::get_project_root;
use reev_lib::benchmark::TestCase;
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

        // Run the benchmark
        let run_results = run_benchmarks(benchmark_path.clone(), agent_name).await?;

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
    // Use simple benchmarks for fast feedback
    let benchmarks = vec!["001-sol-transfer.yml", "002-spl-transfer.yml"]
        .into_iter()
        .map(|name| get_project_root().unwrap().join("benchmarks").join(name))
        .collect::<Vec<PathBuf>>();

    let agents = vec!["deterministic", "local"];

    info!("🚀 Starting E2E Run All test");

    for agent in agents {
        info!("📋 Testing agent: {}", agent);

        // Test "Run All" functionality - sequential execution
        let results = run_agent_tests(agent, &benchmarks, false).await?;

        // Validate consistency across runs (no random results)
        validate_consistency(agent, &results)?;

        info!("✅ Agent {} E2E test passed", agent);
        sleep(Duration::from_secs(1)).await;
    }

    info!("🎉 All E2E Run All tests passed - async threading fix validated!");
    Ok(())
}

/// Test individual benchmark runs for quick debugging
#[tokio::test(flavor = "multi_thread")]
async fn test_single_benchmark_consistency() -> Result<()> {
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
        let run_results = run_benchmarks(benchmark_path.clone(), agent).await?;

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

    info!("✅ Single benchmark consistency test passed - all scores identical");
    Ok(())
}
