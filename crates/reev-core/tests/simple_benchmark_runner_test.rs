//! Simple unit tests for BenchmarkRunner

use anyhow::Result;
use reev_core::benchmark::runner::BenchmarkRunner;

#[tokio::test]
async fn test_benchmark_runner_creation() -> Result<()> {
    // Create a new benchmark runner
    let runner = BenchmarkRunner::new()?;

    // Verify that runner was created successfully
    assert_eq!(runner.pubkey().to_string().len(), 44); // Standard Solana pubkey length

    Ok(())
}

#[tokio::test]
async fn test_summary_report_generation() -> Result<()> {
    let runner = BenchmarkRunner::new()?;

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
