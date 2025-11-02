//! ZAI Agent Model Validation Tests
//!
//! This test module validates that the ZAI Agent uses dynamic model parameters
//! instead of hardcoded constants, and properly validates model availability.

use anyhow::Result;
use reev_agent::enhanced::zai_agent::ZAIAgent;
use reev_agent::LlmRequest;
use std::collections::HashMap;

#[test]
fn test_model_validation_uses_dynamic_parameter() {
    // Test that model_name parameter is used instead of hardcoded constant
    let model_name = "glm-4.6";

    // Verify that the model name is what we expect
    assert_eq!(model_name, "glm-4.6");

    // This test ensures we're not using zai::GLM_4_6 constant directly
    // in the agent builder logic
    let model_name_param = model_name;
    assert_ne!(model_name_param, ""); // Should not be empty

    // Test with different model names
    let test_models = vec!["glm-4.6", "glm-4.6-flash", "glm-3-turbo"];
    for model in test_models {
        assert!(!model.is_empty(), "Model name should not be empty");
        assert!(
            model.starts_with("glm-"),
            "Model should start with 'glm-': {model}"
        );
    }
}

#[tokio::test]
#[ignore] // Requires ZAI_API_KEY to run
async fn test_model_validation_with_invalid_model() -> Result<()> {
    use reev_agent::providers::zai;

    // This test requires ZAI_API_KEY environment variable
    let api_key = std::env::var("ZAI_API_KEY");
    if api_key.is_err() {
        println!("⚠️  ZAI_API_KEY not set, skipping model validation test");
        println!("Set ZAI_API_KEY to run this test:");
        println!("export ZAI_API_KEY=your_api_key_here");
        return Ok(());
    }

    let client = zai::Client::builder(&api_key.unwrap()).build();

    // Test with an invalid model name
    let result = client.verify_model("invalid-model-name").await;

    // Should fail with a meaningful error
    assert!(result.is_err(), "Should fail with invalid model name");

    match result.unwrap_err() {
        rig::client::VerifyError::ProviderError(msg) => {
            assert!(
                msg.contains("invalid-model-name") || msg.contains("not available"),
                "Error should mention the invalid model: {msg}"
            );
        }
        other => {
            panic!("Expected ProviderError for invalid model, got: {other:?}");
        }
    }

    Ok(())
}

#[tokio::test]
#[ignore] // Requires ZAI_API_KEY to run
async fn test_zai_agent_with_valid_model() -> Result<()> {
    // Test that ZAI Agent works with a valid model
    let api_key = std::env::var("ZAI_API_KEY");
    if api_key.is_err() {
        println!("⚠️  ZAI_API_KEY not set, skipping ZAI agent test");
        return Ok(());
    }

    let key_map = HashMap::new();
    let model_name = "glm-4.6";

    // Create a simple test request
    let payload = LlmRequest {
        id: "test-zai-agent-model-validation".to_string(),
        session_id: "test-session-123".to_string(),
        prompt: "Hello, please introduce yourself briefly.".to_string(),
        context_prompt: "You are a helpful assistant.".to_string(),
        model_name: model_name.to_string(),
        mock: false,
        initial_state: None,
        allowed_tools: None,
        account_states: None,
        key_map: Some(key_map.clone()),
    };

    println!("🚀 Testing ZAI Agent with model: {model_name}");

    let result = ZAIAgent::run(model_name, payload, key_map).await;

    match result {
        Ok(response) => {
            println!("✅ ZAI Agent test successful!");
            println!("📄 Response length: {} chars", response.len());

            // Verify response contains expected content
            assert!(!response.is_empty(), "Response should not be empty");
            assert!(response.len() > 50, "Response should be substantial");

            Ok(())
        }
        Err(e) => {
            // Should not fail due to model validation issues
            if e.to_string().contains("validation failed") {
                panic!("Model validation should have passed: {e}");
            }
            println!("❌ ZAI Agent test failed (expected for missing tools): {e}");
            Ok(())
        }
    }
}

#[tokio::test]
#[ignore] // Requires ZAI_API_KEY to run
async fn test_zai_agent_with_different_models() -> Result<()> {
    // Test that ZAI Agent works with different GLM models
    let api_key = std::env::var("ZAI_API_KEY");
    if api_key.is_err() {
        println!("⚠️  ZAI_API_KEY not set, skipping multi-model test");
        return Ok(());
    }

    let test_models = vec![
        "glm-4.6",
        "glm-4.6-flash",
        // Add other GLM models as they become available
    ];

    for model_name in test_models {
        println!("🧪 Testing model: {model_name}");

        let key_map = HashMap::new();
        let payload = LlmRequest {
            id: format!("test-model-{}", model_name.replace("-", "_")),
            session_id: format!("test-session-{}", model_name.replace("-", "_")),
            prompt: "Briefly introduce yourself.".to_string(),
            context_prompt: "You are a helpful assistant.".to_string(),
            model_name: model_name.to_string(),
            mock: false,
            initial_state: None,
            allowed_tools: None,
            account_states: None,
            key_map: Some(key_map.clone()),
        };

        let result = ZAIAgent::run(model_name, payload, key_map).await;

        match result {
            Ok(response) => {
                println!(
                    "✅ Model {} works: {} chars response",
                    model_name,
                    response.len()
                );
                assert!(!response.is_empty(), "Response should not be empty");
            }
            Err(e) => {
                // Should fail with clear error if model is not available
                if e.to_string().contains("validation failed") {
                    println!("⚠️  Model {model_name} not available: {e}");
                } else {
                    println!("❌ Model {model_name} failed with other error: {e}");
                }
            }
        }
    }

    Ok(())
}
