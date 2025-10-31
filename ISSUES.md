# Issues

## Issue #10: Flow benchmarks missing execution_sessions and ASCII tree rendering (2025-10-31)

**Description:** Flow benchmarks (116-jup-lend-redeem-usdc, 200-jup-swap-then-lend-deposit) are missing from execution_sessions table and don't show ASCII tree render, while regular benchmarks work correctly.

**Impact:** 
- Benchmarks 116 and 200 don't appear in `/api/v1/agent-performance` results
- No ASCII tree visualization for flow benchmarks
- Inconsistent data storage between flow and regular benchmarks

**Root Cause:** Flow benchmarks use different execution path (`run_flow_benchmark`) that:
- ✅ Creates `execution_states` records 
- ❌ Does NOT create `execution_sessions` records
- ❌ Does NOT render ASCII trees
While regular benchmarks use normal path that creates all three.

**Files Affected:**
- `crates/reev-runner/src/lib.rs` - `run_flow_benchmark` function missing session creation and tree rendering

**Priority:** High - Affects data consistency and user experience

**Status:** ✅ Resolved (2025-10-31)

**Root Cause Analysis:**
Flow benchmarks had multiple issues due to inconsistent session ID handling:
1. **Session ID Mismatch**: Flow benchmarks generated their own session IDs instead of using provided execution_id
2. **Database Storage**: Raw stdout logs (50KB+) stored instead of clean session data
3. **API Trace Generation**: Limited to 2-line summaries when session files couldn't be matched

**Complete Resolution Applied:**

**Runner Fixes** (`crates/reev-runner/src/lib.rs`):
1. **Session Logger Creation**: Added session logging to `run_flow_benchmark()` function
2. **Execution ID Consistency**: Fixed flow benchmarks to use provided `execution_id` instead of generating their own
   ```rust
   // Before: let session_id = uuid::Uuid::new_v4().to_string();
   // After:  let session_id = execution_id.as_deref().unwrap_or("unknown");
   ```

**API Fixes** (`crates/reev-api/src/handlers/execution_logs.rs`):
1. **Session File Fallback**: When database lookup fails, search for matching session files directly
2. **Clean Result Data**: Strip HUGE stdout from API responses, keep only essential metadata
3. **Enhanced Trace Generation**: Generate full ASCII trees from session file data

**Comprehensive Test Results:**
- ✅ **New Flow Executions**: Full ASCII tree with step-by-step transaction details
- ✅ **Session File Consistency**: Session files now use correct execution_id matching database records  
- ✅ **Clean API Responses**: Result field only `{"duration_ms": 0, "exit_code": 0}` instead of 50KB logs
- ✅ **Backward Compatibility**: Existing executions with session files work via fallback
- ✅ **All Benchmarks**: Both flow (116, 200) and regular benchmarks now work consistently

**Final Verification:**
```bash
# Before: 2-line trace with HUGE stdout
curl ".../execution-logs/116-jup-lend-redeem-usdc?execution_id=old"  
# After: Full ASCII tree with clean result
curl ".../execution-logs/116-jup-lend-redeem-usdc?execution_id=new"
```

**Issue Status:** 🎯 **COMPLETELY RESOLVED** - All flow benchmark issues fixed with comprehensive solution
