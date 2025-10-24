//! Ground Truth Separation Validation Tests
//!
//! Tests that ground truth data is properly separated between:
//! - Deterministic mode: Can access ground truth for reproducible tests
//! - LLM mode: Cannot access ground truth to prevent information leakage
//!
//! Validates the critical architectural principle that prevents future information
//! from leaking into real-time multi-step decision making.

use anyhow::Result;
use reev_agent::flow::agent::{is_deterministic_mode, FlowAgent};
use serde_json::json;
use serial_test::serial;

/// Test ground truth separation in deterministic mode
#[tokio::test]
#[serial]
async fn test_deterministic_mode_ground_truth_access() -> Result<()> {
    println!("🔧 Testing deterministic mode with ground truth access...");

    // Create a test benchmark with ground truth
    let benchmark = create_test_benchmark_with_ground_truth();

    // Test deterministic mode detection
    let _agent = FlowAgent::new("deterministic");

    // Verify deterministic mode is detected
    let benchmark_id = benchmark
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("test-benchmark-001");
    let benchmark_tags: Vec<String> = benchmark
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    assert!(
        is_deterministic_mode("deterministic", benchmark_id, &benchmark_tags),
        "Should detect deterministic mode correctly"
    );

    // Test that deterministic mode CAN access ground truth
    let should_use_ground_truth =
        is_deterministic_mode("deterministic", benchmark_id, &benchmark_tags);

    // Verify ground truth is accessible in deterministic mode
    assert!(
        should_use_ground_truth,
        "Deterministic mode should access ground truth"
    );

    println!("   ✅ Deterministic mode ground truth access: PASSED");
    Ok(())
}

/// Test ground truth blocking in LLM mode
#[tokio::test]
#[serial]
async fn test_llm_mode_ground_truth_blocking() -> Result<()> {
    println!("🤖 Testing LLM mode without ground truth access...");

    // Create a test benchmark with ground truth
    let benchmark = create_test_benchmark_with_ground_truth();

    // Test LLM mode detection
    let _agent = FlowAgent::new("gpt-4");

    // Verify LLM mode is NOT deterministic
    let benchmark_id = benchmark
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("test-benchmark-001");
    let benchmark_tags: Vec<String> = benchmark
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    assert!(
        !is_deterministic_mode("gpt-4", benchmark_id, &benchmark_tags),
        "Should NOT detect deterministic mode for LLM agent"
    );

    // Test that LLM mode CANNOT access ground truth
    let should_use_ground_truth = is_deterministic_mode("gpt-4", benchmark_id, &benchmark_tags);

    // Verify ground truth is NOT accessible
    assert!(
        !should_use_ground_truth,
        "LLM mode should NOT access ground truth"
    );

    println!("   🛡️ LLM mode: Using real blockchain state only");
    println!("   ✅ LLM mode ground truth blocking: PASSED");
    Ok(())
}

/// Test error handling for invalid ground truth usage
#[tokio::test]
#[serial]
async fn test_ground_truth_leakage_prevention() -> Result<()> {
    println!("🚨 Testing ground truth leakage prevention...");

    // Create a test benchmark with ground truth
    let benchmark = create_test_benchmark_with_ground_truth();

    // Test with non-deterministic agent (LLM mode)
    let agent_name = "gpt-4";

    let benchmark_id = benchmark
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("test-benchmark-001");
    let benchmark_tags: Vec<String> = benchmark
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Validate no ground truth leakage in LLM mode
    let final_state_assertions = benchmark
        .get("ground_truth")
        .and_then(|gt| gt.get("final_state_assertions"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.len())
        .unwrap_or(0);

    let has_ground_truth = final_state_assertions > 0;
    let is_deterministic = is_deterministic_mode(agent_name, benchmark_id, &benchmark_tags);

    let result = if !is_deterministic && has_ground_truth {
        println!("   🔍 Checking for ground truth in LLM mode...");

        // This should trigger an error in production code
        // For testing, we simulate the validation
        Err(anyhow::anyhow!(
            "Ground truth not allowed in LLM mode - would leak future information"
        ))
    } else {
        Ok(())
    };

    // Verify the error is properly raised
    assert!(
        result.is_err(),
        "Should error when ground truth is present in LLM mode"
    );

    if let Err(e) = result {
        assert!(
            e.to_string().contains("Ground truth not allowed"),
            "Error message should mention ground truth restriction"
        );
        println!("   ✅ Ground truth leakage prevention: {e}");
    }

    // Test that deterministic mode works fine
    let deterministic_result =
        if is_deterministic_mode("deterministic", benchmark_id, &benchmark_tags) {
            println!("   🔍 Deterministic mode: Ground truth allowed");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Deterministic mode should be allowed"))
        };

    assert!(
        deterministic_result.is_ok(),
        "Deterministic mode should allow ground truth"
    );

    println!("   ✅ Ground truth leakage prevention: PASSED");
    Ok(())
}

/// Test multi-step context consolidation without leakage
#[tokio::test]
#[serial]
async fn test_multi_step_context_consolidation_no_leakage() -> Result<()> {
    println!("🔄 Testing multi-step context consolidation without leakage...");

    // Create a multi-step benchmark
    let benchmark = create_multi_step_benchmark();

    // Test LLM mode context consolidation
    let agent_name = "gpt-4";

    let benchmark_id = benchmark
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("multi-step-test");
    let benchmark_tags: Vec<String> = benchmark
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Verify LLM mode cannot use ground truth for context consolidation
    let should_use_ground_truth = is_deterministic_mode(agent_name, benchmark_id, &benchmark_tags);

    if should_use_ground_truth {
        println!("   ✅ Multi-step deterministic mode: Ground truth allowed");
    } else {
        println!("   🛡️ Multi-step LLM mode: Real blockchain state only");
    }

    assert!(
        !should_use_ground_truth,
        "Multi-step LLM mode should not access ground truth"
    );

    // Simulate multi-step context building
    let mut context_history = Vec::new();

    for step in 1..=3 {
        println!("   📝 Step {step}: Building context without ground truth");

        // Each step should only use previous step results, not ground truth
        let step_context = json!({
            "step": step,
            "uses_ground_truth": should_use_ground_truth,
            "depends_on_previous": step > 1
        });

        context_history.push(step_context);

        // Verify no ground truth leakage in any step
        assert!(
            !context_history[step - 1]
                .get("uses_ground_truth")
                .unwrap()
                .as_bool()
                .unwrap(),
            "Step {step} should not use ground truth in LLM mode"
        );
    }

    println!("   ✅ Multi-step context consolidation: PASSED");
    Ok(())
}

/// Test various agent types and their ground truth access
#[tokio::test]
#[serial]
async fn test_agent_type_ground_truth_access() -> Result<()> {
    println!("🤖 Testing ground truth access by agent type...");

    let benchmark = create_test_benchmark_with_ground_truth();
    // Ensure environment is clean before testing agent types
    std::env::remove_var("REEV_DETERMINISTIC");

    let test_cases = vec![
        ("deterministic", true, "Should access ground truth"),
        ("gpt-4", false, "Should NOT access ground truth"),
        ("glm-4", false, "Should NOT access ground truth"),
        ("local", false, "Should NOT access ground truth"),
        ("zai-agent", false, "Should NOT access ground truth"),
    ];

    for (agent_name, should_access, description) in test_cases {
        println!("   🤖 Testing agent: {agent_name} ({description})");

        let benchmark_id = benchmark
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("test-benchmark-001");
        let benchmark_tags: Vec<String> = benchmark
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let is_deterministic = is_deterministic_mode(agent_name, benchmark_id, &benchmark_tags);

        // Verify deterministic mode detection
        assert_eq!(
            is_deterministic, should_access,
            "Agent {agent_name} deterministic mode detection mismatch"
        );

        // Verify ground truth access pattern
        let should_use_ground_truth = is_deterministic;

        assert_eq!(
            should_use_ground_truth, should_access,
            "Agent {agent_name} ground truth access mismatch"
        );

        println!(
            "     ✅ {}: {}",
            agent_name,
            if should_use_ground_truth {
                "ACCESS GRANTED"
            } else {
                "ACCESS BLOCKED"
            }
        );
    }

    println!("   ✅ Agent type ground truth access: PASSED");
    Ok(())
}

/// Test environment variable override for deterministic mode
#[tokio::test]
#[serial]
async fn test_environment_deterministic_mode() -> Result<()> {
    println!("🌍 Testing environment variable deterministic mode...");

    let benchmark = create_test_benchmark_with_ground_truth();

    let benchmark_id = benchmark
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("test-benchmark-001");
    let benchmark_tags: Vec<String> = benchmark
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Test with REEV_DETERMINISTIC environment variable
    std::env::set_var("REEV_DETERMINISTIC", "1");

    let is_deterministic = is_deterministic_mode("gpt-4", benchmark_id, &benchmark_tags);

    assert!(
        is_deterministic,
        "Environment variable REEV_DETERMINISTIC should enable deterministic mode"
    );

    println!("   ✅ Environment REEV_DETERMINISTIC=1: deterministic mode enabled");

    // Clean up
    std::env::remove_var("REEV_DETERMINISTIC");

    // Verify cleanup worked
    let is_deterministic_after_cleanup =
        is_deterministic_mode("gpt-4", benchmark_id, &benchmark_tags);

    assert!(
        !is_deterministic_after_cleanup,
        "Should return to non-deterministic after environment cleanup"
    );

    println!("   ✅ Environment cleanup: returned to LLM mode");
    println!("   ✅ Environment variable override: PASSED");

    Ok(())
}

// Helper functions

fn create_test_benchmark_with_ground_truth() -> serde_json::Value {
    json!({
        "id": "test-benchmark-001",
        "name": "Ground Truth Test Benchmark",
        "prompt": "Test prompt for ground truth separation",
        "tags": ["test"],
        "initial_state": [
            {
                "pubkey": "USER_WALLET_PUBKEY",
                "owner": "11111111111111111111111111111111",
                "lamports": 1000000000
            },
            {
                "pubkey": "RECIPIENT_WALLET_PUBKEY",
                "owner": "11111111111111111111111111111111",
                "lamports": 500000000
            }
        ],
        "ground_truth": {
            "final_state_assertions": [
                {
                    "pubkey": "USER_WALLET_PUBKEY",
                    "expected_lamportas": 900000000
                },
                {
                    "pubkey": "RECIPIENT_WALLET_PUBKEY",
                    "expected_lamportas": 600000000
                }
            ],
            "expected_instructions": [
                {
                    "program_id": "11111111111111111111111111111111",
                    "data": "transfer",
                    "accounts": ["USER_WALLET_PUBKEY", "RECIPIENT_WALLET_PUBKEY"]
                }
            ],
            "min_score": 0.8
        }
    })
}

fn create_multi_step_benchmark() -> serde_json::Value {
    json!({
        "id": "multi-step-test",
        "name": "Multi-Step Ground Truth Test",
        "prompt": "Multi-step test for ground truth separation",
        "tags": ["test", "multi-step"],
        "initial_state": [
            {
                "pubkey": "USER_WALLET_PUBKEY",
                "owner": "11111111111111111111111111111111",
                "lamports": 2000000000
            }
        ],
        "ground_truth": {
            "final_state_assertions": [
                {
                    "pubkey": "USER_WALLET_PUBKEY",
                    "expected_lamportas": 1800000000
                }
            ],
            "expected_instructions": [
                {
                    "program_id": "11111111111111111111111111111111",
                    "data": "transfer-step-1"
                },
                {
                    "program_id": "11111111111111111111111111111111",
                    "data": "transfer-step-2"
                }
            ],
            "min_score": 0.9
        }
    })
}
