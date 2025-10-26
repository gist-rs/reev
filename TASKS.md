# Tasks

## 🎯 Open Issues

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

### #2 GLM SPL Transfer ATA Resolution Issue - Medium
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
3. **Enhance Key Map Resolution**: Improve how tool looks up and prioritizes ATAs from key_map
4. **Context Clarification**: Strengthen RESOLVED ADDRESSES section instructions for GLM models
5. **Comprehensive Testing**: Test fix across multiple GLM benchmarks and model variants
6. **Documentation Update**: Update documentation with clear examples of correct ATA usage

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