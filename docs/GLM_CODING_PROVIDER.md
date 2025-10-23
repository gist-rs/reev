# GLM Coding Provider Documentation

## Overview

The GLM Coding provider is a custom implementation for integrating GLM Coding models (glm-4.6, etc.) with the reev-agent framework. These models provide enhanced reasoning capabilities with `reasoning_content` in responses, making them particularly suitable for coding and complex logical tasks.

## Features

- **Enhanced Reasoning**: GLM Coding models provide detailed reasoning process in `reasoning_content` field
- **Tool Calling**: Full support for function calling and tool execution
- **OpenAI-Compatible**: Response format converted to OpenAI-compatible structure
- **Environment Configuration**: Flexible configuration via environment variables
- **Streaming Support**: Ready for streaming response implementation

## Quick Start

### 1. Environment Setup

Set the required environment variables:

```bash
export GLM_CODING_API_KEY="your_glm_coding_api_key"
export GLM_CODING_API_URL="https://api.z.ai/api/coding/paas/v4"  # Optional, defaults to this URL
```

### 2. Basic Usage

```rust
use reev_agent::providers::glm_coding::{Client, GLM_4_6};
use reev_agent::providers::glm_coding::completion::{SimpleCompletionRequest, ToolDefinition};
use serde_json::json;

// Create client
let client = Client::from_env();

// Create completion model
let model = client.completion_model(GLM_4_6);

// Create request
let request = SimpleCompletionRequest {
    prompt: "Write a Rust function to calculate fibonacci numbers".to_string(),
    preamble: Some("You are a helpful Rust programming assistant.".to_string()),
    temperature: Some(0.7),
    max_tokens: Some(1000),
    tools: vec![],
};

// Execute completion
let response = model.complete(request).await?;
let (content, tool_calls) = model.extract_response_content(response);

println!("Response: {}", content.unwrap_or_default());
```

### 3. Tool Calling Example

```rust
use reev_agent::providers::glm_coding::completion::ToolDefinition;

// Define a tool
let weather_tool = ToolDefinition {
    name: "get_weather".to_string(),
    description: "Get current weather for a location".to_string(),
    parameters: json!({
        "type": "object",
        "properties": {
            "location": {
                "type": "string",
                "description": "City name"
            },
            "unit": {
                "type": "string",
                "enum": ["celsius", "fahrenheit"]
            }
        },
        "required": ["location"]
    }),
};

// Create request with tool
let request = SimpleCompletionRequest {
    prompt: "What's the weather in Tokyo?".to_string(),
    preamble: None,
    temperature: Some(0.1),
    max_tokens: Some(500),
    tools: vec![weather_tool],
};

// Execute and handle tool calls
let response = model.complete(request).await?;
let (content, tool_calls) = model.extract_response_content(response);

if !tool_calls.is_empty() {
    for tool_call in tool_calls {
        println!("Tool: {}", tool_call.function.name);
        println!("Args: {}", tool_call.function.arguments);
        // Execute tool logic here...
    }
}
```

## API Reference

### Client

#### `Client::new(api_key: &str) -> Client`
Create a new client with API key.

#### `Client::from_env() -> Client`
Create client from environment variables (`GLM_CODING_API_KEY`, `GLM_CODING_API_URL`).

#### `Client::builder(api_key: &str) -> ClientBuilder`
Create a client builder for custom configuration.

#### `client.completion_model(model: &str) -> CompletionModel`
Create a completion model for the specified model name.

### Available Models

- `GLM_4_6` = `"glm-4.6"`

### CompletionModel

#### `model.complete(request: SimpleCompletionRequest) -> Result<CompletionResponse>`
Execute a completion request.

#### `model.extract_response_content(response: CompletionResponse) -> (Option<String>, Vec<ToolCall>)`
Extract content and tool calls from GLM response.

### SimpleCompletionRequest

```rust
pub struct SimpleCompletionRequest {
    pub prompt: String,
    pub preamble: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools: Vec<ToolDefinition>,
}
```

### ToolDefinition

```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
```

## Response Format

GLM Coding models return responses with the following structure:

```json
{
  "choices": [{
    "message": {
      "role": "assistant",
      "reasoning_content": "Detailed reasoning process...",
      "content": "Final answer",
      "tool_calls": [...]
    },
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 20,
    "total_tokens": 30
  }
}
```

The provider automatically combines `reasoning_content` and `content` into a single response for compatibility.

## Integration with reev-agent

### Using GLM Coding Agent

The GLM Coding provider integrates with the reev-agent framework through the `GlmCodingAgent`:

```rust
use reev_agent::enhanced::GlmCodingAgent;
use reev_agent::LlmRequest;
use std::collections::HashMap;

let request = LlmRequest {
    id: "request-123".to_string(),
    prompt: "Help me write a sorting algorithm".to_string(),
    context_prompt: "You are a programming assistant".to_string(),
    model_name: "glm-4.6".to_string(),
    mock: false,
    initial_state: None,
    allowed_tools: None,
};

let key_map = HashMap::new();
let response = GlmCodingAgent::run("glm-4.6", request, key_map).await?;
```

### Environment Variables

The GLM Coding provider uses the following environment variables:

- `GLM_CODING_API_KEY`: Required API key for GLM Coding API
- `GLM_CODING_API_URL`: Optional custom API URL (defaults to `https://api.z.ai/api/coding/paas/v4`)

## Error Handling

The provider provides comprehensive error handling:

```rust
match model.complete(request).await {
    Ok(response) => {
        let (content, tool_calls) = model.extract_response_content(response);
        // Process response
    }
    Err(e) => {
        eprintln!("GLM Coding API error: {}", e);
        // Handle error
    }
}
```

Common error scenarios:
- Missing API key
- Invalid API URL
- Network connectivity issues
- Rate limiting
- Invalid request parameters

## Testing

The provider includes comprehensive tests:

```bash
# Run integration tests
cargo test -p reev-agent --test glm_coding_integration_tests

# Run specific test
cargo test -p reev-agent test_basic_completion
```

Test coverage includes:
- Basic completion functionality
- Tool calling integration
- Multiple tools support
- Response format conversion
- Client builder functionality
- Environment configuration
- Error handling
- Serialization/deserialization

## Migration from GLM

If you're migrating from the previous GLM implementation:

1. **Update Environment Variables**:
   ```bash
   # Old
   export GLM_API_KEY="your_key"
   export GLM_API_URL="your_url"

   # New
   export GLM_CODING_API_KEY="your_key"
   export GLM_CODING_API_URL="your_url"
   ```

2. **Update Code References**:
   ```rust
   // Old
   use reev_agent::enhanced::glm_agent::GlmAgent;

   // New
   use reev_agent::enhanced::glm_coding_agent::GlmCodingAgent;
   ```

3. **Model Names**: Model names remain the same (`glm-4.6`)

## Best Practices

1. **API Key Security**: Never commit API keys to version control
2. **Error Handling**: Always handle potential API errors gracefully
3. **Rate Limiting**: Implement appropriate rate limiting for production use
4. **Tool Design**: Keep tool definitions simple and well-documented
5. **Temperature Settings**: Use lower temperatures (0.1-0.3) for coding tasks
6. **Max Tokens**: Set appropriate limits for your use case

## Troubleshooting

### Common Issues

**Q: Getting "GLM_CODING_API_KEY environment variable not set" error**
A: Ensure the environment variable is set before running your application

**Q: Tool calls are not being executed**
A: Check that tools are properly defined and included in the request

**Q: Responses are empty**
A: Verify API key is valid and has sufficient credits

**Q: Slow response times**
A: Consider reducing max_tokens or optimizing tool definitions

### Debug Mode

Enable debug logging to troubleshoot issues:

```bash
RUST_LOG=debug cargo run -p reev-agent --example your_example
```

## Contributing

When contributing to the GLM Coding provider:

1. Add tests for new features
2. Update documentation
3. Follow the existing code style
4. Test with real API calls when possible
5. Update this documentation as needed

## License

This provider is part of the reev project and follows the same license terms.
