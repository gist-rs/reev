# Development & Testing Guide: API Flow Visualization

## 🎯 **Purpose**
This guide provides curl commands for testing and developing the API flow visualization functionality, focusing on dynamic flows with OTEL integration at orchestrator level.

## 🏗️ **Architecture Update**
**Flow:** `Agent → Orchestrator (OTEL) → JSON + OTEL → DB → YML Parser → Mermaid`

- OTEL initialization moved from agent level to orchestrator level
- Unified tracing across all agents (ZAI, OpenAI, future agents)
- Single OTEL session per flow execution
- Step-by-step ping-pong coordination with OTEL capture

## 🚀 **Quick Start**

### **1. Start API Server**
```bash
cd reev
pkill -f reev-api  # Clean restart
RUST_LOG=info cargo run --bin reev-api --quiet > api_server_test.log 2>&1 &
sleep 3  # Wait for startup
```

### **2. Verify Server Health**
```bash
curl -s http://localhost:3001/api/v1/benchmarks | jq length
# Should return: 18+
```

## 🧪 **Flow Testing Scenarios**

### **Scenario 1: Simple SOL → USDC Swap**
```bash
# Execute flow
curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "swap 0.01 SOL for USDC",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }' | jq -r '.result.flow_id'

# Get flow ID
FLOW_ID=$(curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "swap 1 SOL for USDC",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }' | jq -r '.result.flow_id')

# Visualize flow
curl -s "http://localhost:3001/api/v1/flows/$FLOW_ID" | jq .
```

### **Scenario 2: Complex Multi-Step Flow (Issue #10 RESOLVED)**
```bash
# Execute multiplication strategy
FLOW_ID=$(curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "use my 50% sol to multiply usdc 1.5x on jup",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }' | jq -r '.result.flow_id')

# Check flow visualization (RESOLVED: Enhanced OTEL format compatibility)
curl -s "http://localhost:3001/api/v1/flows/$FLOW_ID" | jq '.metadata | {tool_count, state_count, session_id}'

# Check diagram (RESOLVED: Rich tool call parameters from OTEL)
# Enhanced OTEL logging captures tool execution details for flow visualization
 curl -s "http://localhost:3001/api/v1/flows/$FLOW_ID" | jq -r '.diagram'
```

### **Scenario 3: Bridge Mode with YML Generation**
```bash
# Execute bridge mode
FLOW_ID=$(curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-bridge \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "use 75% of my SOL to get maximum USDC yield on Jupiter",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "shared_surfpool": true
  }' | jq -r '.result.flow_id')

# Check if YML file was created
curl -s "http://localhost:3001/api/v1/flows/$FLOW_ID" | jq '.result | keys'
```

### **Scenario 4: Recovery Mode Testing**
```bash
# Execute with recovery configuration
curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-recovery \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "swap all my SOL to USDC with maximum yield",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "recovery_config": {
      "base_retry_delay_ms": 1000,
      "max_retry_delay_ms": 10000,
      "backoff_multiplier": 2.0,
      "max_recovery_time_ms": 30000,
      "enable_alternative_flows": true,
      "enable_user_fulfillment": false
    }
  }' | jq '.result.execution_mode'
```

## 🔍 **Debugging & Validation**

### **Check Server Logs**
```bash
# Real-time log monitoring
tail -f api_server_test.log | grep -E "(flow|session|tool)"

# Check for errors
grep -i error api_server_test.log | tail -10
```

### **Validate Flow Components**
```bash
# Check tool calls in detail
FLOW_ID="dynamic-1762252083-26f0eb3b"  # Replace with actual ID
curl -s "http://localhost:3001/api/v1/flows/$FLOW_ID" | jq '.tool_calls[0]'

# Check metadata
curl -s "http://localhost:3001/api/v1/flows/$FLOW_ID" | jq '.metadata'

# Check sessions (should show execution data)
curl -s "http://localhost:3001/api/v1/flows/$FLOW_ID" | jq '.sessions'
```

### **Database Inspection**
```bash
# Check if session logs are stored
sqlite3 db/reev_results.db "SELECT session_id, length(log_content) FROM session_logs ORDER BY created_at DESC LIMIT 5;"

# Check session log content
SESSION_ID="dynamic-1762252083-26f0eb3b"
sqlite3 db/reev_results.db "SELECT log_content FROM session_logs WHERE session_id = '$SESSION_ID';" | jq .
```

## 📊 **Expected vs Actual Results**

### **✅ Current Status: Real Agent Execution Working**
```bash
# Test multi-step flow:
curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{"prompt": "use 50% of my SOL to get USDC yield on jupiter", "wallet": "...", "agent": "glm-4.6-coding"}'

# Output: Shows real tool execution
{"tool_calls":[{"tool_name":"account_balance",...},{"tool_name":"jupiter_swap",...}],"metadata":{"tool_count":2}}
```

**Resolved Issues:**
- ✅ Real GLM agent execution via ping-pong orchestrator
- ✅ Multi-step flows (4 steps generated, executed sequentially)
- ✅ Progress tracking (partial completion: 2/4 steps = 50%)
- ✅ Error handling (critical failures terminate flow)
- ✅ OTEL integration at orchestrator level

**Next Enhancement:** YML step references for ping-pong flow connections

### **✅ Desired: Information-Rich Visualization**
### **✅ ACHIEVED: Information-Rich Visualization** (Issue #10 RESOLVED)
**Enhanced OTEL Integration Provides:**
- Tool call parameters (SOL amounts, USDC amounts, Jupiter settings)
- Transaction signatures and execution results
- Real-time error tracking and success rates
- Unified agent tracing across GLM models via orchestrator

**Flow Visualization Now Shows:**
```mermaid
stateDiagram
    [*] --> Swap_2_SOL
    Swap_2_SOL --> Check_USDC_Balance
    Check_USDC_Balance --> Lend_USDC
    Lend_USDC --> [*]

note right of Swap_2_SOL: Enhanced OTEL captures<br/>input: 2.0 SOL<br/>output: 300.0 USDC<br/>signature: captured
note right of Lend_USDC: Enhanced OTEL tracks<br/>input: 300.0 USDC<br/>APY: 5.8%<br/>position: logged
```

## 🛠️ **Development Tasks**

### **Phase 1: Fix Current Mock Data (Immediate)**
```bash
# Test current mock generation
curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{"prompt": "swap 0.5 SOL", "wallet": "debug_wallet", "agent": "glm-4.6-coding", "shared_surfpool": false}' | \
  jq '.tool_calls | length'

# Expected: 1 tool call
# Check: curl -s "http://localhost:3001/api/v1/flows/$FLOW_ID" | jq '.tool_calls[0]'
```

### **Phase 2: Real Execution Integration (Major)**
**Target:** Replace mock data with actual glm-4.6 agent execution results.

**Current Location:** `crates/reev-orchestrator/src/execution/ping_pong_executor.rs`
**Function:** `execute_flow_plan()` - ✅ IMPLEMENTED

**Completed Changes:**
1. ✅ Execute actual glm-4.6 agent with generated flow plan via ping-pong executor
2. ✅ Capture real tool calls, parameters, results via orchestrator-level OTEL
3. ✅ Store transaction signatures, amounts, addresses in dual capture (JSON + OTEL)
4. ✅ OTEL integration at orchestrator level for unified agent tracing

### **Phase 3: Enhanced Visualization (Polish)**
**Target:** Rich flow diagrams with meaningful information.

## 🧪 **Test Script for Validation**

```bash
#!/bin/bash
# test_flow_validation.sh

echo "🧪 Testing API Flow Visualization..."

# Test 1: Basic functionality
echo "📋 Test 1: Basic flow execution"
RESPONSE=$(curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{"prompt": "swap 0.5 SOL", "wallet": "auto_test", "agent": "glm-4.6-coding", "shared_surfpool": false}')

FLOW_ID=$(echo $RESPONSE | jq -r '.result.flow_id')
TOOL_COUNT=$(echo $RESPONSE | jq '.tool_calls | length')

echo "✅ Flow ID: $FLOW_ID"
echo "✅ Tool Count: $TOOL_COUNT"

# Test 2: Flow visualization
echo "📋 Test 2: Flow visualization"
FLOW_RESPONSE=$(curl -s "http://localhost:3001/api/v1/flows/$FLOW_ID")
VISUAL_TOOL_COUNT=$(echo $FLOW_RESPONSE | jq '.metadata.tool_count')
DIAGRAM_STATES=$(echo $FLOW_RESPONSE | jq '.metadata.state_count')

echo "✅ Visualization Tool Count: $VISUAL_TOOL_COUNT"
echo "✅ Diagram States: $DIAGRAM_STATES"

# Test 3: Information quality
echo "📋 Test 3: Information quality check"
TOOL_DETAILS=$(echo $FLOW_RESPONSE | jq '.tool_calls[0]')
HAS_AMOUNT=$(echo $TOOL_DETAILS | jq 'has("input_amount")')
HAS_SIGNATURE=$(echo $TOOL_DETAILS | jq 'has("tx_signature")')

if [ "$HAS_AMOUNT" = "true" ] && [ "$HAS_SIGNATURE" = "true" ]; then
    echo "✅ Tool calls contain real execution data"
else
    echo "❌ Tool calls are mock/synthetic - ISSUE CONFIRMED"
fi

echo "🎉 Test completed!"
```

## 📝 **Development Notes**

### **Key Files Modified:**
- `crates/reev-orchestrator/src/execution/ping_pong_executor.rs` - Real agent execution
- `crates/reev-orchestrator/src/gateway.rs` - Ping-pong coordination
- `crates/reev-agent/src/run.rs` - GLM model routing
- `crates/reev-api/src/handlers/dynamic_flows/mod.rs` - Flow execution API
- `crates/reev-orchestrator/Cargo.toml` - Added reev-agent dependency

### **Next Implementation:**
- ✅ COMPLETED: OTEL logging at orchestrator level (Issue #17)
- ✅ COMPLETED: Session log management at orchestrator level
- ✅ RESOLVED: YML step reference system for ping-pong connections (Issue #10)

### **✅ ACHIEVED Data Flow:**
1. User sends prompt → API
2. Orchestrator creates flow plan
3. **✅ IMPLEMENTED**: Execute agent via ping-pong coordination with OTEL
4. **✅ IMPLEMENTED**: Capture real tool execution data via orchestrator-level OTEL
5. **✅ IMPLEMENTED**: Store dual data (JSON for immediate use + OTEL for rich traces)
6. **✅ IMPLEMENTED**: Visualize execution flow with unified agent tracing

### **✅ TESTED Working Flow:**
```bash
# Both GLM agents working with unified OTEL tracing
curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{"prompt": "swap 1 SOL for USDC", "agent": "glm-4.6-coding", ...}'
# ✅ Response: {"tool_calls":[{"tool_name":"jupiter_swap",...}],"status":"Completed"}

curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{"prompt": "swap 1 SOL for USDC", "agent": "glm-4.6", ...}'
# ✅ Response: {"tool_calls":[{"tool_name":"jupiter_swap",...}],"status":"Completed"}
```
---

### **✅ Execute 300 Benchmark:**
```bash
curl -s -X POST http://localhost:3001/api/v1/benchmarks/300-jup-swap-then-lend-deposit-dyn/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "glm-4.6-coding"
  }' | jq '.'
```

### **✅ Get Mermaid Diagram:**
```bash
# Get execution_id from response, then:
FLOW_ID="5e05380f-627d-4db1-a2bc-b549126a7cf1"
curl -s "http://localhost:3001/api/v1/flows/$FLOW_ID" | jq -r '.diagram'
```

### **✅ Check Execution Status:**
```bash
curl -s "http://localhost:3001/api/v1/benchmarks/300-jup-swap-then-lend-deposit-dyn/status/$EXECUTION_ID" | jq .
```
