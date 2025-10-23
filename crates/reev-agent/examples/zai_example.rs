//! Example of using the ZAI provider with GLM-4.6 model
//!
//! To run this example:
//! ```bash
//! ZAI_API_KEY="your-api-key" cargo run --example zai_example
//! ```

use futures::StreamExt;
use reev_agent::providers::zai;
use reev_tools::tools::SolTransferTool;
use rig::client::{CompletionClient, VerifyClient};
use rig::completion::{CompletionModel, CompletionRequestBuilder};
use rig::streaming::StreamedAssistantContent;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::io::Write;

// Use the actual types from SolTransferTool

// Simple test tool for getting current time
#[derive(Deserialize)]
struct GetCurrentTimeArgs {
    timezone: String,
}

#[derive(Serialize)]
struct TimeResult {
    time: String,
    timezone: String,
    date: String,
}

struct GetCurrentTimeTool;

impl Tool for GetCurrentTimeTool {
    const NAME: &'static str = "get_current_time";

    type Error = std::fmt::Error;
    type Args = GetCurrentTimeArgs;
    type Output = TimeResult;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: "get_current_time".to_string(),
            description: "Get the current time in a specific timezone".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "timezone": {
                        "type": "string",
                        "description": "The timezone to get the time for, e.g., UTC, America/New_York"
                    }
                },
                "required": ["timezone"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // For demonstration purposes, return a mock time
        // In a real implementation, you would use a time library to get the actual time
        Ok(TimeResult {
            time: "14:30:00".to_string(),
            timezone: args.timezone,
            date: "2023-10-22".to_string(),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the ZAI client
    let api_key = std::env::var("ZAI_API_KEY").expect("ZAI_API_KEY environment variable not set");

    let client = zai::Client::builder(&api_key).build();

    // Verify the client configuration
    println!("Verifying ZAI client...");
    client.verify().await?;
    println!("✓ ZAI client verification successful");

    // Create a completion model
    let model = client.completion_model(zai::GLM_4_6);

    // Example 1: Simple completion
    println!("\n=== Example 1: Simple Completion ===");
    let request =
        CompletionRequestBuilder::new(model.clone(), "What is the capital of France?").build();

    let result = model.completion(request).await?;
    println!("Question: What is the capital of France?");

    // Extract text from the choice
    let response_text = result
        .choice
        .iter()
        .find_map(|content| {
            if let rig::message::AssistantContent::Text(text) = content {
                Some(text.text.clone())
            } else {
                None
            }
        })
        .unwrap_or_default();

    println!("Answer: {response_text}");
    println!("Tokens used: {:?}", result.usage);

    // Example 2: Tool calling
    println!("\n=== Example 2: Tool Calling ===");
    let tool = GetCurrentTimeTool;
    let tool_def = tool.definition(String::new()).await;

    let request = CompletionRequestBuilder::new(model.clone(), "What time is it in New York?")
        .tool(tool_def)
        .build();

    let result = model.completion(request).await?;

    println!("Question: What time is it in New York?");

    // Extract tool calls from the choice
    let tool_calls: Vec<_> = result
        .choice
        .iter()
        .filter_map(|content| {
            if let rig::message::AssistantContent::ToolCall(tool_call) = content {
                Some(tool_call)
            } else {
                None
            }
        })
        .collect();

    if !tool_calls.is_empty() {
        let tool_call = &tool_calls[0];
        println!("Tool called: {}", tool_call.function.name);
        println!("Arguments: {}", tool_call.function.arguments);

        // Execute the tool
        let args: GetCurrentTimeArgs =
            serde_json::from_value(tool_call.function.arguments.clone())?;
        let tool_result = tool.call(args).await?;

        println!(
            "Tool result: {} at {} on {}",
            tool_result.time, tool_result.timezone, tool_result.date
        );
    } else {
        // Extract text response
        let response_text = result
            .choice
            .iter()
            .find_map(|content| {
                if let rig::message::AssistantContent::Text(text) = content {
                    Some(text.text.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        println!("Response: {response_text}");
    }

    // Example 3: Test SolTransferTool
    println!("\n=== Example 3: SolTransferTool Test ===");
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_WALLET_PUBKEY".to_string(),
        "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
    );
    key_map.insert(
        "RECIPIENT_WALLET_PUBKEY".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    let sol_tool = SolTransferTool { key_map };
    let sol_tool_def = sol_tool.definition(String::new()).await;

    let sol_request = CompletionRequestBuilder::new(
        model.clone(),
        "Please send 0.1 SOL from USER_WALLET_PUBKEY to RECIPIENT_WALLET_PUBKEY",
    )
    .tool(sol_tool_def)
    .build();

    let sol_result = model.completion(sol_request).await?;

    println!("Question: Please send 0.1 SOL from USER_WALLET_PUBKEY to RECIPIENT_WALLET_PUBKEY");

    // Extract tool calls from the choice
    let sol_tool_calls: Vec<_> = sol_result
        .choice
        .iter()
        .filter_map(|content| {
            if let rig::message::AssistantContent::ToolCall(tool_call) = content {
                Some(tool_call)
            } else {
                None
            }
        })
        .collect();

    if !sol_tool_calls.is_empty() {
        let sol_tool_call = &sol_tool_calls[0];
        println!("Tool called: {}", sol_tool_call.function.name);
        println!("Arguments: {}", sol_tool_call.function.arguments);

        // Execute the tool
        use reev_tools::tools::native::NativeTransferArgs;
        let args: NativeTransferArgs =
            serde_json::from_value(sol_tool_call.function.arguments.clone())?;
        let sol_tool_result = sol_tool.call(args).await?;

        println!("SolTransferTool result: {sol_tool_result}");
    } else {
        // Extract text response
        let response_text = sol_result
            .choice
            .iter()
            .find_map(|content| {
                if let rig::message::AssistantContent::Text(text) = content {
                    Some(text.text.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        println!("Response: {response_text}");
    }

    // Example 4: Streaming
    println!("\n=== Example 4: Streaming ===");
    let request = CompletionRequestBuilder::new(model.clone(), "Tell me a very short joke").build();

    println!("Question: Tell me a very short joke");
    print!("Answer: ");

    let mut stream = model.stream(request).await?;
    let mut full_response = String::new();

    // Process the streaming response
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(StreamedAssistantContent::Text(text)) => {
                print!("{}", text.text);
                std::io::stdout().flush()?;
                full_response.push_str(&text.text);
            }
            Ok(StreamedAssistantContent::ToolCall(tool_call)) => {
                println!("\nTool Call: {tool_call:?}");
            }
            Ok(StreamedAssistantContent::Reasoning(reasoning)) => {
                println!("\nReasoning: {reasoning:?}");
            }
            Ok(StreamedAssistantContent::Final(_)) => {
                // Final response received
                break;
            }
            Err(e) => {
                eprintln!("Error in stream: {e}");
                break;
            }
        }
    }

    // Access usage from the final response
    if let Some(response) = &stream.response {
        println!("\nStreaming completed. Tokens used: {:?}", response.usage);
    } else {
        println!("\nStreaming completed. No usage data available");
    }

    println!("\n✓ All examples completed successfully!");
    Ok(())
}
