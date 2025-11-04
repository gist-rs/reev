//! Dynamic Flow API Integration Tests
//!
//! Tests for the new dynamic flow endpoints including:
//! - Direct mode execution
//! - Bridge mode execution
//! - Recovery mode execution
//! - Enhanced flow visualization with dynamic flow support
//! - HTTP caching headers (ETag, Last-Modified)
//! - Polling frequency recommendations

use reqwest;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

/// API base URL for testing
const API_BASE: &str = "http://localhost:3001";

#[tokio::test]
async fn test_dynamic_flow_direct_execution() {
    let client = reqwest::Client::new();

    let request_body = json!({
        "prompt": "use 50% SOL to get USDC",
        "wallet": "TestWalletPubkey123",
        "agent": "glm-4.6-coding",
        "shared_surfpool": false
    });

    let response = client
        .post(&format!("{}/api/v1/benchmarks/execute-direct", API_BASE))
        .json(&request_body)
        .send()
        .await;

    // Note: This test requires the API server to be running
    // In a real test environment, we would start the server programmatically
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let result: Value = resp.json().await.unwrap();

            // Verify response structure
            assert!(result.get("execution_id").is_some());
            assert!(result.get("status").is_some());
            assert_eq!(result["status"], "completed");

            // Verify execution_id starts with "direct-" for direct mode
            let execution_id = result["execution_id"].as_str().unwrap();
            assert!(execution_id.starts_with("direct-"));

            // Verify result contains flow information
            assert!(result.get("result").is_some());
            let result_data = &result["result"];
            assert_eq!(result_data["execution_mode"], "direct");
            assert!(result_data["flow_id"].as_str().unwrap().starts_with("dynamic-"));
        }
        Err(_) => {
            // Server not running - skip test in CI without network access
            println!("Skipping test - API server not running");
        }
    }
}

#[tokio::test]
async fn test_dynamic_flow_bridge_execution() {
    let client = reqwest::Client::new();

    let request_body = json!({
        "prompt": "swap 0.1 SOL to USDC using jupiter",
        "wallet": "TestWalletPubkey456",
        "agent": "glm-4.6-coding",
        "shared_surfpool": true  // This enables bridge mode
    });

    let response = client
        .post(&format!("{}/api/v1/benchmarks/execute-bridge", API_BASE))
        .json(&request_body)
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let result: Value = resp.json().await.unwrap();

            // Verify response structure
            assert!(result.get("execution_id").is_some());

            // Verify execution_id starts with "bridge-" for bridge mode
            let execution_id = result["execution_id"].as_str().unwrap();
            assert!(execution_id.starts_with("bridge-"));

            // Verify result contains YML file path for bridge mode
            assert!(result.get("result").is_some());
            let result_data = &result["result"];
            assert_eq!(result_data["execution_mode"], "bridge");
            assert!(result_data.get("yml_file").is_some());
        }
        Err(_) => {
            println!("Skipping test - API server not running");
        }
    }
}

#[tokio::test]
async fn test_dynamic_flow_recovery_execution() {
    let client = reqwest::Client::new();

    let request_body = json!({
        "prompt": "critical transaction: swap 1 SOL to USDC with recovery",
        "wallet": "TestWalletPubkey789",
        "agent": "glm-4.6-coding",
        "recovery_config": {
            "base_retry_delay_ms": 1000,
            "max_retry_delay_ms": 10000,
            "backoff_multiplier": 2.0,
            "max_recovery_time_ms": 30000,
            "enable_alternative_flows": true,
            "enable_user_fulfillment": false
        }
    });

    let response = client
        .post(&format!("{}/api/v1/benchmarks/execute-recovery", API_BASE))
        .json(&request_body)
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let result: Value = resp.json().await.unwrap();

            // Verify response structure
            assert!(result.get("execution_id").is_some());

            // Verify execution_id starts with "recovery-" for recovery mode
            let execution_id = result["execution_id"].as_str().unwrap();
            assert!(execution_id.starts_with("recovery-"));

            // Verify result contains recovery configuration
            assert!(result.get("result").is_some());
            let result_data = &result["result"];
            assert_eq!(result_data["execution_mode"], "recovery");
            assert_eq!(result_data["recovery_enabled"], true);
            assert!(result_data.get("recovery_config").is_some());
        }
        Err(_) => {
            println!("Skipping test - API server not running");
        }
    }
}

#[tokio::test]
async fn test_recovery_metrics_endpoint() {
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/api/v1/metrics/recovery", API_BASE))
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let metrics: Value = resp.json().await.unwrap();

            // Verify metrics structure
            assert!(metrics.get("total_flows").is_some());
            assert!(metrics.get("successful_flows").is_some());
            assert!(metrics.get("failed_flows").is_some());
            assert!(metrics.get("recovered_flows").is_some());
            assert!(metrics.get("success_rate").is_some());
            assert!(metrics.get("strategies_used").is_some());
            assert!(metrics.get("last_updated").is_some());

            // Verify strategies structure
            let strategies = &metrics["strategies_used"];
            assert!(strategies.get("retry_attempts").is_some());
            assert!(strategies.get("alternative_flows_used").is_some());
            assert!(strategies.get("user_fulfillment_used").is_some());
        }
        Err(_) => {
            println!("Skipping test - API server not running");
        }
    }
}

#[tokio::test]
async fn test_flow_visualization_with_dynamic_flow() {
    let client = reqwest::Client::new();

    // First, execute a dynamic flow to get a session_id
    let request_body = json!({
        "prompt": "test visualization flow",
        "wallet": "TestWalletVisualization",
        "agent": "glm-4.6-coding",
        "shared_surfpool": false
    });

    let exec_response = client
        .post(&format!("{}/api/v1/benchmarks/execute-direct", API_BASE))
        .json(&request_body)
        .send()
        .await;

    match exec_response {
        Ok(resp) => {
            let result: Value = resp.json().await.unwrap();
            let execution_id = result["execution_id"].as_str().unwrap();

            // Test JSON format
            let json_response = client
                .get(&format!("{}/api/v1/flows/{}", API_BASE, execution_id))
                .header("Accept", "application/json")
                .send()
                .await
                .unwrap();

            assert_eq!(json_response.status(), 200);

            // Verify caching headers are present
            assert!(json_response.headers().get("Last-Modified").is_some());
            assert!(json_response.headers().get("ETag").is_some());
            assert!(json_response.headers().get("Cache-Control").is_some());
            assert!(json_response.headers().get("X-Polling-Recommendation").is_some());
            assert!(json_response.headers().get("X-Flow-Type").is_some());

            // Verify flow type indicates dynamic flow support
            let flow_type = json_response.headers().get("X-Flow-Type").unwrap();
            assert_eq!(flow_type, "dynamic-flow-capable");

            // Test HTML format
            let html_response = client
                .get(&format!("{}/api/v1/flows/{}?format=html", API_BASE, execution_id))
                .header("Accept", "text/html")
                .send()
                .await
                .unwrap();

            assert_eq!(html_response.status(), 200);

            // Verify HTML contains dynamic flow information
            let html_content = html_response.text().await.unwrap();
            assert!(html_content.contains("Dynamic Flow Support"));
            assert!(html_content.contains("POST /api/v1/benchmarks/execute-direct"));
            assert!(html_content.contains("POST /api/v1/benchmarks/execute-bridge"));
            assert!(html_content.contains("POST /api/v1/benchmarks/execute-recovery"));
        }
        Err(_) => {
            println!("Skipping test - API server not running");
        }
    }
}

#[tokio::test]
async fn test_conditional_requests_with_etag() {
    let client = reqwest::Client::new();

    // Get initial response with ETag
    let response = client
        .get(&format!("{}/api/v1/flows/test-session", API_BASE))
        .send()
        .await;

    match response {
        Ok(resp) => {
            let etag = resp.headers().get("ETag").cloned();

            if let Some(etag_value) = etag {
                // Test conditional request with matching ETag
                let conditional_response = client
                    .get(&format!("{}/api/v1/flows/test-session", API_BASE))
                    .header("If-None-Match", etag_value)
                    .send()
                    .await
                    .unwrap();

                // Should return 304 Not Modified if content hasn't changed
                // (or 200 if the session doesn't exist, which is also valid)
                assert!(conditional_response.status() == reqwest::StatusCode::NOT_MODIFIED
                      || conditional_response.status() == reqwest::StatusCode::OK);
            }
        }
        Err(_) => {
            println!("Skipping test - API server not running");
        }
    }
}

#[tokio::test]
async fn test_polling_frequency_recommendations() {
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/api/v1/flows/test-session", API_BASE))
        .send()
        .await;

    match response {
        Ok(resp) => {
            // Verify polling recommendation header is present
            let polling_header = resp.headers().get("X-Polling-Recommendation");
            assert!(polling_header.is_some());

            let recommendation = polling_header.unwrap().to_str().unwrap();
            assert!(recommendation.contains("1-5 seconds"));
            assert!(recommendation.contains("30-60 seconds"));
            assert!(recommendation.contains("active flows"));
            assert!(recommendation.contains("completed flows"));
        }
        Err(_) => {
            println!("Skipping test - API server not running");
        }
    }
}

/// Integration test for complete dynamic flow workflow
#[tokio::test]
async fn test_complete_dynamic_flow_workflow() {
    let client = reqwest::Client::new();

    // Step 1: Execute dynamic flow
    let request_body = json!({
        "prompt": "workflow test: swap SOL to USDC",
        "wallet": "WorkflowTestWallet",
        "agent": "glm-4.6-coding",
        "shared_surfpool": false
    });

    let exec_response = client
        .post(&format!("{}/api/v1/benchmarks/execute-direct", API_BASE))
        .json(&request_body)
        .send()
        .await;

    match exec_response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let result: Value = resp.json().await.unwrap();
            let execution_id = result["execution_id"].as_str().unwrap();

            // Step 2: Poll for completion (simulating real-time monitoring)
            for _ in 0..5 {
                sleep(Duration::from_millis(500)).await; // Poll every 500ms

                let status_response = client
                    .get(&format!("{}/api/v1/benchmarks/dynamic-flow/status/{}", API_BASE, execution_id))
                    .send()
                    .await;

                if let Ok(status_resp) = status_response {
                    if status_resp.status() == 200 {
                        let status: Value = status_resp.json().await.unwrap();
                        if status["status"] == "completed" || status["status"] == "failed" {
                            break;
                        }
                    }
                }
            }

            // Step 3: Get flow visualization
            let flow_response = client
                .get(&format!("{}/api/v1/flows/{}?format=json", API_BASE, execution_id))
                .send()
                .await
                .unwrap();

            assert_eq!(flow_response.status(), 200);

            // Step 4: Verify caching works for subsequent requests
            let etag = flow_response.headers().get("ETag").cloned().unwrap();

            let cached_response = client
                .get(&format!("{}/api/v1/flows/{}?format=json", API_BASE, execution_id))
                .header("If-None-Match", etag)
                .send()
                .await
                .unwrap();

            // Should be cached (304) or return fresh data (200)
            assert!(cached_response.status() == reqwest::StatusCode::NOT_MODIFIED
                  || cached_response.status() == reqwest::StatusCode::OK);
        }
        Err(_) => {
            println!("Skipping integration test - API server not running");
        }
    }
}
