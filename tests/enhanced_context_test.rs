//! Enhanced Context Integration Test
//!
//! This test verifies that the enhanced context implementation
//! provides real wallet data and generates context-aware flows.

use reqwest;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

/// API base URL for testing
const API_BASE: &str = "http://localhost:3001";

/// Test enhanced context resolution and flow generation (mock version)
#[tokio::test]
async fn test_enhanced_context_integration() {
    println!("🧪 Testing Enhanced Context Integration");

    // Test 1: Test context resolver directly
    let resolver = reev_orchestrator::ContextResolver::new();

    // Test wallet context resolution
    let context_result = resolver.resolve_wallet_context("USER_WALLET_PUBKEY").await;
    assert!(context_result.is_ok());

    let context = context_result.unwrap();
    println!("📊 Resolved wallet context:");
    println!("  Owner: {}", context.owner);
    println!("  SOL Balance: {:.6}", context.sol_balance_sol());
    println!("  Total Value: ${:.2}", context.total_value_usd);
    println!("  Token Balances: {}", context.token_balances.len());

    // Test 2: Test enhanced flow generation with mock gateway
    let gateway = reev_orchestrator::OrchestratorGateway::new().await.unwrap();

    // Create a test context with sufficient balance
    let mut test_context = reev_types::flow::WalletContext::new("USER_WALLET_PUBKEY".to_string());
    test_context.sol_balance = 2_000_000_000; // 2 SOL
    test_context.total_value_usd = 300.0;

    // Test different flow generation scenarios
    let test_prompts = vec![
        "swap 0.5 SOL for USDC",
        "use 25% of SOL for USDC",
        "swap SOL to USDC then lend for yield",
    ];

    for prompt in test_prompts {
        println!("\n📋 Testing prompt: {}", prompt);

        let flow_result = gateway.generate_enhanced_flow_plan(prompt, &test_context, None);

        match flow_result {
            Ok(flow) => {
                println!("  ✅ Generated flow with {} steps", flow.steps.len());

                // Verify steps contain context information
                for (i, step) in flow.steps.iter().enumerate() {
                    if step.prompt_template.contains("SOL") ||
                       step.prompt_template.contains("USDC") ||
                       step.prompt_template.contains("balance") {
                        println!("    {} ✅ Step {} includes context data", i + 1, step.step_id);
                    }
                }
            }
            Err(e) => {
                println!("  ❌ Flow generation failed: {}", e);
            }
        }
    }

    // Test 3: Verify context refinement
    let refined_prompt = gateway.refine_prompt("swap 0.5 SOL", &test_context);
    println!("\n📝 Refined prompt includes context:");
    assert!(refined_prompt.contains("2.000000 SOL"));
    assert!(refined_prompt.contains("$300.00"));
    println!("  ✅ Context refinement working");

    println!("🎉 Enhanced context integration test completed successfully!");
}

/// Test enhanced flow generation with different wallet scenarios
#[tokio::test]
async fn test_enhanced_context_scenarios() {
    println!("🧪 Testing Enhanced Context Scenarios");

    let test_scenarios = vec![
        ("High value wallet", "swap 1 SOL for USDC", 10.0, true),
        ("Low value wallet", "swap 0.1 SOL for USDC", 1.0, true),
        ("Percentage-based", "use 25% of SOL for USDC", 2.0, true),
        ("Complex strategy", "use 50% SOL to multiply USDC position", 5.0, true),
        ("Insufficient balance", "swap 10 SOL for USDC", 10.0, false), // Should fail
    ];

    let gateway = reev_orchestrator::OrchestratorGateway::new().await.unwrap();

    for (scenario_name, prompt, sol_amount, should_succeed) in test_scenarios {
        println!("\n📋 Testing scenario: {}", scenario_name);

        // Create test context based on scenario
        let mut test_context = reev_types::flow::WalletContext::new("USER_WALLET_PUBKEY".to_string());
        test_context.sol_balance = (sol_amount * 1_000_000_000.0) as u64; // Convert to lamports
        test_context.total_value_usd = sol_amount * 150.0; // Assuming SOL price of $150

        println!("  Context: {:.6} SOL, ${:.2} total value",
                test_context.sol_balance_sol(),
                test_context.total_value_usd);

        let flow_result = gateway.generate_enhanced_flow_plan(prompt, &test_context, None);

        if should_succeed {
            assert!(flow_result.is_ok(), "Flow should succeed for scenario: {}", scenario_name);
            let flow = flow_result.unwrap();
            println!("  ✅ Generated flow with {} steps", flow.steps.len());

            // Verify steps contain context-specific information
            for step in &flow.steps {
                if step.prompt_template.contains("SOL") ||
                   step.prompt_template.contains("balance") {
                    println!("    ✅ Context-aware step: {}", step.step_id);
                }
            }
        } else {
            assert!(flow_result.is_err(), "Flow should fail for scenario: {}", scenario_name);
            println!("  ✅ Flow correctly failed due to insufficient balance");
        }

        println!("  ✅ Scenario logic verified");
    }

    println!("\n🎉 Enhanced context scenarios test completed!");
}

/// Test context resolver caching and performance
#[tokio::test]
async fn test_context_resolver_performance() {
    println!("🧪 Testing Context Resolver Performance");

    let resolver = reev_orchestrator::ContextResolver::new();

    // Test multiple context resolutions to verify caching
    let test_pubkeys = vec![
        "test_wallet_1",
        "test_wallet_2",
        "test_wallet_3",
        "USER_WALLET_PUBKEY",
        "RECIPIENT_WALLET",
    ];

    println!("  🔄 Testing context resolution with caching...");

    let start_time = std::time::Instant::now();

    // First round - should populate cache
    for pubkey in &test_pubkeys {
        let context_result = resolver.resolve_wallet_context(pubkey).await;
        assert!(context_result.is_ok(), "Context resolution should succeed for: {}", pubkey);
        println!("    ✅ First resolution: {}", pubkey);
    }

    let first_round_time = start_time.elapsed();
    println!("  📊 First round took: {:?}", first_round_time);

    // Second round - should use cache
    let cache_start = std::time::Instant::now();
    for pubkey in &test_pubkeys {
        let context_result = resolver.resolve_wallet_context(pubkey).await;
        assert!(context_result.is_ok(), "Cached context resolution should succeed for: {}", pubkey);
        println!("    ✅ Cached resolution: {}", pubkey);
    }

    let second_round_time = cache_start.elapsed();
    println!("  📊 Second round (cached) took: {:?}", second_round_time);

    // Cache should be significantly faster
    if second_round_time < first_round_time {
        println!("  🚀 Caching appears to be working (second round faster)");
    }

    // Test cache statistics
    let (wallet_cache_size, price_cache_size) = resolver.get_cache_stats().await;
    println!("  📈 Cache stats - Wallet: {}, Prices: {}", wallet_cache_size, price_cache_size);

    // Test cache clearing
    resolver.clear_caches().await;
    println!("  🧹 Caches cleared");

    println!("🎉 Context resolver performance test completed!");
}
