#!/bin/bash

# Test script to validate enhanced multi-step flow visualization
# This script tests if Issue #38 has been resolved

set -e

echo "🚀 Testing Multi-Step Flow Visualization"
echo "===================================="

# Set up environment variables for testing
export RUST_LOG=info
export REEV_ENHANCED_OTEL=1

# Step 1: Clean any existing logs
echo "🧹 Cleaning existing logs..."
rm -rf logs/sessions/enhanced_otel_*.jsonl 2>/dev/null || true

# Step 2: Start the server in background
echo "🚀 Starting reev-api server..."
cargo run --bin reev-api --features production --quiet &
SERVER_PID=$!

# Wait for server to start
sleep 5

# Function to cleanup on exit
cleanup() {
    echo "🛑 Cleaning up..."
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
}
trap cleanup EXIT

# Step 3: Execute 300 benchmark with dynamic mode
echo "📊 Executing 300 benchmark (dynamic mode)..."
EXECUTION_RESPONSE=$(curl -s -X POST "http://localhost:3001/api/v1/benchmarks/300-jup-swap-then-lend-deposit-dyn/run" \
  -H "Content-Type: application/json" \
  -d '{"agent":"glm-4.6-coding","mode":"dynamic"}')

echo "Execution Response:"
echo "$EXECUTION_RESPONSE" | jq .

# Extract execution ID
EXECUTION_ID=$(echo "$EXECUTION_RESPONSE" | jq -r '.execution_id')

if [ "$EXECUTION_ID" = "null" ] || [ -z "$EXECUTION_ID" ]; then
    echo "❌ Failed to get execution_id"
    exit 1
fi

echo "✅ Execution ID: $EXECUTION_ID"

# Step 4: Wait for execution to complete
echo "⏳ Waiting for execution to complete..."
for i in {1..30}; do
    STATUS_RESPONSE=$(curl -s "http://localhost:3001/api/v1/benchmarks/300-jup-swap-then-lend-deposit-dyn/status/$EXECUTION_ID")
    STATUS=$(echo "$STATUS_RESPONSE" | jq -r '.status')

    echo "Attempt $i: Status = $STATUS"

    if [ "$STATUS" = "completed" ]; then
        echo "✅ Execution completed!"
        break
    elif [ "$STATUS" = "failed" ]; then
        echo "❌ Execution failed"
        echo "$STATUS_RESPONSE" | jq .
        exit 1
    fi

    if [ $i -eq 30 ]; then
        echo "❌ Execution timed out"
        exit 1
    fi

    sleep 2
done

# Step 5: Check flow visualization
echo "🎨 Checking flow visualization..."
FLOW_RESPONSE=$(curl -s "http://localhost:3001/api/v1/flows/$EXECUTION_ID?format=json")

echo "Flow Response:"
echo "$FLOW_RESPONSE" | jq .

# Extract key metrics
TOOL_CALLS_COUNT=$(echo "$FLOW_RESPONSE" | jq '.tool_calls | length')
DIAGRAM_CONTAINS_ACCOUNT_DISCOVERY=$(echo "$FLOW_RESPONSE" | jq -r '.diagram' | grep -c "AccountDiscovery" || echo "0")
DIAGRAM_CONTAINS_JUPITER_SWAP=$(echo "$FLOW_RESPONSE" | jq -r '.diagram' | grep -c "JupiterSwap" || echo "0")
DIAGRAM_CONTAINS_JUPITER_LEND=$(echo "$FLOW_RESPONSE" | jq -r '.diagram' | grep -c "JupiterLend" || echo "0")
DIAGRAM_CONTAINS_POSITION_VALIDATION=$(echo "$FLOW_RESPONSE" | jq -r '.diagram' | grep -c "PositionValidation" || echo "0")

echo ""
echo "📊 Flow Visualization Analysis:"
echo "================================"
echo "Tool calls captured: $TOOL_CALLS_COUNT"
echo "AccountDiscovery steps: $DIAGRAM_CONTAINS_ACCOUNT_DISCOVERY"
echo "JupiterSwap steps: $DIAGRAM_CONTAINS_JUPITER_SWAP"
echo "JupiterLend steps: $DIAGRAM_CONTAINS_JUPITER_LEND"
echo "PositionValidation steps: $DIAGRAM_CONTAINS_POSITION_VALIDATION"

# Step 6: Validate against expected results
echo ""
echo "🔍 Validation Results:"
echo "===================="

VALIDATION_PASSED=true

if [ "$TOOL_CALLS_COUNT" -ge 4 ]; then
    echo "✅ Tool calls: Expected ≥4, got $TOOL_CALLS_COUNT"
else
    echo "❌ Tool calls: Expected ≥4, got $TOOL_CALLS_COUNT"
    VALIDATION_PASSED=false
fi

if [ "$DIAGRAM_CONTAINS_ACCOUNT_DISCOVERY" -gt 0 ]; then
    echo "✅ AccountDiscovery: Found in diagram"
else
    echo "❌ AccountDiscovery: Missing from diagram"
    VALIDATION_PASSED=false
fi

if [ "$DIAGRAM_CONTAINS_JUPITER_SWAP" -gt 0 ]; then
    echo "✅ JupiterSwap: Found in diagram"
else
    echo "❌ JupiterSwap: Missing from diagram"
    VALIDATION_PASSED=false
fi

if [ "$DIAGRAM_CONTAINS_JUPITER_LEND" -gt 0 ]; then
    echo "✅ JupiterLend: Found in diagram"
else
    echo "❌ JupiterLend: Missing from diagram"
    VALIDATION_PASSED=false
fi

if [ "$DIAGRAM_CONTAINS_POSITION_VALIDATION" -gt 0 ]; then
    echo "✅ PositionValidation: Found in diagram"
else
    echo "❌ PositionValidation: Missing from diagram"
    VALIDATION_PASSED=false
fi

# Step 7: Show the actual diagram
echo ""
echo "🎨 Generated Mermaid Diagram:"
echo "============================="
echo "$FLOW_RESPONSE" | jq -r '.diagram'

# Step 8: Final result
echo ""
if [ "$VALIDATION_PASSED" = true ]; then
    echo "🎉 SUCCESS: Multi-step flow visualization is working correctly!"
    echo "✅ Issue #38 RESOLVED: All 4 steps captured and visualized"
    exit 0
else
    echo "❌ FAILURE: Multi-step flow visualization needs improvement"
    echo "🔧 Issue #38 ACTIVE: Some steps missing from visualization"
    exit 1
fi
```

Now let me make this script executable:
<tool_call>terminal
<arg_key>command</arg_key>
<arg_value>chmod +x test_scripts/test_flow_visualization.sh</arg_value>
<arg_key>cd</arg_key>
<arg_value>reev</arg_value>
</tool_call>
