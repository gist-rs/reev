//! Integration tests for the ZAI SDK
//!
//! These tests verify that the SDK components work together correctly.

use std::time::Duration;
use zai_sdk::{
    error::{ZaiError, ZaiResult},
    models::{CompletionRequest, GlmVariant, Message, ToolDefinition},
    traits::ZaiModelCapabilities,
    ZaiClient,
};

#[tokio::test]
async fn test_client_builder() {
    // Test creating a client with builder pattern
    std::env::set_var("ZAI_API_KEY", "test-key");
    let client_result = ZaiClient::builder()
        .variant(GlmVariant::Standard)
        .timeout(Duration::from_secs(30))
        .build();

    // This should succeed with an environment variable
    assert!(client_result.is_ok());

    // Clean up
    std::env::remove_var("ZAI_API_KEY");
}

#[tokio::test]
async fn test_client_builder_with_env_key() {
    // Set a temporary environment variable for testing
    std::env::set_var("ZAI_API_KEY", "test-key");

    // Test creating a client with an environment API key
    let client_result = ZaiClient::builder()
        .variant(GlmVariant::Coding)
        .timeout(Duration::from_secs(60))
        .build();

    // This should succeed with the environment variable
    assert!(client_result.is_ok());

    // Clean up
    std::env::remove_var("ZAI_API_KEY");
}

#[tokio::test]
async fn test_glm_variant_properties() {
    // Test Standard variant properties
    let standard = GlmVariant::Standard;
    assert_eq!(standard.display_name(), "glm-4.6");
    assert_eq!(standard.api_model_name(), "glm-4.6");
    assert_eq!(standard.endpoint(), "https://api.z.ai/api/paas/v4");
    assert_eq!(standard.default_timeout_secs(), 30);

    // Test Coding variant properties
    let coding = GlmVariant::Coding;
    assert_eq!(coding.display_name(), "glm-4.6-coding");
    assert_eq!(coding.api_model_name(), "glm-4.6");
    assert_eq!(coding.endpoint(), "https://api.z.ai/api/coding/paas/v4");
    assert_eq!(coding.default_timeout_secs(), 60);
}

#[tokio::test]
async fn test_model_capabilities() {
    // Set a temporary environment variable for testing
    std::env::set_var("ZAI_API_KEY", "test-key");

    // Create a standard model client
    let standard_client = ZaiClient::builder()
        .variant(GlmVariant::Standard)
        .api_key("test-key")
        .build()
        .unwrap();

    // Test standard model capabilities
    assert!(standard_client.supports_function_calling());
    assert!(standard_client.supports_streaming());
    assert_eq!(standard_client.max_context_length(), 200000);
    assert_eq!(standard_client.max_output_tokens(), 32000);

    // Create a coding model client
    let coding_client = ZaiClient::builder()
        .variant(GlmVariant::Coding)
        .api_key("test-key")
        .build()
        .unwrap();

    // Test coding model capabilities
    assert!(!coding_client.supports_function_calling());
    assert!(coding_client.supports_streaming());
    assert_eq!(coding_client.max_context_length(), 200000);
    assert_eq!(coding_client.max_output_tokens(), 32000);

    // Clean up
    std::env::remove_var("ZAI_API_KEY");
}

#[test]
fn test_message_creation() {
    // Test creating different types of messages
    let system_msg = Message::system("You are a helpful assistant.");
    assert!(matches!(
        system_msg.role,
        zai_sdk::models::MessageRole::System
    ));
    assert_eq!(system_msg.content, "You are a helpful assistant.");

    let user_msg = Message::user("What is the capital of France?");
    assert!(matches!(user_msg.role, zai_sdk::models::MessageRole::User));
    assert_eq!(user_msg.content, "What is the capital of France?");

    let assistant_msg = Message::assistant("The capital of France is Paris.");
    assert!(matches!(
        assistant_msg.role,
        zai_sdk::models::MessageRole::Assistant
    ));
    assert_eq!(assistant_msg.content, "The capital of France is Paris.");

    let tool_msg = Message::tool("Weather data: 25°C, sunny");
    assert!(matches!(tool_msg.role, zai_sdk::models::MessageRole::Tool));
    assert_eq!(tool_msg.content, "Weather data: 25°C, sunny");

    // Test adding names to messages
    let named_msg = Message::user("Hello").with_name("User123");
    assert_eq!(named_msg.name, Some("User123".to_string()));
}

#[test]
fn test_completion_request_builder() {
    // Test building a completion request
    let request = CompletionRequest::new("glm-4.6")
        .system("You are a helpful assistant.")
        .user("What is 2+2?")
        .temperature(0.5)
        .max_tokens(100)
        .stream(false);

    assert_eq!(request.model, "glm-4.6");
    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.temperature, Some(0.5));
    assert_eq!(request.max_tokens, Some(100));
    assert!(!request.stream);

    // Test with multiple messages
    let messages = vec![
        Message::system("System prompt"),
        Message::user("User prompt"),
        Message::assistant("Assistant response"),
    ];

    let request_with_messages = CompletionRequest::new("glm-4.6").messages(messages.clone());

    assert_eq!(request_with_messages.messages, messages);
}

#[test]
fn test_tool_definition_builder() {
    // Test creating a tool definition
    let tool_def =
        ToolDefinition::function("get_weather", "Get the current weather for a location")
            .parameter("location", "string", "The city name", true)
            .parameter(
                "units",
                "string",
                "Temperature units (celsius, fahrenheit)",
                false,
            )
            .build();

    assert_eq!(tool_def.name, "get_weather");
    assert_eq!(
        tool_def.description,
        "Get the current weather for a location"
    );

    // Check parameters
    let properties = tool_def
        .parameters
        .get("properties")
        .unwrap()
        .as_object()
        .unwrap();
    assert!(properties.contains_key("location"));
    assert!(properties.contains_key("units"));

    // Check required parameters
    let required = tool_def
        .parameters
        .get("required")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(required.len(), 1);
    assert_eq!(required[0], "location");
}

#[test]
fn test_error_types() {
    // Test creating different error types
    let auth_error = ZaiError::authentication("Invalid API key");
    assert!(matches!(auth_error, ZaiError::Authentication { .. }));

    let api_error = ZaiError::api_request("Request failed");
    assert!(matches!(api_error, ZaiError::ApiRequest { .. }));

    let timeout_error = ZaiError::timeout(30);
    assert!(matches!(timeout_error, ZaiError::Timeout { .. }));
    assert_eq!(
        timeout_error.to_string(),
        "Request timed out after 30 seconds"
    );

    let model_error = ZaiError::model_unavailable("glm-4.7");
    assert!(matches!(model_error, ZaiError::ModelUnavailable { .. }));
    assert_eq!(model_error.to_string(), "Model 'glm-4.7' is not available");
}

#[test]
fn test_zai_result() {
    // Test ZaiResult type alias
    let success: ZaiResult<String> = Ok("Success".to_string());
    assert!(success.is_ok());

    let failure: ZaiResult<String> = Err(ZaiError::other("Something went wrong"));
    assert!(failure.is_err());
}

#[test]
fn test_client_with_custom_http_client() {
    // Set a temporary environment variable for testing
    std::env::set_var("ZAI_API_KEY", "test-key");

    // Create a custom HTTP client
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("zai-sdk-test/0.1.0")
        .build()
        .unwrap();

    // Create a ZAI client with the custom HTTP client and explicit API key
    let client_result = ZaiClient::builder()
        .api_key("test-key")
        .http_client(http_client)
        .build();

    assert!(client_result.is_ok());

    // Clean up
    std::env::remove_var("ZAI_API_KEY");
}

#[test]
fn test_glm_variant_from_str() {
    // Test creating GLM variants from strings
    let standard_from_str = GlmVariant::from("glm-4.6");
    assert_eq!(standard_from_str, GlmVariant::Standard);

    let coding_from_str = GlmVariant::from("glm-4.6-coding");
    assert_eq!(coding_from_str, GlmVariant::Coding);

    // Test with an unknown variant (should default to Standard)
    let unknown_variant = GlmVariant::from("unknown-model");
    assert_eq!(unknown_variant, GlmVariant::Standard);
}

#[test]
fn test_glm_variant_display() {
    // Test displaying GLM variants
    let standard = GlmVariant::Standard;
    assert_eq!(standard.to_string(), "glm-4.6");

    let coding = GlmVariant::Coding;
    assert_eq!(coding.to_string(), "glm-4.6-coding");
}
