# GLM Agent Tool Calling Implementation

## Overview

This document describes the GLM agent's tool calling capabilities, which allow the GLM-4.6 model to execute functions and use external tools to provide more accurate and useful responses.

**⚠️ CURRENT STATUS**: Tool calling infrastructure is complete, but GLM's response format is incompatible with the Rig framework's OpenAI client.

## Architecture

### Core Components

1. **GLM Agent Structure** - Custom implementation for GLM's unique response format
2. **Tool Definitions** - JSON schemas describing available tools (currently mock tools)
3. **Response Processing** - Handles GLM's reasoning_content and tool_calls format
4. **Execution Loop** - Manages multi-turn conversations with tools

### Key Files

- `src/enhanced/glm_agent.rs` - Custom GLM agent with response format handling
- `src/enhanced/openai.rs` - Has GLM URL config but response parsing fails
- `src/run.rs` - GLM routing to custom agent (reverted from OpenAI agent)
- `tests/glm_tool_call_test.rs` - Comprehensive test suite
- `examples/glm_tool_call_demo.rs` - Interactive demonstration
- `run_glm_demo.sh` - Demo runner script

## Tool Calling Flow

### 1. Request Initialization
```rust
let request = LlmRequest {
    id: "unique-id".to_string(),
    prompt: "What time is it in Tokyo?".to_string(),
    context_prompt: "You are a helpful assistant with access to time and weather tools.".to_string(),
    model_name: "glm-4.6".to_string(),
    mock: false,
    initial_state: None,
    allowed_tools: None,
};
```

### 2. Tool Definition
The agent automatically creates tool definitions for available functions:

```json
{
  "type": "function",
  "function": {
    "name": "get_current_time",
    "description": "Get the current time in a specific timezone",
    "parameters": {
      "type": "object",
      "properties": {
        "timezone": {
          "type": "string",
          "description": "The timezone to get the time for, e.g., UTC, America/New_York"
        }
      },
      "required": ["timezone"]
    }
  }
}
```

### 3. Tool Execution Loop
1. Send request with tool definitions to GLM API
2. Receive response with potential tool calls
3. Execute tools with provided parameters
4. Send tool results back in conversation
5. Repeat until final response is received

### Available Tools

#### Current Implementation (Mock Tools)
- **get_current_time** - Get current time for any timezone
- **get_weather** - Get weather information for any location

### Tool Response Format
```json
{
  "time": "14:30:00",
  "timezone": "Asia/Tokyo",
  "timestamp": 1697994600
}
```

## Usage Examples

### Basic Query with Tool
```rust
let response = GlmAgent::run(
    "glm-4.6",
    request,
    HashMap::new()
).await?;
```

### Expected Response Format
```json
{
  "result": {
    "success": true,
    "transactions": [],
    "summary": "The current time in Tokyo is 23:30:00 JST.",
    "signatures": [],
    "flows": null
  }
}
```

## Testing

### Running Tests
```bash
# Set environment variables
export ZAI_API_KEY=your_api_key_here
export GLM_API_URL=https://api.z.ai/api/coding/paas/v4

# Run specific test
cargo test -p reev-agent --test glm_tool_call_test -- --ignored

# Run all GLM tests
cargo test -p reev-agent glm -- --ignored
```

### Test Coverage
- Tool detection and execution
- Parameter parsing and validation
- Multi-tool coordination
- Direct response fallback
- Error handling

### Test Cases
1. **Time Query** - Triggers get_current_time tool
2. **Weather Query** - Triggers get_weather tool
3. **Multiple Tools** - Coordinates time and weather tools
4. **Direct Response** - Handles queries without tool usage
5. **Parameter Parsing** - Validates complex timezone formats

## Demo

### Interactive Demo
```bash
# Set API key
export ZAI_API_KEY=your_api_key_here

# Run demo
./crates/reev-agent/run_glm_demo.sh
```

### Demo Scenarios
1. Time query with timezone
2. Weather query with location
3. Multiple tool calls
4. Direct response without tools
5. Complex multi-tool request

## Environment Variables

### Required
- `ZAI_API_KEY` - Your ZAI API key from z.ai

### Optional
- `GLM_API_URL` - GLM API endpoint (defaults to https://api.z.ai/api/coding/paas/v4)

## Integration with Runner System

### Preparation for Benchmark Integration
The GLM tool calling implementation is designed to integrate seamlessly with the runner benchmark system:

1. **Standardized Response Format** - Consistent with other agents
2. **Tool Abstraction** - Can be extended with reev-tools
3. **Error Handling** - Robust error recovery
4. **Performance Metrics** - Built-in execution tracking

### Next Steps for Integration
1. Add reev-tools integration (Jupiter, Solana operations)
2. Implement proper tool registration system
3. Add performance benchmarking
4. Integrate with runner workflow

## Error Handling

### Common Errors
1. **Missing API Key** - Set ZAI_API_KEY environment variable
2. **Invalid Parameters** - Tool validates parameters before execution
3. **Network Issues** - Automatic retry with exponential backoff
4. **Tool Execution Failure** - Graceful fallback to direct response

### Error Response Format
```json
{
  "result": {
    "success": false,
    "transactions": [],
    "summary": "Error: Unable to execute tool - Invalid timezone format",
    "signatures": [],
    "flows": null
  }
}
```

## Performance Considerations

### Optimization Strategies
1. **Tool Caching** - Cache tool definitions to reduce API calls
2. **Parameter Validation** - Early validation to prevent unnecessary API calls
3. **Concurrent Execution** - Parallel tool execution for independent tools
4. **Response Streaming** - Stream responses for better UX

### Metrics Tracked
- Tool execution time
- API response latency
- Tool usage frequency
- Error rates

## Security Considerations

### Parameter Sanitization
- All tool parameters are validated before execution
- JSON parsing errors are caught and handled gracefully
- Malicious parameter injection is prevented

### API Key Security
- API keys are loaded from environment variables only
- No hardcoded credentials in source code
- Secure transmission to GLM API endpoints

## Current Challenges & Solutions

### Primary Challenge: Response Format Incompatibility
**Issue**: GLM's response format prevents Rig framework integration
**Impact**: Cannot use real reev-tools via OpenAI agent
**Current Solution**: Custom GLM agent with mock tools
**Long-term Solution**: Either fix Rig framework or enhance custom agent

### Trade-offs Analysis
```
Option 1: OpenAI Agent + GLM
✅ Real reev-tools (SolTransferTool, JupiterSwapTool)
✅ Automatic OTEL tracking
❌ Response parsing fails (GLM format incompatibility)

Option 2: Custom GLM Agent
✅ GLM response parsing works
❌ Mock tools only (currently)
❌ No automatic OTEL tracking
```

### Implementation Roadmap
#### Phase 1: Immediate Solution (Custom Agent)
1. Add real reev-tools to custom GLM agent
2. Manual OTEL tracking implementation
3. Test with real Solana operations

#### Phase 2: Long-term Solution (Framework Fix)
1. Investigate Rig framework extension for GLM
2. Create GLM response transformer
3. Unified OTEL tracking across all agents

## Future Enhancements

### Planned Features
1. **Real reev-tools Integration** - Add Solana/Jupiter tools to custom agent
2. **Manual OTEL Tracking** - OpenTelemetry integration for custom GLM agent
3. **Dynamic Tool Registration** - Runtime tool addition/removal
4. **Response Format Transformation** - Bridge GLM and OpenAI formats

### Integration Opportunities
1. **Solana Tools** - Direct blockchain operations via custom agent
2. **Jupiter Integration** - DeFi protocol tools with manual integration
3. **Framework Compatibility** - Enable GLM to use Rig framework tools
4. **Custom Business Logic** - Domain-specific tools with GLM support

## Troubleshooting

### Common Issues

1. **Tool Not Called**
   - Verify prompt requires tool usage
   - Check tool definition format
   - Review model parameters

2. **Parameter Errors**
   - Validate JSON structure
   - Check required parameters
   - Verify parameter types

3. **API Connection Issues**
   - Check ZAI_API_KEY validity
   - Verify GLM_API_URL accessibility
   - Review network connectivity

### Debug Mode
Enable debug logging:
```rust
tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();
```

## Conclusion

The GLM agent's tool calling implementation provides a robust foundation for integrating external tools with the GLM-4.6 model. It handles complex multi-turn conversations, parameter validation, and error recovery while maintaining compatibility with the existing reev ecosystem.

The implementation is ready for integration with the runner benchmark system and can be extended with additional tools as needed for specific use cases.
