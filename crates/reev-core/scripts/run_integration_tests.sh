#!/bin/bash

# Script to run the integration tests for reev-core

# Kill any existing surfpool processes
echo "Cleaning up any existing surfpool processes..."
pkill -f surfpool

# Start surfpool in background
echo "Starting surfpool..."
nohup surfpool start --rpc-url https://api.mainnet-beta.solana.com --port 8899 > surfpool.log 2>&1 &
SURFPOOL_PID=$!
echo "Started surfpool with PID: $SURFPOOL_PID"

# Wait for surfpool to be ready
echo "Waiting for surfpool to be ready..."
for i in {1..30}; do
    if curl -s http://localhost:8899 > /dev/null; then
        echo "✅ Surfpool is ready after $i attempts"
        break
    else
        echo "Attempt $i/30: Waiting for surfpool..."
        sleep 2
    fi
done

# Check if surfpool is ready
if ! curl -s http://localhost:8899 > /dev/null; then
    echo "❌ Failed to start surfpool"
    exit 1
fi

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
cargo test --test end_to_end_swap test_swap_1_sol_for_usdc --release -- --ignored

echo "Running sell all SOL test..."
cargo test --test end_to_end_swap test_sell_all_sol_for_usdc --release -- --ignored

echo "✅ All tests completed!"

# Clean up
echo "Cleaning up surfpool..."
kill $SURFPOOL_PID
echo "✅ Done!"
