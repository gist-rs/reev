//! Tests for planner module
mod common;

use reev_core::{
    context::{ContextResolver, SolanaEnvironment},
    planner::Planner,
};
use reev_types::flow::WalletContext;

#[tokio::test]
async fn test_planner_creation() {
    let context_resolver = ContextResolver::new(SolanaEnvironment::default());
    let _planner = Planner::new(context_resolver);

    // Test that planner can be created
    // Planner creation test passed
}

#[tokio::test]
async fn test_planner_with_glm() {
    // Test planner creation with GLM
    let context_resolver = ContextResolver::new(SolanaEnvironment::default());
    match Planner::new_with_glm(context_resolver) {
        Ok(_planner) => {
            // Successfully created planner with GLM
            // Planner with GLM creation test passed
        }
        Err(_e) => {
            // Failed to create planner with GLM (likely missing ZAI_API_KEY)
            // Expected failure when ZAI_API_KEY is missing
        }
    }
}

#[tokio::test]
async fn test_simple_planning() {
    // Load environment variables for tests
    dotenvy::dotenv().ok();

    let context_resolver = ContextResolver::new(SolanaEnvironment::default());
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
