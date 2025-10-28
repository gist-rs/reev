# API Usage with cURL

This document provides examples of how to interact with the Reev API using cURL commands.

## 🚀 Quick Start

```bash
# 1. Check API health
curl http://localhost:3001/api/v1/health

# 2. List available benchmarks  
curl http://localhost:3001/api/v1/benchmarks | jq .

# 3. Run a benchmark
EXECUTION_RESPONSE=$(curl -s -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{"agent": "deterministic"}' | jq -r '.execution_id')

# 4. Get execution trace (ASCII tree format)
curl http://localhost:3001/api/v1/execution-logs/001-sol-transfer | jq -r '.trace'
```

## 🔍 Tool Call Logging with OpenTelemetry

Flow visualization is now handled by the reev-api web interface using database session data.

### Generate Tool Call Logs

To generate flow diagrams:

```bash
# Use the reev-api web interface
curl http://localhost:3001/api/v1/flows/{session-id}

# Or run benchmarks and access via web UI
cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent deterministic
```

### Generate Flow Diagram from Tool Logs

```bash
# Flow visualization is now handled via the API
# See FLOW.md for web interface usage
```

### Tool Log Format

Flow diagram data is stored in database sessions and accessible via API:
```
2024-01-15T10:30:00.123Z INFO [OpenAIAgent] Starting agent execution with OpenTelemetry tracing
2024-01-15T10:30:01.456Z INFO [AccountBalanceTool] Starting tool execution in accountbalance_tool_call with args: {"pubkey": "USER_1"}
2024-01-15T10:30:02.789Z INFO [AccountBalanceTool] Tool completed successfully in accountbalance_tool_call with result: {"balance": "100 USDC"}
2024-01-15T10:30:03.012Z INFO [JupiterSwapTool] Starting tool execution in jupiterswap_tool_call with args: {"amount": "0.1", "input_mint": "So11111111111111111111111111111111111111112", "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"}
2024-01-15T10:30:04.345Z INFO [JupiterSwapTool] Tool completed successfully in jupiterswap_tool_call with result: {"output_amount": "5.23"}
2024-01-15T10:30:05.678Z INFO [OpenAIAgent] Agent execution completed
```

### Environment Variables

```bash
# Flow visualization is enabled by default
# Session data is stored in database for web interface access
```

## 🚀 Running Benchmarks

### Basic Benchmark Execution

```bash
curl -X POST http://localhost:3001/api/v1/benchmarks/{benchmark-id}/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "deterministic"
  }'
```

**Example:**
```bash
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "deterministic"
  }'
```

### With Different Agent Types

```bash
# Local agent
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "local"
  }'

# Gemini model
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "glm-4.6"
  }'

# GLM model
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "glm-4-6"
  }'
```

### With Agent Configuration

```bash
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "local",
    "config": {
      "model": "qwen3-coder-30b-a3b-instruct-mlx",
      "api_base": "http://localhost:8000",
      "api_key": "your-api-key-here",
      "max_tokens": 2000,
      "temperature": 0.1
    }
  }'
```

## 📋 Available Endpoints

### List Benchmarks
```bash
curl http://localhost:3001/api/v1/benchmarks
```

### Check Execution Status
```bash
curl http://localhost:3001/api/v1/benchmarks/{benchmark-id}/status/{execution-id}
```

### Stop Execution
```bash
curl -X POST http://localhost:3001/api/v1/benchmarks/{benchmark-id}/stop/{execution-id}
```

### Get Execution Trace (ASCII Tree Format)
```bash
# Get execution trace with formatted ASCII tree
curl http://localhost:3001/api/v1/execution-logs/{benchmark-id}

# Get raw JSON response
curl http://localhost:3001/api/v1/execution-logs/{benchmark-id} | jq .

# Get just the formatted trace
curl -s http://localhost:3001/api/v1/execution-logs/{benchmark-id} | jq -r '.trace'
```

### Get Flow Diagram (Session-based)
```bash
# Get flow for specific session
curl http://localhost:3001/api/v1/flows/{session-id}

# Get HTML version
curl http://localhost:3001/api/v1/flows/{session-id}?format=html
```

### Get Flow Logs
```bash
# Get flow logs for benchmark
curl http://localhost:3001/api/v1/flow-logs/{benchmark-id}

# Get transaction logs
curl http://localhost:3001/api/v1/transaction-logs/{benchmark-id}

# Get demo transaction logs
curl http://localhost:3001/api/v1/transaction-logs/demo
```

### Agent Configuration
```bash
# Save agent config
curl -X POST http://localhost:3001/api/v1/agents/config \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "local",
    "model": "qwen3-coder-30b-a3b-instruct-mlx",
    "api_base": "http://localhost:8000",
    "api_key": "your-api-key"
  }'

# Get agent config
curl http://localhost:3001/api/v1/agents/config/{agent-type}

# Test agent connection
curl -X POST http://localhost:3001/api/v1/agents/test \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "local",
    "model": "qwen3-coder-30b-a3b-instruct-mlx",
    "api_base": "http://localhost:8000",
    "api_key": "your-api-key"
  }'
```

## 📊 Available Benchmarks

- `001-sol-transfer` - SOL transfer
- `002-spl-transfer` - SPL token transfer
- `003-spl-transfer-fail` - Failed SPL transfer
- `004-partial-score-spl-transfer` - Partial score SPL transfer
- `100-jup-swap-sol-usdc` - Jupiter SOL/USDC swap
- `110-jup-lend-deposit-sol` - Jupiter lending SOL deposit
- `111-jup-lend-deposit-usdc` - Jupiter lending USDC deposit
- `112-jup-lend-withdraw-sol` - Jupiter lending SOL withdraw
- `113-jup-lend-withdraw-usdc` - Jupiter lending USDC withdraw
- `114-jup-positions-and-earnings` - Jupiter positions
- `115-jup-lend-mint-usdc` - Jupiter lending USDC mint
- `116-jup-lend-redeem-usdc` - Jupiter lending USDC redeem
- `200-jup-swap-then-lend-deposit` - Swap then lend deposit

## 🔄 Complete Workflow Example

```bash
# 1. Check API health
curl http://localhost:3001/api/v1/health

# 2. List available benchmarks
curl http://localhost:3001/api/v1/benchmarks

# 3. Run a benchmark
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "deterministic"
  }'

# Response: {"execution_id":"uuid-here","status":"started"}

# 4. Check execution status
curl http://localhost:3001/api/v1/benchmarks/001-sol-transfer/status/uuid-here

# 5. Once complete, get the execution trace (ASCII tree format)
curl http://localhost:3001/api/v1/execution-logs/001-sol-transfer

# 6. Get the formatted trace only
curl -s http://localhost:3001/api/v1/execution-logs/001-sol-transfer | jq -r '.trace'

# 7. Get flow diagram using session_id
curl http://localhost:3001/api/v1/flows/{session-id}

# 8. Get HTML version for visualization
curl http://localhost:3001/api/v1/flows/{session-id}?format=html > flow.html
```

## 🛠️ Tips

1. **Pretty Print JSON**: Add `| jq` to any curl command for formatted output
   ```bash
   curl http://localhost:3001/api/v1/benchmarks | jq
   ```

2. **Save Responses**: Redirect output to files
   ```bash
   curl http://localhost:3001/api/v1/flows/{session-id}?format=html > flow_diagram.html
   ```

3. **Include Headers**: Add `-v` for verbose output to debug issues
   ```bash
   curl -v -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
     -H "Content-Type: application/json" \
     -d '{"agent": "deterministic"}'
   ```

4. **Check Response Times**: Use `-w` to measure API performance
   ```bash
   curl -w "@curl-format.txt" -o /dev/null -s http://localhost:3001/api/v1/health
   ```

## 📝 Session ID vs Execution ID

- **Execution ID**: Temporary ID for tracking running benchmark execution
- **Session ID**: Permanent ID for completed benchmark sessions (used for flow diagrams)
- After execution completes, use the session_id from the results to access flow diagrams

## 🔄 Complete Workflow with Flow Visualization

```bash
# 1. Check API health
curl http://localhost:3001/api/v1/health

# 2. List available benchmarks
curl http://localhost:3001/api/v1/benchmarks

# 3. Run a benchmark (generates session with tool calls)
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "local"
  }'

# Response: {"execution_id":"uuid-here","status":"started"}

# 4. Check execution status
curl http://localhost:3001/api/v1/benchmarks/001-sol-transfer/status/uuid-here

# 5. Once complete, get the execution trace (ASCII tree format)
curl http://localhost:3001/api/v1/execution-logs/001-sol-transfer | jq -r '.trace'

# 6. Get flow diagram using session_id
curl http://localhost:3001/api/v1/flows/{session-id}

# 7. Flow visualization is now handled via the web API
# Use the reev-api endpoints for flow diagrams
```

## 🌳 Execution Trace Format

The execution trace endpoint returns formatted ASCII tree structure:

**Example Output:**
```
✅ 001-sol-transfer (Score: 100.0%): succeeded
 └─ Step 1
    ├─ ACTION:
     Program ID: 11111111111111111111111111111111
     Accounts:
     [ 0] 🖋️ ➕ D8desbmY7LG2R9Abe7m1LThxS3Vsq6fvmK7FXY5xvPGR
     [ 1] 🖍️ ➕ 7c877QpPvxcU9vNcEWjX4EfBekGMB55zsSuwaTKbqG9S
     Data (Base58): 3Bxs411Dtc7pkFQj
    └─ OBSERVATION: Success
```

**Legend:**
- 🖋️ = Signer account
- 🖍️ = Non-signer account  
- ➕ = Writable account
- ➖ = Read-only account

## 📚 Complete API Reference

### Core Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/health` | GET | API health check |
| `/api/v1/benchmarks` | GET | List all available benchmarks |
| `/api/v1/benchmarks/{id}/run` | POST | Execute a benchmark |
| `/api/v1/benchmarks/{id}/status/{execution_id}` | GET | Check execution status |
| `/api/v1/benchmarks/{id}/status` | GET | Get latest execution status |
| `/api/v1/benchmarks/{id}/stop/{execution_id}` | POST | Stop running execution |

### Trace & Log Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/execution-logs/{benchmark_id}` | GET | **Get execution trace (ASCII tree)** |
| `/api/v1/flow-logs/{benchmark_id}` | GET | Get flow logs for benchmark |
| `/api/v1/transaction-logs/{benchmark_id}` | GET | Get transaction logs |
| `/api/v1/flows/{session_id}` | GET | Get flow diagram for session |
| `/api/v1/transaction-logs/demo` | GET | Get demo transaction logs |

### Agent Management

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/agents` | GET | List available agents |
| `/api/v1/agents/config` | POST | Save agent configuration |
| `/api/v1/agents/config/{agent_type}` | GET | Get agent configuration |
| `/api/v1/agents/test` | POST | Test agent connection |
| `/api/v1/agent-performance` | GET | Get agent performance metrics |

### Admin & Debug

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/upsert-yml` | POST | Upsert YAML benchmark |
| `/api/v1/sync` | POST | Sync benchmarks |
| `/api/v1/debug/benchmarks` | GET | Debug benchmark data |
| `/api/v1/debug/agent-performance-raw` | GET | Raw performance data |
| `/api/v1/debug/execution-sessions` | GET | Debug execution sessions |
| `/api/v1/debug/insert-test-data` | GET | Insert test data |

## 🛠️ Advanced Usage

### Batch Operations

```bash
# Run multiple benchmarks
BENCHMARKS=("001-sol-transfer" "002-spl-transfer" "100-jup-swap-sol-usdc")
for bench in "${BENCHMARKS[@]}"; do
  echo "Running $bench..."
  curl -s -X POST http://localhost:3001/api/v1/benchmarks/$bench/run \
    -H "Content-Type: application/json" \
    -d '{"agent": "deterministic"}' | jq '.execution_id'
done
```

### Real-time Monitoring

```bash
# Monitor execution status
watch -n 2 'curl -s http://localhost:3001/api/v1/benchmarks/001-sol-transfer/status/latest | jq "{status, progress}"'
```

### Export Results

```bash
# Export all execution traces
for bench in $(curl -s http://localhost:3001/api/v1/benchmarks | jq -r '.[].id'); do
  echo "=== $bench ==="
  curl -s http://localhost:3001/api/v1/execution-logs/$bench | jq -r '.trace' 2>/dev/null || echo "No trace available"
  echo ""
done > all_traces.txt
```

## 🔧 Troubleshooting

### Common Issues

1. **404 Not Found on execution trace**
   ```bash
   # Wrong endpoint - this won't work
   curl http://localhost:3001/api/v1/benchmarks/001-sol-transfer/trace
   
   # Correct endpoint
   curl http://localhost:3001/api/v1/execution-logs/001-sol-transfer
   ```

2. **Empty trace response**
   ```bash
   # Check if execution completed
   curl http://localhost:3001/api/v1/benchmarks/{id}/status/{execution_id} | jq '.status'
   
   # Run new execution if needed
   curl -X POST http://localhost:3001/api/v1/benchmarks/{id}/run \
     -H "Content-Type: application/json" \
     -d '{"agent": "deterministic"}'
   ```

3. **Connection refused**
   ```bash
   # Check if API server is running
   curl http://localhost:3001/api/v1/health
   
   # Start API server if needed
   cargo run -p reev-api
   ```

### Debug Commands

```bash
# Check API connectivity
curl -v http://localhost:3001/api/v1/health

# Check available benchmarks with details
curl http://localhost:3001/api/v1/benchmarks | jq '.[] | {id, name, last_run}'

# Check running executions
curl http://localhost:3001/api/v1/debug/execution-sessions

# Get raw execution data for debugging
curl http://localhost:3001/api/v1/debug/agent-performance-raw
```

## 🎯 Flow Visualization Sources

There are two ways to generate flow diagrams:

1. **Session-based API**: `/flows/{session-id}` - Uses session logs from database
2. **Web API**: Use reev-api endpoints for flow visualization

The web API provides real-time flow diagrams showing agent decision flows and tool execution patterns.

## 🎬 Complete Example Workflow

```bash
#!/bin/bash
# Complete benchmark execution and trace analysis workflow

set -e

API_BASE="http://localhost:3001/api/v1"
BENCHMARK="001-sol-transfer"
AGENT="deterministic"

echo "🚀 Starting Complete Benchmark Workflow"
echo "====================================="

# 1. Health check
echo "1️⃣ Checking API health..."
curl -s "$API_BASE/health" | jq '.status' || exit 1
echo "✅ API is healthy"

# 2. List benchmarks
echo ""
echo "2️⃣ Listing available benchmarks..."
curl -s "$API_BASE/benchmarks" | jq -r '.[] | "\(.id): \(.name)"' | head -5

# 3. Run benchmark
echo ""
echo "3️⃣ Running benchmark: $BENCHMARK..."
EXECUTION_RESPONSE=$(curl -s -X POST "$API_BASE/benchmarks/$BENCHMARK/run" \
  -H "Content-Type: application/json" \
  -d "{\"agent\": \"$AGENT\"}")

EXECUTION_ID=$(echo "$EXECUTION_RESPONSE" | jq -r '.execution_id')
STATUS=$(echo "$EXECUTION_RESPONSE" | jq -r '.status')

echo "📋 Execution ID: $EXECUTION_ID"
echo "📊 Status: $STATUS"

# 4. Monitor execution
echo ""
echo "4️⃣ Monitoring execution progress..."
while true; do
    STATUS_CHECK=$(curl -s "$API_BASE/benchmarks/$BENCHMARK/status/$EXECUTION_ID")
    CURRENT_STATUS=$(echo "$STATUS_CHECK" | jq -r '.status')
    PROGRESS=$(echo "$STATUS_CHECK" | jq -r '.progress')
    
    echo "🔄 Status: $CURRENT_STATUS ($PROGRESS%)"
    
    if [[ "$CURRENT_STATUS" == "Completed" || "$CURRENT_STATUS" == "Failed" ]]; then
        echo "✅ Execution finished with status: $CURRENT_STATUS"
        break
    fi
    
    sleep 2
done

# 5. Get execution trace (ASCII tree)
echo ""
echo "5️⃣ Getting execution trace..."
TRACE_RESPONSE=$(curl -s "$API_BASE/execution-logs/$BENCHMARK")
FORMATTED_TRACE=$(echo "$TRACE_RESPONSE" | jq -r '.trace')

echo "🌳 Execution Trace:"
echo "$FORMATTED_TRACE"

# 6. Get raw JSON response
echo ""
echo "6️⃣ Raw response data..."
echo "$TRACE_RESPONSE" | jq '.'

# 7. Check other logs
echo ""
echo "7️⃣ Checking flow logs..."
curl -s "$API_BASE/flow-logs/$BENCHMARK" | jq '.trace' 2>/dev/null || echo "No flow logs available"

echo ""
echo "8️⃣ Checking transaction logs..."
curl -s "$API_BASE/transaction-logs/$BENCHMARK" | jq '.trace' 2>/dev/null || echo "No transaction logs available"

echo ""
echo "🎉 Workflow completed successfully!"
echo "=================================="
```

## 📖 Additional Resources

- **FLOW.md**: Detailed flow visualization documentation
- **ARCHITECTURE.md**: System architecture overview  
- **ISSUES.md**: Current issues and tracking
- **REFLECT.md**: Implementation reflections
- **TASKS.md**: Development tasks and progress

For more advanced usage and troubleshooting, see the debug endpoints section above.
