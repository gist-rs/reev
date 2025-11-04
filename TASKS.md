# TASKS.md - 300-Series Dynamic Flow Benchmark Implementation

## 🎯 **Mission Statement**

Create comprehensive 300-series benchmarks to demonstrate reev's dynamic flow capabilities through realistic DeFi scenarios that showcase natural language processing, intelligent decision-making, and multi-step orchestration with proper tool call validation via OpenTelemetry.

## 📋 **Implementation Tasks**

### Phase 1: Foundation - Benchmark 300 ✅ COMPLETED
**Task 1.1: Create Multiplication Strategy Benchmark** ✅ COMPLETED
- [x] Create `300-swap-sol-then-mul-usdc.yml`
- [x] Define multiplication scenario: 50% SOL → 1.5x USDC increase
- [x] Set initial state: 4 SOL, 20 USDC existing
- [x] Craft natural language prompt: "use my 50% sol to multiply usdc 1.5x on jup"
- [x] Design expected tool calls (not API calls):
  - `account_balance` - Wallet discovery
  - `jupiter_swap` - 50% SOL → USDC conversion
  - `jupiter_lend` - USDC yield generation
  - `jupiter_positions` - Final validation
- [x] Define success criteria and OTEL tracking
- [x] Implement proper agent encapsulation

**Task 1.2: Correct Design Philosophy** ✅ COMPLETED
- [x] Fix fundamental flaw: `expected_api_calls` → `expected_tool_calls`
- [x] Update documentation to reflect agent-only tool knowledge
- [x] Implement OpenTelemetry tracking expectations
- [x] Create proper encapsulation pattern for benchmarks

### Phase 2: Design Framework - 301-305 Series 🟡 IN PROGRESS

#### Task 2.1: Fix Existing Benchmarks 🟡 PARTIAL
- [x] Fix `301-dynamic-yield-optimization.yml` tool calls
- [ ] Fix `302-portfolio-rebalancing.yml` API call patterns
- [ ] Fix `303-risk-adjusted-growth.yml` market analysis expectations
- [ ] Fix `304-emergency-exit-strategy.yml` emergency response patterns
- [ ] Fix `305-yield-farming-optimization.yml` multi-pool expectations

**Task 2.2: Create Consistent Patterns** 🟡 NOT STARTED
- [ ] Standardize tool call validation across all benchmarks
- [ ] Create progressive complexity ladder (simple → expert)
- [ ] Implement proper OTEL tracking specifications
- [ ] Design comprehensive success criteria for each scenario

#### Task 2.3: Complete Series Implementation 🟡 NOT STARTED
- [ ] **301**: Dynamic yield optimization (simple complexity)
  - Natural language: "Use my 50% SOL to maximize returns"
  - Tools: `account_balance` → `jupiter_swap` → `jupiter_lend` → `jupiter_positions`
  - Focus: Single optimization goal with clear percentage

- [ ] **302**: Portfolio rebalancing (medium complexity)
  - Natural language: "Rebalance portfolio based on market conditions"
  - Tools: `account_balance` → `jupiter_positions` → analysis → `jupiter_swap` → `jupiter_lend`
  - Focus: Multi-variable analysis and optimal allocation

- [ ] **303**: Risk-adjusted growth (medium-high complexity)
  - Natural language: "Implement conservative growth using 30% of SOL"
  - Tools: `account_balance` → `jupiter_lend_rates` → `jupiter_swap` → `jupiter_lend`
  - Focus: Capital preservation with controlled growth

- [ ] **304**: Emergency exit strategy (high complexity)
  - Natural language: "Emergency exit due to market stress"
  - Tools: `account_balance` → `jupiter_positions` → `jupiter_withdraw` → `jupiter_swap` → stable assets
  - Focus: Crisis response with rapid liquidation

- [ ] **305**: Yield farming optimization (expert complexity)
  - Natural language: "Optimize yield farming using 70% capital"
  - Tools: `account_balance` → `jupiter_pools` → `jupiter_lend_rates` → `jupiter_swap` → `jupiter_lend` (multi-pool)
  - Focus: Advanced multi-pool optimization and auto-compounding

### Phase 3: Test Implementation 🟡 NOT STARTED

#### Task 3.1: Unit Test Suite 🟡 NOT STARTED
- [ ] Create `tests/dynamic_flow_300_series_test.rs`
- [ ] Test percentage calculation accuracy for each benchmark
- [ ] Test tool call sequence validation
- [ ] Test parameter passing correctness
- [ ] Test final state achievement validation

**Task 3.2: Integration Test Suite 🟡 NOT STARTED
- [ ] Create end-to-end execution tests
- [ ] Test OpenTelemetry tracking for all tool calls
- [ ] Test flow diagram generation from tool sequences
- [ ] Test API integration with dynamic flow endpoints

**Task 3.3: Performance Test Suite 🟡 NOT STARTED
- [ ] Benchmark flow generation time (<200ms target)
- [ ] Test tool call execution overhead (<50ms per call)
- [ ] Validate memory usage (<2KB per flow)
- [ ] Test caching effectiveness (>80% hit rate)

### Phase 4: Validation & Documentation 🟡 NOT STARTED

#### Task 4.1: Success Criteria Validation 🟡 NOT STARTED
- [ ] Validate natural language parsing accuracy (>95%)
- [ ] Validate tool sequence logic (>90% correct)
- [ ] Validate parameter accuracy (>90% correct)
- [ ] Validate final state achievement (>85% success)

**Task 4.2: Documentation Completion 🟡 NOT STARTED
- [ ] Update `DYNAMIC_BENCHMARK_DESIGN.md` with 300-series patterns
- [ ] Create API usage examples for each benchmark
- [ ] Document OpenTelemetry integration patterns
- [ ] Create troubleshooting guides for common failures

**Task 4.3: Production Readiness 🟡 NOT STARTED
- [ ] Ensure all benchmarks compile without warnings
- [ ] Verify test suite passes (100% coverage)
- [ ] Validate API integration success
- [ ] Confirm performance targets met
- [ ] Update `ARCHITECTURE.md` with 300-series capabilities

## 🏗️ **Technical Requirements**

### Benchmark Design Standards
```yaml
# ✅ CORRECT PATTERN
expected_tool_calls:
  - tool_name: "jupiter_swap"
    description: "Swap SOL to USDC using Jupiter"
    critical: true
    expected_params: ["input_token", "output_token", "amount"]
    weight: 0.4

# ❌ INCORRECT PATTERN (removed)
expected_api_calls:
  - service: "jupiter_prices"
    method: "GET"
    endpoint_pattern: "quote-api.jup.ag/v6/quote"
    critical: true
```

### OpenTelemetry Integration Requirements
```rust
// Expected tracking for all tool calls
expected_otel_tracking:
  - type: "tool_call_logging"
    description: "OpenTelemetry should track all tool calls"
    required_tools: ["account_balance", "jupiter_swap", "jupiter_lend", "jupiter_positions"]
    weight: 0.3

  - type: "parameter_validation"
    description: "Tool parameters should be validated via OTEL"
    required_params: ["amount", "mint", "wallet_pubkey"]
    weight: 0.3

  - type: "execution_timing"
    description: "Each tool call should have execution time tracking"
    required_metrics: ["tool_call_duration_ms", "total_flow_time_ms"]
    weight: 0.2
```

### Progressive Complexity Matrix
| Benchmark | Natural Language | Tool Sequence | Decision Complexity | Recovery Needed |
|-----------|-------------------|---------------|-------------------|-----------------|
| **300** | "use my 50% sol to multiply usdc 1.5x" | 4 tools | Simple | Basic |
| **301** | "Use my 50% SOL to maximize returns" | 4 tools | Simple | Basic |
| **302** | "Rebalance portfolio based on market conditions" | 5 tools | Medium | Basic |
| **303** | "Conservative growth using 30% of SOL" | 5 tools | Medium-High | Intermediate |
| **304** | "Emergency exit due to market stress" | 6 tools | High | Advanced |
| **305** | "Optimize yield farming using 70% capital" | 7 tools | Expert | Advanced |

## 📊 **Success Metrics**

### Quantitative Targets
- **Benchmark Completion Rate**: 100% (all 6 benchmarks)
- **Test Coverage**: >90% of scenarios
- **Natural Language Success**: >95% prompt parsing
- **Tool Call Accuracy**: >90% correct sequences
- **Parameter Accuracy**: >90% correct values
- **Performance Targets**: <200ms flow generation, <5s total execution
- **OTEL Coverage**: 100% tool call tracking

### Qualitative Targets
- **Progressive Complexity**: Clear difficulty progression from 300 to 305
- **Real-World Scenarios**: Practical DeFi use cases
- **Robust Error Handling**: Recovery mechanisms for failures
- **Comprehensive Documentation**: Complete implementation and usage guides
- **Production Readiness**: All benchmarks ready for CI/CD pipeline

## ⚠️ **Current Blockers**

### Design Philosophy Inconsistency
- **Issue**: Benchmarks 301-305 still use `expected_api_calls` pattern
- **Impact**: Agent doesn't know about APIs, only tools
- **Solution**: Update all benchmarks to use `expected_tool_calls` pattern
- **Priority**: HIGH - Must be fixed before testing

### Missing Test Infrastructure
- **Issue**: No dedicated test suite for 300-series benchmarks
- **Impact**: Cannot validate tool call sequences or OTEL tracking
- **Solution**: Create comprehensive test framework
- **Priority**: HIGH - Required for validation

### Documentation Gaps
- **Issue**: DYNAMIC_BENCHMARK_DESIGN.md mentions API calls
- **Impact**: Confusing guidance for developers
- **Solution**: Update documentation with tool call approach
- **Priority**: MEDIUM - Documentation consistency

## 🚀 **Implementation Roadmap**

### Week 1: Foundation & Fixes
- [ ] Fix benchmark design philosophy across 301-305 series
- [ ] Create test infrastructure for 300-series validation
- [ ] Update documentation with correct tool call patterns
- [ ] Validate benchmark 300 with comprehensive tests

### Week 2: Series Completion
- [ ] Complete benchmarks 301-303 implementation
- [ ] Implement advanced scenarios 304-305
- [ ] Add comprehensive test coverage for all scenarios
- [ ] Create progressive complexity validation

### Week 3: Integration & Validation
- [ ] Complete API integration testing
- [ ] Validate OpenTelemetry tracking for all benchmarks
- [ ] Performance optimization and benchmarking
- [ ] Production deployment readiness validation
- [ ] Complete documentation and examples

## 🎯 **Success Criteria**

### Phase Completion Gates
- **Phase 1 Complete**: Benchmark 300 working, design philosophy fixed
- **Phase 2 Complete**: All 301-305 benchmarks implemented and tested
- **Phase 3 Complete**: Full validation, documentation, production readiness
- **Phase 4 Complete**: CI/CD integration, performance monitoring

### Final Deliverables
- **6 Complete Benchmarks**: 300, 301, 302, 303, 304, 305
- **Comprehensive Test Suite**: Unit, integration, and performance tests
- **Updated Documentation**: Correct design patterns and usage guides
- **API Integration**: Full REST API support with dynamic flow detection
- **Production Ready**: Zero warnings, >90% test coverage, <50ms overhead

---

**Status**: 🟡 **IN PROGRESS** - Foundation complete, series implementation needed
**Priority**: 🟢 **HIGH** - Core capability demonstration
**Dependencies**: reev-orchestrator, OpenTelemetry, reev-tools, test framework