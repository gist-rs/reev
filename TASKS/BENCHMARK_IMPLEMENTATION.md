# Reev Project Tasks

## Benchmark Implementation Plan

This document outlines the implementation plan for enhancing the benchmark system based on `PLAN_CORE_BENCHMARK.md` and integrating it with `crates/reev-runner`. The goal is to create a comprehensive benchmarking framework that can evaluate both static and dynamic flows with sophisticated scoring mechanisms.

For current issues related to this implementation, see [ISSUES.md](./ISSUES.md).

## Current State Analysis

### What We Have
1. **Core Benchmark Module** (`crates/reev-core/src/benchmark/`):
   - `StaticBenchmarkRunner` - Executes predefined YML flows
   - `DynamicBenchmarkRunner` - Generates flows from prompts
   - `BenchmarkScorer` - Evaluates execution results
   - Basic scoring infrastructure

2. **Legacy Runner** (`crates/reev-runner`):
   - Uses a "mock perfect" approach for benchmark testing
   - Located in `tests/benchmarks_test.rs`
   - Focuses on validating that benchmarks are solvable
   - Limited to basic scoring (pass/fail with 1.0)

3. **Test Framework** (`crates/reev-core/tests/common/framework/mod.rs`):
   - Standardized test runner for e2e tests
   - Environment setup and management
   - Could be moved out of tests for reuse as a real runner

### What We Need
1. **Enhanced Scoring**: Implement the sophisticated evaluation criteria from PLAN_CORE_BENCHMARK.md
2. **Performance Metrics**: Add comprehensive performance tracking
3. **Error Recovery Testing**: Implement benchmarks for error recovery scenarios
4. **Integration**: Unify the benchmark runners into a cohesive system

## Implementation Plan

### Phase 1: Enhance Basic Benchmark Infrastructure

#### Task 1.1: Create Unified Benchmark Runner
- **Description**: Move the test framework from `crates/reev-core/tests/common/framework/mod.rs` to `crates/reev-core/src/benchmark/runner/` as a reusable component
- **Priority**: High
- **Related Issues**: #325, #326
- **Dependencies**: None
- **Estimated Effort**: 3 days
- **Implementation Details**:
  - Refactor `TestRunner` to `BenchmarkRunner`
  - Add support for both static and dynamic benchmark execution
  - Implement environment management for benchmarking

#### Task 1.2: Implement Advanced Scoring Criteria
- **Description**: Extend `BenchmarkScorer` to support the evaluation criteria from PLAN_CORE_BENCHMARK.md
- **Priority**: High
- **Related Issues**: #327
- **Dependencies**: Task 1.1
- **Estimated Effort**: 5 days
- **Implementation Details**:
  - Implement multiplication strategy evaluation
  - Add yield optimization assessment
  - Implement flow complexity validation
  - Add success criteria validation
  - Implement expected data structure validation

#### Task 1.3: Enhance Performance Metrics
- **Description**: Extend performance metrics collection to match PLAN_CORE_BENCHMARK.md requirements
- **Priority**: Medium
- **Related Issues**: #329
- **Dependencies**: Task 1.2
- **Estimated Effort**: 2 days
- **Implementation Details**:
  - Add flow generation time tracking
  - Implement step execution time metrics
  - Add resource usage tracking (memory, CPU)
  - Implement recovery overhead metrics

### Phase 2: Error Recovery Benchmarking

#### Task 2.1: Implement Error Recovery Scenarios
- **Description**: Create benchmark scenarios for testing error recovery mechanisms
- **Priority**: Medium
- **Related Issues**: #328
- **Dependencies**: Phase 1 completion
- **Estimated Effort**: 4 days
- **Implementation Details**:
  - Create YML files for error recovery scenarios
  - Implement error injection mechanisms
  - Add error recovery validation
  - Create recovery expectation validation

#### Task 2.2: Integrate Error Recovery with Runner
- **Description**: Enhance the runner to execute error recovery benchmarks
- **Priority**: Medium
- **Related Issues**: #328
- **Dependencies**: Task 2.1
- **Estimated Effort**: 3 days
- **Implementation Details**:
  - Add error injection support to runner
  - Implement recovery strategy tracking
  - Add recovery success metrics
  - Integrate with existing scoring system

### Phase 3: Integration with reev-runner

#### Task 3.1: Refactor reev-runner to Use New Benchmark System
- **Description**: Update reev-runner to use the new unified benchmark system instead of the legacy approach
- **Priority**: High
- **Related Issues**: #325, #327
- **Dependencies**: Phase 2 completion
- **Estimated Effort**: 5 days
- **Implementation Details**:
  - Replace mock perfect approach with advanced scoring
  - Integrate unified benchmark runner
  - Update CLI to support enhanced benchmarking
  - Migrate existing test cases

#### Task 3.2: Add Benchmark Categories to reev-runner
- **Description**: Implement support for different benchmark categories in reev-runner
- **Priority**: Medium
- **Related Issues**: #325
- **Dependencies**: Task 3.1
- **Estimated Effort**: 3 days
- **Implementation Details**:
  - Add CLI options for benchmark categories
  - Implement category filtering
  - Add category-specific reporting
  - Integrate with PLAN_CORE_BENCHMARK.md categories

### Phase 4: Reporting and Visualization

#### Task 4.1: Enhanced Benchmark Reports
- **Description**: Implement comprehensive benchmark reporting as specified in PLAN_CORE_BENCHMARK.md
- **Priority**: Medium
- **Related Issues**: #327, #329
- **Dependencies**: Phase 3 completion
- **Estimated Effort**: 3 days
- **Implementation Details**:
  - Implement detailed category scoring
  - Add improvement suggestions generation
  - Create benchmark trend analysis
  - Implement export formats (JSON, CSV, HTML)

#### Task 4.2: Benchmark Visualization
- **Description**: Add visualization capabilities for benchmark results
- **Priority**: Low
- **Related Issues**: #330
- **Dependencies**: Task 4.1
- **Estimated Effort**: 4 days
- **Implementation Details**:
  - Create Mermaid diagram generation for flows
  - Implement performance charts
  - Add trend visualization
  - Create dashboard interface

## Implementation Approach

### 1. Incremental Development
- Implement each phase incrementally
- Ensure backward compatibility
- Maintain existing functionality while adding new features

### 2. Test-Driven Development
- Write tests for each component before implementation
- Use existing benchmarks as test cases
- Validate implementation against expected outcomes

### 3. Modular Design
- Keep components loosely coupled
- Design for extensibility
- Implement clear interfaces between components

### 4. Performance Considerations
- Minimize overhead during benchmark execution
- Optimize scoring algorithms
- Implement efficient data collection

## Success Criteria

1. **Functionality**: All benchmark types from PLAN_CORE_BENCHMARK.md are supported
2. **Accuracy**: Scoring accurately reflects the quality of execution
3. **Performance**: Benchmark execution overhead is minimal
4. **Usability**: Easy to run and interpret benchmarks
5. **Extensibility**: Easy to add new benchmark types and evaluation criteria

## Risk Mitigation

1. **Complexity**: Break down complex tasks into smaller, manageable parts
2. **Integration**: Implement integration tests early
3. **Performance**: Profile and optimize critical paths
4. **Compatibility**: Maintain backward compatibility during transition

## Timeline

- **Phase 1**: 10 days
- **Phase 2**: 7 days
- **Phase 3**: 8 days
- **Phase 4**: 7 days
- **Total**: 32 days (approx. 6 weeks)

## Next Steps

1. Start with Task 1.1: Create Unified Benchmark Runner
2. Set up a development branch for this work
3. Begin implementing the enhanced scoring system
4. Regularly test against existing benchmarks to ensure compatibility