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

**Status:** 🔄 Open
