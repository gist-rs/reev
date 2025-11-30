# Reev Benchmark Implementation Tasks - Two-Stage Protocol Approach

This document outlines the remaining work to fully implement the benchmark requirements from PLAN_CORE_BENCHMARK.md, with a focus on the two-stage protocol approach defined in PLAN_PROTOCOLS.md. Each task is modular, testable independently, and aligned with the comprehensive benchmark architecture.

## 📋 Task Categories

### 1. Stage 1 Protocol Interface Implementation
**Priority**: High
**Estimated Effort**: 4-5 days

#### 1.1 Core Protocol Interface
**File**: `crates/reev-core/src/protocols/executor.rs`
**Description**: Implement minimal protocol interface for immediate needs
**Acceptance Criteria**:
- [ ] Create `ProtocolExecutor` trait with execute method
- [ ] Define `ProtocolOperation` and `OperationType` structures
- [ ] Implement error types for protocol operations
- [ ] Keep interface minimal but extensible

**Test Cases**:
```rust
#[tokio::test]
async fn test_protocol_executor_trait() {
    // Test ProtocolExecutor trait implementation
}

#[tokio::test]
async fn test_protocol_operation() {
    // Test ProtocolOperation structure and serialization
}
```

#### 1.2 Jupiter Protocol Wrapper
**File**: `crates/reev-core/src/protocols/jupiter/mod.rs`
**Description**: Wrap existing Jupiter handlers with protocol interface
**Acceptance Criteria**:
- [ ] Implement `JupiterProtocol` struct with protocol trait
- [ ] Wrap existing swap, lend, and earn handlers
- [ ] Convert between protocol and handler parameter formats
- [ ] Maintain compatibility with existing functionality

**Test Cases**:
```rust
#[tokio::test]
async fn test_jupiter_protocol_swap() {
    // Test Jupiter protocol swap operation
}

#[tokio::test]
async fn test_jupiter_protocol_lend() {
    // Test Jupiter protocol lend operation
}

#[tokio::test]
async fn test_jupiter_protocol_earn() {
    // Test Jupiter protocol earn operation
}
```

#### 1.3 Protocol Registry
**File**: `crates/reev-core/src/protocols/registry.rs`
**Description**: Simple protocol registry for protocol discovery
**Acceptance Criteria**:
- [ ] Implement protocol registration and retrieval
- [ ] Support protocol selection by operation type
- [ ] Prepare for future protocol additions
- [ ] Maintain protocol instances during execution

**Test Cases**:
```rust
#[tokio::test]
async fn test_protocol_registry() {
    // Test protocol registration and retrieval
}

#[tokio::test]
async fn test_protocol_selection() {
    // Test protocol selection by operation type
}
```

### 2. Enhanced Validation Framework
**Priority**: High
**Estimated Effort**: 3-4 days

#### 2.1 Blockchain State Validation
**File**: `crates/reev-core/src/benchmark/validators/blockchain_state.rs`
**Description**: Implement actual blockchain state validation for assertions
**Acceptance Criteria**:
- [ ] Verify SOL balance changes after transactions
- [ ] Verify token account balances after swaps, lends, transfers
- [ ] Handle edge cases (insufficient funds, slippage)
- [ ] Support all assertion types in YmlAssertion

**Test Cases**:
```rust
#[tokio::test]
async fn test_sol_balance_validation() {
    // Test SolBalanceChange assertion against actual blockchain state
}

#[tokio::test]
async fn test_token_balance_validation() {
    // Test TokenAccountBalance assertion against actual token holdings
}
```

#### 2.2 Assertion Engine
**File**: `crates/reev-core/src/benchmark/validators/assertion_engine.rs`
**Description**: Create a generic assertion validation engine
**Acceptance Criteria**:
- [ ] Support for all assertion types (SolBalanceChange, TokenAccountBalance)
- [ ] Weighted assertion scoring
- [ ] Parameter validation (expected_gte, expected_lte)
- [ ] Custom assertion parameter handling

**Test Cases**:
```rust
#[tokio::test]
async fn test_assertion_engine() {
    // Test assertion validation with various types and parameters
}
```

### 3. Benchmark Runner
**Priority**: High
**Estimated Effort**: 4-5 days

#### 3.1 Static Benchmark Runner
**File**: `crates/reev-core/src/benchmark/runner/static_runner.rs`
**Description**: Execute predefined benchmark flows
**Acceptance Criteria**:
- [ ] Load YML flows from file system
- [ ] Initialize SURFPOOL with deterministic state
- [ ] Execute flows using protocol interface
- [ ] Collect metrics and generate reports
- [ ] Support multiple flow categories

**Test Cases**:
```rust
#[tokio::test]
async fn test_static_swap_benchmark() {
    // Test running a predefined swap benchmark flow
}

#[tokio::test]
async fn test_static_multi_step_benchmark() {
    // Test running a predefined multi-step benchmark flow
}
```

#### 3.2 Dynamic Benchmark Runner
**File**: `crates/reev-core/src/benchmark/runner/dynamic_runner.rs`
**Description**: Generate and test flows from prompts
**Acceptance Criteria**:
- [ ] Accept prompt input in various languages
- [ ] Generate flows using QueryHandler
- [ ] Execute flows using protocol interface
- [ ] Score generation quality against expectations

**Test Cases**:
```rust
#[tokio::test]
async fn test_dynamic_swap_benchmark() {
    // Test generating and scoring a swap flow from prompt
}

#[tokio::test]
async fn test_dynamic_multi_step_benchmark() {
    // Test generating and scoring a multi-step flow from prompt
}
```

#### 3.3 Benchmark CLI
**File**: `crates/reev-core/src/bin/benchmark_runner.rs`
**Description**: Command-line interface for running benchmarks
**Acceptance Criteria**:
- [ ] Support both static and dynamic benchmarks
- [ ] Configuration via command-line arguments
- [ ] Output in multiple formats (JSON, CSV, pretty-print)
- [ ] Integration with CI/CD

**Test Cases**:
```bash
cargo run --bin benchmark_runner --static --path benchmarks/flows/ --output json
cargo run --bin benchmark_runner --dynamic --category swap --prompt "swap 1 sol to usdc"
```

### 4. SURFPOOL Integration
**Priority**: High
**Estimated Effort**: 3-4 days

#### 4.1 SURFPOOL Environment Setup
**File**: `crates/reev-core/src/surfpool/environment.rs`
**Description**: Initialize and manage SURFPOOL environments
**Acceptance Criteria**:
- [ ] Create mainnet fork at specific block height
- [ ] Set up deterministic state for tests
- [ ] Reset state between test runs
- [ ] Handle environment cleanup

**Test Cases**:
```rust
#[tokio::test]
async fn test_surfpool_setup() {
    // Test setting up and tearing down SURFPOOL environment
}
```

#### 4.2 State Initialization
**File**: `crates/reev-core/src/surfpool/state_manager.rs`
**Description**: Initialize deterministic blockchain state
**Acceptance Criteria**:
- [ ] Set account balances via surfnet_setAccount
- [ ] Create token accounts with specific holdings
- [ ] Set up Jupiter pools with known liquidity
- [ ] Verify state matches expectations

**Test Cases**:
```rust
#[tokio::test]
async fn test_state_initialization() {
    // Test initializing deterministic blockchain state for benchmarks
}
```

### 5. Performance Monitoring
**Priority**: Medium
**Estimated Effort**: 2-3 days

#### 5.1 Resource Usage Tracker
**File**: `crates/reev-core/src/benchmark/performance/resource_tracker.rs`
**Description**: Track memory, CPU, and network usage
**Acceptance Criteria**:
- [ ] Monitor memory usage during execution
- [ ] Track CPU usage percentage
- [ ] Measure API call count and duration
- [ ] Collect timing data at step level

**Test Cases**:
```rust
#[tokio::test]
async fn test_resource_tracking() {
    // Test resource usage tracking during flow execution
}
```

#### 5.2 Performance Metrics Collector
**File**: `crates/reev-core/src/benchmark/performance/metrics_collector.rs`
**Description**: Collect and aggregate performance metrics
**Acceptance Criteria**:
- [ ] Collect metrics for each step in a flow
- [ ] Aggregate metrics to flow level
- [ ] Generate performance reports
- [ ] Export metrics in standard formats

**Test Cases**:
```rust
#[tokio::test]
async fn test_metrics_collection() {
    // Test performance metrics collection and aggregation
}
```

### 6. Error Recovery Framework
**Priority**: Medium
**Estimated Effort**: 3-4 days

#### 6.1 Error Recovery Strategies
**File**: `crates/reev-core/src/executor/recovery/strategies.rs`
**Description**: Implement specific recovery strategies for different errors
**Acceptance Criteria**:
- [ ] Handle insufficient_balance with reduced amount
- [ ] Handle slippage_exceeded with alternative routes
- [ ] Handle network_error with retries and fallback
- [ ] Support protocol-specific recovery strategies

**Test Cases**:
```rust
#[tokio::test]
async fn test_insufficient_balance_recovery() {
    // Test recovery from insufficient balance errors
}

#[tokio::test]
async fn test_slippage_recovery() {
    // Test recovery from high slippage scenarios
}
```

#### 6.2 Recovery Executor
**File**: `crates/reev-core/src/executor/recovery/executor.rs`
**Description**: Execute recovery strategies and track success
**Acceptance Criteria**:
- [ ] Detect errors and select appropriate strategy
- [ ] Execute recovery actions with monitoring
- [ ] Track recovery success/failure rates
- [ ] Limit recovery attempts to prevent infinite loops

**Test Cases**:
```rust
#[tokio::test]
async fn test_recovery_execution() {
    // Test recovery strategy execution and monitoring
}
```

### 7. Stage 2 Protocol Framework (Future Priority)
**Priority**: Low
**Estimated Effort**: 5-6 days

#### 7.1 Enhanced Protocol Interface
**File**: `crates/reev-core/src/protocols/validator.rs` (extension)
**Description**: Add validation and gas estimation to protocol interface
**Acceptance Criteria**:
- [ ] Extend ProtocolExecutor with validation methods
- [ ] Implement ProtocolValidator trait
- [ ] Implement ProtocolResultMapper trait
- [ ] Add protocol metadata support

**Test Cases**:
```rust
#[tokio::test]
async fn test_enhanced_protocol_interface() {
    // Test enhanced protocol interface with validation
}

#[tokio::test]
async fn test_protocol_validation() {
    // Test protocol validation implementation
}
```

#### 7.2 Marinade Protocol Implementation
**File**: `crates/reev-core/src/protocols/marinade/mod.rs`
**Description**: Implement Marinade protocol using enhanced interface
**Acceptance Criteria**:
- [ ] Implement Marinade protocol with validation
- [ ] Add staking operation support
- [ ] Implement result mapping for Marinade
- [ ] Add gas estimation for staking operations

**Test Cases**:
```rust
#[tokio::test]
async fn test_marinade_protocol_stake() {
    // Test Marinade protocol staking operation
}
```

### 8. Benchmark Test Suite
**Priority**: Medium
**Estimated Effort**: 2-3 days

#### 8.1 Language Variation Tests
**File**: `crates/reev-core/tests/benchmark/language_variation.rs`
**Description**: Test handling of different languages and typos
**Acceptance Criteria**:
- [ ] Support Thai, English, and simplified English
- [ ] Handle common typos and variations
- [ ] Score accuracy across language variations
- [ ] Generate appropriate tool calls for each variation

**Test Cases**:
```rust
#[tokio::test]
async fn test_thai_language_swap() {
    // Test "แลก 1 SOL เป็น USDC" generates correct swap flow
}

#[tokio::test]
async fn test_typo_handling() {
    // Test "swp 1 sol 2 usdc" generates correct swap flow
}
```

#### 8.2 Complexity Tests
**File**: `crates/reev-core/tests/benchmark/complexity_tests.rs`
**Description**: Test handling of different complexity levels
**Acceptance Criteria**:
- [ ] Execute simple flows correctly
- [ ] Handle medium complexity with context awareness
- [ ] Execute complex multi-step strategies
- [ ] Score appropriately based on complexity

**Test Cases**:
```rust
#[tokio::test]
async fn test_simple_swap() {
    // Test "swap 1 SOL to USDC" generates simple flow
}

#[tokio::test]
async fn test_complex_multiplication() {
    // Test "use 50% SOL to multiply USDC 1.5x on jup" generates complex flow
}
```

### 9. CI/CD Integration
**Priority**: Low
**Estimated Effort**: 2-3 days

#### 9.1 GitHub Actions Workflow
**File**: `.github/workflows/benchmark.yml`
**Description**: Automated benchmark execution in CI/CD
**Acceptance Criteria**:
- [ ] Trigger on push and pull request
- [ ] Run benchmarks against mainnet fork
- [ ] Upload results as artifacts
- [ ] Generate trend reports over time

**Implementation**:
```yaml
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
          cargo run --bin benchmark_runner --category all --format json --output benchmark-results.json
      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: benchmark-results.json
```

#### 9.2 Benchmark Dashboard
**File**: `scripts/benchmark_dashboard.py`
**Description**: Visualize benchmark results over time
**Acceptance Criteria**:
- [ ] Parse JSON benchmark results
- [ ] Generate trend charts
- [ ] Show score breakdown by category
- [ ] Export interactive HTML dashboard

**Implementation**:
```python
import json
import plotly.express as px
import pandas as pd

def generate_dashboard(results_file):
    # Parse results and generate dashboard
    pass
```

## 🚀 Implementation Sequence

### Week 1: Stage 1 Protocol Foundation (High Priority)
1. Core Protocol Interface
   - ProtocolExecutor trait
   - ProtocolOperation and OperationType
   - Protocol error types

2. Jupiter Protocol Wrapper
   - JupiterProtocol struct
   - Handler wrapping for swap, lend, earn
   - Parameter format conversion

### Week 2: Validation Framework (High Priority)
1. Enhanced Validation Framework
   - Blockchain State Validation
   - Assertion Engine

2. Protocol Registry
   - Protocol registration and retrieval
   - Protocol selection by operation type

### Week 3: Benchmark Runner (High Priority)
1. Benchmark Runner Foundation
   - Static Benchmark Runner
   - SURFPOOL Integration
   - Environment Setup

2. SURFPOOL Integration
   - State Initialization
   - Environment management

### Week 4: Advanced Features (Medium Priority)
1. Dynamic Benchmark Runner
   - Prompt-based flow generation
   - Protocol interface integration

2. Performance Monitoring
   - Resource Usage Tracker
   - Metrics Collector

### Week 5: Error Handling (Medium Priority)
1. Error Recovery Framework
   - Recovery Strategies
   - Recovery Executor

2. Benchmark Test Suite
   - Language Variation Tests
   - Complexity Tests

### Week 6: CI/CD & Future Prep (Low Priority)
1. CI/CD Integration
   - GitHub Actions Workflow
   - Benchmark Dashboard

2. Stage 2 Preparation
   - Enhanced Protocol Interface design
   - Marinade Protocol planning

## 📊 Success Metrics

### Technical Metrics
- [ ] All tests pass with >95% code coverage
- [ ] Benchmark execution time < 5 seconds per flow
- [ ] Memory usage < 500MB during execution
- [ ] Zero security vulnerabilities in dependencies

### Quality Metrics
- [ ] Documentation for all public APIs
- [ ] Consistent error handling throughout
- [ ] Modular design with clear separation of concerns
- [ ] Alignment with PLAN_CORE_BENCHMARK.md requirements

### Performance Metrics
- [ ] Flow generation time < 2 seconds
- [ ] Total execution time < 10 seconds
- [ ] Tool call accuracy > 95%
- [ ] Parameter accuracy > 90%
- [ ] Error recovery success > 80%
- [ ] Overall benchmark score > 0.8

## 🔍 Testing Strategy

Each task includes specific test cases for verification. The testing strategy follows these principles:

1. **Unit Tests**: Test individual components in isolation
2. **Integration Tests**: Test component interactions
3. **End-to-End Tests**: Test complete workflows
4. **Performance Tests**: Verify performance targets
5. **Regression Tests**: Prevent performance regressions

## 📝 Documentation Requirements

Each task should include:
1. **Rustdoc Comments**: Document all public APIs
2. **Example Usage**: Provide code examples for each component
3. **Architecture Overview**: Explain how components interact
4. **Troubleshooting Guide**: Common issues and solutions

## 🔄 Iterative Development

This task list is designed for iterative development:
1. Implement tasks in sequence by priority
2. Test each task independently
3. Integrate with existing components
4. Validate against PLAN_CORE_BENCHMARK.md
5. Update task list based on learnings

---

This task list provides a modular, step-by-step approach to implementing the comprehensive benchmark requirements from PLAN_CORE_BENCHMARK.md, with a focus on the two-stage protocol approach outlined in PLAN_PROTOCOLS.md. Each task is designed to be testable independently while contributing to the overall benchmark infrastructure.