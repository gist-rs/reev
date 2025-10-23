#!/bin/bash

# GLM Agent Tool Calling Demo Runner
# This script runs the GLM tool calling demonstration

echo "🚀 GLM Agent Tool Calling Demo Runner"
echo "====================================="

# Check if GLM_API_KEY is set
if [ -z "$GLM_API_KEY" ]; then
    echo "❌ GLM_API_KEY environment variable not set"
    echo ""
    echo "Please set your GLM API key:"
    echo "export GLM_API_KEY=your_api_key_here"
    echo ""
    echo "Then run this script again."
    exit 1
fi

echo "✅ GLM_API_KEY is set"
echo "🔑 Using API key: ${GLM_API_KEY:0:8}..."
echo ""

# Set GLM API URL if not already set
export GLM_API_URL="${GLM_API_URL:-https://api.z.ai/api/coding/paas/v4}"
echo "🌐 Using GLM API URL: $GLM_API_URL"
echo ""

# Run the demo
echo "🎬 Starting GLM tool calling demo..."
echo ""

# Change to the project root directory
cd "$(dirname "$0")/../.."

# Run the example
cargo run --example glm_tool_call_demo --features native

echo ""
echo "✅ Demo completed!"
