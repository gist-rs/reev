# Issues

## Issue #13: Local tab showing execution traces from other tests when status is "untested" (2025-11-01)

**Description:** When clicking local tab on web UI, items with "untested" status correctly appear but display "Execution Trace" content from other test executions instead of showing no data.

**Current Behavior:**
1. User clicks local tab on web
2. API call: `GET http://localhost:3001/api/v1/benchmarks/001-sol-transfer`
3. Response includes recent_executions with execution_id "b5b4ff25-c90c-4045-a19b-1ca674da608b"
4. Shows Execution Trace of that execution even though status is "untested" and agent_type doesn't match local tab

**Expected Behavior:**
- When benchmark status is "untested" OR agent_type doesn't match current tab filter
- Should show no execution trace data (empty state)
- Only show traces for matching agent_type and non-untested status

**API Response Analysis:**
```json
"recent_executions": [
    {
        "execution_id": "b5b4ff25-c90c-4045-a19b-1ca674da608b",
        "agent_type": "deterministic", 
        "status": "completed",
        "created_at": "2025-10-31T12:57:02+00:00",
        "score": 1.0
    }
]
```

**Issue:** Frontend showing trace for "deterministic" agent execution when viewing "local" tab (agent_type mismatch)

**Root Cause Identified:**
The issue is in the `getExecutionTraceWithLatestId()` function in `/web/src/hooks/useBenchmarkExecution.ts`. When a user clicks a benchmark with "untested" status:

1. **Function Logic**: Since `isRunning=false`, it skips running execution checks
2. **Fallback to Database**: Uses `getBenchmarkWithExecutions(benchmarkId)` API call  
3. **Missing Agent Filter**: Takes `latest_execution_id` regardless of agent_type match
4. **Wrong Trace Display**: Shows trace from "deterministic" agent when viewing "local" tab

**API Response Analysis:**
```json
{
  "recent_executions": [
    {"execution_id": "b5b4ff25...", "agent_type": "deterministic", "status": "completed"},
    {"execution_id": "38f514cf...", "agent_type": "glm-4.6-coding", "status": "completed"}
  ],
  "latest_execution_id": "b5b4ff25..."  // Always the most recent, not agent-specific
}
```

**Problem Code Location:**
`/web/src/hooks/useBenchmarkExecution.ts:220-230` - Uses `latest_execution_id` without filtering by `selectedAgent`

**Detailed Implementation Plan:**

**Step 1: Add Helper Function**
```typescript
// Helper to find matching execution by agent type and valid status
const findMatchingExecution = (
  executions: Array<{agent_type: string, status: string, execution_id: string}>,
  selectedAgent?: string
) => {
  if (!selectedAgent) return null;
  
  // Find most recent execution for selected agent with valid status
  const validStatuses = ["completed", "failed", "running", "queued"];
  return executions.find(exec => 
    exec.agent_type === selectedAgent && 
    validStatuses.includes(exec.status.toLowerCase())
  );
};
```

**Step 2: Modify Fallback Logic**
Replace lines 220-230 in `getExecutionTraceWithLatestId()`:
```typescript
// OLD CODE (problematic):
const latestExecutionId = benchmarkData.latest_execution_id;

// NEW CODE (fixed):
const matchingExecution = findMatchingExecution(
  benchmarkData.recent_executions, 
  selectedAgent
);
const latestExecutionId = matchingExecution?.execution_id || null;
```

**Step 3: Preserve Empty Result Structure**
Ensure consistent return format when no matching execution found.

**Test Scenarios:**
1. ✅ Local tab + untested status → Empty state (fixes main bug)
2. ✅ Local tab + completed local execution → Shows correct trace
3. ✅ Deterministic tab → Works exactly as before  
4. ✅ Running executions → Priority logic unchanged
5. ✅ Edge case: No selectedAgent → Falls back gracefully

**Priority:** Medium - Affects user experience but doesn't break core functionality

**Status:** ✅ **RESOLVED** - Implementation completed successfully

**Implementation Details:**
- Added `findMatchingExecution()` helper function to filter by agent_type and valid status
- Modified `getExecutionTraceWithLatestId()` to use agent-filtered executions instead of `latest_execution_id`
- Preserved empty result structure when no matching execution found for selected agent
- Maintained existing priority logic for running executions

**Code Changes Made:**
1. **Helper Function Added** (`/web/src/hooks/useBenchmarkExecution.ts:183-201`)
2. **Fallback Logic Updated** (`/web/src/hooks/useBenchmarkExecution.ts:254-286`)
3. **Enhanced Logging** for better debugging and tracing

**Test Results:**
- ✅ Build successful with no TypeScript errors
- ✅ Local tab + untested status → Empty state (main issue fixed)
- ✅ Local tab + completed local execution → Shows correct trace
- ✅ Deterministic tab → Works exactly as before
- ✅ Running executions → Priority logic unchanged

**Files Modified:**
- `reev/web/src/hooks/useBenchmarkExecution.ts` - Core fix implementation

**Implementation Priority:** ✅ **COMPLETED** - User experience issue resolved

**Verification Results:**
- ✅ **API Test**: No "local" executions exist for benchmark 001-sol-transfer 
- ✅ **Fix Verification**: Local tab now shows empty state instead of deterministic trace
- ✅ **Regression Test**: Deterministic tab still shows correct execution trace "b5b4ff25-c90c-4045-a19b-1ca674da608b"
- ✅ **Build Success**: TypeScript compilation completed without errors
- ✅ **Clippy Pass**: Rust code quality checks passed with only acceptable warnings

**Before Fix:**
- Local tab + untested benchmark → Shows deterministic execution trace (WRONG)

**After Fix:**
- Local tab + untested benchmark → Shows empty state (CORRECT)
- Local tab + completed execution → Shows correct local trace (CORRECT)
- Other agent tabs → Work exactly as before (CORRECT)

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

## Issue #12: Investigate Uuid::new_v4() collision causing execution_id conflicts (2025-10-31)

**Description:** Multiple executions being created with same execution_id causing database conflicts and frontend stuck in "Queued" state. Investigation needed into Uuid::new_v4() usage causing inconsistent ID generation.

**Current Impact:**
- **Execution ID Collisions**: Multiple executions created with same execution_id `a71c1214-c295-4aa6-abc9-3df47e0364cc` causing database conflicts
- **Frontend Stuck**: Benchmarks stuck in "Queued" state when duplicate execution_ids occur
- **Root Cause**: `Uuid::new_v4()` generating inconsistent IDs in different code paths
- **No Graceful Recovery**: Hard failures prevent continuation to remaining benchmarks

**Error Analysis:**

**Issue #1: Invalid Share Parameter (116)**
```json
{
  "tool": "jupiter_lend_earn_redeem",
  "arguments": {"asset":"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v","shares":-1,"signer":"BtsJz59TJDphedMqqPQPeECjJg8wKWypXKDBoGWjRDs7"}
}
```
- LLM agent passes -1 as shares parameter to JupiterLendEarnRedeemTool
- Query for actual jUSDC balance failing and returning -1 instead of error
- Should query token balance and return 0 if not found, not -1

**Issue #2: Insufficient Funds (200)**
```json
{
  "error": "Jupiter lend deposit error: Balance validation failed: Insufficient funds: requested 394358118, available 0"
}
``- Agent trying to deposit tokens it doesn't have after swap
- Missing balance validation before attempting deposit
- Should handle insufficient funds gracefully and provide helpful error

**Investigation Areas:**

**🔍 Primary Investigation Points:**

1. **UUID Generation Analysis**
   - Compare `Uuid::new_v4()` usage patterns across codebase
   - Identify where consistent vs inconsistent generation occurs
   - Check if timestamp or system state affects UUID generation
   - Verify collision probability in concurrent execution scenarios

2. **Execution ID Flow Tracing**
   - Map execution_id lifecycle from API request → CLI runner → Database storage
   - Identify where ID transformation/collision occurs
   - Check if temporary IDs vs final IDs cause confusion
   - Verify `execute_cli_command()` vs `execute_benchmark()` ID handling

3. **Database State Consistency**
   - Check if database operations use consistent ID formats
   - Verify if logging IDs differ from storage IDs
   - Identify race conditions in concurrent execution scenarios
   - Check if session file naming conflicts with database records

**🧪 Reproduction Steps:**
1. Set up concurrent benchmark execution scenario
2. Monitor execution_id generation across multiple API calls
3. Trace ID transformation through each system component
4. Identify exact point where collision occurs
5. Verify database conflict resolution behavior

**📊 Data Collection Needed:**
- UUID collision frequency and patterns
- Execution state transition timelines  
- Database operation success/failure rates by ID
- Frontend state change correlation with ID conflicts
- Session file creation vs database storage timing

**🎯 Expected Findings:**
- Root cause of UUID generation inconsistency
- Specific code path causing execution_id conflicts
- Impact assessment on frontend stability
- Recommended fix approach for ID management

**Implementation Details:**
- Modify flow step execution to catch and log errors without propagating
- Update session logging schema to include error details
- Add balance validation before Jupiter operations
- Improve frontend state transitions for failed benchmarks
- Add comprehensive error logging to aid debugging

**Status:** 🔍 **UNDER INVESTIGATION** - Analyzing UUID collision patterns

**Current Analysis Status:**
- ✅ **Initial Fix Applied**: Resolved execution_id collision in `execute_cli_command()`
- 📊 **Monitoring Active**: Watching for recurring collision patterns
- 🔧 **Temporary Solution**: Using consistent execution_id across API and CLI
- ⚠️ **Root Cause**: `Uuid::new_v4()` may have deeper consistency issues

**Next Investigation Steps:**
1. **Pattern Analysis**: Collect data on collision frequency and timing
2. **Code Review**: Systematic review of all UUID generation points
3. **Stress Testing**: Concurrent execution scenarios to reproduce collisions
4. **Root Cause Analysis**: Determine if issue is algorithmic or environmental
5. **Permanent Fix**: Implement robust ID generation strategy

**Investigation Priority: High** - Potential for systemic ID generation issues

**Testing Strategy:**
1. Test with various balance scenarios (0, insufficient, adequate)
2. Verify flow step failures don't stop entire benchmark suite
3. Confirm frontend updates properly for failed states
4. Validate error information appears in session logs
5. End-to-end testing with problematic benchmarks 116 and 200
