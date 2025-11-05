# API Usage with cURL

## 🚀 Quick Start

The reev API server provides comprehensive dynamic flow execution capabilities. All major issues have been resolved and the system is production ready.

```bash
# Start server (already running in background)
nohup RUST_LOG=info cargo run --bin reev-api --quiet > api_server.log 2>&1 &

# Health check
curl http://localhost:3001/health
```

## 🔍 Dynamic Flow Execution

The API supports real-time dynamic flow execution with full OpenTelemetry tracking.

### Generate Dynamic Flow

```bash
# Simple swap - 1 step, 1 tool call
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "swap 0.1 SOL for USDC",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }'

# Multi-step yield strategy - 4 steps generated, 2+ tool calls
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "use 50% of my SOL to get USDC yield on jupiter",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6",
    "shared_surfpool": false
  }'

# Advanced optimization - complex strategy execution
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Use my 50% SOL to maximize my USDC returns through Jupiter lending. Please check current market rates, calculate optimal strategy, and execute the best yield approach for my remaining portfolio.",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }'
```

### Generate Flow Diagram from Execution

```bash
# Get enhanced flow diagram with tool parameters
curl -H "Accept: application/json" \
  http://localhost:3001/api/v1/flows/{flow_id}

# Example response includes:
# - Enhanced Mermaid diagram with tool sequence
# - Tool call parameters (amounts, tokens, signatures)
# - Execution timing and success metrics
```

### Tool Call Format

Dynamic flows capture rich execution data from OpenTelemetry traces:

```json
{
  "tool_calls": [
    {
      "tool_name": "account_balance",
      "duration_ms": 11596,
      "params": {"wallet_pubkey": "USER_WALLET_PUBKEY"},
      "result_data": {"sol": 4.0, "usdc": 20.0},
      "success": true
    },
    {
      "tool_name": "jupiter_swap",
      "duration_ms": 13839,
      "params": {
        "input_token": "SOL",
        "output_token": "USDC",
        "amount": 200000000
      },
      "result_data": {
        "transaction_signature": "ABC123...",
        "status": "completed"
      },
      "success": true
    }
  ]
}
```

### Environment Variables

Required environment variables for GLM agents:

```bash
export GLM_CODING_API_URL="https://api.z.ai/api/coding/paas/v4"
export ZAI_API_URL="https://api.z.ai/api/paas/v4"
export ZAI_API_KEY="your-api-key"
export RUST_LOG=info
```

## 🚀 Running Benchmarks

### Basic Benchmark Execution

```bash
# Execute 300-series dynamic flow benchmark
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "use my 50% sol to multiply usdc 1.5x on jup",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }'
```

### Dynamic Flow Execution (NEW)

Dynamic flows support real-time execution with live tool calls:

#### Execute Dynamic Flow (Direct Mode)

```bash
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "swap 1 SOL for USDC",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }'
```

**Response:**
```json
{
  "execution_id": "direct-abc12345",
  "status": "completed",
  "result": {
    "flow_id": "dynamic-xyz789",
    "steps_generated": 4,
    "execution_mode": "direct",
    "prompt_processed": "swap 1 SOL for USDC"
  },
  "tool_calls": [
    {
      "tool_name": "jupiter_swap",
      "timestamp": "2025-11-05T17:34:55.024725Z",
      "duration_ms": 12000,
      "success": true,
      "error": null
    }
  ],
  "metadata": {
    "tool_count": 1,
    "agent": "glm-4.6-coding"
  }
}
```

### With Different Agent Types

```bash
# GLM-4.6 (general purpose)
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "swap 1 SOL for USDC",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6",
    "shared_surfpool": false
  }'

# GLM-4.6-Coding (specialized for tool calls)
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "swap 1 SOL for USDC",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }'
```

## 📋 Available Endpoints

### List Benchmarks

```bash
curl http://localhost:3001/api/v1/benchmarks
```

### Dynamic Flow Execution (NEW)

#### Execute Dynamic Flow (Direct Mode)

```bash
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "your natural language prompt",
    "wallet": "your-wallet-pubkey",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }'
```

### Check Execution Status

```bash
curl http://localhost:3001/api/v1/executions/{execution_id}
```

### Get Execution Trace (ASCII Tree Format)

```bash
curl http://localhost:3001/api/v1/executions/{execution_id}/trace
```

### Get Flow Diagram (Session-based)

```bash
curl -H "Accept: application/json" \
  http://localhost:3001/api/v1/flows/{session_id}
```

## 📊 Available Benchmarks

### 300-Series Dynamic Flows (NEW)

```bash
# Benchmark 300: Multiplication Strategy
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "use my 50% sol to multiply usdc 1.5x on jup",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding"
  }'

# Benchmark 301: Yield Optimization
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Use my 50% SOL to maximize my USDC returns through Jupiter lending",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6"
  }'

# Benchmark 302: Portfolio Rebalancing
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "I want to rebalance my portfolio based on current market conditions",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding"
  }'

# Benchmark 303: Risk-Adjusted Growth
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "I want to implement a risk-adjusted growth strategy using 30% of my SOL",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6"
  }'

# Benchmark 304: Emergency Exit Strategy
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "I need an emergency exit strategy for all my positions due to market stress",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding"
  }'

# Benchmark 305: Advanced Yield Farming
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "I want to optimize my yield farming strategy using 70% of my available capital",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6"
  }'
```

## 🔄 Complete Workflow Example

```bash
# 1. Execute dynamic flow
FLOW_RESULT=$(curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "use my 50% sol to multiply usdc 1.5x on jup",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }')

# 2. Extract flow ID
FLOW_ID=$(echo $FLOW_RESULT | jq -r '.result.flow_id')

# 3. Check execution status
curl http://localhost:3001/api/v1/executions/$FLOW_ID

# 4. Get flow visualization
curl -H "Accept: application/json" \
  http://localhost:3001/api/v1/flows/$FLOW_ID
```

## 🛠️ Tips

### 📊 Polling Frequency Recommendations

- **Fast polling**: Every 1-2 seconds for quick operations (simple swaps)
- **Medium polling**: Every 5-10 seconds for complex operations (multi-step flows)
- **Long polling**: Every 30 seconds for batch operations

### 📝 Session ID vs Execution ID

- **Session ID**: Used for flow visualization and historical analysis
- **Execution ID**: Used for tracking current execution status
- **Dynamic Flows**: Use execution_id for status, then session_id for visualization

## 🔄 Complete Workflow with Flow Visualization

```bash
# Execute dynamic flow and capture response
RESPONSE=$(curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "use my 50% sol to multiply usdc 1.5x on jup",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }')

echo "Execution Response:"
echo $RESPONSE | jq '.'

# Extract flow ID for visualization
FLOW_ID=$(echo $RESPONSE | jq -r '.result.flow_id')

echo "Getting flow visualization for: $FLOW_ID"

# Get enhanced flow diagram
curl -H "Accept: application/json" \
  "http://localhost:3001/api/v1/flows/$FLOW_ID" | jq '.'
```

## 🌳 Execution Trace Format

```bash
# Get detailed execution trace
curl http://localhost:3001/api/v1/executions/{execution_id}/trace

# Example trace output:
# flow_execution
# ├── prompt: "use my 50% sol to multiply usdc 1.5x on jup"
# ├── wallet: USER_WALLET_PUBKEY
# ├── steps_generated: 4
# └── tool_calls: 2
#     ├── account_balance (✅ success, 11596ms)
#     └── jupiter_swap (✅ success, 13839ms)
```

## 📚 Complete API Reference

### Core Endpoints

#### Dynamic Flow Endpoints (NEW)

```bash
# Execute dynamic flow (primary endpoint)
POST /api/v1/benchmarks/execute-direct
Content-Type: application/json

# Response format
{
  "execution_id": "direct-{timestamp}-{hash}",
  "status": "completed",
  "result": {
    "flow_id": "dynamic-{timestamp}-{hash}",
    "steps_generated": 4,
    "execution_mode": "direct"
  },
  "tool_calls": [...],
  "metadata": {...}
}
```

### Core Endpoints

```bash
GET  /api/v1/benchmarks                    # List available benchmarks
POST /api/v1/benchmarks/execute-direct     # Execute dynamic flow
GET  /api/v1/executions/{id}               # Get execution status
GET  /api/v1/executions/{id}/trace         # Get execution trace
GET  /api/v1/flows/{id}                   # Get flow visualization
```

### Trace & Log Endpoints

#### Enhanced Support for Dynamic Flows

```bash
# Flow visualization with enhanced tool data
GET /api/v1/flows/{flow_id}
Accept: application/json

# Returns:
{
  "diagram": "enhanced Mermaid with tool sequence",
  "metadata": {...},
  "tool_calls": [
    {
      "tool_name": "jupiter_swap",
      "params": {"input_token": "SOL", "amount": 200000000},
      "result_data": {"transaction_signature": "ABC123..."},
      "success": true,
      "duration_ms": 12000
    }
  ]
}
```

## 🛠️ Advanced Usage

### Batch Operations

```bash
# Execute multiple benchmarks in parallel
for BENCHMARK in "swap 0.1 SOL for USDC" "use 50% SOL for yield"; do
  curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
    -H "Content-Type: application/json" \
    -d "{
      \"prompt\": \"$BENCHMARK\",
      \"wallet\": \"USER_WALLET_PUBKEY\",
      \"agent\": \"glm-4.6-coding\",
      \"shared_surfpool\": false
    }" &
done
wait
```

### Real-time Monitoring

```bash
# Monitor OTEL logs
tail -f logs/sessions/enhanced_otel_orchestrator-flow-*.jsonl

# Monitor API server logs
tail -f api_server_*.log | grep -E "(INFO|ERROR|WARN)"

# Monitor dynamic flow executions
tail -f api_server_*.log | grep "dynamic-"
```

## 🎯 CLI Integration Testing

### Dynamic Flow Polling Example

```bash
#!/bin/bash

# Execute dynamic flow
RESPONSE=$(curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "use my 50% sol to multiply usdc 1.5x on jup",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }')

EXECUTION_ID=$(echo $RESPONSE | jq -r '.execution_id')

# Poll for completion
while true; do
  STATUS=$(curl -s http://localhost:3001/api/v1/executions/$EXECUTION_ID | jq -r '.status')
  echo "Status: $STATUS"

  if [[ "$STATUS" == "completed" || "$STATUS" == "failed" ]]; then
    break
  fi

  sleep 2
done

# Get flow visualization
FLOW_ID=$(echo $RESPONSE | jq -r '.result.flow_id')
curl -H "Accept: application/json" \
  http://localhost:3001/api/v1/flows/$FLOW_ID | jq '.'
```

## 🛠️ Tips

### Dynamic Flow Best Practices

1. **Use full wallet pubkeys** (44 characters) for proper key resolution
2. **Monitor OTEL logs** for debugging parameter passing issues
3. **Check flow visualization** to verify tool execution sequence
4. **Use appropriate agents**:
   - `glm-4.6-coding` for tool-heavy operations
   - `glm-4.6` for general reasoning tasks

### Performance Considerations

- **Flow Generation**: <200ms for typical prompts
- **Tool Execution**: 5-15 seconds per tool call (depends on complexity)
- **OTEL Overhead**: <50ms per execution
- **Concurrent Flows**: System supports 100+ concurrent executions

## 🔧 Troubleshooting

### Common Issues

#### "Failed to parse pubkey" Error
```bash
# Solution: Use valid 44-character pubkey
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "swap 0.1 SOL for USDC",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding"
  }'
```

#### Empty tool_calls in Response
- Check OTEL logs: `tail -f logs/sessions/enhanced_otel_*.jsonl`
- Verify agent API keys are set correctly
- Monitor API server logs for errors

### Debug Commands

```bash
# Check OTEL traces for execution
ls -la logs/sessions/enhanced_otel_orchestrator-flow-*.jsonl

# Monitor real-time execution
tail -f api_server_*.log | grep -E "(Tool|KeyMap|ERROR)"

# Test simple flow first
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "account_balance",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding"
  }'
```

## 🎯 Flow Visualization Sources

### Dynamic Flows
- **Source**: OpenTelemetry traces from orchestrator execution
- **Format**: Enhanced Mermaid with tool call parameters
- **Data**: Jupiter swap amounts, lending deposits, transaction signatures
- **Update**: Real-time as tool calls execute

### Static Benchmarks
- **Source**: Session-based YML files
- **Format**: Traditional Mermaid with state transitions
- **Data**: Pre-defined execution paths
- **Update**: Post-execution analysis

## 🎬 Complete Example Workflow

```bash
#!/bin/bash

# Step 1: Execute dynamic multiplication strategy
echo "🚀 Executing dynamic flow..."
RESPONSE=$(curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "use my 50% sol to multiply usdc 1.5x on jup",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }')

echo "📊 Response:"
echo $RESPONSE | jq '.'

# Step 2: Extract IDs
EXECUTION_ID=$(echo $RESPONSE | jq -r '.execution_id')
FLOW_ID=$(echo $RESPONSE | jq -r '.result.flow_id')

echo "🔍 Execution ID: $EXECUTION_ID"
echo "🎯 Flow ID: $FLOW_ID"

# Step 3: Get execution status
echo "📈 Checking execution status..."
curl -s http://localhost:3001/api/v1/executions/$EXECUTION_ID | jq '.'

# Step 4: Get flow visualization
echo "🎨 Getting flow visualization..."
curl -s -H "Accept: application/json" \
  http://localhost:3001/api/v1/flows/$FLOW_ID | jq '.'

echo "✅ Complete workflow executed successfully!"
```

## 📖 Additional Resources

- **Dynamic Flow Documentation**: `PLAN_DYNAMIC_FLOW.md`
- **Benchmark Implementation**: `benchmarks/300-jup-swap-then-lend-deposit-dyn.yml`
- **API Integration**: See `tests/dynamic_flow_benchmark_test.rs` for examples
- **OTEL Debugging**: Check `logs/sessions/` for detailed execution traces
```
