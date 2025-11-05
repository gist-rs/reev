//! OpenTelemetry Tracking Test: 300-Series Dynamic Flow Benchmarks
//!
//! This test validates that all 300-series benchmarks have proper OpenTelemetry
//! tracking expectations and that tool calls are correctly instrumented.
//! It ensures comprehensive observability for dynamic flow execution.

use crate::tests::dynamic_flow::300_series::{create_test_wallet_context, load_benchmark_yaml, TestUtils};
use reev_orchestrator::OrchestratorGateway;
use reev_types::flow::WalletContext;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[tokio::test]
async fn test_300_series_otel_tracking_completeness() -> anyhow::Result<()> {
    println!("🎯 Testing 300-Series: OpenTelemetry Tracking Completeness");

    let benchmark_ids = vec![
        "300-jup-swap-then-lend-deposit-dyn",
        "301-dynamic-yield-optimization",
        "302-portfolio-rebalancing",
        "303-risk-adjusted-growth",
        "304-emergency-exit-strategy",
        "305-yield-farming-optimization",
    ];

    for benchmark_id in benchmark_ids {
        println!("\n📊 Validating OTEL tracking for: {}", benchmark_id);

        let benchmark = load_benchmark_yaml(benchmark_id);

        // Extract expected OTEL tracking from benchmark
        let expected_otel = benchmark
            .get("ground_truth")
            .and_then(|gt| gt.get("expected_otel_tracking"))
            .and_then(|eot| eot.as_sequence())
            .unwrap_or(&serde_json::Value::Null);

        let expected_tracking_types = vec![
            "tool_call_logging",
            "execution_tracing",
            "mermaid_generation",
            "performance_metrics",
        ];

        for tracking_type in &expected_tracking_types {
            let found = expected_otel
                .as_sequence()
                .unwrap_or(&serde_json::json!([]))
                .iter()
                .any(|tracking| {
                    tracking.get("type")
                        .and_then(|t| t.as_str())
                        .map(|t| t == tracking_type)
                        .unwrap_or(false)
                });

            assert!(found, "Should expect {} OTEL tracking in {}", tracking_type, benchmark_id);
        }

        println!("  ✅ All OTEL tracking types present");
    }

    println!("\n✅ OTEL tracking completeness validation passed");
    Ok(())
}

#[tokio::test]
async fn test_300_series_tool_call_logging_requirements() -> anyhow::Result<()> {
    println!("🎯 Testing 300-Series: Tool Call Logging Requirements");

    let expected_tool_requirements = vec![
        ("300-jup-swap-then-lend-deposit-dyn", vec![
            "account_balance", "jupiter_swap", "jupiter_lend", "jupiter_positions"
        ]),
        ("301-dynamic-yield-optimization", vec![
            "account_balance", "jupiter_swap", "jupiter_lend", "jupiter_positions"
        ]),
        ("302-portfolio-rebalancing", vec![
            "account_balance", "jupiter_swap", "jupiter_lend", "jupiter_positions"
        ]),
        ("303-risk-adjusted-growth", vec![
            "account_balance", "jupiter_swap", "jupiter_lend", "jupiter_positions"
        ]),
        ("304-emergency-exit-strategy", vec![
            "account_balance", "jupiter_positions", "jupiter_withdraw", "jupiter_swap"
        ]),
        ("305-yield-farming-optimization", vec![
            "account_balance", "jupiter_pools", "jupiter_lend_rates",
            "jupiter_swap", "jupiter_lend", "jupiter_positions"
        ]),
    ];

    for (benchmark_id, expected_tools) in expected_tool_requirements {
        println!("\n🔍 Validating tool call logging for: {}", benchmark_id);

        let benchmark = load_benchmark_yaml(benchmark_id);

        // Find tool_call_logging section
        let tool_logging = benchmark
            .get("ground_truth")
            .and_then(|gt| gt.get("expected_otel_tracking"))
            .and_then(|eot| eot.as_sequence())
            .and_then(|seq| {
                seq.iter().find(|tracking| {
                    tracking.get("type")
                        .and_then(|t| t.as_str())
                        .map(|t| t == "tool_call_logging")
                        .unwrap_or(false)
                })
            });

        assert!(tool_logging.is_some(), "Should have tool_call_logging section in {}", benchmark_id);

        if let Some(logging) = tool_logging {
            let required_tools = logging
                .get("required_tools")
                .and_then(|rt| rt.as_sequence())
                .map(|seq| {
                    seq.iter()
                        .filter_map(|tool| tool.as_str())
                        .collect::<HashSet<_>>()
                })
                .unwrap_or_default();

            println!("  Required tools: {:?}", required_tools);
            println!("  Expected tools: {:?}", expected_tools);

            // All expected tools should be in required tools
            for expected_tool in &expected_tools {
                assert!(required_tools.contains(expected_tool),
                       "Should require {} tool in {} OTEL logging", expected_tool, benchmark_id);
            }

            // Validate weight distribution
            let weight = logging
                .get("weight")
                .and_then(|w| w.as_f64())
                .unwrap_or(0.0);

            assert!(weight > 0.0, "Tool call logging should have positive weight in {}", benchmark_id);
        }

        println!("  ✅ Tool call logging requirements validated");
    }

    println!("\n✅ Tool call logging requirements validation passed");
    Ok(())
}

#[tokio::test]
async fn test_300_series_execution_tracing_spans() -> anyhow::Result<()> {
    println!("🎯 Testing 300-Series: Execution Tracing Spans");

    let expected_span_requirements = vec![
        ("300-jup-swap-then-lend-deposit-dyn", vec![
            "prompt_processing", "context_resolution", "swap_execution", "lend_execution"
        ]),
        ("301-dynamic-yield-optimization", vec![
            "prompt_processing", "yield_analysis", "swap_execution", "lend_execution"
        ]),
        ("302-portfolio-rebalancing", vec![
            "prompt_processing", "portfolio_analysis", "rebalancing_execution"
        ]),
        ("303-risk-adjusted-growth", vec![
            "prompt_processing", "risk_analysis", "conservative_execution"
        ]),
        ("304-emergency-exit-strategy", vec![
            "prompt_processing", "position_analysis", "emergency_withdrawal", "asset_conversion"
        ]),
        ("305-yield-farming-optimization", vec![
            "prompt_processing", "pool_analysis", "apy_comparison", "multi_pool_execution"
        ]),
    ];

    for (benchmark_id, expected_spans) in expected_span_requirements {
        println!("\n🔍 Validating execution tracing spans for: {}", benchmark_id);

        let benchmark = load_benchmark_yaml(benchmark_id);

        // Find execution_tracing section
        let execution_tracing = benchmark
            .get("ground_truth")
            .and_then(|gt| gt.get("expected_otel_tracking"))
            .and_then(|eot| eot.as_sequence())
            .and_then(|seq| {
                seq.iter().find(|tracking| {
                    tracking.get("type")
                        .and_then(|t| t.as_str())
                        .map(|t| t == "execution_tracing")
                        .unwrap_or(false)
                })
            });

        assert!(execution_tracing.is_some(), "Should have execution_tracing section in {}", benchmark_id);

        if let Some(tracing) = execution_tracing {
            let required_spans = tracing
                .get("required_spans")
                .and_then(|rs| rs.as_sequence())
                .map(|seq| {
                    seq.iter()
                        .filter_map(|span| span.as_str())
                        .collect::<HashSet<_>>()
                })
                .unwrap_or_default();

            println!("  Required spans: {:?}", required_spans);
            println!("  Expected spans: {:?}", expected_spans);

            // All expected spans should be in required spans
            for expected_span in &expected_spans {
                assert!(required_spans.contains(expected_span),
                       "Should require {} span in {} OTEL tracing", expected_span, benchmark_id);
            }

            // Validate weight and description
            let weight = tracing
                .get("weight")
                .and_then(|w| w.as_f64())
                .unwrap_or(0.0);

            let description = tracing
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");

            assert!(weight > 0.0, "Execution tracing should have positive weight in {}", benchmark_id);
            assert!(!description.is_empty(), "Execution tracing should have description in {}", benchmark_id);
        }

        println!("  ✅ Execution tracing spans validated");
    }

    println!("\n✅ Execution tracing spans validation passed");
    Ok(())
}

#[tokio::test]
async fn test_300_series_mermaid_generation_expectations() -> anyhow::Result<()> {
    println!("🎯 Testing 300-Series: Mermaid Generation Expectations");

    let benchmark_ids = vec![
        "300-jup-swap-then-lend-deposit-dyn",
        "301-dynamic-yield-optimization",
        "302-portfolio-rebalancing",
        "303-risk-adjusted-growth",
        "304-emergency-exit-strategy",
        "305-yield-farming-optimization",
    ];

    for benchmark_id in benchmark_ids {
        println!("\n📈 Validating Mermaid generation for: {}", benchmark_id);

        let benchmark = load_benchmark_yaml(benchmark_id);

        // Find mermaid_generation section
        let mermaid_generation = benchmark
            .get("ground_truth")
            .and_then(|gt| gt.get("expected_otel_tracking"))
            .and_then(|eot| eot.as_sequence())
            .and_then(|seq| {
                seq.iter().find(|tracking| {
                    tracking.get("type")
                        .and_then(|t| t.as_str())
                        .map(|t| t == "mermaid_generation")
                        .unwrap_or(false)
                })
            });

        assert!(mermaid_generation.is_some(), "Should have mermaid_generation section in {}", benchmark_id);

        if let Some(mermaid) = mermaid_generation {
            let required = mermaid
                .get("required")
                .and_then(|r| r.as_bool())
                .unwrap_or(false);

            let weight = mermaid
                .get("weight")
                .and_then(|w| w.as_f64())
                .unwrap_or(0.0);

            let description = mermaid
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");

            println!("  Required: {}", required);
            println!("  Weight: {:.2}", weight);
            println!("  Description: {}", description);

            assert!(required, "Mermaid generation should be required in {}", benchmark_id);
            assert!(weight > 0.0, "Mermaid generation should have positive weight in {}", benchmark_id);
            assert!(!description.is_empty(), "Mermaid generation should have description in {}", benchmark_id);
        }

        println!("  ✅ Mermaid generation expectations validated");
    }

    println!("\n✅ Mermaid generation expectations validation passed");
    Ok(())
}

#[tokio::test]
async fn test_300_series_performance_metrics_tracking() -> anyhow::Result<()> {
    println!("🎯 Testing 300-Series: Performance Metrics Tracking");

    let expected_metrics_requirements = vec![
        ("300-jup-swap-then-lend-deposit-dyn", vec![
            "execution_time_ms", "tool_call_count", "success_rate"
        ]),
        ("301-dynamic-yield-optimization", vec![
            "execution_time_ms", "tool_call_count", "yield_optimization_score"
        ]),
        ("302-portfolio-rebalancing", vec![
            "execution_time_ms", "tool_call_count", "rebalancing_efficiency"
        ]),
        ("303-risk-adjusted-growth", vec![
            "execution_time_ms", "tool_call_count", "capital_preservation_ratio"
        ]),
        ("304-emergency-exit-strategy", vec![
            "execution_time_ms", "tool_call_count", "capital_preservation_ratio"
        ]),
        ("305-yield-farming-optimization", vec![
            "execution_time_ms", "tool_call_count", "apy_optimization_score"
        ]),
    ];

    for (benchmark_id, expected_metrics) in expected_metrics_requirements {
        println!("\n📊 Validating performance metrics for: {}", benchmark_id);

        let benchmark = load_benchmark_yaml(benchmark_id);

        // Find performance_metrics section
        let performance_metrics = benchmark
            .get("ground_truth")
            .and_then(|gt| gt.get("expected_otel_tracking"))
            .and_then(|eot| eot.as_sequence())
            .and_then(|seq| {
                seq.iter().find(|tracking| {
                    tracking.get("type")
                        .and_then(|t| t.as_str())
                        .map(|t| t == "performance_metrics")
                        .unwrap_or(false)
                })
            });

        assert!(performance_metrics.is_some(), "Should have performance_metrics section in {}", benchmark_id);

        if let Some(metrics) = performance_metrics {
            let required_metrics = metrics
                .get("required_metrics")
                .and_then(|rm| rm.as_sequence())
                .map(|seq| {
                    seq.iter()
                        .filter_map(|metric| metric.as_str())
                        .collect::<HashSet<_>>()
                })
                .unwrap_or_default();

            println!("  Required metrics: {:?}", required_metrics);
            println!("  Expected metrics: {:?}", expected_metrics);

            // All expected metrics should be in required metrics
            for expected_metric in &expected_metrics {
                assert!(required_metrics.contains(expected_metric),
                       "Should require {} metric in {} OTEL performance tracking", expected_metric, benchmark_id);
            }

            // Validate basic structure
            let weight = metrics
                .get("weight")
                .and_then(|w| w.as_f64())
                .unwrap_or(0.0);

            let description = metrics
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");

            assert!(weight > 0.0, "Performance metrics should have positive weight in {}", benchmark_id);
            assert!(!description.is_empty(), "Performance metrics should have description in {}", benchmark_id);

            // Validate that execution_time_ms is always included (core metric)
            assert!(required_metrics.contains("execution_time_ms"),
                   "Should always track execution_time_ms in {}", benchmark_id);
        }

        println!("  ✅ Performance metrics validated");
    }

    println!("\n✅ Performance metrics tracking validation passed");
    Ok(())
}

#[tokio::test]
async fn test_300_series_otel_weight_distribution() -> anyhow::Result<()> {
    println!("🎯 Testing 300-Series: OTEL Weight Distribution");

    let benchmark_ids = vec![
        "300-jup-swap-then-lend-deposit-dyn",
        "301-dynamic-yield-optimization",
        "302-portfolio-rebalancing",
        "303-risk-adjusted-growth",
        "304-emergency-exit-strategy",
        "305-yield-farming-optimization",
    ];

    for benchmark_id in benchmark_ids {
        println!("\n⚖️ Validating OTEL weight distribution for: {}", benchmark_id);

        let benchmark = load_benchmark_yaml(benchmark_id);

        // Extract OTEL tracking sections
        let expected_otel = benchmark
            .get("ground_truth")
            .and_then(|gt| gt.get("expected_otel_tracking"))
            .and_then(|eot| eot.as_sequence())
            .unwrap_or(&serde_json::json!([]));

        let mut total_weight = 0.0;
        let mut weight_distribution = HashMap::new();

        for tracking in expected_otel.as_sequence().unwrap_or(&serde_json::json!([])) {
            let tracking_type = tracking
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown");

            let weight = tracking
                .get("weight")
                .and_then(|w| w.as_f64())
                .unwrap_or(0.0);

            total_weight += weight;
            weight_distribution.insert(tracking_type.to_string(), weight);

            println!("  {}: weight {:.2}", tracking_type, weight);
        }

        println!("  Total weight: {:.2}", total_weight);

        // Validate weight distribution
        assert!((total_weight - 1.0).abs() < 0.01,
               "OTEL tracking weights should sum to 1.0 in {}, got {:.2}",
               benchmark_id, total_weight);

        // Validate that all major tracking types have reasonable weights
        let expected_types = vec![
            "tool_call_logging", "execution_tracing", "mermaid_generation", "performance_metrics"
        ];

        for expected_type in &expected_types {
            let weight = weight_distribution.get(expected_type).unwrap_or(&0.0);
            assert!(*weight > 0.0,
                   "Tracking type {} should have positive weight in {}",
                   expected_type, benchmark_id);
            assert!(*weight <= 0.5,
                   "Tracking type {} weight should not exceed 0.5 in {}, got {:.2}",
                   expected_type, benchmark_id, weight);
        }

        println!("  ✅ Weight distribution validated");
    }

    println!("\n✅ OTEL weight distribution validation passed");
    Ok(())
}

#[tokio::test]
async fn test_300_series_otel_description_completeness() -> anyhow::Result<()> {
    println!("🎯 Testing 300-Series: OTEL Description Completeness");

    let benchmark_ids = vec![
        "300-jup-swap-then-lend-deposit-dyn",
        "301-dynamic-yield-optimization",
        "302-portfolio-rebalancing",
        "303-risk-adjusted-growth",
        "304-emergency-exit-strategy",
        "305-yield-farming-optimization",
    ];

    for benchmark_id in benchmark_ids {
        println!("\n📝 Validating OTEL descriptions for: {}", benchmark_id);

        let benchmark = load_benchmark_yaml(benchmark_id);

        // Extract OTEL tracking sections
        let expected_otel = benchmark
            .get("ground_truth")
            .and_then(|gt| gt.get("expected_otel_tracking"))
            .and_then(|eot| eot.as_sequence())
            .unwrap_or(&serde_json::json!([]));

        for tracking in expected_otel.as_sequence().unwrap_or(&serde_json::json!([])) {
            let tracking_type = tracking
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown");

            let description = tracking
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");

            println!("  {}: '{}'", tracking_type, description);

            // Validate description quality
            assert!(!description.is_empty(),
                   "OTEL tracking type {} should have description in {}",
                   tracking_type, benchmark_id);

            assert!(description.len() >= 10,
                   "OTEL tracking description should be substantial for {} in {}, got: '{}'",
                   tracking_type, benchmark_id, description);

            // Description should mention the benchmark context
            let context_keywords = vec!["tool", "track", "execution", "flow"];
            let has_context_keyword = context_keywords.iter()
                .any(|keyword| description.to_lowercase().contains(keyword));

            assert!(has_context_keyword,
                   "OTEL tracking description for {} in {} should contain context keywords, got: '{}'",
                   tracking_type, benchmark_id, description);
        }

        println!("  ✅ OTEL descriptions validated");
    }

    println!("\n✅ OTEL description completeness validation passed");
    Ok(())
}

#[tokio::test]
async fn test_300_series_otel_critical_path_tracking() -> anyhow::Result<()> {
    println!("🎯 Testing 300-Series: OTEL Critical Path Tracking");

    let benchmark_ids = vec![
        "300-jup-swap-then-lend-deposit-dyn",
        "301-dynamic-yield-optimization",
        "302-portfolio-rebalancing",
        "303-risk-adjusted-growth",
        "304-emergency-exit-strategy",
        "305-yield-farming-optimization",
    ];

    for benchmark_id in benchmark_ids {
        println!("\n🛤️ Validating critical path tracking for: {}", benchmark_id);

        let benchmark = load_benchmark_yaml(benchmark_id);

        // Extract critical tools from expected_tool_calls
        let critical_tools: HashSet<String> = benchmark
            .get("ground_truth")
            .and_then(|gt| gt.get("expected_tool_calls"))
            .and_then(|etc| etc.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter(|tool| {
                        tool.get("critical")
                            .and_then(|c| c.as_bool())
                            .unwrap_or(false)
                    })
                    .filter_map(|tool| tool.get("tool_name"))
                    .filter_map(|name| name.as_str())
                    .map(|s| s.to_string())
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();

        println!("  Critical tools: {:?}", critical_tools);

        // Validate that tool_call_logging includes critical tools
        let tool_logging = benchmark
            .get("ground_truth")
            .and_then(|gt| gt.get("expected_otel_tracking"))
            .and_then(|eot| eot.as_sequence())
            .and_then(|seq| {
                seq.iter().find(|tracking| {
                    tracking.get("type")
                        .and_then(|t| t.as_str())
                        .map(|t| t == "tool_call_logging")
                        .unwrap_or(false)
                })
            });

        if let Some(logging) = tool_logging {
            let logged_tools = logging
                .get("required_tools")
                .and_then(|rt| rt.as_sequence())
                .map(|seq| {
                    seq.iter()
                        .filter_map(|tool| tool.as_str())
                        .collect::<HashSet<_>>()
                })
                .unwrap_or_default();

            // All critical tools should be in logged tools
            for critical_tool in &critical_tools {
                assert!(logged_tools.contains(critical_tool),
                       "Critical tool {} should be tracked in OTEL logging for {}",
                       critical_tool, benchmark_id);
            }

            // Critical tools should have higher tracking weight
            let critical_weight_bonus = if critical_tools.len() > 2 { 0.1 } else { 0.05 };
            let weight = logging
                .get("weight")
                .and_then(|w| w.as_f64())
                .unwrap_or(0.0);

            assert!(weight >= 0.25 + critical_weight_bonus,
                   "Tool call logging weight should account for critical tools in {}, got {:.2}",
                   benchmark_id, weight);
        }

        println!("  ✅ Critical path tracking validated");
    }

    println!("\n✅ OTEL critical path tracking validation passed");
    Ok(())
}
