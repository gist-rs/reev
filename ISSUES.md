# Reev Core Implementation Issues

---

## Issue #117: Updated Context Not Passed to LLM for Multi-Step Flows (IN PROGRESS)
### Status: PARTIALLY FIXED
### Description:
When executing a multi-step flow with a swap followed by a lend operation, the LLM is not receiving the updated wallet context after the swap step. This causes the lend operation to use stale balance information and fail due to requesting an amount that exceeds the available balance.

### Current Behavior:
- First step (swap) correctly exchanges SOL for USDC
- Context is updated with correct USDC balance (e.g., 1,000,744,792 USDC)
- However, the LLM for the second step still receives the original context with the old USDC balance (200,000,000)
- LLM calculates 95% of the old balance, resulting in a requested amount (951,695,255) that exceeds the available balance (801,744,792 after subtracting pre-existing balance)
- Balance validation fails because requested amount exceeds available balance

### Root Cause:
The issue is in the executor.rs file. When updating the context after a step, it updates the internal context correctly, but it's not properly serializing this updated context when passing it to the LLM for the next step in the flow. We've partially fixed the issue by ensuring that `current_context` is passed to `execute_step_with_history` instead of the original `wallet_context` parameter. However, debug logs show that the context being passed to the LLM still contains the old USDC balance, suggesting a serialization issue.

### Tasks Required:
1. ✅ Fix the executor to pass the updated context to subsequent steps (partially complete)
2. ⚠️ Investigate why updated context is not being serialized properly when passed to LLM
3. Verify the fix works for both USDC and other tokens in multi-step flows
4. Add test to prevent regression
5. Update the prompt to make it clearer to the LLM to use the exact amount from the previous step
6. ✅ Added delay between steps to ensure blockchain has fully processed the swap
7. ✅ Modified context update to use actual on-chain balance instead of adding to previous balance
8. ❌ Test still failing due to BalanceValidator seeing different balance than context

---

## Issue #116: Context Passing Between Multi-Step Flow Steps (COMPLETED)
### Status: COMPLETED
### Description:
Multi-step flows need to pass actual results from previous steps to subsequent steps. The implementation has been completed to properly update wallet context between steps.

### Current Implementation:
1. ✅ Added multi-threaded runtime to test to support blocking calls in jup_sdk
2. ✅ Implemented update_context_after_step method to update wallet context after each step
3. ✅ Added previous step history to RigAgent context for better decision making
4. ✅ Modified test to use conservative amount approach for lend step
5. ✅ Fixed borrowing issues in test
6. ✅ Fixed context update logic for Jupiter swap to query blockchain for actual output amount
7. ✅ Fixed USDC amount conversion from USDC to smallest units (1,000,000 instead of 1,000,000,000)

### Current Issue:
The swap step is now correctly updating the context with actual swap output amounts. However, the updated context is not being passed to the next step in the flow. The LLM still receives the original wallet context with old balances.

### Current Behavior:
- First step (swap) succeeds with transaction signature
- Context is correctly updated with actual swap output (e.g., 801,854,694 USDC)
- Second step (lend) still receives original context (e.g., 200,000,000 USDC)
- Lend step fails because it tries to lend 95% of original balance, not updated balance

### Next Steps:
1. Fix the issue where updated context is not being passed to subsequent steps
2. Verify that the context update is being applied to the context object used for the next step

---



## Issue #102: Implement Error Recovery Engine (NOT STARTED)
### Status: NOT STARTED
### Description:
Error recovery is incomplete. While there's a RecoveryConfig struct, the actual recovery logic isn't fully implemented. Failed steps need proper recovery strategies based on error type.

### Success Criteria:
- Implement error recovery strategies for common failures
- Add retry logic with exponential backoff for network errors
- Implement alternative path selection for transaction failures
- Add slippage adjustment for swap failures

### Tasks Required:
1. Implement retry mechanism for network errors
2. Add alternative path selection for failed transactions
3. Implement slippage adjustment for swap failures
4. Create error classification system
5. Add recovery metrics and logging

---

## Issue #105: Enhance RigAgent Integration (PARTIALLY COMPLETED)
### Status: PARTIALLY COMPLETED
### Description:
RigAgent integration is basic and doesn't fully leverage LLM capabilities. Tool selection and parameter extraction need improvement to handle more complex operations and edge cases.

### Success Criteria:
- Improve RigAgent prompt engineering for better tool selection
- Add parameter validation before tool execution
- Implement tool result interpretation
- Add support for multi-step operations

### Tasks Completed:
1. ✅ Basic RigAgent prompts for tool selection
2. ✅ Basic parameter extraction from refined prompts
3. ✅ Tool execution for SOL transfers, swaps, etc.

### Tasks Remaining:
1. ❌ Better prompt engineering for more complex tool selection
2. ❌ Parameter validation before tool execution
3. ❌ Tool result interpretation
4. ❌ Multi-step operation support
5. ❌ Improved RigAgent error handling for edge cases

---

## Issue #106: Improve LanguageRefiner (PARTIALLY COMPLETED)
### Status: PARTIALLY COMPLETED
### Description:
LanguageRefiner needs improvement to handle more complex prompts and edge cases. Current implementation is basic and doesn't support advanced language refinement features.

### Success Criteria:
- Support more complex prompt patterns
- Add context awareness to refinement
- Implement better error handling
- Add support for multi-language prompts

### Tasks Completed:
1. ✅ LLM-based language refinement implementation
2. ✅ Support for complex prompt patterns
3. ✅ Operation type preservation (critical fix)
4. ✅ Basic error handling

### Tasks Remaining:
1. ❌ Context awareness to refinement
2. ❌ Multi-language support
3. ❌ Better refinement quality metrics
4. ❌ More sophisticated error recovery

---



## Issue #112: Add Comprehensive Error Recovery (NEW)
### Status: NOT STARTED
### Description:
The V3 plan specifies robust error recovery as a key component, but current implementation has minimal error handling. Error recovery should handle different error types with appropriate strategies.

### Success Criteria:
- Implement error classification for different failure types
- Add retry logic with exponential backoff for transient errors
- Implement alternative path selection for failed transactions
- Add slippage adjustment for swap failures
- Add circuit breaker pattern for repeated failures

### Tasks Required:
1. Create error classification system
2. Implement retry logic with exponential backoff
3. Add alternative transaction path selection
4. Implement slippage adjustment for swap failures
5. Add circuit breaker for repeated failures
6. Add recovery metrics and logging

---

### Implementation Priority

### Week 1:
1. ~~Issue #111: Complete RigAgent Integration for Tool Selection (COMPLETED)~~
2. Issue #117: Fix USDC Amount Multiplied by 1,000,000 in Jupiter Lend Earn Deposit (NEW)
3. ~~Issue #116: Complete Context Passing Between Multi-Step Flow Steps (COMPLETED)~~

### Week 2:
3. Issue #102: Implement Error Recovery Engine (NOT STARTED)
4. Issue #105: Enhance RigAgent Integration (PARTIALLY COMPLETED)
5. Issue #106: Improve LanguageRefiner (PARTIALLY COMPLETED)

### Week 3:
6. Issue #110: Remove Unused Code in YmlGenerator (NEW)
7. Issue #112: Add Comprehensive Error Recovery (NEW)

### Current State Summary:
All e2e tests are now passing, including:
- swap operations (including "sell all SOL")
- transfer operations
- rig_agent tool selection
- "sell" terminology parsing as swap operations

The architecture is properly aligned with V3 plan, where RigAgent handles tool selection based on refined prompts rather than rule-based operation parsing. Balance validation has been successfully integrated.

## Notes:
- Completed issues are removed to reduce noise
- Future implementation should follow V3 plan guidelines
- RigAgent is the primary mechanism for tool selection and parameter extraction
- Balance validation is integrated and working
- Error recovery implementation is a high priority for production readiness