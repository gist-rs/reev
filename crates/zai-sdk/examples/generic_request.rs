//! Request/response example using the ZAI SDK
//!
//! This example demonstrates how to use the SDK's request/response capabilities
//! with the CompletionRequest type.

use std::time::Duration;
use zai_sdk::{models::ToolDefinition, CompletionRequest, GlmVariant, ZaiClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt::init();

    // Create a client with the builder pattern
    let client = ZaiClient::builder()
        .variant(GlmVariant::Standard)
        .timeout(Duration::from_secs(30))
        .build()?;

    println!("Request/response example:");
    println!("-----------------------");

    // Create a completion request with all available options
    let request = CompletionRequest::new("glm-4.6")
        .system("You are a helpful assistant who provides reasoning for your answers.")
        .user("What is the capital of France? Provide a brief explanation.")
        .temperature(0.1)
        .max_tokens(500);

    // Send the request and get the response
    let response = client.send_completion_request(request).await?;

    // Print the response
    println!("Response ID: {}", response.id);

    if let Some(content) = response.content() {
        println!("Content: {}", content);
    }

    if let Some(usage) = response.usage() {
        println!("Tokens used: {}", usage.total_tokens);
    }

    // Example with tools
    println!("\nTool calling example:");
    println!("---------------------");

    let weather_tool =
        ToolDefinition::function("get_weather", "Get the current weather for a location")
            .parameter("location", "string", "The city name", true)
            .parameter(
                "units",
                "string",
                "Temperature units (celsius, fahrenheit)",
                false,
            )
            .build()
            .into();

    let tool_request = CompletionRequest::new("glm-4.6")
        .system("You are a helpful assistant.")
        .user("What's the weather in Paris?")
        .tool(weather_tool)
        .temperature(0.1)
        .max_tokens(500);

    let tool_response = client.send_completion_request(tool_request).await?;

    if let Some(content) = tool_response.content() {
        println!("Response: {}", content);
    }

    // Example with the Coding model
    println!("\nCoding model example:");
    println!("---------------------");

    let coding_client = ZaiClient::builder()
        .variant(GlmVariant::Coding)
        .timeout(Duration::from_secs(60))
        .build()?;

    let coding_request = CompletionRequest::new("glm-4.6")
        .user("Write a simple Rust function to calculate the factorial of a number.")
        .temperature(0.1)
        .max_tokens(1000);

    let coding_response = coding_client
        .send_completion_request(coding_request)
        .await?;

    if let Some(content) = coding_response.content() {
        println!("Code:\n{}", content);
    }

    Ok(())
}
