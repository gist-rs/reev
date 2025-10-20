# Handover: Flow Visualization Tool Call Tracking

## Current State (2025-10-20)

### ✅ What's Working
1. **Flow API**: `/flows/{session_id}` endpoint working correctly
2. **Session logs**: Generated with proper structure in `logs/sessions/`
3. **Flow diagrams**: Basic stateDiagram generation working
4. **Architecture**: Decoupled design implemented (reev-api layer tracking)

### ❌ What's Broken
1. **Tool Call Tracking**: `tools: []` arrays are empty in session logs
2. **OTEL Integration**: Created spans but not extracting data correctly
3. **Agent Dependency**: Relying on agent code changes (wrong approach)

## 🎯 Real Challenge

**We must extract tool calls from OpenTelemetry traces via reev-api layer**

### Current Misconceptions
- ❌ Relying on agent code changes (we can't control 3rd party agents)
- ❌ Modifying LlmAgent logging (wrong layer)
- ❌ Binary OTEL format parsing (not needed)

### Correct Approach
- ✅ **OTEL spans are being created** in reev-api layer around HTTP requests
- ✅ **Flow visualization is now handled by reev-api** web interface
- ✅ **Session-based flow diagrams** generated via API endpoints

## 📋 Next Steps (CRITICAL)

### Step 1: Capture OTEL Trace Data in reev-api
```rust
// In LlmAgent::get_action() - AFTER HTTP request
// Extract trace data from current OTEL span
let trace_data = extract_current_otel_trace();
// Convert to ToolCallInfo format
let tool_calls = parse_otel_trace_to_tools(trace_data);
// Store in session logs
```

### Step 2: OTEL Trace Extraction Methods
- Find how to get current span data from global tracer
- Parse OTEL span attributes to extract tool information
- Map HTTP request/response to logical tool calls

### Step 3: Integration Points
- Hook into existing LlmAgent HTTP request/response cycle
- Parse LLM responses to identify intended tool actions
- Convert to existing `ToolCallInfo` structure format

## 🔧 Implementation Focus Areas

### 1. OTEL Data Extraction (HIGH PRIORITY)
```rust
// Need to implement these functions:
fn extract_current_otel_trace() -> OtelTraceData
fn parse_otel_trace_to_tools(trace: OtelTraceData) -> Vec<ToolCallInfo>
```

### 2. Response Parsing (HIGH PRIORITY)
- Parse LLM responses to identify tool intentions
- Map "get account balance" → "get_account_balance" tool call
- Extract parameters and results from natural language

### 3. Integration (MEDIUM PRIORITY)
- Modify LlmAgent to call extraction methods
- Update session logging to include extracted tools
- Test with real agent responses

## 📁 Key Files to Modify

### Primary Targets
- `crates/reev-lib/src/llm_agent.rs` - Add OTEL extraction after HTTP calls
- `crates/reev-lib/src/otel_extraction.rs` - New module for trace parsing

### Secondary Targets
- `crates/reev-lib/src/session_logger/mod.rs` - Tool integration (already works)
- `crates/reev-runner/src/lib.rs` - Pass tool calls (already works)

## 🧪 Testing Strategy

### 1. Manual Testing
```bash
# Run benchmark with real agent
curl -X POST http://localhost:3001/api/v1/benchmarks/100-jup-swap-sol-usdc/run \
  -H "Content-Type: application/json" \
  -d '{"agent": "glm-4.6"}'

# Check session logs for tools array
cat logs/sessions/session_*.json | jq '.final_result.tools'
```

### 2. Flow API Testing
```bash
# Test flow diagram with extracted tools
curl http://localhost:3001/api/v1/flows/{session_id}
```

## 🎯 Success Criteria

### MUST HAVE
- ✅ Session logs contain non-empty `tools: []` arrays
- ✅ Tool calls have proper timing (start_time, end_time)
- ✅ Tool calls have parameters and results
- ✅ Flow diagrams show real tool execution paths

### NICE TO HAVE
- ✅ Multiple tool calls in single session
- ✅ Error handling for failed tool calls
- ✅ Tool categorization (swap, transfer, etc.)

## 🚨 BLOCKERS

### Current Blocker
- **How to extract OTEL trace data from global tracer?**
- **What's the correct OTEL API for getting current span?**
- **How to parse LLM responses for tool intentions?**

### Need Research
- OpenTelemetry Rust API documentation
- OTEL span data extraction methods
- LLM response parsing for tool detection

## 📚 Resources

### OpenTelemetry Rust
- https://docs.rs/opentelemetry/0.30/opentelemetry/
- Focus on trace span data extraction

### Existing Code Patterns
- `reev-agent/src/enhanced/openai.rs` - Tool logging format
- `reev-agent/src/flow/visualization/` - Parsing patterns

### Working Examples
- OTEL spans are created (seen in logs)
- Flow API works with mock data
- Session structure supports tools array

## 🔄 Next Developer

Focus should be on:
1. **OTEL trace extraction research** - Find correct API calls
2. **Implement extraction methods** - Get trace data programmatically
3. **Integration testing** - Ensure tools appear in session logs

This is the final piece to make flow visualization work with real agent executions!
