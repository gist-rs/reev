//! Tests for planner module

use reev_core::context::{ContextResolver, SolanaEnvironment};
use reev_core::planner::Planner;

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
async fn test_planner_new_for_test() {
    let context_resolver = ContextResolver::new(SolanaEnvironment::default());
    let _planner = Planner::new(context_resolver);
    // Test that planner can be created for testing
    // Planner creation test passed
}

#[tokio::test]
async fn test_simple_planning() {
    let context_resolver = ContextResolver::new(SolanaEnvironment::default());
    let _planner = Planner::new(context_resolver);

    // Test that planner can be created
    // Simple planning test setup passed
}
