# Implementation Tasks

## Issue #33: Flow Type Field Implementation - PENDING ⏳

### 🎯 **Objective**
Add `flow_type` field to benchmark YAML files to clearly distinguish between static and dynamic flow execution modes, fixing 300-series benchmark routing issues.

### 📋 **Current Problems**
1. 300 benchmark hardcoded as dynamic but uses deterministic agent incorrectly
2. No explicit configuration for flow execution mode in YML
3. Static vs dynamic routing depends on hardcoded benchmark ID patterns
4. Deterministic agent returns hardcoded responses defeating dynamic flow purpose
5. System tries to parse deterministic responses for dynamic benchmarks causing failures

### 🏗️ **Proposed Solution**
Add `flow_type: "dynamic"` field to YAML with default `"static"` behavior:
- **Static**: Use deterministic agent with pre-defined instruction generation
- **Dynamic**: Use LLM agent with Jupiter tools for real-time execution
- **Backward compatible**: Existing benchmarks default to static behavior
- **Clean separation**: No more hardcoded routing based on benchmark IDs

### 📝 **Implementation Steps**

#### Step 1: Update 300 Benchmark YML
```yaml
id: 300-jup-swap-then-lend-deposit-dyn
description: Dynamic multiplication strategy...
flow_type: "dynamic"  # <-- ADD THIS FIELD
tags: ["dynamic", "multiplication", "jupiter"]
prompt: "use my 50% sol to multiply usdc 1.5x on jup"
```

#### Step 2: Modify Agent Router (`crates/reev-agent/src/lib.rs`)
```rust
// In run_deterministic_agent function, add flow_type check
let flow_type = benchmark.get("flow_type")
    .and_then(|ft| ft.as_str())
    .unwrap_or("static");

match flow_type {
    "dynamic" => {
        // Skip deterministic agent, let LLM handle it directly
        info!("[reev-agent] Dynamic flow detected, skipping deterministic agent");
        anyhow::bail!("Use LLM agent for dynamic flows");
    }
    "static" | _ => {
        // Use current deterministic routing logic
        // ... existing handler calls ...
    }
}
```

#### Step 3: Update Runner Logic (`crates/reev-runner/src/lib.rs`)
```rust
// Modify evaluation loop to check flow_type
let flow_type = benchmark.get("flow_type")
    .and_then(|ft| ft.as_str())
    .unwrap_or("static");

let agent_type = match flow_type {
    "dynamic" => request.agent.clone(), // glamour, glm-4.6-coding, etc.
    "static" | _ => "deterministic".to_string(), // default
};
```

#### Step 4: Remove Hardcoded 300 Handler
Clean up the 300-specific code in `handle_flow_benchmarks()` since flow_type will handle routing properly.

#### Step 5: Add Flow Type Validation
```rust
// Add validation in benchmark synchronization
fn validate_flow_type(benchmark: &Value) -> Result<()> {
    if let Some(flow_type) = benchmark.get("flow_type").and_then(|ft| ft.as_str()) {
        match flow_type {
            "static" | "dynamic" => Ok(()),
            _ => anyhow::bail!("Invalid flow_type: {}", flow_type),
        }
    } else {
        Ok(()) // default to static
    }
}
```

### 🧪 **Testing Strategy**

#### Test 1: Static Flow (200 Benchmark)
```bash
# Should use deterministic agent as before
curl -X POST "http://localhost:3001/api/v1/benchmarks/200-jup-swap-then-lend-deposit/run" \
  -H "Content-Type: application/json" \
  -d '{"agent":"deterministic","mode":"static"}'
# Expected: Success with deterministic instructions, tool calls captured
```

#### Test 2: Dynamic Flow (300 Benchmark)  
```bash
# Should use LLM agent with Jupiter tools
curl -X POST "http://localhost:3001/api/v1/benchmarks/300-jup-swap-then-lend-deposit-dyn/run" \
  -H "Content-Type: application/json" \
  -d '{"agent":"glamour","mode":"dynamic"}'
# Expected: Success with LLM tool calls, real Jupiter execution
```

#### Test 3: Flow Type Validation
```bash
# Invalid flow_type should fail validation
# Missing flow_type should default to static
```

### 📊 **Expected Results**

#### Before Fix (Current State)
- 300 benchmark: ❌ "Agent returned no actions to execute"
- Tool calls: ❌ 0 tool calls captured
- Flow visualization: ❌ "Prompt --> Agent --> [*]" (no tool states)
- LLM usage: ❌ Deterministic agent used instead

#### After Fix (Expected State)
- 300 benchmark: ✅ Dynamic flow with LLM agent execution
- Tool calls: ✅ Jupiter swap/lend calls captured in OTEL
- Flow visualization: ✅ Detailed mermaid with tool states
- LLM usage: ✅ Glamour agent with actual Jupiter tools
- 200 benchmark: ✅ Static flow unchanged, backward compatible

### 🔧 **Files to Modify**
1. `benchmarks/300-jup-swap-then-lend-deposit-dyn.yml` - Add flow_type field
2. `crates/reev-agent/src/lib.rs` - Update agent routing logic
3. `crates/reev-runner/src/lib.rs` - Modify evaluation loop
4. `crates/reev-api/src/services/benchmark_executor.rs` - Handle flow_type
5. `crates/reev-lib/src/benchmark.rs` - Add flow_type validation

### ⚠️ **Breaking Changes**
None - backward compatible with existing benchmarks that lack flow_type field.

### 🎉 **Benefits**
1. **Clear Intent**: Explicit flow configuration in YML
2. **Clean Architecture**: No more hardcoded routing
3. **Backward Compatible**: Existing benchmarks unchanged
4. **Flexible**: Easy to add new flow types
5. **Testable**: Clear separation for testing

---

## Previous Issue #32: Jupiter Tool Call Capture - COMPLETED ✅

### **📊 Validation Results:**
- ✅ Jupiter swap tool captured with 1164ms execution time
- ✅ Enhanced OTEL logs containing full tool metadata
- ✅ Session ID propagation working correctly
- ✅ Tool calls captured in database for flow visualization

### **✅ Final Status:**
The dynamic benchmark system now has **complete Jupiter tool call integration** with enhanced OTEL logging and mermaid flow visualization! 

Both 200 and 300 benchmarks can now properly track:
- Jupiter swap operations (real tool execution)
- Jupiter lend operations (when depth limits allow)
- Transaction details, timing, and success metrics
- Complete flow sequences with proper state transitions

**All core tasks implemented and validated successfully.**