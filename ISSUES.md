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

**Next Steps**: Test with updated code, may require prompt engineering if issue persists.

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