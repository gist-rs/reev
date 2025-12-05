# ZAI SDK

A generic Rust SDK for interacting with ZAI's GLM-4.6 models, including standard, coding, and future variants.

## Features

- **Builder Pattern**: Fluent API for client configuration
- **Generic Types**: Support for custom request and response types
- **Multiple Variants**: Support for all GLM-4.6 model variants
- **Streaming**: Built-in streaming response capabilities
- **Retry Logic**: Configurable retry policies for resilience
- **Error Handling**: Comprehensive error types with proper context

## Quick Start

```rust
use zai_sdk::{ZaiClient, GlmVariant};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a client with the builder pattern
    let client = ZaiClient::builder()
        .variant(GlmVariant::Standard)
        .api_key("your-api-key")
        .build()?;

    // Simple completion
    let response = client
        .completion("What is the capital of France?")
        .await?;

    println!("Response: {}", response);

    // Completion with messages
    use zai_sdk::Message;
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::user("What is 2+2?"),
    ];

    let response = client
        .completion_with_messages(messages)
        .await?;

    println!("Response: {}", response);
    
    Ok(())
}
```

## Configuration

The SDK supports various configuration options through the builder pattern:

```rust
use zai_sdk::{ZaiClient, GlmVariant};
use std::time::Duration;
use reqwest::Client;

let client = ZaiClient::builder()
    .variant(GlmVariant::Coding)
    .api_key("your-api-key")
    .base_url("https://api.z.ai/api/coding/paas/v4")
    .timeout(Duration::from_secs(60))
    .http_client(Client::new())
    .build()?;
```

### Model Variants

- **Standard**: General-purpose model (default)
- **Coding**: Specialized for code generation and analysis

Each variant has its own endpoint, default timeout, and capabilities.

### Environment Variables

The SDK will automatically use the `ZAI_API_KEY` environment variable if no API key is explicitly provided.

## Advanced Usage

### Streaming Responses

```rust
use futures::StreamExt;

let mut stream = client
    .stream_completion("Write a short story about AI.")
    .await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(content) => print!("{}", content),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Tool Calling

The SDK supports function calling with the GLM-4.6 model:

```rust
use zai_sdk::{ToolDefinition, CompletionRequest};

let tool = ToolDefinition::function(
    "get_weather",
    "Get the current weather for a location"
)
.parameter("location", "string", "The city name", true)
.parameter("units", "string", "Temperature units (celsius, fahrenheit)", false)
.build();

let request = CompletionRequest::new("glm-4.6")
    .user("What's the weather in Tokyo?")
    .tool(tool);

let response = client.send_completion_request(request).await?;
```

## Error Handling

The SDK provides comprehensive error types for different failure scenarios:

```rust
match client.completion("test").await {
    Ok(response) => println!("Success: {}", response),
    Err(e) => {
        match e {
            zai_sdk::ZaiError::Authentication { .. } => {
                eprintln!("Authentication failed");
            }
            zai_sdk::ZaiError::RateLimit { seconds } => {
                eprintln!("Rate limited. Retry after {} seconds", seconds);
            }
            zai_sdk::ZaiError::Network { .. } => {
                eprintln!("Network error");
            }
            _ => {
                eprintln!("Other error: {}", e);
            }
        }
    }
}
```

## Architecture

The SDK is built with a modular architecture:

- **Client**: Core ZAI client implementation
- **Models**: Data structures for requests and responses
- **Traits**: Generic interfaces for extensibility
- **Error**: Comprehensive error handling

## Design Principles

- **Modular Design**: Each module has a single responsibility
- **Type Safety**: Strong typing with proper error handling
- **Extensibility**: Generic traits for custom implementations
- **Reusability**: Project-agnostic design for use in different contexts

## License

This project is licensed under MIT OR Apache-2.0.