#!/bin/bash

# Test script to run benchmarks with local agents
# Usage: ./test_local_agent.sh [--local] [benchmark1.yml benchmark2.yml ...]
# Default: deterministic agents, all benchmarks

# Cleanup function to kill existing processes
cleanup_processes() {
    echo "🧹 Cleaning up processes..."

    # Kill all cargo processes first (most aggressive)
    pkill -9 -f "cargo" 2>/dev/null || true

    # Kill reev-agent processes
    pkill -9 -f "reev-agent" 2>/dev/null || true

    # Kill surfpool processes
    pkill -9 -f "surfpool" 2>/dev/null || true

    # Kill any remaining processes on ports
    pids=$(lsof -ti:9090 2>/dev/null)
    if [ ! -z "$pids" ]; then
        echo "  Killing processes on port 9090: $pids"
        kill -9 $pids 2>/dev/null || true
    fi

    pids=$(lsof -ti:8899 2>/dev/null)
    if [ ! -z "$pids" ]; then
        echo "  Killing processes on port 8899: $pids"
        kill -9 $pids 2>/dev/null || true
    fi

    # Kill any remaining processes that might hold shared state
    pkill -9 -f "target/debug/reev-agent" 2>/dev/null || true
    pkill -9 -f "target/debug/reev-runner" 2>/dev/null || true

    # Wait a moment for processes to fully terminate
    sleep 1

    # Double-check ports are clear
    pids=$(lsof -ti:9090 2>/dev/null)
    if [ ! -z "$pids" ]; then
        echo "  Force killing remaining processes on port 9090: $pids"
        kill -9 $pids 2>/dev/null || true
    fi

    pids=$(lsof -ti:8899 2>/dev/null)
    if [ ! -z "$pids" ]; then
        echo "  Force killing remaining processes on port 8899: $pids"
        kill -9 $pids 2>/dev/null || true
    fi

    echo "✅ Cleanup complete"
}

    # Function to show help
    show_help() {
        echo "Usage: $0 [--kill] [--local] [benchmark1.yml benchmark2.yml ...]"
        echo ""
        echo "Options:"
        echo "  --kill          Kill all existing reev processes and exit"
        echo "  --local         Test with enhanced agents (requires tool-calling capable model)"
        echo "                  Default: deterministic agents (works with any model)"
        echo "  [benchmarks...] Optional specific benchmark files to test"
        echo "                  If not provided, tests all benchmarks"
        echo "  -h, --help       Show this help message"
    }

# Default to deterministic agents
AGENT_TYPE="deterministic"
AGENT_FLAG="deterministic"
SPECIFIC_BENCHMARKS=()

# Parse command line arguments
for arg in "$@"; do
    case $arg in
        --kill)
            echo "🧹 Killing all reev processes..."
            cleanup_processes
            exit 0
            ;;
        --local)
            AGENT_TYPE="local"
            AGENT_FLAG="local"
            shift
            ;;
        -h|--help)
            show_help
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

# Use specific benchmarks if provided, otherwise dynamically discover all benchmarks
if [ ${#SPECIFIC_BENCHMARKS[@]} -gt 0 ]; then
    benchmarks=("${SPECIFIC_BENCHMARKS[@]}")
else
    # Dynamically discover all benchmark files
    echo "Discovering benchmark files..."

    # Get all .yml files in benchmarks directory, sorted alphabetically
    # Use a simpler method compatible with both macOS and Linux
    benchmarks=($(find benchmarks -name "*.yml" -type f | sort))

    if [ ${#benchmarks[@]} -eq 0 ]; then
        echo "❌ No benchmark files found in benchmarks/ directory"
        exit 1
    fi

    echo "Found ${#benchmarks[@]} benchmark files"
    for benchmark in "${benchmarks[@]}"; do
        echo "  - $(basename "$benchmark")"
    done
fi

# Cleanup before starting
cleanup_processes

# Set up cleanup on script exit and interruption
trap 'echo ""; echo "🛑 Interrupted! Cleaning up..."; cleanup_processes; exit 130' INT TERM

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

    # Run the benchmark and capture the output
    output=$(cargo run -p reev-runner -- "$full_benchmark" --agent $AGENT_FLAG 2>&1)
    cargo_exit_code=$?

    # Check if we were interrupted
    if [ $cargo_exit_code -ne 0 ]; then
        echo ""
        echo "🛑 Benchmark failed or interrupted (exit code: $cargo_exit_code)!"
        echo "Last 10 lines of output:"
        echo "$output" | tail -n 10
        echo "---"
        # Don't exit - continue to next benchmark
    fi

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
