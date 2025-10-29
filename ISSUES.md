# Issues

## 🚧 #21: API Decoupling - CLI-Based Runner Communication
**Status**: Phase 1-2 Complete, Phase 3 In Progress  
**Priority**: High - Architecture improvement  
**Target**: Eliminate direct dependencies from reev-api to reev-runner/flow/tools

### Problem
reev-api currently builds and imports reev-runner directly, creating tight coupling:
```toml
reev-runner = { path = "../reev-runner" }           # ❌ Remove
reev-flow = { path = "../reev-flow", features = ["database"] }  # ❌ Remove  
reev-tools = { path = "../reev-tools" }            # ❌ Remove
```

### Solution
Transform to CLI-based communication with JSON-RPC protocol through reev-db state management:
```
reev-api (web server)
    ↓ (CLI calls, JSON-RPC)
reev-runner (standalone process)
    ↓ (state communication)
reev-db (shared state)
```

### Phase 1 ✅ COMPLETED
- Created `reev-types` crate for shared type definitions
- Implemented JSON-RPC 2.0 protocol structures
- Added execution state management types
- Created CLI command/response types
- Added timeout and error handling
- Zero compilation warnings, all modules <320 lines

### Phase 2 ✅ COMPLETED
- Implemented `RunnerProcessManager` for CLI execution
- Added execution state database tables
- Implemented hybrid `BenchmarkExecutor` with fallback mechanism
- Added feature flag system (`cli_runner` default, `direct_runner` optional)
- Created simplified CLI-based benchmark executor
- Preserved backward compatibility during migration
- Integrated execution state management via database
- Fixed all compilation errors and trait compatibility issues
- Created generic BenchmarkExecutor supporting both DatabaseWriter and PooledDatabaseWriter
- Implemented DatabaseWriterTrait for both connection types

### Phase 3 🚧 IN PROGRESS
- [x] Remove direct dependencies from reev-api Cargo.toml (imports still exist but not used)
- [x] Update handlers to use new BenchmarkExecutor (PooledBenchmarkExecutor implemented)
- [x] Fixed all compilation errors and trait compatibility issues
- [x] Created generic BenchmarkExecutor supporting both DatabaseWriter and PooledDatabaseWriter
- [x] Implemented DatabaseWriterTrait for both connection types
- [ ] Add comprehensive testing framework
- [ ] Update CURL.md with new CLI test commands
- [ ] Performance validation and optimization
- [ ] Implement real CLI execution in BenchmarkExecutor (currently placeholder)

### Impact
- ✅ Enable hot-swapping runner implementation
- ✅ Reduce binary size and compilation time
- ✅ Improve modularity and testability
- ✅ Enable independent scaling of components

---

## 🚧 #22: CLI Implementation in BenchmarkExecutor
**Status**: Ready to Start  
**Priority**: High - Complete Phase 3 of API decoupling  
**Target**: Implement actual CLI-based benchmark execution

### Problem
Current `BenchmarkExecutor.execute_benchmark()` uses placeholder implementation instead of real CLI calls:
```rust
// Placeholder code - needs CLI integration
execution_state.update_status(ExecutionStatus::Completed);
execution_state.complete(serde_json::json!({
    "message": "Benchmark execution placeholder - CLI integration next"
}));
```

### Solution
Implement real CLI execution using `RunnerProcessManager` and JSON-RPC protocol:

```rust
// Replace placeholder with:
let runner_manager = RunnerProcessManager::new(config, db, timeout);
let execution_id = runner_manager.execute_benchmark(params).await?;
```

### Tasks
- [ ] Implement real CLI execution in `BenchmarkExecutor.execute_benchmark()`
- [ ] Connect benchmark execution handlers to use CLI path
- [ ] Add proper error handling and timeout management
- [ ] Test with actual benchmark files
- [ ] Add CLI execution metrics and monitoring

### Dependencies
- `RunnerProcessManager` ✅ Implemented
- `DatabaseWriterTrait` ✅ Implemented  
- JSON-RPC protocol structures ✅ Implemented
- Execution state management ✅ Implemented

### Success Criteria
- API can execute benchmarks via CLI
- Execution states are properly tracked
- Error handling and timeouts work correctly
- Performance is comparable to direct library calls (within 20%)

---
