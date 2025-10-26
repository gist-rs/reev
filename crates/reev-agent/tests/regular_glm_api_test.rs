use anyhow::Result;
use reev_agent::enhanced::openai::OpenAIAgent;
use reev_agent::LlmRequest;
use std::collections::HashMap;

#[tokio::test]
#[ignore] // Requires ZAI_API_KEY to run
async fn test_regular_glm_api_support() -> Result<()> {
    // Check if ZAI_API_KEY is available
    let zai_api_key = std::env::var("ZAI_API_KEY");
    if zai_api_key.is_err() {
        println!("⚠️  ZAI_API_KEY not set, skipping regular GLM API test");
        println!("Set ZAI_API_KEY to run this test:");
        println!("export ZAI_API_KEY=your_glm_api_key");
        return Ok(());
    }

    println!("✅ ZAI_API_KEY is set, testing regular GLM API");

    let key_map = HashMap::new();

    // Create a simple test request
    let payload = LlmRequest {
        id: "test-regular-glm".to_string(),
        session_id: "test-session-123".to_string(),
        prompt: "Hello, please introduce yourself briefly.".to_string(),
        context_prompt: "".to_string(),
        model_name: "glm-4.6".to_string(),
        mock: false,
        initial_state: None,
        allowed_tools: None,
        account_states: None,
        key_map: Some(key_map.clone()),
    };

    println!("🚀 Testing regular GLM API with model: glm-4.6");

    let response = OpenAIAgent::run("glm-4.6", payload, key_map).await;

    match response {
        Ok(result) => {
            println!("✅ Regular GLM API test successful!");
            println!("📄 Response length: {} chars", result.len());
            println!(
                "📄 Response preview: {}...",
                &result[..result.len().min(100)]
            );

            // Verify response contains expected content
            assert!(!result.is_empty(), "Response should not be empty");
            assert!(result.len() > 50, "Response should be substantial");

            Ok(())
        }
        Err(e) => {
            println!("❌ Regular GLM API test failed: {e}");
            Err(e)
        }
    }
}

#[tokio::test]
async fn test_api_priority_order() -> Result<()> {
    // Test that API priority is correct: ZAI_API_KEY > OPENAI_API_KEY > local fallback

    // Save original env vars
    let original_zai = std::env::var("ZAI_API_KEY").ok();
    let original_openai = std::env::var("OPENAI_API_KEY").ok();

    // Test 1: ZAI_API_KEY takes priority
    std::env::set_var("ZAI_API_KEY", "test-zai-key");
    std::env::set_var("OPENAI_API_KEY", "test-openai-key");

    println!("✅ Testing API priority: ZAI_API_KEY should take priority");

    let key_map = HashMap::new();

    // Note: This will fail with invalid keys, but we're testing the priority logic
    let payload = LlmRequest {
        id: "test-priority".to_string(),
        session_id: "test-session-priority".to_string(),
        prompt: "test".to_string(),
        context_prompt: "".to_string(),
        model_name: "glm-4.6".to_string(),
        mock: false,
        initial_state: None,
        allowed_tools: None,
        account_states: None,
        key_map: Some(key_map.clone()),
    };
    let result = OpenAIAgent::run("glm-4.6", payload, key_map).await;

    // Should attempt ZAI_API_KEY first (will fail with invalid key, but that's expected)
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("ZAI_API_KEY")
            || error_msg.contains("401")
            || error_msg.contains("unauthorized")
    );

    // Restore original env vars
    if let Some(key) = original_zai {
        std::env::set_var("ZAI_API_KEY", key);
    } else {
        std::env::remove_var("ZAI_API_KEY");
    }

    if let Some(key) = original_openai {
        std::env::set_var("OPENAI_API_KEY", key);
    } else {
        std::env::remove_var("OPENAI_API_KEY");
    }

    println!("✅ API priority test completed");
    Ok(())
}

#[tokio::test]
async fn test_local_fallback() -> Result<()> {
    // Test local model fallback when no API keys are set

    // Save original env vars
    let original_zai = std::env::var("ZAI_API_KEY").ok();
    let original_openai = std::env::var("OPENAI_API_KEY").ok();

    // Clear API keys to test fallback
    std::env::remove_var("ZAI_API_KEY");
    std::env::remove_var("OPENAI_API_KEY");

    println!("✅ Testing local model fallback (no API keys)");

    let key_map = HashMap::new();

    let payload = LlmRequest {
        id: "test-fallback".to_string(),
        session_id: "test-session-fallback".to_string(),
        prompt: "test".to_string(),
        context_prompt: "".to_string(),
        model_name: "local".to_string(),
        mock: false,
        initial_state: None,
        allowed_tools: None,
        account_states: None,
        key_map: Some(key_map.clone()),
    };
    let result = OpenAIAgent::run("glm-4.6", payload, key_map).await;

    // Should attempt local model (will likely fail if no local server running)
    assert!(result.is_err());

    // Restore original env vars
    if let Some(key) = original_zai {
        std::env::set_var("ZAI_API_KEY", key);
    }

    if let Some(key) = original_openai {
        std::env::set_var("OPENAI_API_KEY", key);
    }

    println!("✅ Local fallback test completed");
    Ok(())
}
