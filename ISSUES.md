# Issues

## Open Issues

### #2 Jupiter Lend Deposit Amount Parsing Issue - Medium
**Date**: 2025-10-26  
**Status**: Open  
**Priority**: Medium  

**Issue**: GLM-4.6 model passes `amount: 0` to Jupiter lend deposit tool instead of reading actual balance from context, even when correct balance is clearly visible in context.

**Symptoms**:
- Context shows `"USER_USDC_ATA": {"amount": 394358118, ...}` (correct)
- LLM tool call: `{"amount":0,"asset_mint":"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v","user_pubkey":"USER_WALLET_PUBKEY"}` (wrong)
- Error: `Jupiter lend deposit error: Invalid amount: Amount must be greater than 0`
- Step 1 (Jupiter swap) works perfectly, step 2 (lend deposit) fails

**Root Cause**:
- LLM can see correct balance in context but doesn't parse it correctly for amount parameter
- Possible confusion about amount field interpretation in context vs tool parameter
- Tool description may not be clear enough about using exact balance from context

**Required Fix**:
- Improve Jupiter lend deposit tool description to be more explicit about balance reading
- Add clearer instructions for reading amounts from context
- Possibly add balance validation hints in tool description

**Impact**: Affects all Jupiter lend deposit operations after successful swaps

---

### #3 GLM SPL Transfer ATA Resolution Issue - Medium
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

### #1 Jupiter Earn Tool Scope Issue - Fixed
**Date**: 2025-10-26  
**Status**: Fixed  
**Priority**: Critical  

**Issue**: `jupiter_earn` tool is incorrectly available to all benchmarks instead of only `114-jup-positions-and-earnings.yml`, causing API calls that bypass surfpool's forked mainnet state.

**Symptoms**:
- `200-jup-swap-then-lend-deposit.yml` shows "0 balance" errors from jupiter_earn calls
- Jupiter earn tool fetches data directly from live mainnet APIs, bypassing surfpool
- Data inconsistency between surfpool's forked state and Jupiter API responses

**Root Cause**:
- `jupiter_earn_tool` added unconditionally in OpenAIAgent normal mode
- Tool should only be available for position/earnings benchmarks (114-*.yml)
- Surfpool is a forked mainnet, but jupiter_earn calls live mainnet APIs directly, bypassing the fork

**Fixes Applied**:
- ✅ Removed jupiter_earn_tool from OpenAIAgent normal mode
- ✅ Made jupiter_earn_tool conditional in ZAI agent based on allowed_tools
- ✅ Removed jupiter_earn references from general agent contexts
- ✅ Added safety checks in tool execution
- ✅ Updated documentation (AGENTS.md, ARCHITECTURE.md, RULES.md)
- ✅ Code compiles successfully with restrictions in place

**Resolution**: Jupiter earn tool now properly restricted to position/earnings benchmarks only, preventing API calls that bypass surfpool's forked mainnet state.

**Impact**: Fixed for all benchmarks except 114-jup-positions-and-earnings.yml (where it's intended to be used)


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