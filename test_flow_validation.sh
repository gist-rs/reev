#!/bin/bash
# test_flow_validation.sh

echo "🧪 Testing API Flow Visualization..."

# Test 1: Basic functionality
echo "📋 Test 1: Basic flow execution"
RESPONSE=$(curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{"prompt": "swap 0.01 SOL", "wallet": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM", "agent": "glm-4.6-coding", "shared_surfpool": false}')

FLOW_ID=$(echo $RESPONSE | jq -r '.result.flow_id')
TOOL_COUNT=$(echo $RESPONSE | jq '.tool_calls | length')

echo "✅ Flow ID: $FLOW_ID"
echo "✅ Tool Count: $TOOL_COUNT"

# Test 2: Flow visualization
echo "📋 Test 2: Flow visualization"
sleep 3  # Wait for execution to complete
FLOW_RESPONSE=$(curl -s "http://localhost:3001/api/v1/flows/$FLOW_ID")
VISUAL_TOOL_COUNT=$(echo $FLOW_RESPONSE | jq '.metadata.tool_count // 0')
DIAGRAM_STATES=$(echo $FLOW_RESPONSE | jq '.metadata.state_count // 0')

echo "✅ Visualization Tool Count: $VISUAL_TOOL_COUNT"
echo "✅ Diagram States: $DIAGRAM_STATES"

# Test 3: Information quality
echo "📋 Test 3: Information quality check"
TOOL_DETAILS=$(echo $FLOW_RESPONSE | jq '.tool_calls[0] // empty')

# Check if tool_calls exist
if [ "$TOOL_DETAILS" = "null" ] || [ "$TOOL_DETAILS" = "" ] || [ "$TOOL_DETAILS" = "empty" ]; then
    echo "❌ No tool calls found in visualization - ISSUE CONFIRMED"
    echo "🔍 Debug: Full response: $FLOW_RESPONSE"
    exit 1
fi

HAS_AMOUNT=$(echo $TOOL_DETAILS | jq 'has("input_amount") or has("params")')
HAS_SIGNATURE=$(echo $TOOL_DETAILS | jq 'has("tx_signature") or has("result")')
HAS_REAL_DATA=$(echo $TOOL_DETAILS | jq 'has("tool_name") and has("duration_ms")')

echo "🔍 Tool Details Analysis:"
echo "   Has Amount/Data: $HAS_AMOUNT"
echo "   Has Signature/Result: $HAS_SIGNATURE"
echo "   Has Basic Fields: $HAS_REAL_DATA"

if [ "$HAS_AMOUNT" = "true" ] && [ "$HAS_SIGNATURE" = "true" ]; then
    echo "✅ Tool calls contain real execution data"
    RESULT="PASS"
elif [ "$HAS_REAL_DATA" = "true" ]; then
    echo "⚠️  Tool calls contain basic mock data - PARTIALLY FIXED"
    RESULT="PARTIAL"
else
    echo "❌ Tool calls are missing or completely synthetic - MAJOR ISSUE"
    RESULT="FAIL"
fi

# Test 4: Diagram meaningfulness
echo "📋 Test 4: Diagram meaningfulness"
DIAGRAM=$(echo $FLOW_RESPONSE | jq -r '.diagram')
HAS_NULL_TRANSITIONS=$(echo "$DIAGRAM" | grep -c "Null" 2>/dev/null || echo "0")

if [ "${HAS_NULL_TRANSITIONS:-0}" -gt 0 ]; then
    echo "❌ Diagram contains $HAS_NULL_TRANSITIONS useless ': Null' transitions"
    if [ "$RESULT" = "PASS" ]; then
        RESULT="PARTIAL"
    fi
else
    echo "✅ Diagram has meaningful transitions"
fi

# Test 5: Show actual user information
echo "📋 Test 5: User information availability"
echo "🔍 Current Information Available:"
echo "   Tool Names: $(echo $FLOW_RESPONSE | jq -r '.tool_calls[].tool_name' | tr '\n' ', ' | sed 's/,$//')"
echo "   Tool Count: $VISUAL_TOOL_COUNT"
echo "   Session ID: $(echo $FLOW_RESPONSE | jq -r '.metadata.session_id')"
echo "   Execution Time: $(echo $FLOW_RESPONSE | jq -r '.metadata.execution_time_ms')ms"

echo ""
echo "🎯 MISSING INFORMATION (What users need):"
echo "   ❌ Transaction amounts (SOL, USDC)"
echo "   ❌ Wallet addresses (from, to)"
echo "   ❌ Transaction signatures"
echo "   ❌ Slippage percentages"
echo "   ❌ Balance changes"
echo "   ❌ Error messages (if failed)"
echo "   ❌ Recovery paths (if enabled)"

echo ""
echo "🎉 Test completed with result: $RESULT"

if [ "$RESULT" = "FAIL" ]; then
    echo ""
    echo "🚨 MAJOR ISSUE: Flow visualization provides almost no user value"
    echo "📝 See Issue #13 in ISSUES.md for detailed problem analysis"
    exit 1
elif [ "$RESULT" = "PARTIAL" ]; then
    echo ""
    echo "⚠️  PARTIAL FIX: Some improvements made but key information still missing"
    echo "📝 See Issue #12 and #13 in ISSUES.md for current status"
    exit 2
else
    echo ""
    echo "✅ SUCCESS: Flow visualization provides useful information"
    exit 0
fi
