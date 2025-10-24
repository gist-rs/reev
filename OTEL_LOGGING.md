# Enhanced OpenTelemetry Logging for Tool Calls

## Overview

This document describes the enhanced OpenTelemetry (otel) logging capabilities for tracking tool call information in the Reev system. The enhanced logging provides detailed visibility into tool execution, parameters, results, and performance metrics.

## Features

### 1. Enhanced Tool Call Logging
- **Tool Name Tracking**: Automatically logs the name of each tool being executed
- **Parameter Logging**: Captures all input parameters passed to tools
- **Execution Timing**: Tracks start time, completion time, and duration
- **Result Logging**: Records tool execution results and outputs
- **Error Tracking**: Captures detailed error information when tools fail

### 2. Standardized Logging Macros
Two main macros provide consistent logging across all tools:

#### `log_tool_call!`
```rust
log_tool_call!("tool_name", &args);
```
- Records tool start time
- Logs all input parameters as otel attributes
- Sets tool name in current span

#### `log_tool_completion!`
```rust
log_tool_completion!("tool_name", execution_time_ms, &result, success);
```
- Records execution completion time
- Logs execution results
- Sets success/error status
- Captures error details on failure

### 3. Environment Variable Control
Enhanced otel logging is **enabled by default** and can be controlled via environment variable:

```bash
# Enhanced otel logging is enabled by default (no action needed)

# Disable enhanced otel logging if needed
export REEV_ENHANCED_OTEL=0

# Re-enable explicitly (optional)
export REEV_ENHANCED_OTEL=1
```

## Implementation Details

### Tool Integration
Tools automatically get enhanced logging when they use the provided macros:

```rust
#[instrument(skip(self), fields(tool_name = "sol_transfer"))]
async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
    // Start enhanced logging
    crate::log_tool_call!("sol_transfer", &args);
    
    let start_time = Instant::now();
    
    // Tool execution logic...
    
    let execution_time = start_time.elapsed().as_millis() as u32;
    
    match result {
        Ok(output) => {
            let result_data = json!({"key": "value"});
            crate::log_tool_completion!("sol_transfer", execution_time, &result_data, true);
        }
        Err(e) => {
            let error_data = json!({"error": e.to_string()});
            crate::log_tool_completion!("sol_transfer", execution_time, &error_data, false);
        }
    }
}
```

### Otel Attributes Structure
The enhanced logging creates standardized otel attributes:

#### Tool Start Attributes
- `tool.name`: Name of the tool being executed
- `tool.start_time`: ISO 8601 timestamp when tool started
- `tool.args.*`: All input parameters (e.g., `tool.args.amount`, `tool.args.pubkey`)

#### Tool Completion Attributes
- `tool.execution_time_ms`: Execution time in milliseconds
- `tool.completion_time`: ISO 8601 timestamp when tool completed
- `tool.status`: "success" or "error"
- `tool.result.*`: Output data fields
- `tool.error.message`: Error message (if failed)

## Usage Examples

### Running with Enhanced Otel Logging
```bash
# Enhanced otel logging is enabled by default, just set trace file and run
REEV_TRACE_FILE=traces.log RUST_LOG=info cargo run -p reev-runner -- benchmarks/100-jup-swap-sol-usdc.yml --agent local

# Or disable if needed
REEV_ENHANCED_OTEL=0 REEV_TRACE_FILE=traces.log RUST_LOG=info cargo run -p reev-runner -- benchmarks/100-jup-swap-sol-usdc.yml --agent local
```

### Viewing Otel Trace Data
The enhanced logging integrates with the existing otel extraction system:

```rust
// Extract tool calls from current otel trace
if let Some(otel_trace) = reev_lib::otel_extraction::extract_current_otel_trace() {
    let tool_calls = reev_lib::otel_extraction::parse_otel_trace_to_tools(otel_trace);
    
    for tool_call in tool_calls {
        println!("Tool: {}", tool_call.tool_name);
        println!("Args: {}", tool_call.tool_args);
        println!("Status: {:?}", tool_call.result_status);
        println!("Time: {}ms", tool_call.execution_time_ms);
    }
}
```

## Benefits

### 1. Debugging
- Detailed parameter tracking helps identify incorrect input values
- Error context provides quick identification of failure causes
- Execution timing helps identify performance bottlenecks

### 2. Monitoring
- Standardized metrics enable consistent monitoring across all tools
- Performance data helps identify slow or problematic tools
- Error rates can be tracked per tool type

### 3. Auditing
- Complete audit trail of tool execution with parameters and results
- Timestamps provide chronological order of operations
- Success/failure status enables reliability analysis

### 4. Flow Visualization
- Enhanced data supports better Mermaid diagram generation
- Tool sequence and timing information for flow analysis
- Parameter and result data for detailed operation tracking

## Current Implementation Status

### ✅ Implemented
- Enhanced logging macros in `reev/crates/reev-agent/src/enhanced/common/mod.rs`
- SolTransferTool integration in `reev/crates/reev-agent/src/tools/native.rs`
- JupiterSwapTool integration in `reev/crates/reev-agent/src/tools/jupiter_swap.rs`
- Enhanced otel extraction in `reev/crates/reev-lib/src/otel_extraction/mod.rs`
- Environment variable control (`REEV_ENHANCED_OTEL`)

### 🔄 In Progress
- Integration with remaining tools (Jupiter Lend/Earn tools)
- SPL tool integration
- Balance tool integration

### 📋 Planned
- Performance dashboard integration
- Alerting on tool failures
- Tool-specific metrics aggregation
- Integration with external monitoring systems

## Troubleshooting

### Logs Not Appearing
1. Enhanced otel logging is enabled by default, verify it's not disabled (`REEV_ENHANCED_OTEL` != "0")
2. Check `RUST_LOG` includes appropriate level (info or debug)
3. Ensure tools are using the enhanced logging macros

### Performance Impact
- Enhanced logging adds minimal overhead (< 1ms per tool call)
- Can be disabled by unsetting `REEV_ENHANCED_OTEL`
- Only affects development and debugging, not production execution

### Missing Attributes
- Ensure tool structs implement `Serialize` for JSON conversion
- Check that all fields are public or have proper serialization attributes
- Verify tool parameters are properly formatted in JSON

## Best Practices

1. **Always Use Macros**: Use `log_tool_call!` and `log_tool_completion!` for consistency
2. **Include Context**: Log relevant context in result data for better debugging
3. **Error Handling**: Always log errors with sufficient context
4. **Performance**: Keep parameter serialization lightweight for performance
5. **Consistency**: Use standardized field names across similar tools