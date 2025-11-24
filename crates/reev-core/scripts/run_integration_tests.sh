#!/bin/bash

# Script to run the integration tests for reev-core

# Check if surfpool is running
echo "Checking if surfpool is running..."
if ! curl -s http://localhost:8899 > /dev/null; then
    echo "❌ Surfpool is not running at http://localhost:8899"
    echo "Please start surfpool with: surfpool --fork-url https://api.mainnet-beta.solana.com --port 8899"
    exit 1
fi

echo "✅ Surfpool is running"

# Check if ZAI_API_KEY is set
if [ -z "$ZAI_API_KEY" ]; then
    echo "❌ ZAI_API_KEY is not set"
    echo "Please set ZAI_API_KEY in your environment or .env file"
    exit 1
fi

echo "✅ ZAI_API_KEY is configured"

# Run the integration tests
echo "Running integration tests..."

# Run the specific test without the ignore flag
cargo test -p reev-core --release -- --ignored test_swap_1_sol_for_usdc

echo "Running sell all SOL test..."
cargo test -p reev-core --release -- --ignored test_sell_all_sol_for_usdc

echo "✅ All tests completed!"
