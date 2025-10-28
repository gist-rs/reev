# Tasks

## 🎯 Open Issues

### #1 Jupiter Swap Tool Response Format Inconsistency - CRITICAL 🔥

**Date**: 2025-10-28
**Status**: Identified
**Priority**: Critical

**Issue**: Same benchmark (`200-jup-swap-then-lend-deposit`) succeeds via CLI (100% score) but fails via API due to different Jupiter swap tool response formats between execution paths.

**Root Cause**: Two different Jupiter swap tool implementations with inconsistent response structures:
- **CLI path**: Uses `jupiter_swap_flow.rs` (flow-aware tool)
- **API path**: Uses `jupiter_swap.rs` (standard tool)

**Impact**: Step 2 (deposit) receives no swap amount data from step 1, causing LLM to guess wrong amount.

**Evidence**:
```
CLI Success: amount=394358118 (394.358 USDC) - Uses swap_details.output_amount
API Failure: amount=1000000000 (1000 USDC) - Missing swap_details structure
```

**Critical Files**:
- `crates/reev-tools/src/tools/jupiter_swap_flow.rs` - Flow-aware tool with swap_details
- `crates/reev-tools/src/tools/jupiter_swap.rs` - Standard tool without swap_details
- `crates/reev-agent/src/flow/agent.rs` - Tool routing logic
- `crates/reev-context/src/lib.rs` - process_step_result_for_context() expects swap_details

**Fix Strategy**: Unify Jupiter swap tool implementations to ensure consistent response format across all execution paths.

---


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

---

#### 🎯 GLM SPL Transfer ATA Resolution Issue - Medium

**Date**: 2025-10-26
**Status**: In Progress
**Priority**: High

**Issue**: GLM models (glm-4.6-coding) through reev-agent are generating wrong recipient ATAs for SPL transfers. Instead of using pre-created ATAs from benchmark setup, the LLM generates new ATAs or uses incorrect ATA names.

**Symptoms**:
- `002-spl-transfer` score: 56.2% with "invalid account data for instruction" error
- LLM generates transaction with wrong recipient ATA: "8RXifzZ34i3E7qTcvYFaUvCRaswcJBDBXrPGgrwPZxTo" instead of expected "BmCGQJCPZHrAzbLCjHd1JBQAxF24jrReU3fPwN6ri6a7"

**Root Cause**:
- LLM should use placeholder name `"RECIPIENT_USDC_ATA"` in tool calls, but is generating new recipient ATA
- Context confusion from RESOLVED ADDRESSES section (already fixed but still affecting GLM behavior)
- Possible misinterpretation of recipient parameters vs ATA placeholders
- **FIXED**: Different GLM agents had inconsistent context and wallet handling

**✅ COMPLETED FIXES**:
1. **UNIFIED GLM LOGIC**: Created `UnifiedGLMAgent` with shared context and wallet handling
2. **IDENTICAL CONTEXT**: Both `OpenAIAgent` and `ZAIAgent` now use same context building logic
3. **SHARED COMPONENTS**: Wallet info creation and prompt mapping are now identical
4. **PROVIDER-SPECIFIC**: Only request/response handling differs between implementations

**Technical Requirements**:
1. **Test Unified Logic**: Verify unified GLM logic resolves context inconsistencies
2. **Improve ATA Resolution Logic**: Enhance SPL transfer tool to better prioritize pre-created ATAs from key_map over generated ones
3. **Strengthen Context Instructions**: Make context warnings more explicit about using placeholder names vs direct addresses
4. **Test Across GLM Variants**: Verify fix works with different GLM model implementations
5. **Documentation Update**: Update documentation with clear examples of correct ATA usage

---

## 📋 Tasks

#### 🎯 GLM SPL Transfer ATA Resolution Fix - High Priority
**Status**: In Progress
**Priority**: High
**Description**: Fix SPL transfer tool to properly resolve pre-created ATAs and prevent LLM from generating incorrect recipient addresses

**Background**:
- SOL transfer issue was successfully resolved by improving context instructions
- However, SPL transfers still fail because GLM models generate wrong recipient ATAs despite having pre-created ones in key_map
- Local agents work perfectly, indicating the issue is specific to GLM model routing through reev-agent
- ✅ **COMPLETED**: Unified GLM logic architecture - both OpenAIAgent and ZAIAgent now use identical context and wallet handling

**Technical Requirements**:
1. **Investigate LLM Tool Calls**: Debug exactly what recipient_pubkey value LLM is using in spl_transfer calls
2. **Improve ATA Resolution Logic**: Enhance SPL transfer tool to better prioritize pre-created ATAs from key_map over generated ones
3. **Strengthen Context Instructions**: Make context warnings more explicit about using placeholder names vs direct addresses
4. **Test Across GLM Variants**: Verify fix works with different GLM model implementations
5. **Documentation Update**: Update documentation with clear examples of correct ATA usage

**Implementation Steps**:
1. **✅ UNIFIED GLM ARCHITECTURE**: Refactored both OpenAIAgent and ZAIAgent to use shared logic
2. **Debug Current Behavior**: Add extensive logging to SPL transfer tool to track LLM parameter usage
3. **Enhance Key Map Resolution**: Improve how tool looks up and prioritizes ATAs from key_map over generated ones
4. **Context Clarification**: Strengthen RESOLVED ADDRESSES section instructions for GLM models
5. **Comprehensive Testing**: Test fix across multiple GLM benchmarks and model variants
6. **Documentation Update**: Update documentation with clear examples of correct ATA usage

**Current Critical Priority**: Jupiter swap tool unification takes precedence over GLM ATA resolution fix.

---

#### 🧪 Jupiter Swap Tool Implementation Review**


---
*Add this task to TASKS.md and track progress in ISSUES.md*

#### 🧪 Jupiter Swap Tool Implementation Review
   - Analyze why two different implementations exist
   - Determine if `jupiter_swap_flow.rs` should replace `jupiter_swap.rs` entirely
   - Update tool registration and discovery mechanisms
   - Document proper usage patterns for each tool type

**Acceptance Criteria**:
- [x] Unified GLM logic implemented in common module
- [x] Both OpenAIAgent and ZAIAgent use identical context building
- [x] Wallet info creation and prompt mapping are shared
- [x] Only request/response handling differs between implementations
- [ ] Local agent works correctly (SOL and SPL transfers)
- [ ] GLM-4.6-coding agent works correctly (SOL transfers)
- [ ] GLM-4.6-coding agent works correctly (SPL transfers)
- [ ] No regression in other agent types
- [ ] Improved context prevents address confusion
- [ ] All diagnostics pass

---
#### 🎯 Jupiter Swap Tool Unification - HIGH PRIORITY ✅ IN PROGRESS

**Status**: In Progress  
**Priority**: High  
**Description**: Unify Jupiter swap tool implementations to ensure consistent `swap_details` response format across CLI and API execution paths.

**Background**: 
Same benchmark (`200-jup-swap-then-lend-deposit`) succeeds via CLI (100% score) but fails via API due to different Jupiter swap tool response formats between execution paths.

**Root Cause**: 
Two different Jupiter swap tool implementations with inconsistent response structures:
- **CLI path**: Uses `jupiter_swap_flow.rs` (flow-aware tool)
- **API path**: Uses `jupiter_swap.rs` (standard tool)

**Impact**: 
Step 2 (deposit) receives no swap amount data from step 1, causing LLM to guess wrong amount.

**Evidence**:
```
CLI Success: amount=394358118 (394.358 USDC) - Uses swap_details.output_amount
API Failure: amount=1000000000 (1000 USDC) - Missing swap_details structure
```

**Critical Files**:
- `crates/reev-tools/src/tools/jupiter_swap_flow.rs` - Flow-aware tool with swap_details
- `crates/reev-tools/src/tools/jupiter_swap.rs` - Standard tool without swap_details
- `crates/reev-agent/src/flow/agent.rs` - Tool routing logic
- `crates/reev-context/src/lib.rs` - `process_step_result_for_context()` expects swap_details

**Fix Strategy**: 
Unify Jupiter swap tool implementations to ensure both CLI and API use consistent tool that provides structured `swap_details` for multi-step flow communication.

**Implementation Options**:
- **Option A: Tool Unification** (Recommended)
  - Merge both implementations into single flow-aware Jupiter swap tool
  - Ensure consistent `swap_details` response format
  - Update tool registration to use unified tool only
- **Option B: Response Format Standardization** (Quick)
  - Modify `jupiter_swap.rs` to also return `swap_details` structure
  - Ensure both tools provide same data format

---

#### 🛠️ Immediate Action Required

1. **Investigate routing logic** - Why do CLI and API use different tools?
2. **Compare response formats** - Document exact structure differences
3. **Implement unification** - Choose Option A or B based on complexity
4. **Test thoroughly** - Ensure API achieves same success rate as CLI
5. **Document changes** - Update tool usage patterns for each tool type

**Technical Requirements**:
1. **Preserve CLI functionality** - Don't break the working path
2. **Consistent response format** - Both tools return `swap_details.output_amount`
3. **Context flow verification** - Step 2 receives correct swap result data
4. **No performance regression** - API path matches CLI success rate

**Acceptance Criteria**:
- [x] Unified GLM logic implemented in common module
- [x] Both OpenAIAgent and ZAIAgent use identical context building
- [x] Wallet info creation and prompt mapping are shared
- [x] Only request/response handling differs between implementations
- [ ] Local agent works correctly (SOL and SPL transfers)
- [ ] GLM-4.6-coding agent works correctly (SOL transfers)
- [ ] GLM-4.6-coding agent works correctly (SPL transfers)
- [ ] No regression in other agent types
- [ ] Improved context prevents address confusion
- [ ] All diagnostics pass

---
*Add this task to TASKS.md and track progress in ISSUES.md*
