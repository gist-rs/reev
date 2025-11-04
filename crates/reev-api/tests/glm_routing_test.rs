//! Test GLM-4.6 model routing to verify it uses OpenAI client with correct endpoint

use reev_agent::{run::run_agent, LlmRequest};
use std::collections::HashMap;

/// Helper function to check if error indicates wrong routing
fn is_routing_error(error: &str) -> bool {
    // "Invalid API parameter" indicates glm-4.6 went to ZAI client with coding endpoint
    error.contains("Invalid API parameter") ||
    // "Model not found" might indicate wrong endpoint
    error.contains("Model not found") ||
    // "Not Found" with 404 might indicate wrong path
    error.contains("404 Not Found")
}

/// Helper function to check if error indicates correct routing
fn is_correct_routing_error(error: &str) -> bool {
    // "token expired or incorrect" indicates correct OpenAI client routing
    error.contains("token expired or incorrect") ||
    // "invalid authentication" indicates correct ZAI client routing
    error.contains("invalid authentication") ||
    // "unauthorized" indicates correct routing but bad credentials
    error.contains("unauthorized") ||
    // Authentication errors from either endpoint
    error.contains("401") ||
    error.contains("authentication")
}

#[tokio::test]
async fn test_glm_4_6_routing() {
    // Set up environment for testing
    std::env::set_var("ZAI_API_KEY", "test_key");
    std::env::set_var("ZAI_API_URL", "https://api.z.ai/api/paas/v4");

    let payload = LlmRequest {
        id: "test-123".to_string(),
        session_id: "test-session".to_string(),
        prompt: "use my 50% sol to multiply usdc 1.5x on jup".to_string(),
        context_prompt: "Test context".to_string(),
        model_name: "glm-4.6".to_string(),
        mock: false, // Important: set to false to test real routing
        initial_state: None,
        allowed_tools: None,
        account_states: None,
        key_map: Some({
            let mut map = HashMap::new();
            map.insert("wallet".to_string(), "test_wallet".to_string());
            map
        }),
    };

    // This should route to OpenAI client with ZAI endpoint
    // If it goes to ZAI client with coding endpoint, it will fail with "Invalid API parameter"
    match run_agent("glm-4.6", payload).await {
        Ok(response) => {
            println!("✅ Success! GLM-4.6 routed correctly: {response}");
            assert!(!response.contains("Invalid API parameter"));
            assert!(!is_routing_error(&response));
        }
        Err(e) => {
            let error_str = e.to_string();
            println!("Error: {error_str}");

            // Check if this is a routing error (wrong endpoint)
            if is_routing_error(&error_str) {
                panic!("❌ GLM-4.6 incorrectly routed - expected OpenAI client at /api/paas/v4 but got ZAI client at /api/coding/paas/v4");
            }

            // Check if this is an authentication error (correct routing)
            if is_correct_routing_error(&error_str) {
                println!("✅ GLM-4.6 correctly routed (authentication error expected with test credentials)");
            } else {
                panic!("❌ Unexpected error for GLM-4.6 routing: {error_str}");
            }
        }
    }
}

#[tokio::test]
async fn test_glm_4_6_coding_routing() {
    // Set up environment for testing
    std::env::set_var("ZAI_API_KEY", "test_key");
    std::env::set_var("ZAI_API_URL", "https://api.z.ai/api/coding/paas/v4");

    let payload = LlmRequest {
        id: "test-456".to_string(),
        session_id: "test-session".to_string(),
        prompt: "code a smart contract".to_string(),
        context_prompt: "Test context".to_string(),
        model_name: "glm-4.6-coding".to_string(),
        mock: false,
        initial_state: None,
        allowed_tools: None,
        account_states: None,
        key_map: Some({
            let mut map = HashMap::new();
            map.insert("wallet".to_string(), "test_wallet".to_string());
            map
        }),
    };

    // This should route to ZAI client with coding endpoint
    match run_agent("glm-4.6-coding", payload).await {
        Ok(response) => {
            println!("✅ Success! GLM-4.6-coding routed correctly: {response}");
        }
        Err(e) => {
            let error_str = e.to_string();
            println!("Error for glm-4.6-coding: {error_str}");

            // Check if this is the expected authentication error for ZAI client
            if is_correct_routing_error(&error_str) {
                println!("✅ GLM-4.6-coding correctly routed to ZAI client (authentication error expected)");
            } else {
                panic!("❌ Unexpected error for GLM-4.6-coding routing: {error_str}");
            }
        }
    }
}
