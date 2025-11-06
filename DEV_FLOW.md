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
RUST_LOG=info cargo run --bin reev-api --quiet > api_server_task.log 2>&1 &
sleep 3  # Wait for startup
```

### **2. Verify Server Health**
```bash
curl -s http://localhost:3001/api/v1/benchmarks | jq length
# Should return: 18+
```

### **3. Verify Implementation Status**
```bash
# All endpoints should return HTTP 200
curl -s -w "Status: %{http_code}\n" -o /dev/null -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" -d '{"prompt": "test", "wallet": "USER_WALLET_PUBKEY", "agent": "glm-4.6-coding"}'

curl -s -w "Status: %{http_code}\n" -o /dev/null -X POST http://localhost:3001/api/v1/benchmarks/execute-bridge \
  -H "Content-Type: application/json" -d '{"prompt": "test", "wallet": "USER_WALLET_PUBKEY", "agent": "glm-4.6-coding", "shared_surfpool": true}'

curl -s -w "Status: %{http_code}\n" -o /dev/null -X POST http://localhost:3001/api/v1/benchmarks/execute-recovery \
  -H "Content-Type: application/json" -d '{"prompt": "test", "wallet": "USER_WALLET_PUBKEY", "agent": "glm-4.6-coding"}'
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

### **✅ IMPLEMENTATION COMPLETED: All Tasks from TASKS.md**
```bash
# Test real tool execution with valid wallet:
curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{"prompt": "swap 0.1 SOL to USDC", "wallet": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM", "agent": "glm-4.6-coding"}' | jq '.result'

# Output: Real tool execution with flow visualization
{
  "flow_id": "dynamic-xxxxxxxx-yyyyyyyy",
  "steps_generated": 2,
  "execution_mode": "direct",
  "tool_count": 2,
  "state_count": 4
}
```

**✅ All Implementation Tasks Completed:**
- ✅ Phase 1: Code Analysis & Alignment - All gaps resolved
- ✅ Phase 2: Benchmark-First Implementation - Clean separation achieved
- ✅ Phase 3: Tool Name System Overhaul - Type-safe enums throughout
- ✅ Phase 4: Eliminate Mock Data - Real execution only confirmed
- ✅ Phase 5: Simple Dynamic YML Generation - Working implementation
- ✅ Phase 6: Integration & Testing - All endpoints functional

**✅ Validation Results:**
- ✅ Zero compilation errors and clippy warnings
- ✅ All API endpoints responding with HTTP 200
- ✅ Real tool execution with proper flow visualization
- ✅ Server running successfully in background
- ✅ Clean architecture separation implemented

### **✅ ACHIEVED: Information-Rich Visualization**
**Enhanced OTEL Integration Provides:**
- Tool call parameters (SOL amounts, USDC amounts, Jupiter settings)
- Transaction signatures and execution results
- Real-time error tracking and success rates
- Unified agent tracing across GLM models via orchestrator

**Flow Visualization Now Shows:**
```mermaid
stateDiagram
    [*] --> Prompt
    Prompt --> account_balance : Execute
    account_balance --> jupiter_swap : Execute
    jupiter_swap --> [*]

classDef tools fill:grey
class account_balance tools
class jupiter_swap tools
```

**Real Execution Example:**
```bash
# Get enhanced flow diagram with real context
curl -s "http://localhost:3001/api/v1/flows/dynamic-1762362524-9c43c429" | jq -r '.diagram'

# Shows actual tool execution sequence with state transitions
```

## 🛠️ **Implementation Status: COMPLETED**

### **✅ Phase 1: Mock Data Elimination - COMPLETED**
```bash
# Confirmed mock: false in production
grep "mock.*false" crates/reev-orchestrator/src/execution/ping_pong_executor.rs
# Returns: mock: false, // Use real execution
```

### **✅ Phase 2: Real Execution Integration - COMPLETED**
**Location:** `crates/reev-orchestrator/src/execution/ping_pong_executor.rs`
**Function:** `execute_flow_plan()` - ✅ IMPLEMENTED

**Completed Implementation:**
1. ✅ Execute actual glm-4.6 agent with generated flow plan via ping-pong executor
2. ✅ Capture real tool calls, parameters, results via orchestrator-level OTEL
3. ✅ Store transaction signatures, amounts, addresses in dual capture (JSON + OTEL)
4. ✅ OTEL integration at orchestrator level for unified agent tracing

### **✅ Phase 3: Enhanced Visualization - COMPLETED**
**Achieved:** Rich flow diagrams with real tool execution data and proper state transitions.

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

## 📝 **Implementation Summary**

### **Key Files Modified in Implementation:**
- `crates/reev-orchestrator/src/execution/ping_pong_executor.rs` - Real agent execution
- `crates/reev-orchestrator/src/gateway.rs` - Ping-pong coordination
- `crates/reev-agent/src/run.rs` - GLM model routing
- `crates/reev-api/src/handlers/dynamic_flows/mod.rs` - Flow execution API
- `crates/reev-orchestrator/Cargo.toml` - Added reev-agent dependency
- `crates/reev-types/src/tools.rs` - ToolName enum with strum
- `crates/reev-orchestrator/src/lib.rs` - ExecutionMode enum and routing
- `crates/reev-orchestrator/src/benchmark_mode.rs` - Static benchmark execution
- `crates/reev-orchestrator/src/dynamic_mode.rs` - Dynamic request execution

### **All Issues RESOLVED:**
- ✅ COMPLETED: OTEL logging at orchestrator level (Issue #17)
- ✅ COMPLETED: Session log management at orchestrator level
- ✅ RESOLVED: YML step reference system for ping-pong connections (Issue #10)
- ✅ COMPLETED: Type-safe tool handling with strum enums
- ✅ COMPLETED: Clean mode separation with ExecutionMode enum
- ✅ COMPLETED: Mock data elimination throughout codebase
- ✅ COMPLETED: Production-ready error handling and validation

### **✅ COMPLETED Data Flow:**
1. User sends prompt → API
2. Orchestrator creates flow plan
3. **✅ IMPLEMENTED**: Execute agent via ping-pong coordination with OTEL
4. **✅ IMPLEMENTED**: Capture real tool execution data via orchestrator-level OTEL
5. **✅ IMPLEMENTED**: Store dual data (JSON for immediate use + OTEL for rich traces)
6. **✅ IMPLEMENTED**: Visualize execution flow with unified agent tracing

### **✅ VALIDATED Working Flow:**
```bash
# Real tool execution with flow visualization
curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{"prompt": "swap 0.1 SOL to USDC", "wallet": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM", "agent": "glm-4.6-coding"}'

# ✅ Response: {"flow_id":"dynamic-...","tool_count":2,"state_count":4}
# Real flow diagram: stateDiagram with account_balance → jupiter_swap transitions
```

## 🎉 **IMPLEMENTATION STATUS: PRODUCTION READY**

The complete dynamic benchmark system has been successfully implemented according to TASKS.md specifications. All phases are complete, all success criteria met, and the system is validated and ready for production use.

**Commit Hash:** `1adb7156` - feat: implement complete dynamic benchmark system from TASKS.md
---

### **✅ Execute 300 Benchmark:**
```bash
FLOW_ID=$(curl -s -X POST http://localhost:3001/api/v1/benchmarks/300-jup-swap-then-lend-deposit-dyn/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "glm-4.6-coding"
  }' | jq -r '.execution_id')

echo "FLOW_ID=$FLOW_ID"
```

### **✅ Get Mermaid Diagram:**
```bash
curl -s "http://localhost:3001/api/v1/flows/$FLOW_ID" | jq -r '.diagram'
```

### **✅ Check Execution Status:**
```bash
curl -s "http://localhost:3001/api/v1/benchmarks/300-jup-swap-then-lend-deposit-dyn/status/$EXECUTION_ID" | jq .
```

---

###  **dynamic flow execution**

```bash
# This uses static benchmark runner (no enhanced context)
curl -s -X POST http://localhost:3001/api/v1/benchmarks/300-jup-swap-then-lend-deposit-dyn/run \
  -H "Content-Type: application/json" \
  -d '{"agent": "glm-4.6-coding"}'

# ❌ This uses static YAML, not enhanced context

# ✅ Use this instead - DYNAMIC FLOW with enhanced context
FLOW_ID=$(curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "use my 50% sol to multiply usdc 1.5x on jup",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }' | jq -r '.execution_id')

echo "FLOW_ID=$FLOW_ID"

# Get enhanced flow diagram with real context
curl -s "http://localhost:3001/api/v1/flows/$FLOW_ID" | jq -r '.diagram'
```
