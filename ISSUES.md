# Issues

## 🎯 Current Status - All Critical Issues Resolved

### ✅ **API Architecture Verification Complete**
- **Issue #30**: Frontend API Calls Analysis - **RESOLVED** ✅
- **Issue #31**: Status/Trace Endpoints CLI Dependencies - **RESOLVED** ✅
- **Issue #29**: API Architecture Fix - Remove CLI Dependency - **RESOLVED** ✅

### 🏆 **Architecture Achievements**
- **Zero CLI conflicts** during frontend load and API discovery
- **Database-only operations** for all status, trace, and sync endpoints
- **CLI usage isolated** to intentional benchmark execution only
- **Fast response times** with direct database queries
- **Server stability** - no crashes or cargo conflicts

### 📊 **Verified Endpoints**
**Auto-called on App Load (All Safe):**
- ✅ `/api/v1/health` - Health check
- ✅ `/api/v1/benchmarks` - Database discovery
- ✅ `/api/v1/agent-performance` - Database queries

**Status/Trace Operations (All DB-only):**
- ✅ `/api/v1/benchmarks/{id}/status/{execution_id}` - DB read
- ✅ `/api/v1/benchmarks/{id}/status` - DB read
- ✅ `/api/v1/flows/{session_id}` - DB read + file fallback
- ✅ `/api/v1/execution-logs/{benchmark_id}` - DB read
- ✅ `/api/v1/flow-logs/{benchmark_id}` - DB read
- ✅ `/api/v1/transaction-logs/{benchmark_id}` - DB read

**Sync Operations (File System + DB):**
- ✅ `/api/v1/sync` - File system scan + DB upsert (no CLI)
- ✅ `/api/v1/upsert-yml` - Database operations

**Execution Operations (CLI Intended):**
- ⚠️ `/api/v1/benchmarks/{id}/run` - **CLI/Runner** (intentional execution)

### 🔧 **Key Implementation**
- **CLI-based Runner**: Process isolation for benchmark execution
- **Database Discovery**: Fast, conflict-free benchmark listing
- **State Management**: Cross-process execution tracking via database
- **Error Handling**: Robust timeout and failure recovery

### 🚨 **Current Issue - #32** 
- **Title**: Database connection locks + Session file feedback loop missing
- **Description**: 
  1. **Database Lock Contention**: API and runner both trying to access same database causing `Query execution failed` errors
  2. **Missing Session Feedback**: API server never reads session files created by runner to update execution state
- **Root Causes**: 
  - Dual database access patterns causing SQLite lock conflicts
  - BenchmarkExecutor not reading `logs/sessions/session_{id}.json` after CLI completion
  - Missing `--no-db` flag to prevent runner database conflicts
- **Impact**: **Critical** - benchmark executions fail immediately or show "Queued" forever
- **Status**: **IN PROGRESS** - implementing runner database bypass + session file reading

### 📝 **Architecture Flow Issues**
```
🔴 BROKEN STATE:
Frontend → API Server → Database ←[LOCK CONFLICT]→ Runner 
            ↓                  ↑ Missing feedback loop
CLI/Runner → Session Files → [NEVER READ] → API memory

✨ TARGET STATE: 
Frontend → API Server → Database (discovery, status, traces)
            ↓                  ✅ Feedback loop
CLI/Runner (--no-db) → Session Files → API reads → Database storage
```

**Problems**: 
1. Database lock conflicts between API and runner processes
2. Session files created but never read by API
3. Execution state never updates from "Queued" status

### 🎯 **Solution Required**
**Two-Phase Fix:**

**Phase 1: Prevent Database Conflicts**
1. Add `--no-db` flag to reev-runner to skip database operations
2. Ensure runner only writes session files to `logs/sessions/`
3. API handles all database operations exclusively

**Phase 2: Complete Feedback Loop** 
1. Add session file reading to `BenchmarkExecutor.execute_cli_benchmark()` after CLI completion
2. Poll for `logs/sessions/session_{execution_id}.json` with retry logic
3. Parse session JSON to extract `final_result.success` and `final_result.score`
4. Update in-memory `execution_state` with actual results
5. Store final state in database via API (no runner DB conflicts)

### 🔧 **Implementation Progress**
- [✅] Identified database lock contention as root cause
- [✅] Removed database dependency from BenchmarkExecutor
- [✅] Fixed database column indices in `row_to_execution_state()`
- [🔄] Added `--no-db` flag to reev-runner CLI
- [🔄] Implemented session file reading in `BenchmarkExecutor.read_session_file_results()`
- [ ] Re-enable database storage in API handlers (after session reading works)
- [ ] Test end-to-end execution with `--no-db` flag
- [ ] Verify session files are read and parsed correctly
- [ ] Confirm final state stored in database without conflicts

### 📊 **Technical Details**
- **Database Lock**: SQLite WAL files causing `database is locked (5)` errors
- **Session Location**: `logs/sessions/session_{execution_id}.json` (project root)
- **Key Fields**: `final_result.success`, `final_result.score`, `execution_id`
- **Runner Command**: `cargo run -p reev-runner -- --no-db benchmarks/{file}.yml --agent={type}`
- **Retry Logic**: 10 attempts with 100ms delay for session file creation
- **File I/O**: Async tokio::fs operations for non-blocking file access