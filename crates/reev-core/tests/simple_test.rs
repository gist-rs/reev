//! Simple tests for reev-core

use reev_core::context::ContextResolver;
use reev_core::planner::{Planner, UserIntent};
use reev_types::flow::WalletContext;

#[tokio::test]
async fn test_context_resolver() {
    let _context_resolver = ContextResolver::new(reev_core::context::SolanaEnvironment::default());
    // ContextResolver is created with default timeout of 30 seconds
}

#[tokio::test]
async fn test_planner_creation() {
    let context_resolver = ContextResolver::new(reev_core::context::SolanaEnvironment::default());
    let planner = Planner::new(context_resolver);

    // Test intent parsing
    let intent = planner.parse_intent("swap 1 SOL to USDC");
    match intent {
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
}

#[tokio::test]
async fn test_simple_planning() {
    let context_resolver = ContextResolver::new(reev_core::context::SolanaEnvironment::default());
    let planner = Planner::new(context_resolver);

    // Test rule-based planning
    let _wallet_context = WalletContext::new("test_wallet".to_string());
    // The refine_and_plan method expects wallet_pubkey as string, not WalletContext
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
    assert!(!yml_flow.steps.is_empty(), "No steps generated");
}
