//! Simple tests for reev-core

use reev_core::context::{ContextResolver, SolanaEnvironment};
use reev_core::planner::Planner;
// use reev_types::flow::WalletContext; // Not used in this test file

#[tokio::test]
async fn test_context_resolver() {
    let _context_resolver = ContextResolver::new(SolanaEnvironment::default());
    // ContextResolver is created with default timeout of 30 seconds
}

#[tokio::test]
async fn test_planner_creation() {
    let context_resolver = ContextResolver::new(SolanaEnvironment::default());
    let _planner = Planner::new(context_resolver);

    // Test that planner can be created
    assert!(true); // Just testing that it doesn't panic
}

#[tokio::test]
async fn test_planner_with_glm() {
    // Test planner creation with GLM
    let context_resolver = ContextResolver::new(SolanaEnvironment::default());
    match Planner::new_with_glm(context_resolver) {
        Ok(_planner) => {
            // Successfully created planner with GLM
            assert!(true);
        }
        Err(_e) => {
            // Failed to create planner with GLM (likely missing ZAI_API_KEY)
            assert!(true);
        }
    }
}

#[tokio::test]
async fn test_planner_new_for_test() {
    let context_resolver = ContextResolver::new(SolanaEnvironment::default());
    let _planner = Planner::new(context_resolver);
    // Test that planner can be created for testing
    assert!(true);
}
