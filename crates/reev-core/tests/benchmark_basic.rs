//! Basic Benchmark Test
//!
//! This test demonstrates the basic benchmark functionality by creating a simple flow,
//! executing it, and scoring the execution against ground truth.

use anyhow::Result;
use reev_core::{
    benchmark::{BenchmarkScorer, ExecutionMetrics},
    query_handler::QueryHandler,
    yml_schema::{YmlAssertion, YmlFlow, YmlGroundTruth, YmlStep, YmlToolCall, YmlWalletInfo},
};
use serial_test::serial;
use std::time::Instant;
use tracing::info;

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_basic_swap_benchmark() -> Result<()> {
    info!("Testing basic swap benchmark functionality");
    info!("===========================================");

    // Initialize test runner
    let mut runner = common::framework::TestRunner::new()?;
    runner.initialize().await?;

    // Initialize query handler
    let query_handler = QueryHandler::new().await?;

    // Create a simple swap flow manually for testing
    let flow_id = uuid::Uuid::new_v4().to_string();
    let prompt = "swap 1 sol to usdc";

    // Create wallet info
    let wallet_info = YmlWalletInfo::new(
        runner.pubkey.to_string(),
        5_000_000_000, // 5 SOL
    )
    .with_total_value(500.0);

    // Create swap step
    let swap_step = YmlStep::new(
        uuid::Uuid::new_v4().to_string(),
        prompt.to_string(),
        "Swap SOL to USDC".to_string(),
    )
    .with_tool_call(YmlToolCall::new(
        reev_types::tools::ToolName::JupiterSwap,
        true, // critical
    ));

    // Create comprehensive ground truth for benchmarking
    let ground_truth = YmlGroundTruth::new()
        .with_min_score(0.7)
        .with_assertion(
            YmlAssertion::new("SolBalanceChange".to_string())
                .with_pubkey(runner.pubkey.to_string())
                .with_expected_change_lte(-1_100_000_000.0), // 1.1 SOL max (swap + fees)
        )
        .with_tool_call(YmlToolCall::new(
            reev_types::tools::ToolName::JupiterSwap,
            true, // critical
        ));

    // Create the flow
    let flow = YmlFlow::new(flow_id, prompt.to_string(), wallet_info)
        .with_step(swap_step)
        .with_ground_truth(ground_truth);

    // Process the query with benchmark scoring
    info!("Processing query with benchmark scoring");
    let execution_start_time = Instant::now();

    // Execute the flow
    let wallet_context = query_handler
        .context_resolver
        .resolve_wallet_context(&runner.pubkey.to_string())
        .await?;

    let execution_result = query_handler
        .executor
        .execute_flow(&flow, &wallet_context)
        .await?;

    // Score the execution
    info!("Scoring execution against ground truth");
    let scorer = BenchmarkScorer::new();
    let scored_result = scorer
        .score_flow_execution(&flow, &execution_result, execution_start_time)
        .await?;

    // Create execution metrics
    let execution_time_ms = execution_start_time.elapsed().as_millis() as u64;
    let execution_metrics = ExecutionMetrics {
        total_execution_time_ms: execution_time_ms,
        steps_executed: flow.steps.len() as u32,
        tool_calls_made: execution_result.step_results.len() as u32,
        successful_tool_calls: execution_result
            .step_results
            .iter()
            .filter(|step| step.success)
            .count() as u32,
        recovery_attempts: 0,
        memory_usage_bytes: None,
        cpu_usage_percent: None,
    };

    // Generate benchmark report
    let benchmark_report = scorer.generate_report(&flow, &scored_result, execution_metrics);

    // Print results
    info!("Benchmark results:");
    info!("  Overall score: {:.2}", benchmark_report.overall_score);
    info!("  Execution time: {}ms", execution_time_ms);
    info!(
        "  Passed assertions: {}",
        scored_result.score.passed_assertions
    );
    info!(
        "  Failed assertions: {}",
        scored_result.score.failed_assertions
    );

    // Print improvement suggestions
    if !benchmark_report.improvement_suggestions.is_empty() {
        info!("Improvement suggestions:");
        for suggestion in &benchmark_report.improvement_suggestions {
            info!("  - {}", suggestion);
        }
    }

    // Verify the benchmark components are working
    assert!(
        benchmark_report.overall_score >= 0.0,
        "Score should be non-negative"
    );
    assert!(execution_time_ms > 0, "Execution time should be positive");

    info!("Basic swap benchmark test completed successfully");
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_query_handler_benchmark() -> Result<()> {
    info!("Testing QueryHandler benchmark integration");
    info!("=========================================");

    // Initialize test runner
    let mut runner = common::framework::TestRunner::new()?;
    runner.initialize().await?;

    // Initialize query handler
    let mut query_handler = QueryHandler::new().await?;

    // Reset wallet balance to ensure we have enough SOL for tests
    runner.reset_wallet_balance().await?;

    // Process a simple query with benchmark scoring
    let prompt = "swap 0.1 sol to usdc";
    info!("Processing query with benchmark: {}", prompt);

    let (result, benchmark_report) = query_handler
        .process_query_with_benchmark(prompt, &runner.pubkey)
        .await?;

    // Print results
    info!("Query result: {}", result.success);
    info!("Benchmark report:");
    info!("  Overall score: {:.2}", benchmark_report.overall_score);
    info!(
        "  Execution time: {}ms",
        benchmark_report.execution_metrics.total_execution_time_ms
    );
    info!(
        "  Steps executed: {}",
        benchmark_report.execution_metrics.steps_executed
    );
    info!(
        "  Tool calls made: {}",
        benchmark_report.execution_metrics.tool_calls_made
    );

    // Print improvement suggestions
    if !benchmark_report.improvement_suggestions.is_empty() {
        info!("Improvement suggestions:");
        for suggestion in &benchmark_report.improvement_suggestions {
            info!("  - {}", suggestion);
        }
    }

    // Verify the benchmark components are working
    assert!(
        benchmark_report.overall_score >= 0.0,
        "Score should be non-negative"
    );
    assert!(
        benchmark_report.execution_metrics.total_execution_time_ms > 0,
        "Execution time should be positive"
    );

    info!("QueryHandler benchmark integration test completed successfully");
    Ok(())
}

mod common;
