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

## Issue #11: Flow benchmark hard failures causing frontend to hang (2025-10-31)

**Description:** Flow benchmarks experiencing hard failures that stop entire execution process instead of graceful degradation, causing frontend to hang indefinitely.

**Current Impact:**
- **Benchmark 116-jup-lend-redeem-usdc**: Fails on Step 2 with "Number is not a valid u64" error when -1 passed as shares parameter
- **Benchmark 200-jup-swap-then-lend-deposit**: Fails on Step 2 with "Insufficient funds: requested 394358118, available 0" error
- **Frontend Stuck**: Both benchmarks get stuck in "Failed" state, preventing further execution
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

**Root Cause Problems:**
1. **Hard Failures**: Errors propagate up and stop entire execution instead of being caught and logged
2. **Poor Error Recovery**: No fallback mechanisms for common failure scenarios  
3. **Missing Error Session Fields**: Errors not properly logged to session files for debugging
4. **Frontend State**: Frontend gets stuck waiting for completion that never comes
5. **Balance Query Failures**: Token balance queries return invalid values (-1) instead of proper error handling

**Files Affected:**
- `crates/reev-tools/src/tools/jupiter_lend_earn_mint_redeem.rs` - Balance query error handling
- `crates/reev-runner/src/lib.rs` - Flow benchmark error recovery  
- `crates/reev-lib/src/llm_agent.rs` - Agent error propagation
- `crates/reev-protocols/src/jupiter/mod.rs` - Token balance query validation
- Frontend state management for failed flows

**Priority:** Critical - Blocks flow benchmark execution and hangs frontend

**Status:** 🔴 **ACTIVE** - Requires immediate fix

**Proposed Solution Plan:**

**Phase 1: Soft Error Handling**
1. **Catch Flow Step Failures**: Modify `run_flow_benchmark()` to catch individual step failures without stopping entire flow
2. **Graceful Degradation**: Log error and continue with next benchmark instead of hard failure
3. **Session Error Logging**: Add error field to session logs for failed steps
4. **Frontend Recovery**: Update execution state management to handle soft failures

**Phase 2: Balance Query Fixes**  
1. **Validate Balance Responses**: Fix `query_token_balance()` to return 0 instead of -1 on errors
2. **Better Error Messages**: Provide meaningful error context for insufficient funds
3. **Pre-Validation**: Check balances before attempting operations that require them

**Phase 3: Enhanced Error Recovery**
1. **Retry Logic**: Add configurable retry for transient failures
2. **Fallback Mechanisms**: Alternative approaches when primary operation fails  
3. **Error Classification**: Distinguish between recoverable vs non-recoverable errors

**Implementation Details:**
- Modify flow step execution to catch and log errors without propagating
- Update session logging schema to include error details
- Add balance validation before Jupiter operations
- Improve frontend state transitions for failed benchmarks
- Add comprehensive error logging to aid debugging

**Success Criteria:**
- Flow benchmarks continue execution even when individual steps fail
- Frontend receives proper error states and can continue
- Session logs contain detailed error information for debugging
- Balance queries handle edge cases properly
- Users can see what failed and why in the UI

**Testing Strategy:**
1. Test with various balance scenarios (0, insufficient, adequate)
2. Verify flow step failures don't stop entire benchmark suite
3. Confirm frontend updates properly for failed states
4. Validate error information appears in session logs
5. End-to-end testing with problematic benchmarks 116 and 200
