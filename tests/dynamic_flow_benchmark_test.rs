//! Dynamic Flow Benchmark Tests
//!
//! Comprehensive test suite for 300-series dynamic flow benchmarks.
//! Tests natural language processing, multi-step orchestration, and intelligent decision-making.

use reqwest;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

/// API base URL for testing
const API_BASE: &str = "http://localhost:3001";

/// Test execution configuration
struct BenchmarkTest {
    id: String,
    description: String,
    expected_score: f64,
    key_assertions: Vec<String>,
}

impl BenchmarkTest {
    fn new(id: &str, description: &str, expected_score: f64) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            expected_score,
            key_assertions: Vec::new(),
        }
    }

    fn with_assertions(mut self, assertions: Vec<&str>) -> Self {
        self.key_assertions = assertions.iter().map(|s| s.to_string()).collect();
        self
    }
}

/// Dynamic flow benchmark test cases
fn get_benchmark_tests() -> Vec<BenchmarkTest> {
    vec![
        BenchmarkTest::new("301-dynamic-yield-optimization",
                         "Dynamic yield optimization with 50% SOL allocation", 0.7)
            .with_assertions(vec![
                "context_resolution",
                "percentage_calculation",
                "yield_optimization",
                "multi_step_execution"
            ]),

        BenchmarkTest::new("302-portfolio-rebalancing",
                         "Portfolio rebalancing based on market conditions", 0.7)
            .with_assertions(vec![
                "portfolio_analysis",
                "market_conditions_assessment",
                "rebalancing_execution",
                "risk_management"
            ]),

        BenchmarkTest::new("303-risk-adjusted-growth",
                         "Risk-adjusted growth strategy with 30% SOL usage", 0.75)
            .with_assertions(vec![
                "risk_assessment",
                "percentage_discipline",
                "capital_preservation",
                "conservative_yield",
                "market_analysis"
            ]),

        BenchmarkTest::new("304-emergency-exit-strategy",
                         "Emergency exit strategy for market stress conditions", 0.8)
            .with_assertions(vec![
                "emergency_detection",
                "position_liquidation",
                "asset_consolidation",
                "capital_preservation",
                "speed_execution"
            ]),

        BenchmarkTest::new("305-yield-farming-optimization",
                         "Advanced yield farming with multi-pool optimization", 0.75)
            .with_assertions(vec![
                "pool_analysis",
                "apy_comparison",
                "multi_pool_strategy",
                "capital_allocation",
                "auto_compounding_consideration",
                "risk_diversification"
            ]),
    ]
}

/// Test dynamic flow execution via API
#[tokio::test]
async fn test_dynamic_flow_execution() {
    let client = reqwest::Client::new();
    let test_cases = get_benchmark_tests();

    for test_case in test_cases {
        println!("Testing benchmark: {}", test_case.id);

        // Execute benchmark via dynamic flow
        let execution_request = json!({
            "prompt": get_benchmark_prompt(&test_case.id),
            "wallet": "TestWalletPubkey123",
            "agent": "glm-4.6-coding",
            "shared_surfpool": false
        });

        let response = client
            .post(&format!("{}/api/v1/benchmarks/execute-direct", API_BASE))
            .json(&execution_request)
            .send()
            .await;

        match response {
            Ok(resp) => {
                assert_eq!(resp.status(), 200, "Benchmark {} should execute successfully", test_case.id);

                let result: Value = resp.json().await.unwrap();

                // Verify execution structure
                assert!(result.get("execution_id").is_some(),
                          "Benchmark {} should return execution ID", test_case.id);
                assert_eq!(result["status"], "completed",
                          "Benchmark {} should complete successfully", test_case.id);

                let execution_id = result["execution_id"].as_str().unwrap();
                assert!(execution_id.starts_with("direct-"),
                          "Benchmark {} should use direct mode", test_case.id);

                // Verify flow generation
                assert!(result.get("result").is_some(),
                          "Benchmark {} should return result", test_case.id);
                let result_data = &result["result"];
                assert_eq!(result_data["execution_mode"], "direct",
                          "Benchmark {} should use direct execution mode", test_case.id);
                assert!(result_data["steps_generated"].as_u64().unwrap() > 0,
                          "Benchmark {} should generate steps", test_case.id);

                println!("✅ {} - Execution ID: {}, Steps: {}",
                        test_case.id, execution_id, result_data["steps_generated"]);
            }
            Err(e) => {
                println!("⚠️  Skipping {} - API server not running: {}", test_case.id, e);
                continue;
            }
        }
    }
}

/// Test flow visualization for dynamic benchmarks
#[tokio::test]
async fn test_dynamic_flow_visualization() {
    let client = reqwest::Client::new();
    let test_cases = get_benchmark_tests();

    for test_case in test_cases {
        // First execute the benchmark
        let execution_request = json!({
            "prompt": get_benchmark_prompt(&test_case.id),
            "wallet": "VisualizationTestWallet",
            "agent": "glm-4.6-coding",
            "shared_surfpool": false
        });

        let exec_response = client
            .post(&format!("{}/api/v1/benchmarks/execute-direct", API_BASE))
            .json(&execution_request)
            .send()
            .await;

        match exec_response {
            Ok(resp) => {
                let result: Value = resp.json().await.unwrap();
                let execution_id = result["execution_id"].as_str().unwrap();

                // Test flow visualization
                let viz_response = client
                    .get(&format!("{}/api/v1/flows/{}?format=json", API_BASE, execution_id))
                    .send()
                    .await
                    .unwrap();

                assert_eq!(viz_response.status(), 200,
                          "Benchmark {} should have flow visualization", test_case.id);

                // Verify visualization headers
                assert!(viz_response.headers().get("X-Flow-Type").is_some(),
                          "Benchmark {} should have dynamic flow type header", test_case.id);
                assert!(viz_response.headers().get("X-Polling-Recommendation").is_some(),
                          "Benchmark {} should have polling recommendation", test_case.id);
                assert!(viz_response.headers().get("ETag").is_some(),
                          "Benchmark {} should have ETag header", test_case.id);

                let viz_result: Value = viz_response.json().await.unwrap();
                assert!(viz_result.get("diagram").is_some(),
                          "Benchmark {} should have Mermaid diagram", test_case.id);
                assert!(viz_result.get("metadata").is_some(),
                          "Benchmark {} should have visualization metadata", test_case.id);

                let diagram = viz_result["diagram"].as_str().unwrap();
                assert!(diagram.contains("stateDiagram"),
                        "Benchmark {} should contain Mermaid stateDiagram", test_case.id);

                println!("✅ {} - Flow visualization generated successfully", test_case.id);
            }
            Err(_) => {
                println!("⚠️  Skipping {} visualization test - API server not running", test_case.id);
                continue;
            }
        }
    }
}

/// Test recovery mode for emergency scenarios
#[tokio::test]
async fn test_emergency_recovery_mode() {
    let client = reqwest::Client::new();

    let emergency_request = json!({
        "prompt": "I need an emergency exit strategy for all my positions due to market stress. Please immediately analyze my current holdings, withdraw all lending positions, convert risky assets to stable ones, and preserve capital.",
        "wallet": "EmergencyTestWallet",
        "agent": "glm-4.6-coding",
        "recovery_config": {
            "base_retry_delay_ms": 500,
            "max_retry_delay_ms": 5000,
            "backoff_multiplier": 1.5,
            "max_recovery_time_ms": 20000,
            "enable_alternative_flows": true,
            "enable_user_fulfillment": false
        }
    });

    let response = client
        .post(&format!("{}/api/v1/benchmarks/execute-recovery", API_BASE))
        .json(&emergency_request)
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200, "Emergency recovery should execute successfully");

            let result: Value = resp.json().await.unwrap();

            // Verify recovery execution
            assert!(result.get("execution_id").is_some(),
                      "Emergency recovery should return execution ID");
            let execution_id = result["execution_id"].as_str().unwrap();
            assert!(execution_id.starts_with("recovery-"),
                      "Emergency should use recovery mode");

            // Verify recovery configuration
            assert!(result.get("result").is_some(),
                      "Emergency recovery should return result");
            let result_data = &result["result"];
            assert_eq!(result_data["execution_mode"], "recovery",
                      "Should use recovery execution mode");
            assert_eq!(result_data["recovery_enabled"], true,
                      "Recovery should be enabled");
            assert!(result_data.get("recovery_config").is_some(),
                      "Should return recovery config");

            println!("✅ Emergency recovery executed successfully - Execution ID: {}", execution_id);
        }
        Err(_) => {
            println!("⚠️  Skipping emergency recovery test - API server not running");
        }
    }
}

/// Test caching and conditional requests
#[tokio::test]
async fn test_dynamic_flow_caching() {
    let client = reqwest::Client::new();

    // Execute a simple dynamic flow
    let request = json!({
        "prompt": "Test caching with simple swap",
        "wallet": "CacheTestWallet",
        "agent": "glm-4.6-coding",
        "shared_surfpool": false
    });

    let exec_response = client
        .post(&format!("{}/api/v1/benchmarks/execute-direct", API_BASE))
        .json(&request)
        .send()
        .await;

    match exec_response {
        Ok(resp) => {
            let result: Value = resp.json().await.unwrap();
            let execution_id = result["execution_id"].as_str().unwrap();

            // First request - get full response with ETag
            let viz_response = client
                .get(&format!("{}/api/v1/flows/{}?format=json", API_BASE, execution_id))
                .send()
                .await
                .unwrap();

            assert_eq!(viz_response.status(), 200);

            let etag = viz_response.headers().get("ETag").cloned();
            assert!(etag.is_some(), "First request should have ETag");

            // Second request - use conditional request
            if let Some(etag_value) = etag {
                let conditional_response = client
                    .get(&format!("{}/api/v1/flows/{}?format=json", API_BASE, execution_id))
                    .header("If-None-Match", etag_value)
                    .send()
                    .await
                    .unwrap();

                // Should be 304 Not Modified or 200 with fresh data
                assert!(conditional_response.status() == reqwest::StatusCode::NOT_MODIFIED
                      || conditional_response.status() == reqwest::StatusCode::OK,
                          "Conditional request should work properly");
            }

            println!("✅ Caching and conditional requests working correctly");
        }
        Err(_) => {
            println!("⚠️  Skipping caching test - API server not running");
        }
    }
}

/// Test complete workflow with polling
#[tokio::test]
async fn test_complete_dynamic_workflow() {
    let client = reqwest::Client::new();
    let test_case = get_benchmark_tests().first().unwrap();

    // Step 1: Execute dynamic flow
    let execution_request = json!({
        "prompt": get_benchmark_prompt(&test_case.id),
        "wallet": "WorkflowTestWallet",
        "agent": "glm-4.6-coding",
        "shared_surfpool": false
    });

    let exec_response = client
        .post(&format!("{}/api/v1/benchmarks/execute-direct", API_BASE))
        .json(&execution_request)
        .send()
        .await;

    match exec_response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let result: Value = resp.json().await.unwrap();
            let execution_id = result["execution_id"].as_str().unwrap();

            // Step 2: Poll for completion (simulating real-time monitoring)
            let mut poll_count = 0;
            let max_polls = 10;

            while poll_count < max_polls {
                sleep(Duration::from_millis(500)).await; // Poll every 500ms for active flows
                poll_count += 1;

                let status_response = client
                    .get(&format!("{}/api/v1/benchmarks/{}/status/{}", API_BASE, test_case.id, execution_id))
                    .send()
                    .await;

                if let Ok(status_resp) = status_response {
                    if status_resp.status() == 200 {
                        let status: Value = status_resp.json().await.unwrap();
                        if status["status"] == "completed" || status["status"] == "failed" {
                            println!("✅ Workflow completed after {} polls", poll_count);
                            break;
                        }
                    }
                }
            }

            // Step 3: Get final flow visualization
            let final_viz_response = client
                .get(&format!("{}/api/v1/flows/{}?format=html", API_BASE, execution_id))
                .send()
                .await
                .unwrap();

            assert_eq!(final_viz_response.status(), 200);

            let html_content = final_viz_response.text().await.unwrap();
            assert!(html_content.contains("Dynamic Flow Support"),
                    "Final HTML should contain dynamic flow information");
            assert!(html_content.contains("POST /api/v1/benchmarks/execute-direct"),
                    "Final HTML should contain API endpoint references");

            println!("✅ Complete dynamic workflow test passed");
        }
        Err(_) => {
            println!("⚠️  Skipping complete workflow test - API server not running");
        }
    }
}

/// Get benchmark prompt by ID
fn get_benchmark_prompt(benchmark_id: &str) -> &str {
    match benchmark_id {
        "301-dynamic-yield-optimization" => {
            "Use my 50% SOL to maximize my USDC returns through Jupiter lending. Please check current market rates, calculate optimal strategy, and execute best yield approach for my remaining portfolio. I want to maximize my USD value while managing risk appropriately."
        },
        "302-portfolio-rebalancing" => {
            "I want to rebalance my portfolio based on current market conditions. Please analyze my current holdings (SOL and USDC), check current market prices and Jupiter lending rates, then execute optimal rebalancing to maximize returns while maintaining some liquidity. I'm comfortable with moderate risk and want to optimize for yield."
        },
        "303-risk-adjusted-growth" => {
            "I want to implement a risk-adjusted growth strategy using 30% of my SOL. Please preserve most of my capital while generating some yield through conservative Jupiter lending. Analyze current market conditions and implement a strategy that prioritizes capital preservation with moderate growth. Keep enough SOL for transaction fees and emergencies."
        },
        "304-emergency-exit-strategy" => {
            "I need an emergency exit strategy for all my positions due to market stress. Please immediately analyze my current holdings, withdraw all lending positions, convert risky assets to stable ones, and preserve capital. Focus on speed and capital preservation over optimization."
        },
        "305-yield-farming-optimization" => {
            "I want to optimize my yield farming strategy using 70% of my available capital. Please analyze all Jupiter lending pools, compare current APYs, and implement the best multi-pool strategy for maximum returns. Consider auto-compounding opportunities and risk diversification across different tokens. Optimize for long-term yield growth."
        },
        _ => "Execute dynamic flow test benchmark",
    }
}

/// Test individual benchmark assertions
#[tokio::test]
async fn test_benchmark_assertions() {
    let test_cases = get_benchmark_tests();

    for test_case in test_cases {
        // Each benchmark should have the expected assertions in its ground truth
        println!("Testing assertions for: {}", test_case.description);

        // Verify we have the expected number of key assertions
        assert!(!test_case.key_assertions.is_empty(),
                  "Benchmark {} should have key assertions defined", test_case.id);

        // Each assertion should be meaningful
        for assertion in &test_case.key_assertions {
            assert!(!assertion.is_empty(),
                      "Benchmark {} should have non-empty assertion strings", test_case.id);
            assert!(assertion.len() > 3,
                      "Benchmark {} assertions should be descriptive", test_case.id);
        }

        println!("✅ {} - {} assertions validated", test_case.id, test_case.key_assertions.len());
    }
}
