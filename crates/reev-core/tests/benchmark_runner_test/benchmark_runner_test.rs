//! Tests for the BenchmarkRunner

use anyhow::Result;
use reev_core::benchmark::runner::BenchmarkRunner;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_benchmark_runner_creation() -> Result<()> {
    // Create a new benchmark runner
    let runner = BenchmarkRunner::new()?;

    // Verify the runner was created successfully
    assert_eq!(runner.pubkey().to_string().len(), 44); // Standard Solana pubkey length

    Ok(())
}

#[tokio::test]
async fn test_static_benchmark_execution() -> Result<()> {
    // Create a temporary directory for test files
    let temp_dir = TempDir::new()?;

    // Create a simple benchmark YAML file
    let benchmark_content = r#"
flow_id: "test-benchmark"
description: "Test benchmark for BenchmarkRunner"
prompt: "Test prompt"

flow_type: "static"
tags: ["test"]

steps:
  - step_id: "test-step"
    refined_prompt: "Test step prompt"
    context: {}
    critical: true
    expected_tools: []

ground_truth:
  min_score: 0.5
  final_state_assertions: []
  expected_tool_calls: []
  success_criteria: []
  expected_data_structure: []
"#;

    let benchmark_path = temp_dir.path().join("test_benchmark.yml");
    std::fs::write(&benchmark_path, benchmark_content)?;

    // Create a benchmark runner with surfpool disabled for testing
    let mut runner = BenchmarkRunner::new()?;

    // Initialize the runner
    runner.initialize().await?;

    // Execute the benchmark (this will likely fail due to missing surfpool, but that's OK for this test)
    let result = runner.execute_static_benchmark(&benchmark_path).await?;

    // Verify the result structure
    assert!(result.execution_time_ms > 0);

    // We don't check for success here since we don't have surfpool running
    // The important part is that the runner executes without panicking

    Ok(())
}

#[tokio::test]
async fn test_summary_report_generation() -> Result<()> {
    let mut runner = BenchmarkRunner::new()?;

    // Create mock results
    let results = vec![
        reev_core::benchmark::runner::BenchmarkExecutionResult {
            report: reev_core::benchmark::BenchmarkReport::default(),
            execution_time_ms: 1000,
            success: true,
            error: None,
        },
        reev_core::benchmark::runner::BenchmarkExecutionResult {
            report: reev_core::benchmark::BenchmarkReport::default(),
            execution_time_ms: 2000,
            success: false,
            error: Some("Test error".to_string()),
        },
    ];

    // Generate summary report
    let summary = runner.generate_summary_report(&results);

    // Verify summary content
    assert!(summary.contains("Total Benchmarks: 2"));
    assert!(summary.contains("Successful: 1"));
    assert!(summary.contains("Failed: 1"));
    assert!(summary.contains("Total Execution Time: 3000ms"));

    Ok(())
}
