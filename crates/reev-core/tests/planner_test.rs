//! Tests for the planner module

mod common;

use common::mock_helpers::MockLLMClient;
use reev_core::{
    context::ContextResolver,
    planner::{Planner, UserIntent},
};
use reev_types::flow::WalletContext;

#[tokio::test]
async fn test_planner_with_mock_llm() {
    // Create a mock LLM client
    let mock_llm = MockLLMClient::new().with_response(
        "swap 1 SOL to USDC",
        r#"{
            "flow_id": "test-flow-id",
            "user_prompt": "swap 1 SOL to USDC",
            "subject_wallet_info": [{
                "pubkey": "USER_WALLET_PUBKEY",
                "lamports": 4000000000,
                "total_value_usd": 100.0,
                "tokens": []
            }],
            "steps": [{
                "step_id": "test-step-id",
                "prompt": "swap SOL to USDC",
                "context": "Executing swap through Jupiter",
                "critical": true,
                "estimated_time_seconds": 30,
                "expected_tool_calls": [{
                    "tool_name": "JupiterSwap",
                    "critical": true
                }]
            }],
            "ground_truth": {
                "final_state_assertions": [],
                "expected_tool_calls": []
            }
        }"#,
    );

    // Create a planner with the mock LLM client
    let context_resolver = ContextResolver::new(reev_core::context::SolanaEnvironment::default());
    let mut planner = Planner::new(context_resolver);
    planner = planner.with_llm_client(Box::new(mock_llm));

    // Create a wallet context
    let wallet_context = WalletContext::new("test_wallet".to_string());

    // Test flow generation
    let result = planner
        .refine_and_plan("swap 1 SOL to USDC", "test_wallet")
        .await;

    assert!(
        result.is_ok(),
        "Failed to generate flow: {:?}",
        result.err()
    );

    let yml_flow = result.unwrap();
    assert_eq!(yml_flow.user_prompt, "swap 1 SOL to USDC");
    assert_eq!(yml_flow.steps.len(), 1);
    assert_eq!(yml_flow.steps[0].prompt, "swap SOL to USDC");
}

#[tokio::test]
async fn test_planner_intent_parsing() {
    let context_resolver = ContextResolver::new(reev_core::context::SolanaEnvironment::default());
    let planner = Planner::new(context_resolver);

    // Test swap intent
    let swap_intent = planner.parse_intent("swap 1 SOL to USDC");
    match swap_intent {
        Ok(UserIntent::Swap {
            from, to, amount, ..
        }) => {
            assert_eq!(from, "SOL");
            assert_eq!(to, "USDC");
            assert_eq!(amount, 1.0);
        }
        Ok(_) => panic!("Expected Swap intent"),
        Err(_) => panic!("Failed to parse intent"),
    }

    // Test lend intent
    let lend_intent = planner.parse_intent("lend 100 USDC");
    match lend_intent {
        Ok(UserIntent::Lend { mint, amount, .. }) => {
            assert_eq!(mint, "USDC");
            assert_eq!(amount, 100.0);
        }
        Ok(_) => panic!("Expected Lend intent"),
        Err(_) => panic!("Failed to parse intent"),
    }

    // Test unknown intent
    let unknown_intent = planner.parse_intent("what is the weather today");
    match unknown_intent {
        Ok(UserIntent::Unknown) => {
            // Expected result
        }
        Ok(_) => panic!("Expected Unknown intent"),
        Err(_) => panic!("Failed to parse intent"),
    }
}

#[tokio::test]
async fn test_planner_rule_based_fallback() {
    // Create a planner without LLM client to trigger rule-based fallback
    let context_resolver = ContextResolver::new(reev_core::context::SolanaEnvironment::default());
    let planner = Planner::new(context_resolver);

    // Create a wallet context
    let wallet_context = WalletContext::new("test_wallet".to_string());

    // Test swap flow
    let result = planner
        .refine_and_plan("swap 1 SOL to USDC", "test_wallet")
        .await;

    assert!(
        result.is_ok(),
        "Failed to generate swap flow: {:?}",
        result.err()
    );

    let yml_flow = result.unwrap();
    assert_eq!(yml_flow.user_prompt, "swap 1 SOL to USDC");
    assert_eq!(yml_flow.steps.len(), 1);
    assert_eq!(yml_flow.steps[0].prompt, "swap 0.05 SOL to USDC");

    // Test lend flow
    let result = planner
        .refine_and_plan("lend 100 USDC", "test_wallet")
        .await;

    assert!(
        result.is_ok(),
        "Failed to generate lend flow: {:?}",
        result.err()
    );

    let yml_flow = result.unwrap();
    assert_eq!(yml_flow.user_prompt, "lend 100 USDC");
    assert_eq!(yml_flow.steps.len(), 1);
    assert_eq!(yml_flow.steps[0].prompt, "lend USDC");
}

#[tokio::test]
async fn test_planner_with_llm_failure() {
    // Create a mock LLM client that always fails
    let mock_llm = MockLLMClient::new().with_success(false);

    // Create a planner with the failing mock LLM client
    let context_resolver = ContextResolver::new(reev_core::context::SolanaEnvironment::default());
    let mut planner = Planner::new(context_resolver);
    planner = planner.with_llm_client(Box::new(mock_llm));

    // Create a wallet context
    let wallet_context = WalletContext::new("test_wallet".to_string());

    // Test that it falls back to rule-based planning
    let result = planner
        .refine_and_plan("swap 1 SOL to USDC", "test_wallet")
        .await;

    assert!(
        result.is_ok(),
        "Failed to fallback to rule-based planning: {:?}",
        result.err()
    );

    // Check if result is ok and then use value
    let yml_flow = result.unwrap();
    assert_eq!(yml_flow.user_prompt, "swap 1 SOL to USDC");
    assert_eq!(yml_flow.steps.len(), 1);
    assert_eq!(yml_flow.steps[0].prompt, "swap 0.05 SOL to USDC");
}
