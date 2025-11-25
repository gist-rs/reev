#!/bin/bash

set -e

echo "🔍 Testing Flow Visualization Issue #38"
echo "======================================"

# Check if server is running
if ! curl -s "http://localhost:3001/api/v1/health" > /dev/null; then
    echo "❌ Server not running, starting it..."
    cargo run --bin reev-api --features production --quiet &
    SERVER_PID=$!
    sleep 5

    # Function to cleanup on exit
    cleanup() {
        echo "🛑 Cleaning up..."
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    }
    trap cleanup EXIT
fi

echo "✅ Server is running"

# Step 1: Get existing execution sessions
echo ""
echo "📊 Checking existing execution sessions..."
curl -s "http://localhost:3001/api/v1/debug/execution-sessions?limit=5" | jq -r '.[].session_id' | head -5

# Step 2: Check if we have the target session
TARGET_SESSION="7521e491-0cab-47b1-936a-b2d195ef4ddd"
echo ""
echo "🎯 Checking flow for session: $TARGET_SESSION"

# Check session log content first
echo "📋 Session log content (first 3 lines):"
curl -s "http://localhost:3001/api/v1/flows/$TARGET_SESSION?format=json" 2>/dev/null | jq -r '.diagram' | head -3 2>/dev/null || echo "Session not found or error"

# Step 3: Test flow generation
echo ""
echo "🎨 Testing flow diagram generation..."
FLOW_RESPONSE=$(curl -s "http://localhost:3001/api/v1/flows/$TARGET_SESSION?format=json" 2>/dev/null || echo '{"error": "Request failed"}')

if echo "$FLOW_RESPONSE" | jq -e '.error' > /dev/null 2>&1; then
    echo "❌ Flow generation failed: $(echo "$FLOW_RESPONSE" | jq -r '.error')"

    # Try to get any available flow
    echo ""
    echo "🔄 Trying to get any available flow..."
    AVAILABLE_SESSIONS=$(curl -s "http://localhost:3001/api/v1/debug/execution-sessions?limit=5" 2>/dev/null | jq -r '.[].session_id' 2>/dev/null || echo "")

    if [ -n "$AVAILABLE_SESSIONS" ]; then
        FIRST_SESSION=$(echo "$AVAILABLE_SESSIONS" | head -1)
        echo "📝 Testing with session: $FIRST_SESSION"
        curl -s "http://localhost:3001/api/v1/flows/$FIRST_SESSION?format=json" | jq '.metadata, .tool_calls | length' 2>/dev/null || echo "Failed to get flow"
    fi
else
    echo "✅ Flow generated successfully!"
    echo "$FLOW_RESPONSE" | jq '{session_id, metadata: .metadata, tool_count: .tool_calls | length}'

    # Check if we have the expected 4-step flow
    TOOL_COUNT=$(echo "$FLOW_RESPONSE" | jq '.tool_calls | length' 2>/dev/null || echo "0")
    echo ""
    echo "📊 Flow Analysis:"
    echo "- Tool calls captured: $TOOL_COUNT"
    echo "- Expected for 4-step flow: ≥4 tool calls"

    if [ "$TOOL_COUNT" -ge 4 ]; then
        echo "✅ SUCCESS: Multi-step flow visualization working!"
        echo "🎉 Issue #38 appears to be RESOLVED"
    else
        echo "❌ INCOMPLETE: Only $TOOL_COUNT tool calls captured"
        echo "🔧 Issue #38 still needs work - enhanced flow not captured"

        # Show tool call details
        echo ""
        echo "🔍 Tool call details:"
        echo "$FLOW_RESPONSE" | jq '.tool_calls[] | {tool_name, timestamp}' 2>/dev/null || echo "No tool calls found"
    fi

    # Show the diagram
    echo ""
    echo "🎨 Generated Diagram:"
    echo "===================="
    echo "$FLOW_RESPONSE" | jq -r '.diagram' 2>/dev/null || echo "No diagram content"
fi

echo ""
echo "📝 Summary:"
echo "==========="
echo "Issue #38 Status: Multi-step flow visualization validation"
echo "Expected: 4-step flow with AccountDiscovery → JupiterSwap → JupiterLend → PositionValidation"
echo "Actual: Check tool count and diagram above"
