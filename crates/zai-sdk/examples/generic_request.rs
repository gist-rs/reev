//! Generic request/response example using the ZAI SDK
//!
//! This example demonstrates how to use the SDK's generic request/response capabilities
//! with custom request and response types.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use zai_sdk::{GlmVariant, ZaiClient};

// Custom request type with additional parameters
#[derive(Serialize)]
struct CustomCompletionRequest {
    model: String,
    messages: Vec<CustomMessage>,
    temperature: f32,
    max_tokens: u32,
    // Custom field specific to this request type
    include_reasoning: bool,
}

// Custom message type
#[derive(Serialize)]
struct CustomMessage {
    role: String,
    content: String,
    // Optional custom fields
    reasoning: Option<String>,
}

// Custom response type with additional fields
#[derive(Debug, Deserialize)]
struct CustomCompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<CustomChoice>,
    usage: CustomUsage,
    // Custom field not in standard response
    processing_time_ms: u64,
}

#[derive(Debug, Deserialize)]
struct CustomChoice {
    index: u32,
    message: CustomMessageResponse,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CustomMessageResponse {
    role: String,
    content: String,
    // Custom fields
    reasoning_content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CustomUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
    // Custom field
    prompt_tokens_details: Option<PromptTokensDetails>,
}

#[derive(Debug, Deserialize)]
struct PromptTokensDetails {
    cached_tokens: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt::init();

    // Create a client with the builder pattern
    let client = ZaiClient::builder()
        .variant(GlmVariant::Standard)
        .timeout(Duration::from_secs(30))
        .build()?;

    println!("Generic request/response example:");
    println!("-------------------------------");

    // Create a custom request
    let custom_request = CustomCompletionRequest {
        model: "glm-4.6".to_string(),
        messages: vec![
            CustomMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant who provides reasoning for your answers."
                    .to_string(),
                reasoning: None,
            },
            CustomMessage {
                role: "user".to_string(),
                content: "What is the capital of France? Provide a brief explanation.".to_string(),
                reasoning: None,
            },
        ],
        temperature: 0.1,
        max_tokens: 500,
        include_reasoning: true, // Custom parameter
    };

    // Send the request and get a custom-typed response
    let response: CustomCompletionResponse =
        client.send_typed_chat_request(&custom_request).await?;

    // Print the response
    println!("Response ID: {}", response.id);
    println!("Processing time: {}ms", response.processing_time_ms);

    if let Some(choice) = response.choices.first() {
        println!("Content: {}", choice.message.content);

        if let Some(reasoning) = &choice.message.reasoning_content {
            println!("Reasoning: {reasoning}");
        }
    }

    println!("Tokens used: {}", response.usage.total_tokens);

    // Example with the Coding model
    println!("\nCoding model example:");
    println!("---------------------");

    let coding_client = ZaiClient::builder()
        .variant(GlmVariant::Coding)
        .timeout(Duration::from_secs(60))
        .build()?;

    let coding_request = CustomCompletionRequest {
        model: "glm-4.6".to_string(),
        messages: vec![CustomMessage {
            role: "user".to_string(),
            content: "Write a simple Rust function to calculate the factorial of a number."
                .to_string(),
            reasoning: None,
        }],
        temperature: 0.1,
        max_tokens: 1000,
        include_reasoning: true,
    };

    let response: CustomCompletionResponse = coding_client
        .send_typed_chat_request(&coding_request)
        .await?;

    if let Some(choice) = response.choices.first() {
        println!("Code:\n{}", choice.message.content);
    }

    Ok(())
}
