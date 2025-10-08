# 🧪 Benchmark Testing & Validation Guide

## 📋 Overview

This document provides comprehensive guidance for testing and validating the Reev framework's benchmark suite and scoring system. It covers test strategies, validation procedures, and troubleshooting techniques.

## 🎯 Testing Objectives

### Primary Goals
1. **Score Validation**: Ensure scoring system works across full spectrum (0%, 50%, 75%, 100%)
2. **Anti-False-Positive Testing**: Differentiate between failure modes
3. **Regression Prevention**: Maintain consistency across framework changes
4. **Performance Validation**: Ensure benchmarks execute efficiently

### Secondary Goals
1. **Edge Case Discovery**: Identify boundary conditions and unusual scenarios
2. **Documentation Accuracy**: Keep test cases aligned with documentation
3. **User Experience**: Validate framework usability and interpretability

## 🧪 Test Suite Categories

### 1. Scoring Validation Tests

#### Purpose
Validate the two-tiered scoring system across all possible outcomes.

#### Test Cases

| Test ID | Benchmark | Expected Score | Purpose |
|---------|-----------|---------------|---------|
| T001 | `001-sol-transfer` | 100% | Perfect execution baseline |
| T002 | `002-spl-transfer` | 100% | SPL token success case |
| T003 | `003-spl-transfer-fail` | 0% | Complete failure (no instructions) |
| T004 | `004-partial-score-spl-transfer` | ~50% | Partial credit system |
| T005 | `100-jup-swap-sol-usdc` | 100% | Complex DeFi success |

#### Execution
```bash
# Run all scoring validation tests
for benchmark in 001-sol-transfer 002-spl-transfer 003-spl-transfer-fail 004-partial-score-spl-transfer 100-jup-swap-sol-usdc; do
    echo "Testing $benchmark..."
    cargo run -p reev-runner -- benchmarks/$benchmark.yml --agent deterministic
done
```

#### Validation Criteria
- ✅ Actual scores match expected scores within ±5%
- ✅ Score breakdown shows correct component weighting
- ✅ On-chain execution status aligns with expectations
- ✅ Database records are created correctly

### 2. Protocol Integration Tests

#### Jupiter Protocol Tests
```bash
# Jupiter swap
cargo run -p reev-runner -- benchmarks/100-jup-swap-sol-usdc.yml --agent deterministic

# Jupiter lending operations
cargo run -p reev-runner -- benchmarks/110-jup-lend-deposit-sol.yml --agent deterministic
cargo run -p reev-runner -- benchmarks/111-jup-lend-deposit-usdc.yml --agent deterministic
cargo run -p reev-runner -- benchmarks/112-jup-lend-withdraw-sol.yml --agent deterministic
cargo run -p reev-runner -- benchmarks/113-jup-lend-withdraw-usdc.yml --agent deterministic

# Jupiter mint/redeem
cargo run -p reev-runner -- benchmarks/115-jup-lend-mint-usdc.yml --agent deterministic
cargo run -p reev-runner -- benchmarks/116-jup-lend-redeem-usdc.yml --agent deterministic

# Jupiter positions/earnings (API)
cargo run -p reev-runner -- benchmarks/114-jup-positions-and-earnings.yml --agent deterministic
```

#### Multi-Step Flow Tests
```bash
# Swap then lend
cargo run -p reev-runner -- benchmarks/200-jup-swap-then-lend-deposit.yml --agent deterministic

# Compound strategies
cargo run -p reev-runner -- benchmarks/201-compound-strategy.yml --agent deterministic
```

### 3. Agent Compatibility Tests

#### Deterministic Agent Tests
```bash
# Test all benchmarks with deterministic agent
cargo run -p reev-runner -- benchmarks/ --agent deterministic
```

#### AI Agent Tests
```bash
# Local model tests (requires LM Studio)
cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent local-model

# Gemini tests (requires API key)
GEMINI_API_KEY=your_key cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent gemini-2.5-pro
```

## 🔍 Validation Procedures

### Score Validation Checklist

#### For Each Test Run:
1. **Check Final Score**: Does it match expected range?
2. **Verify Component Breakdown**: Are individual components scored correctly?
3. **Confirm On-Chain Status**: Does execution status align with expectations?
4. **Validate Database Entry**: Are results persisted correctly?
5. **Review Logs**: Are there any unexpected warnings or errors?

#### Example Validation for 50% Score Test:
```bash
# Run the test
cargo run -p reev-runner -- benchmarks/004-partial-score-spl-transfer.yml --agent deterministic

# Expected output should show:
# - Instruction Score: ~71.4% (correct program ID + accounts, wrong data)
# - On-Chain Score: 0% (transaction fails)
# - Final Score: ~53.6% (weighted average)
```

### Regression Testing

#### Automated Regression Checks
```bash
# Run full test suite and save results
cargo run -p reev-runner -- benchmarks/ --agent deterministic > test_results.log

# Compare with baseline results
diff test_results.log baseline_results.log
```

#### Score Consistency Validation
```bash
# Run same test multiple times to ensure consistency
for i in {1..5}; do
    echo "Run $i:"
    cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent deterministic | grep "Score:"
done
```

## 🐛 Troubleshooting Guide

### Common Issues and Solutions

#### 1. Score Mismatches
**Symptom**: Actual score differs significantly from expected score

**Debugging Steps**:
```bash
# Enable detailed scoring logs
RUST_LOG=debug cargo run -p reev-runner -- benchmarks/004-partial-score-spl-transfer.yml --agent deterministic

# Look for these log entries:
# - [SCORING-DEBUG] messages showing component comparisons
# - Final weighted score calculation
# - Individual component scores
```

**Common Causes**:
- Benchmark configuration errors (incorrect weights)
- Placeholder resolution issues
- Account metadata mismatches

#### 2. Transaction Failures
**Symptom**: Expected successful transaction fails on-chain

**Debugging Steps**:
```bash
# Check transaction simulation logs
RUST_LOG=info cargo run -p reev-runner -- benchmarks/100-jup-swap-sol-usdc.yml --agent deterministic

# Look for:
# - Simulation logs showing specific error
# - Account state issues
# - Instruction data problems
```

**Common Fixes**:
- Increase slippage tolerance for Jupiter swaps
- Verify account initializations
- Check instruction data encoding

#### 3. Database Lock Issues
**Symptom**: "SQL execution failure: Locking error"

**Solution**:
```bash
# Kill existing processes
pkill -f reev-agent
pkill -f surfpool

# Wait and retry
sleep 5
cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent deterministic
```

### Performance Issues

#### Slow Test Execution
```bash
# Check system resources
htop  # Monitor CPU/memory usage

# Optimize by running tests in parallel where possible
cargo test -p reev-runner --release
```

#### Memory Leaks
```bash
# Monitor memory usage during test runs
watch -n 1 'ps aux | grep -E "(reev-agent|surfpool)" | grep -v grep'
```

## 📊 Test Result Analysis

### Score Distribution Analysis

#### Expected Distribution
- **0%**: 20% of tests (intentional failures)
- **50%**: 20% of tests (partial credit scenarios)
- **75%**: 10% of tests (reasoning success, execution failure)
- **100%**: 50% of tests (normal successful operations)

#### Analysis Script
```bash
#!/bin/bash
# analyze_scores.sh - Analyze test score distribution

echo "Score Distribution Analysis"
echo "=========================="

# Run all tests and extract scores
cargo run -p reev-runner -- benchmarks/ --agent deterministic | grep "Score:" | \
  sed 's/.*Score: \([0-9.]*\)%/\1/' | \
  awk '
  {
    if ($1 == 0) zero++
    else if ($1 < 60) fifty++
    else if ($1 < 90) seventy_five++
    else if ($1 >= 90) hundred++
    total++
  }
  END {
    printf "0%%: %d (%.1f%%)\n", zero, zero/total*100
    printf "~50%%: %d (%.1f%%)\n", fifty, fifty/total*100
    printf "~75%%: %d (%.1f%%)\n", seventy_five, seventy_five/total*100
    printf "100%%: %d (%.1f%%)\n", hundred, hundred/total*100
    printf "Total: %d tests\n", total
  }'
```

### Database Query Analysis

#### Query Test Results
```sql
-- Connect to the database
sqlite3 db/reev_results.db

-- View recent test results
SELECT benchmark_id, score, timestamp 
FROM benchmark_results 
ORDER BY timestamp DESC 
LIMIT 10;

-- Analyze score distribution
SELECT 
  CASE 
    WHEN score = 0 THEN '0%'
    WHEN score < 60 THEN '~50%'
    WHEN score < 90 THEN '~75%'
    ELSE '100%'
  END as score_range,
  COUNT(*) as count,
  ROUND(COUNT(*) * 100.0 / (SELECT COUNT(*) FROM benchmark_results), 1) as percentage
FROM benchmark_results 
GROUP BY score_range 
ORDER BY score_range;
```

## 🔄 Continuous Integration

### GitHub Actions Workflow

```yaml
name: Benchmark Testing

on: [push, pull_request]

jobs:
  test-benchmarks:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v2
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Run benchmark tests
      run: |
        # Test scoring validation
        cargo run -p reev-runner -- benchmarks/003-spl-transfer-fail.yml --agent deterministic
        cargo run -p reev-runner -- benchmarks/004-partial-score-spl-transfer.yml --agent deterministic
        cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent deterministic
        
        # Test Jupiter integration
        cargo run -p reev-runner -- benchmarks/100-jup-swap-sol-usdc.yml --agent deterministic
        
    - name: Validate scores
      run: |
        # Add score validation logic here
        ./scripts/validate_scores.sh
```

### Score Validation Script
```bash
#!/bin/bash
# validate_scores.sh - Validate test scores against expected ranges

declare -A expected_scores=(
  ["003-spl-transfer-fail"]="0"
  ["004-partial-score-spl-transfer"]="50"
  ["001-sol-transfer"]="100"
  ["100-jup-swap-sol-usdc"]="100"
)

for benchmark in "${!expected_scores[@]}"; do
  expected=${expected_scores[$benchmark]}
  
  # Run test and extract score
  result=$(cargo run -p reev-runner -- benchmarks/$benchmark.yml --agent deterministic 2>/dev/null | grep "Score:" | sed 's/.*Score: \([0-9.]*\)%/\1/')
  
  # Validate score
  if [[ $benchmark == "003-spl-transfer-fail" ]]; then
    if [[ $result -eq 0 ]]; then
      echo "✅ $benchmark: $result% (expected 0%)"
    else
      echo "❌ $benchmark: $result% (expected 0%)"
      exit 1
    fi
  elif [[ $benchmark == "004-partial-score-spl-transfer" ]]; then
    if [[ $result -gt 40 && $result -lt 60 ]]; then
      echo "✅ $benchmark: $result% (expected ~50%)"
    else
      echo "❌ $benchmark: $result% (expected ~50%)"
      exit 1
    fi
  else
    if [[ $result -gt 95 ]]; then
      echo "✅ $benchmark: $result% (expected 100%)"
    else
      echo "❌ $benchmark: $result% (expected 100%)"
      exit 1
    fi
  fi
done

echo "All benchmark scores validated! ✅"
```

## 📈 Best Practices

### Test Development
1. **Clear Purpose**: Each test should have a specific validation goal
2. **Expected Results**: Document expected scores and behaviors
3. **Isolation**: Tests should not depend on each other
4. **Reproducibility**: Tests must produce consistent results

### Score Validation
1. **Component Testing**: Test individual scoring components
2. **Integration Testing**: Test complete scoring workflow
3. **Edge Cases**: Validate boundary conditions
4. **Regression Testing**: Ensure consistency over time

### Documentation
1. **Test Descriptions**: Clearly document each test's purpose
2. **Expected Results**: Document expected scores and behaviors
3. **Troubleshooting**: Include common issues and solutions
4. **Maintenance**: Keep documentation updated with changes

## 🎯 Conclusion

The Reev framework's benchmark testing suite provides comprehensive validation of the scoring system and agent performance. By following the procedures and best practices outlined in this guide, you can ensure reliable, consistent testing and validation of Solana LLM agents across the full spectrum of possible outcomes.

Regular execution of these tests and validation of results ensures the framework maintains its accuracy and reliability as it evolves and expands.