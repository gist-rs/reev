# Reev Core Architecture Plan - Benchmark

## 🎯 **Why: Benchmark Strategy for Verifiable AI-Generated DeFi Flows**

This document complements PLAN_CORE_V2.md by focusing on benchmark-specific requirements, evaluation criteria, and testing strategies for the novel verifiable AI-generated DeFi flows architecture.

## 🔄 **Benchmark Architecture Overview**

### **Key Concept: Deterministic Verification**
Using SURFPOOL's mainnet-forking capability to create deterministic test environments where:
- AI-generated flows can be verified against known outcomes
- Performance can be measured objectively
- Error scenarios can be reproduced consistently

### **Dual Purpose YML in Benchmarking**
```yaml
# Ground truth YML serves dual purposes:
# 1. Runtime guardrails (during execution)
# 2. Evaluation criteria (after execution)

ground_truth:
  # Guardrails for execution
  final_state_assertions:
    - type: SolBalanceChange
      expected_change_gte: -200500000
      error_tolerance: 0.01
  
  # Evaluation criteria for scoring
  success_criteria:
    - type: "percentage_calculation"
      weight: 0.25
```

## 📊 **Benchmark Categories**

### **1. Flow Generation Benchmarks**
Test AI's ability to generate correct YML flows from various inputs:
- Language variations: "แลก", "swp", "exchange"
- Typo handling: "swp", "lned", "muliply"
- Complexity levels: Simple swap vs multi-step strategies
- Intent clarity: Clear vs ambiguous user prompts

### **2. Execution Accuracy Benchmarks**
Measure execution quality against expected outcomes:
- Parameter accuracy: Correct amounts, tokens, slippage
- Step coordination: Proper sequence and dependencies
- Error handling: Recovery from network issues and slippage
- Efficiency: Minimal unnecessary operations

### **3. Performance Benchmarks**
Evaluate system performance under various conditions:
- Planning time: Phase 1 LLM call duration
- Execution time: Individual step and total flow time
- Resource usage: Memory, CPU, API calls
- Recovery overhead: Time and cost of error recovery

## 🏗️ **Benchmark YML Structure**

### **Complete Benchmark YML Example**
```yaml
id: "300-jup-swap-then-lend-deposit-dyn"
description: "Dynamic multiplication strategy - Agent uses 50% of SOL to multiply USDC position by 1.5x using Jupiter swap and lending strategies with dynamic orchestration."
flow_type: "dynamic"
tags: ["dynamic", "multiplication", "jupiter", "yield", "strategy"]

initial_state:
  # User's main wallet with SOL for multiplication strategy
  - pubkey: "USER_WALLET_PUBKEY"
    owner: "11111111111111111111111111111111"
    lamports: 4000000000 # 4 SOL

  # User's existing USDC position to be multiplied
  - pubkey: "USER_USDC_ATA"
    owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    lamports: 2039280
    data:
      mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
      owner: "USER_WALLET_PUBKEY"
      amount: "20000000" # 20 USDC existing

prompt: "use my 50% sol to multiply usdc 1.5x on jup"

ground_truth:
  min_score: 0.7

  final_state_assertions:
    # Should have used ~2 SOL (50% of 4 SOL + fees)
    - type: SolBalanceChange
      pubkey: "USER_WALLET_PUBKEY"
      expected_change_gte: -200500000 # Should not use more than 2 SOL + fees
      weight: 0.3

    # Should have ~50 USDC total (20 existing + 30 from multiplication)
    - type: TokenAccountBalance
      pubkey: "USER_USDC_ATA"
      mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
      expected_gte: 45000000 # ~45 USDC (20 existing + 25 from swap + yield)
      expected_lte: 55000000 # ~55 USDC maximum
      weight: 0.4

    # Should have reduced USDC balance after lending (proof of deposit)
    - type: TokenAccountBalance
      pubkey: "USER_USDC_ATA"
      mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
      expected_lte: 50000000 # Should be <= 50 USDC after lending deposit (some tokens moved to lending)
      weight: 0.15

    # Balance deduction validation - proves lending occurred
    - type: TokenAccountBalance
      pubkey: "USER_USDC_ATA"
      mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
      expected_lte: 45000000 # Should be less than maximum acquired (proves tokens moved to lending)
      weight: 0.15

  expected_tool_calls:
    - tool_name: "get_account_balance"
      description: "Check current wallet balances and positions"
      critical: false
      expected_params: ["wallet_pubkey"]
      weight: 0.1

    - tool_name: "jupiter_swap"
      description: "Swap SOL to USDC using Jupiter"
      critical: true
      expected_params: ["input_token", "output_token", "amount"]
      weight: 0.4

    - tool_name: "jupiter_lend_earn_deposit"
      description: "Deposit USDC into Jupiter lending for yield"
      critical: true
      expected_params: ["mint", "amount"]
      weight: 0.4

  success_criteria:
    - type: "percentage_calculation"
      description: "Correctly calculate and use 50% of SOL balance"
      required: true
      weight: 0.25

    - type: "multiplication_strategy"
      description: "Implement strategy to multiply USDC by 1.5x"
      required: true
      weight: 0.3

    - type: "tool_coordination"
      description: "Coordinate swap and lend operations properly"
      required: true
      weight: 0.25

    - type: "yield_generation"
      description: "Generate additional yield through lending"
      required: true
      weight: 0.2

  expected_data_structure:
    - path: "$.result.data.wallet_context"
      type: "object"
      required_fields: ["sol_balance", "usdc_balance", "total_portfolio_value"]
      weight: 0.2

    - path: "$.result.data.swap_execution"
      type: "object"
      required_fields: ["input_amount", "output_amount", "exchange_rate"]
      weight: 0.2

    - path: "$.result.data.lend_execution"
      type: "object"
      required_fields: ["deposit_amount", "expected_yield", "apy_rate"]
      weight: 0.2

    - path: "$.result.data.final_state"
      type: "object"
      required_fields:
        ["final_sol_balance", "final_usdc_balance", "multiplication_ratio"]
      weight: 0.2

    - path: "$.result.data.tool_calls"
      type: "array"
      min_items: 2
      required_fields: ["tool_name", "parameters", "success"]
      weight: 0.2

  expected_flow_complexity:
    - type: "multi_step_execution"
      description: "Should execute swap → lend sequence"
      min_steps: 2
      weight: 0.3

    - type: "context_awareness"
      description: "Should use wallet context for calculations"
      required: true
      weight: 0.3

    - type: "strategic_planning"
      description: "Should plan multiplication strategy rather than simple actions"
      required: true
      weight: 0.4

  expected_multiplication_metrics:
    - type: "target_achievement"
      description: "Should achieve ~1.5x USDC multiplication"
      target_ratio: 1.5
      tolerance: 0.2 # Allow 1.3x to 1.7x range
      weight: 0.4

    - type: "capital_efficiency"
      description: "Should efficiently use 50% of SOL for multiplication"
      min_efficiency: 0.8 # 80% capital efficiency
      weight: 0.3

    - type: "yield_contribution"
      description: "Yield should contribute to multiplication goal"
      min_yield_contribution: 0.1 # 10% of target from yield
      weight: 0.3

  expected_otel_tracking:
    - type: "tool_call_logging"
      description: "OpenTelemetry should track all tool calls"
      required_tools:
        ["get_account_balance", "jupiter_swap", "jupiter_lend_earn_deposit"]
      weight: 0.3

    - type: "execution_tracing"
      description: "Flow execution should be traceable end-to-end"
      required_spans:
        [
          "prompt_processing",
          "context_resolution",
          "swap_execution",
          "lend_execution",
        ]
      weight: 0.3

    - type: "mermaid_generation"
      description: "Should generate flow diagram from tool calls"
      required: true
      weight: 0.2

    - type: "performance_metrics"
      description: "Should track execution time and resource usage"
      required_metrics: ["execution_time_ms", "tool_call_count", "success_rate"]
      weight: 0.2

  recovery_expectations:
    - type: "swap_failure_recovery"
      description: "Should recover from failed swap with alternative routes"
      required: true
      weight: 0.2

    - type: "lend_failure_recovery"
      description: "Should recover from failed lend with retry mechanism"
      required: true
      weight: 0.2

    - type: "atomic_execution"
      description: "Multiplication strategy should execute atomically"
      required: true
      weight: 0.3

    - type: "partial_success_handling"
      description: "Should handle partial successes gracefully"
      required: true
      weight: 0.3
```

## 🧪 **Benchmark Testing Strategy**

### **Static vs Dynamic Benchmarking**
1. **Static Benchmarking**: Use pre-defined YML files
   - Reproducible results across runs
   - Consistent evaluation criteria
   - Controlled test scenarios

2. **Dynamic Benchmarking**: Generate YML from prompts
   - Test AI generation capabilities
   - Real-world prompt variations
   - Natural language understanding

### **Test Categories**
```yaml
# Language variation tests
language_tests:
  - prompt: "แลก 1 SOL เป็น USDC"
    expected_tool: "jupiter_swap"
    expected_amount: "1000000000"
    
  - prompt: "swp 1 sol 2 usdc"
    expected_tool: "jupiter_swap"
    expected_amount: "1000000000"

# Complexity tests
complexity_tests:
  - simple: "swap 1 SOL to USDC"
  - medium: "use 50% SOL to get USDC"
  - complex: "use 50% SOL to multiply USDC 1.5x on jup"

# Error scenario tests
error_tests:
  - network_failure: Simulate Jupiter API timeout
  - slippage: Simulate high slippage scenarios
  - insufficient_balance: Test with insufficient funds
```

## 📊 **Evaluation Framework**

### **Scoring Algorithm**
```rust
fn calculate_benchmark_score(
    ground_truth: &GroundTruth,
    execution_result: &ExecutionResult,
    performance_metrics: &PerformanceMetrics,
) -> BenchmarkScore {
    let mut score = BenchmarkScore::new();
    
    // Final state assertions (40% of total)
    score.add_category("final_state", evaluate_final_state(
        &ground_truth.final_state_assertions,
        &execution_result.final_state,
    ));
    
    // Tool call accuracy (30% of total)
    score.add_category("tool_calls", evaluate_tool_calls(
        &ground_truth.expected_tool_calls,
        &execution_result.tool_calls,
    ));
    
    // Success criteria (20% of total)
    score.add_category("success_criteria", evaluate_success_criteria(
        &ground_truth.success_criteria,
        &execution_result,
    ));
    
    // Performance metrics (10% of total)
    score.add_category("performance", evaluate_performance(
        &ground_truth.performance_expectations,
        performance_metrics,
    ));
    
    score.calculate_weighted_average()
}
```

### **Benchmark Reports**
```yaml
benchmark_report:
  flow_id: "300-jup-swap-then-lend-deposit-dyn"
  execution_id: "01H9X2X3Y4Z5A6B7C8D9E0F1G2"
  
  overall_score: 0.85
  
  category_scores:
    final_state: 0.9
    tool_calls: 0.8
    success_criteria: 0.85
    performance: 0.75
  
  detailed_results:
    passed_assertions: 12
    failed_assertions: 2
    execution_time_ms: 3450
    recovery_attempts: 1
    
  improvement_suggestions:
    - "Reduce slippage tolerance for better price execution"
    - "Optimize tool call sequence to reduce execution time"
```

## 🚀 **Benchmark Implementation**

### **Benchmark Execution Flow**
```
1. Initialize SURFPOOL with mainnet fork
2. Set up initial state using surfnet_setAccount
3. Execute benchmark flow with monitoring
4. Collect execution results and metrics
5. Calculate score against ground truth
6. Generate benchmark report
```

### **Benchmark CI/CD Integration**
```yaml
# .github/workflows/benchmark.yml
name: Reev Benchmark Tests
on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run Benchmarks
        run: |
          cargo run --bin benchmark-runner \
            -- --category all \
            --format json \
            --output benchmark-results.json
      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: benchmark-results.json
```

## 📈 **Performance Targets**

### **Benchmark Performance Goals**
| Metric | Target | Measurement |
|--------|--------|-------------|
| **Flow Generation Time** | < 2 seconds | Time from prompt to YML generation |
| **Total Execution Time** | < 10 seconds | Complete flow execution time |
| **Tool Call Accuracy** | > 95% | Correct tool calls / total tool calls |
| **Parameter Accuracy** | > 90% | Correct parameters / total parameters |
| **Error Recovery Success** | > 80% | Successful recoveries / total failures |
| **Benchmark Score** | > 0.8 | Weighted score across all criteria |

### **Continuous Improvement**
- Track benchmark scores over time
- Identify regression patterns
- Compare performance across LLM models
- Optimize based on benchmark insights

## 🔄 **Next Steps**

1. **Implementation Phase 1**: Create benchmark runner with static YML tests
2. **Implementation Phase 2**: Add dynamic prompt-based benchmarks
3. **Implementation Phase 3**: Integrate with CI/CD pipeline
4. **Implementation Phase 4**: Add performance tracking and optimization

## 📝 **Related Documents**

- PLAN_CORE_V2.md: Core architecture for production implementation
- SURFPOOL.md: SURFPOOL integration details for deterministic testing
- REEV-BENCHMARKING.md: Implementation details for benchmark runner

---

This document focuses specifically on benchmark requirements, evaluation criteria, and testing strategies for the novel verifiable AI-generated DeFi flows architecture, complementing the production-focused PLAN_CORE_V2.md.