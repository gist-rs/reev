# Issues

## Open Issues

### #1 AI Model Amount Request Issue - High
**Date**: 2025-06-17  
**Status**: Open  
**Priority**: High  

AI model was requesting 1,000,000,000,000 USDC (1 trillion) for deposit in benchmark `200-jup-swap-then-lend-deposit` step 2, despite only having 383,193,564 USDC available in context.

**Status**: Significant Improvement 🎉
- **Before**: Complete failure due to trillion USDC requests
- **After**: 75% score with custom program errors (0x1, 0xffff)
- **Issue**: No longer requesting insane amounts, now has execution errors

**Fixes Applied**:
- Fixed context serialization to use numbers instead of strings
- Enhanced tool description to be more explicit about reading exact balances
- Fixed dependency guard scope regression (see #4)

**Current Issues**:
- Custom program errors (0x1, 0xffff) suggest Jupiter tool integration edge cases
- 75% score is acceptable but could be improved

**Next Steps**: Investigate Jupiter tool execution errors for further score improvement.

---

### #4 Dependency Guard Scope Regression - Fixed
**Date**: 2025-10-26  
**Status**: Fixed  
**Priority**: Critical  

Processes (reev-agent, surfpool) were terminated before benchmark execution due to premature dependency guard dropping.

**Root Cause**: `_dependency_guard` scoped inside if/else blocks in `run_benchmarks()` function, causing `Drop` implementation to terminate managed processes before benchmark execution began.

**Fix**: Moved guard declaration outside if/else blocks to maintain proper lifecycle throughout function execution.

**Impact**: 
- Fixed critical regression affecting all benchmark execution
- Both fresh and shared surfpool modes now work correctly
- All key benchmarks now achieve good scores (75-100%)

---

## Closed Issues

### #2 Database Test Failure - Fixed
**Date**: 2025-06-20  
**Status**: Fixed  
**Priority**: Medium  

SQL query in `get_session_tool_calls` referencing non-existent `metadata` column in `session_tool_calls` table.

**Root Cause**: SQL query included `metadata` column that doesn't exist in database schema.

**Fix**: Removed `metadata` column from SELECT query in `crates/reev-db/src/writer/sessions.rs` line 527.

---

### #3 Flow Test Assertion Failure - Fixed  
**Date**: 2025-06-20  
**Status**: Fixed  
**Priority**: Low  

Test expecting `.json` extension but log files use `.jsonl` (JSON Lines format).

**Root Cause**: Test assertion mismatched with actual file extension used by EnhancedOtelLogger.

**Fix**: Updated test in `crates/reev-flow/src/enhanced_otel.rs` line 568 to expect `.jsonl` extension.

---