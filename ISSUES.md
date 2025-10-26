# Issues

## Open Issues

### #2 Jupiter Lend Deposit Amount Parsing Issue - RESOLVED ✅
**Date**: 2025-10-26  
**Status**: Closed  
**Priority**: Medium  

**Resolution**: Enhanced context format implemented to clearly separate INITIAL vs CURRENT state with step numbers and visual indicators.

**Test Results**:
- ✅ **Context Format Works**: LLM now sees STEP 0 (initial) vs STEP 2+ (current) clearly separated
- ✅ **Amount Confusion Resolved**: Explicit instructions to use CURRENT STATE amounts
- 🎯 **Goal Achieved**: LLM can distinguish between old vs new token amounts

**Implementation**:
- Enhanced `LlmAgent.get_action()` in `reev-lib/src/llm_agent.rs`
- Added step-aware context formatting with visual indicators
- Clear labeling: "STEP 0 - INITIAL STATE (BEFORE FLOW START)" vs "STEP N - CURRENT STATE (AFTER PREVIOUS STEPS)"
- Explicit instruction: "💡 IMPORTANT: Use amounts from CURRENT STATE (STEP N) for operations"

**Impact**: Fixes primary confusion where LLM used `amount: 0` from initial state instead of current balance for Jupiter lend deposit operations.

---

### #4 SOL Transfer Placeholder Resolution Issue - High
**Date**: 2025-10-26  
**Status**: Open  
**Priority**: Medium  

**Issue**: GLM-4.6 LLM uses placeholder names directly instead of resolved addresses from key_map, causing "Failed to parse pubkey: Invalid Base58 string" errors.

**Symptoms**:
- Context shows resolved addresses like `"RECIPIENT_WALLET_PUBKEY": "3FHqkBwzaasvorCVvS6wSgzVHE7T8mhWmYD6F2Jjyqmg"`
- LLM tool call: `{"to_pubkey":"RECIPIENT_WALLET_PUBKEY",...}` (using placeholder instead of resolved address)
- Error: `SOL transfer error: Failed to parse pubkey: Invalid Base58 string`
- Affects SOL transfer and other operations requiring resolved addresses

**Root Cause**:
- LLM sees resolved addresses in key_map but doesn't understand to use them instead of placeholders
- Context shows both placeholder names AND resolved addresses, creating confusion
- Missing explicit guidance about using resolved addresses from key_map section
- Placeholder names like 'RECIPIENT_WALLET_PUBKEY' look like valid pubkeys to LLM

**Fixes Applied**:
- ✅ **Enhanced tool description**: Made tools more explicit about using resolved addresses
- ✅ **Added RAW balance display**: Context now shows both formatted and raw amounts (e.g., "394,358.118 USDC (RAW: 394358118)")
- ✅ **Improved debugging**: Added better error messages to show available vs requested amounts
- ✅ **Enhanced context format**: Step-aware separation of INITIAL vs CURRENT state
- ✅ **Enhanced context display**: Added explicit "🔑 RESOLVED ADDRESSES FOR OPERATIONS" section
- ✅ **Tool description updates**: Explicit instructions to use resolved addresses, not placeholders
- 🔍 **Current Issue**: Despite all context enhancements, LLM still passes placeholder names to tools

**Auto-Resolution Implementation Applied**:
- ✅ **Smart placeholder detection**: Identifies placeholders using `_` and keywords like WALLET/PUBKEY/TOKEN/ATA
- ✅ **Automatic resolution**: `self.key_map.get(&args.recipient_pubkey)` resolves placeholders to addresses
- ✅ **Fallback handling**: Uses original address if placeholder not found in key_map
- ✅ **Debug logging**: Auto-resolution logging to track behavior

**Current Debugging Findings**:
- Context properly includes resolved addresses: `"RECIPIENT_WALLET_PUBKEY": "AFsX1jD6JTb2hLFsLBzkHMWGy6UWDMaEY8UVnacwRWUH"`
- Tool receives correct key_map with resolved addresses  
- Auto-resolution logic: detects placeholder and should resolve to real address
- LLM still calls tool with: `{"recipient_pubkey":"RECIPIENT_WALLET_PUBKEY"}`
- Issue: Despite auto-resolution, parsing still fails with "Invalid Base58 string"
- Root cause: Tool execution may not be using new binary or caching issue

**Investigation Required**:
- Debug messages not appearing in logs suggests binary caching issue
- Need to verify auto-resolution code is actually being executed
- May need to restart processes or rebuild completely
- Alternative: Consider resolving at prompt level instead of tool level

**Next Steps Required**:
- 🔍 **Binary Caching**: Verify new code is actually executing in running processes
- 🛠️ **Force Restart**: Kill all reev-agent processes and rebuild to ensure new code
- 📝 **Alternative Approach**: Consider prompt-level placeholder resolution if tool-level fails
- 🔧 **Test Auto-Resolution**: Verify resolved address appears in parsing step
- 📊 **Monitor Behavior**: Track whether LLM adapts to better error messages

**Impact**: 
- Issue #2: Resolved - Enhanced context prevents amount confusion
- Issue #4: Active - LLM still ignores resolved address guidance despite clear context
- Affects all operations requiring resolved addresses from key_map

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

### #2 Jupiter Lend Deposit Amount Parsing Issue - Fixed ✅
**Date**: 2025-10-26  
**Status**: Closed  
**Resolution**: Enhanced context format with step-aware labeling

**Implementation**: Enhanced `LlmAgent.get_action()` in `reev-lib/src/llm_agent.rs` to create step-aware context that clearly separates INITIAL STATE (STEP 0) from CURRENT STATE (STEP N+). Added visual indicators and explicit instructions to use amounts from current state.

**Impact**: Resolves LLM confusion between original amounts and current balances in multi-step flows.

---

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