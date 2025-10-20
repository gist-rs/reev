# Flow Diagram System Implementation Plan

## 🎯 Overview
Implement Mermaid `stateDiagram` visualizations from agent execution logs via web API, displayed in UI hero section.

## 🏗️ Architecture: Decoupled Design

**Target Architecture:**
```
3rd Party Agent → reev-api (tool call tracking) → session logs → flow API → Web UI
```

**Key Decisions:**
- Tool tracking in reev-api layer (we control this)
- Session-based flows from real executions
- Remove reev-agent flow visualizer (deprecated)

## 📋 Requirements

### Backend
- Endpoint: `GET /api/v1/flows/{session-id}`
- Return Mermaid `stateDiagram` format
- Parse session logs for tool calls with timing
- Track tools in session logs array

### Web
- Display Mermaid diagram in hero section from grid box clicks
- Use Mermaid.js for rendering
- Session-specific execution flows

### Session Log Format
```json
{
  "session_id": "...",
  "benchmark_id": "...",
  "tools": [
    {
      "tool_name": "...",
      "start_time": "...",
      "end_time": "...",
      "params": {"pubkey": "..."},
      "result": {"balance": "..."},
      "status": "success|error"
    }
  ]
}
```

## 🚀 Implementation Plan (4 Phases)

### Phase 1: Tool Call Tracking in LlmAgent
- Add `active_tool_calls` and `tool_call_sequence` to LlmAgent
- Implement `start_tool_call()` and `end_tool_call()` methods
- Update SessionFileLogger to embed tools array

### Phase 2: StateDiagram Generator (Existing)
- Leverage existing StateDiagramGenerator module
- Update SessionParser for tools array support
- Remove deprecated OTEL parser

### Phase 3: Flow API Enhancement
- Update flow handler to use session_id
- No changes needed for session log integration
- Response format unchanged

### Phase 4: Testing & Integration
- cURL testing for quick validation
- Full localhost integration test
- Session format validation
- Flow diagram validation

## 🔧 Key Implementation Details

### LlmAgent Enhancement
```rust
pub struct LlmAgent {
    // existing fields...
    active_tool_calls: HashMap<String, ToolCallInfo>,
    tool_call_sequence: Vec<ToolCallInfo>,
}

impl LlmAgent {
    fn start_tool_call(&mut self, tool_name: &str, params: Value);
    fn end_tool_call(&mut self, tool_name: &str, result: Value, status: ToolStatus);
    pub fn get_tool_calls(&self) -> &[ToolCallInfo];
}
```

### Session Logger Integration
```rust
impl SessionFileLogger {
    pub fn complete_with_trace_and_tools(
        &mut self,
        session_id: &str,
        benchmark_id: &str,
        trace: String,
        tools: Vec<ToolCallInfo>,
    );
}
```

## ✅ Success Criteria

### Functional
- Generate valid Mermaid stateDiagram from session logs
- Accurate tool call sequence and timing
- Session-specific flow visualization
- Web UI displays diagrams correctly

### Non-Functional
- Performance: <500ms response time
- Reliability: Graceful error handling
- Maintainability: Modular, testable code

### Integration
- Backward compatible with existing APIs
- Real tool calls from actual agent executions
- Clean rollback strategy

## 🔄 Next Steps

1. Implement Phase 1: Tool call tracking
2. Quick cURL test validation
3. Verify Phase 2: Existing flow system
4. Integration test following benchmarks_test.rs pattern
5. Phase 3: API endpoint update
6. End-to-end testing
7. Documentation updates
8. Web UI integration

## 🧪 Quick Testing

```bash
# 1. Start reev-api
cargo run --bin reev-api

# 2. Run benchmark with tool calls
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{"agent": "local"}'

# 3. Get flow diagram
curl http://localhost:3001/api/v1/flows/{session_id}
```

This plan ensures systematic implementation with clear success criteria and risk mitigation.
