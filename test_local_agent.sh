#!/bin/bash

# Test script to run benchmarks with local agents
# Usage: ./test_local_agent.sh [--local] [benchmark1.yml benchmark2.yml ...]
# Default: deterministic agents, all benchmarks

# Default to deterministic agents
AGENT_TYPE="deterministic"
AGENT_FLAG="local"
SPECIFIC_BENCHMARKS=()

# Parse command line arguments
for arg in "$@"; do
    case $arg in
        --local)
            AGENT_TYPE="enhanced"
            AGENT_FLAG="local-model"
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--local] [benchmark1.yml benchmark2.yml ...]"
            echo "  --local         Test with enhanced agents (requires tool-calling capable model)"
            echo "                  Default: deterministic agents (works with any model)"
            echo "  [benchmarks...]  Optional specific benchmark files to test"
            echo "                  If not provided, tests all benchmarks"
            echo "  -h, --help       Show this help message"
            exit 0
            ;;
        *.yml)
            SPECIFIC_BENCHMARKS+=("$arg")
            shift
            ;;
        *)
            echo "Unknown option: $arg"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Use specific benchmarks if provided, otherwise use all benchmarks
if [ ${#SPECIFIC_BENCHMARKS[@]} -gt 0 ]; then
    benchmarks=("${SPECIFIC_BENCHMARKS[@]}")
else
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
fi

echo "Testing ${#benchmarks[@]} benchmark(s) with $AGENT_TYPE agents (flag: --agent $AGENT_FLAG)"
if [ ${#SPECIFIC_BENCHMARKS[@]} -gt 0 ]; then
    echo "Specific benchmarks: ${benchmarks[*]}"
else
    echo "All benchmarks"
fi
echo "==========================================================================="

results=()

for benchmark in "${benchmarks[@]}"; do
    echo -n "Testing $benchmark... "

    # Run the benchmark and capture the output
    # Ensure the benchmark path includes the benchmarks/ directory
    if [[ "$benchmark" != benchmarks/* ]]; then
        full_benchmark="benchmarks/$benchmark"
    else
        full_benchmark="$benchmark"
    fi
    output=$(cargo run -p reev-runner -- "$full_benchmark" --agent $AGENT_FLAG 2>&1)

    # Debug: Print last few lines of output if it failed
    if ! echo "$output" | grep -q "✅.*Succeeded\|❌.*Failed"; then
        echo "❌ ERROR - Last 10 lines of output:"
        echo "$output" | tail -n 10
        echo "---"
    fi

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
