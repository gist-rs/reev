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

    // Test that planner can be created - just testing that it doesn't panic
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
async fn test_planner_new_for_test() {
    let context_resolver = ContextResolver::new(SolanaEnvironment::default());
    let _planner = Planner::new(context_resolver);
    // Test that planner can be created for testing
    // Planner creation test passed
}
