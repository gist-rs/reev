# Reev Project Issues

## Issue #325: Duplicate Benchmark Systems
- **Description**: Project currently has two separate benchmark systems - a legacy system in `crates/reev-runner` and a newer system in `crates/reev-core/src/benchmark/`
- **Impact**: Code duplication, maintenance overhead, inconsistent scoring approaches
- **Status**: Open
- **Priority**: High
- **Solution**: Integrate systems by creating unified benchmark runner (see TASKS.md)

## Issue #326: Test Framework Location
- **Description**: Valuable test framework in `crates/reev-core/tests/common/framework/mod.rs` is only available to tests
- **Impact**: Code reuse limited, benchmark runner missing key functionality
- **Status**: Open
- **Priority**: High
- **Solution**: Move to `crates/reev-core/src/benchmark/runner/` as reusable component

## Issue #327: Limited Benchmark Scoring
- **Description**: Legacy system in reev-runner only supports binary pass/fail scoring (1.0)
- **Impact**: Cannot evaluate nuanced performance metrics required by PLAN_CORE_BENCHMARK.md
- **Status**: Open
- **Priority**: Medium
- **Solution**: Implement advanced scoring criteria from new benchmark system

## Issue #328: Missing Error Recovery Testing
- **Description**: No benchmark scenarios for testing error recovery mechanisms
- **Impact**: Cannot validate error recovery strategies
- **Status**: Open
- **Priority**: Medium
- **Solution**: Create error recovery benchmark scenarios (Phase 2 of TASKS.md)

## Issue #329: Incomplete Performance Metrics
- **Description**: Current performance tracking doesn't match PLAN_CORE_BENCHMARK.md requirements
- **Impact**: Cannot measure performance against specification
- **Status**: Open
- **Priority**: Medium
- **Solution**: Implement comprehensive performance metrics (Task 1.3)

## Issue #330: No Visualization for Results
- **Description**: Benchmark results lack visualization capabilities
- **Impact**: Difficult to interpret results and identify patterns
- **Status**: Open
- **Priority**: Low
- **Solution**: Implement visualization for benchmark results (Phase 4 of TASKS.md)

## Issue #331: Deterministic Agent Location
- **Description**: Deterministic agents in `crates/reev-agent/src/agents/coding/` could be better utilized in the benchmark system
- **Impact**: Valuable reference implementations are not integrated with benchmark framework
- **Status**: Open
- **Priority**: High
- **Solution**: Move to `crates/reev-core/src/benchmark/deterministic_agents/` and integrate with benchmark runner (Task 1.4 of TASKS.md)