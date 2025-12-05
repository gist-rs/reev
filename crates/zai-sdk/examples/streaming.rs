//! Streaming response example using the ZAI SDK
//!
//! This example demonstrates how to use streaming responses with the ZAI SDK.

use futures::StreamExt;
use std::time::Duration;
use zai_sdk::{GlmVariant, ZaiClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt::init();

    // Create a client with the builder pattern
    let client = ZaiClient::builder()
        .variant(GlmVariant::Standard)
        .timeout(Duration::from_secs(30))
        .build()?;

    // Simple streaming example
    println!("Simple streaming example:");
    println!("------------------------");

    let mut stream = client
        .stream_completion(
            "Write a short poem about programming. Each line should be short and simple.",
        )
        .await?;

    println!("Response (streaming): ");
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(content) => {
                print!("{content}");
                // Flush stdout to see the streaming effect immediately
                std::io::Write::flush(&mut std::io::stdout())?;
            }
            Err(e) => {
                eprintln!("Error in stream: {e}");
                break;
            }
        }
    }
    println!("\n");

    // Streaming with the coding model
    println!("Coding model streaming example:");
    println!("------------------------------");

    let coding_client = ZaiClient::builder()
        .variant(GlmVariant::Coding)
        .timeout(Duration::from_secs(60))
        .build()?;

    let mut stream = coding_client
        .stream_completion("Write a Rust function that implements binary search. Include comments explaining each step.")
        .await?;

    println!("Response (streaming): ");
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(content) => {
                print!("{content}");
                // Flush stdout to see the streaming effect immediately
                std::io::Write::flush(&mut std::io::stdout())?;
            }
            Err(e) => {
                eprintln!("Error in stream: {e}");
                break;
            }
        }
    }
    println!();

    Ok(())
}
