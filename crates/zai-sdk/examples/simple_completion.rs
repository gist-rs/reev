//! Simple completion example using the ZAI SDK
//!
//! This example demonstrates basic usage of the ZAI SDK for text completion.

use std::time::Duration;
use zai_sdk::{GlmVariant, Message, ZaiClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt::init();

    // Create a client with the enhanced builder pattern
    // API key can be set via parameter or ZAI_API_KEY environment variable
    // API URL defaults to the variant's endpoint if not specified
    let client = ZaiClient::builder()
        .variant(GlmVariant::Standard)
        .api_key("your-api-key-here") // Optional, defaults to ZAI_API_KEY env var
        .api_url("https://api.z.ai/api/paas/v4") // Optional, defaults to variant endpoint
        .timeout(Duration::from_secs(30))
        .build()?;

    // Simple completion with a single prompt
    println!("Simple completion example:");
    println!("-------------------------");

    let response = client
        .completion("What is the capital of France? Respond in one sentence.")
        .await?;

    println!("Response: {response}\n");

    // Completion with multiple messages
    println!("Multi-message conversation example:");
    println!("----------------------------------");

    let messages = vec![
        Message::system("You are a helpful assistant who answers questions concisely."),
        Message::user("What is the largest planet in our solar system?"),
        Message::assistant("Jupiter is the largest planet in our solar system."),
        Message::user("What about the smallest planet?"),
    ];

    let response = client.completion_with_messages(messages).await?;

    println!("Response: {response}\n");

    // Using the coding variant
    println!("Coding model example:");
    println!("--------------------");

    let coding_client = ZaiClient::builder()
        .variant(GlmVariant::Coding)
        .timeout(Duration::from_secs(60))
        .build()?;

    let response = coding_client
        .completion("Write a Rust function to calculate the factorial of a number.")
        .await?;

    println!("Response: {response}\n");

    Ok(())
}
