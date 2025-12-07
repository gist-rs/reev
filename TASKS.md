# Reev Project Tasks

This directory contains organized task files for the Reev project. Each task file focuses on a specific area of development.

## Available Task Files

1. **[BENCHMARK_IMPLEMENTATION.md](./TASKS/BENCHMARK_IMPLEMENTATION.md)**
   - Implementation plan for enhancing the benchmark system
   - Integration of legacy and new benchmark systems
   - Advanced scoring and performance metrics
   - Error recovery testing
   - Visualization and reporting

## Current State Analysis: Benchmark Systems

The project currently has two separate benchmark systems that need to be unified:

### 1. Legacy System (crates/reev-runner)
- Uses "mock perfect" approach for benchmark testing
- Located in `tests/benchmarks_test.rs`
- Focuses on validating that benchmarks are solvable
- Limited scoring (binary pass/fail with 1.0 score)
- Has basic flow execution in `run_flow_benchmark` function

### 2. Modern System (crates/reev-core/src/benchmark/)
- `StaticBenchmarkRunner` - Executes predefined YML flows
- `DynamicBenchmarkRunner` - Generates flows from prompts
- `BenchmarkScorer` - Evaluates execution results
- Sophisticated scoring system with support for complex evaluation criteria
- Ready to implement evaluation criteria from PLAN_CORE_BENCHMARK.md

### 3. Test Framework (crates/reev-core/tests/common/framework/mod.rs)
- `TestRunner` provides standardized setup and environment management
- Currently only used for tests, could be moved to a real runner
- Contains valuable code that shouldn't be limited to tests

### Key Insight
The `StaticBenchmarkRunner` and `DynamicBenchmarkRunner` in `crates/reev-core/src/benchmark/` represent the modern implementation we should be using. The legacy system in `crates/reev-runner` needs to be updated to use this newer approach.

## Recommended Approach: Integration Plan

1. **Move test framework** from `tests/common/framework/mod.rs` to `crates/reev-core/src/benchmark/runner/` as a reusable component
2. **Create unified interface** that supports both static and dynamic flows
3. **Integrate advanced scoring** from the modern system with the legacy runner
4. **Maintain backward compatibility** during transition

## Related Documents

- [ISSUES.md](./ISSUES.md) - Current issues and their status
- [PLAN_CORE_V3.md](./PLAN_CORE_V3.md) - Core architecture plan V3
- [PLAN_CORE_BENCHMARK.md](./PLAN_CORE_BENCHMARK.md) - Benchmark-specific requirements

## Task Management Guidelines

1. Each task should be linked to one or more issues in ISSUES.md
2. Tasks should be broken down into manageable phases
3. Each phase should have clear dependencies and effort estimates
4. Progress should be tracked by updating the relevant issue in ISSUES.md

## Adding New Task Files

When creating a new task file:

1. Create the file in the `TASKS/` directory
2. Add a link to this index file
3. Reference related issues from ISSUES.md
4. Include clear phases and implementation details
5. Link to related plan documents when applicable

## Implementation Status

Based on the analysis in BENCHMARK_IMPLEMENTATION.md:

### Phase 1: Basic Infrastructure (10 days)
- Create unified benchmark runner (Task 1.1)
- Implement advanced scoring criteria (Task 1.2)
- Enhance performance metrics (Task 1.3)

### Phase 2: Error Recovery (7 days)
- Implement error recovery scenarios (Task 2.1)
- Integrate error recovery with runner (Task 2.2)

### Phase 3: Integration (8 days)
- Refactor reev-runner to use new system (Task 3.1)
- Add benchmark categories (Task 3.2)

### Phase 4: Reporting (7 days)
- Enhanced benchmark reports (Task 4.1)
- Benchmark visualization (Task 4.2)

### Total Estimated Timeline: 32 days (approx. 6 weeks)