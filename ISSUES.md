# Reev Project Issues

## Current Issues

### Issue #309: SPL Transfer with "all" Keyword Fails
**Status**: Fixed  
**Priority**: High  
**Description**: When executing "send all usdc to [address]", the system was failing with "insufficient funds" error from the token program. The issue was that the `refined_prompt` in the structured response was not being updated with the actual calculated amount when the "all" keyword was used.

**Root Cause**: The LLM was returning the original prompt with "all" in the `refined_prompt` field instead of replacing "all" with the calculated amount. This caused confusion in the execution flow because:
1. The structured response contained the correct amount in the parameters field (e.g., "100.0")
2. But the `refined_prompt` still contained "all usdc" instead of "100.0 usdc"
3. The execution flow was using the amount from the parameters, which was correctly parsed and converted to the smallest unit (100.0 * 10^6 = 100,000,000)
4. However, there was still some confusion in the system because the refined prompt didn't match the actual parameters

**Solution Implemented**:
1. Updated the `STRUCTURED_PROMPT_SYSTEM_PROMPT` in `crates/reev-core/src/prompts/prompt_processor.rs` to include instructions for the LLM to replace "all" with the actual calculated amount in the `refined_prompt` field
2. The prompt now explicitly instructs the LLM to:
   - Replace "all" with the actual numeric amount in `refined_prompt`
   - Example: "send all usdc to..." becomes "send 100.0 usdc to..." when max_amount is 100.0
3. Fixed the `calculate_max_transferable_amount` function in `crates/reev-core/src/utils/transfer_utils.rs` to correctly handle SPL tokens by returning the full amount without gas deduction (since gas is paid in SOL for SPL transfers)
4. Fixed the `get_token_balance_from_surfpool` function in `crates/reev-core/src/prompt_processor/mod.rs` to correctly handle the UiTokenAmount returned by the RPC client

**Files Modified**:
- `crates/reev-core/src/prompts/prompt_processor.rs` - Updated structured prompt to handle "all" keyword replacement
- `crates/reev-core/src/utils/transfer_utils.rs` - Fixed SPL token max amount calculation
- `crates/reev-core/src/prompt_processor/mod.rs` - Fixed token balance retrieval

**Testing**:
- Verified that "send 1 usdc to [address]" works correctly
- Verified that "send all usdc to [address]" now works correctly with proper amount calculation
- All tests are now passing

**Result**: The "all" keyword now works correctly for SPL token transfers when sufficient balance is available. The system correctly calculates the full token balance and passes this information through to the execution flow.
