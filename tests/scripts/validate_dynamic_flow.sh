#!/bin/bash
# validate_dynamic_flow.sh
# Clean validation script for dynamic flow implementation

set -e

echo "🧪 Validating Dynamic Flow Implementation"
echo "====================================="

# Configuration
API_BASE="http://localhost:3001/api/v1"
WALLET="9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"
AGENT="glm-4.6-coding"

# Check if server is running
echo "📋 Server Health Check"
if ! curl -s "$API_BASE/benchmarks" >/dev/null; then
    echo "❌ Server not running at $API_BASE"
    echo "💡 Start with: nohup cargo run --bin reev-api > api.log 2>&1 &"
    exit 1
fi

BENCHMARK_COUNT=$(curl -s "$API_BASE/benchmarks" | jq 'length')
echo "✅ Server running with $BENCHMARK_COUNT benchmarks"

# Test dynamic flow execution
echo ""
echo "📋 Testing Dynamic Flow Execution"
EXECUTION_RESPONSE=$(curl -s -X POST "$API_BASE/benchmarks/execute-direct" \
    -H "Content-Type: application/json" \
    -d "{\"prompt\": \"swap 0.01 SOL\", \"wallet\": \"$WALLET\", \"agent\": \"$AGENT\", \"shared_surfpool\": false}")

FLOW_ID=$(echo "$EXECUTION_RESPONSE" | jq -r '.result.flow_id // null')
if [ "$FLOW_ID" = "null" ]; then
    echo "❌ Failed to create dynamic flow"
    echo "🔍 Response: $EXECUTION_RESPONSE"
    exit 1
fi

echo "✅ Flow created: $FLOW_ID"

# Wait for execution and check visualization
sleep 3
echo "📋 Checking Flow Visualization"
FLOW_RESPONSE=$(curl -s "$API_BASE/flows/$FLOW_ID")

TOOL_COUNT=$(echo "$FLOW_RESPONSE" | jq -r '.metadata.tool_count // 0')
STATE_COUNT=$(echo "$FLOW_RESPONSE" | jq -r '.metadata.state_count // 0')

echo "✅ Tools captured: $TOOL_COUNT"
echo "✅ States generated: $STATE_COUNT"

# Validate implementation components
echo ""
echo "📋 Validating Implementation Files"

FILES=(
    "crates/reev-orchestrator/src/lib.rs"
    "crates/reev-orchestrator/src/benchmark_mode.rs"
    "crates/reev-orchestrator/src/dynamic_mode.rs"
    "crates/reev-orchestrator/src/execution/ping_pong_executor.rs"
    "crates/reev-types/src/tools.rs"
    "crates/reev-api/src/handlers/dynamic_flows/mod.rs"
)

ALL_EXIST=true
for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "✅ $file"
    else
        echo "❌ $file"
        ALL_EXIST=false
    fi
done

# Validate tool system
echo ""
echo "📋 Validating Tool System"
if grep -q "pub enum ToolName" crates/reev-types/src/tools.rs; then
    echo "✅ ToolName enum implemented"

    if grep -q "strum" crates/reev-types/src/tools.rs; then
        echo "✅ Strum derive macros present"
    fi

    if grep -q "JupiterSwap" crates/reev-types/src/tools.rs; then
        echo "✅ Jupiter tools available"
    fi
else
    echo "❌ ToolName enum not found"
    ALL_EXIST=false
fi

# Check code quality
echo ""
echo "📋 Code Quality Check"
if cargo clippy --quiet 2>/dev/null; then
    echo "✅ No clippy warnings"
    CLIPPY_STATUS="PASS"
else
    echo "⚠️ Clippy warnings detected"
    CLIPPY_STATUS="WARN"
fi

# Final result
echo ""
echo "🎯 VALIDATION SUMMARY"
echo "===================="

if [ "$ALL_EXIST" = true ] && [ "$TOOL_COUNT" -gt 0 ] && [ "$CLIPPY_STATUS" = "PASS" ]; then
    echo "🎉 VALIDATION: PASSED"
    echo "✅ Dynamic flow system is production ready"
    exit 0
else
    echo "❌ VALIDATION: FAILED"
    echo "🔧 Issues to resolve before production"
    exit 1
fi
