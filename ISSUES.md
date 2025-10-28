# Issues

## 🚧 #21: API Decoupling - CLI-Based Runner Communication
**Status**: In Progress - Phase 1 Complete, Phase 2 Starting  
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
Transform to CLI-based communication with JSON-RPC protocol:
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
- Zero compilation warnings, all modules <320 lines

### Phase 2 🚧 IN PROGRESS
- Implement `RunnerProcessManager` for CLI execution
- Add JSON-RPC communication via stdin/stdout
- Create execution state database tables
- Implement timeout and error handling

### Remaining Work
- Migrate API endpoints progressively (read-only first)
- Add comprehensive CLI testing framework
- Update CURL.md with new test commands
- Remove direct dependencies
- Performance validation and optimization

### Impact
- ✅ Enable hot-swapping runner implementation
- ✅ Reduce binary size and compilation time
- ✅ Improve modularity and testability
- ✅ Enable independent scaling of components

---
