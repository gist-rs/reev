#!/bin/bash

# Script to run the end-to-end sell all test with proper logging filters
# This follows the recommended logging approach from the project rules

echo "🧪 Running Sell All Test with Filtered Logging"
echo "=============================================="
echo ""
echo "This test follows the 6-step process:"
echo "1. Prompt: 'sell all SOL for USDC'"
echo "2. YML prompt with wallet info from SURFPOOL sent to GLM-coding"
echo "3. Swap tool calling from LLM"
echo "4. Generated transaction"
echo "5. Transaction signed with default keypair at ~/.config/solana/id.json"
echo "6. Transaction completion result from SURFPOOL"
echo ""

# Run the test with RUST_LOG set to filter out noise
# This shows only the relevant logs for the swap flow
RUST_LOG=reev_core::planner=info,reev_core::executor=info,jup_sdk=info,warn cargo test -p reev-core --test end_to_end_swap test_sell_all_sol_for_usdc -- --nocapture --ignored

# Check exit code
if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Test completed successfully!"
else
    echo ""
    echo "❌ Test failed with exit code $?"
    echo ""
    echo "Make sure:"
    echo "1. SURFPOOL is installed and running on port 8899"
    echo "2. ZAI_API_KEY is set in your .env file"
    echo "3. Default Solana keypair exists at ~/.config/solana/id.json"
fi
