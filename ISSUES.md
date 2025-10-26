# Issues

## Open Issues

### #2 GLM SPL Transfer ATA Resolution Issue - Medium
**Date**: 2025-10-26  
**Status**: In Progress  
**Priority**: Medium  

**Issue**: GLM models (glm-4.6-coding) through reev-agent are generating wrong recipient ATAs for SPL transfers. Instead of using pre-created ATAs from benchmark setup, the LLM generates new ATAs or uses incorrect ATA names.

**Symptoms**:
- `002-spl-transfer` score: 56.2% with "invalid account data for instruction" error
- LLM generates transaction with wrong recipient ATA: "8RXifzZ34i3E7qTcvYFaUvCRaswcJBDBXrPGgrwPZxTo" instead of expected "BmCGQJCPZHrAzbLCjHd1JBQAxF24jrReU3fPwN6ri6a7"
- Local agent works perfectly (100% score)

**Root Cause**:
- LLM should use placeholder name `"RECIPIENT_USDC_ATA"` in tool calls, but is generating new recipient ATA.
- Context confusion from RESOLVED ADDRESSES section (already fixed but still affecting GLM behavior)
- Possible misinterpretation of recipient parameters vs ATA placeholders

**Fixes Applied**:
- ✅ **UNIFIED GLM LOGIC IMPLEMENTED**: Created `UnifiedGLMAgent` with shared context and wallet handling
- ✅ **IDENTICAL CONTEXT**: Both `OpenAIAgent` and `ZAIAgent` now use same context building logic
- ✅ **SHARED COMPONENTS**: Wallet info creation and prompt mapping are now identical
- 🔄 **PROVIDER-SPECIFIC WRAPPER**: Only request/response handling differs between implementations
- Fixed context serialization to use numbers instead of strings
- Enhanced tool description to be more explicit about reading exact balances

**Next Steps**: 
- Test unified GLM logic with updated code
- Verify SPL transfer tool prioritizes pre-created ATAs from key_map
- Check if LLM correctly uses placeholder names in recipient_pubkey field

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