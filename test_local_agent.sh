#!/bin/bash

# Test script to run all benchmarks with the local agent

echo "Testing all benchmarks with --agent local"
echo "=========================================="

benchmarks=(
    "001-sol-transfer.yml"
    "002-spl-transfer.yml"
    "003-spl-transfer-fail.yml"
    "100-jup-swap-sol-usdc.yml"
    "110-jup-lend-deposit-sol.yml"
    "111-jup-lend-deposit-usdc.yml"
    "112-jup-lend-withdraw-sol.yml"
    "113-jup-lend-withdraw-usdc.yml"
    "114-jup-positions-and-earnings.yml"
    "200-jup-swap-then-lend-deposit.yml"
)

results=()

for benchmark in "${benchmarks[@]}"; do
    echo -n "Testing $benchmark... "

    # Run the benchmark and capture the output
    output=$(cargo run -p reev-runner -- "benchmarks/$benchmark" --agent local 2>&1)

    # Check if it succeeded
    if echo "$output" | grep -q "✅.*Succeeded"; then
        score=$(echo "$output" | grep -o "Score: [0-9.]*%" | head -1)
        echo "✅ $score"
        results+=("$benchmark: SUCCESS ($score)")
    elif echo "$output" | grep -q "❌.*Failed"; then
        score=$(echo "$output" | grep -o "Score: [0-9.]*%" | head -1)
        echo "❌ $score"
        results+=("$benchmark: FAILED ($score)")
    else
        echo "❌ ERROR"
        results+=("$benchmark: ERROR")
    fi

    # Small delay between tests
    sleep 2
done

echo ""
echo "Summary:"
echo "========"
for result in "${results[@]}"; do
    echo "$result"
done
