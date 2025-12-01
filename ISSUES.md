# Reev Project Issues

## Current Issues 306

### Issue 301: E2E Test Failures - FIXED ✅

**Description:**
While the protocol interface implementation is solid, several e2e tests were failing. These have now been resolved:

1. **e2e_swap test**: Both test cases are now passing consistently. The "all sol for usdc" case was failing with Jupiter transaction error (0xffff) but has been fixed.
   - **Fix**: Added a new constant `JUPITER_SWAP_FEE_RESERVE` (0.01 SOL) and updated all Jupiter swap implementations to use this centralized constant. The PromptProcessor now correctly uses 0.01 SOL fee for swaps instead of 0.001 SOL.

2. **e2e_transfer test**: Both test cases are now passing consistently. The "all sol" case was failing with insufficient funds error.
   - **Fix**: Reverted to LLM-based approach for handling "all" keyword instead of rule-based u64::MAX approach. The prompt processor now correctly calculates the max transferable amount and provides it to the LLM, which then generates a prompt with the specific amount (e.g., "send 4.999 SOL" instead of "send all SOL").

3. **e2e_multi_step test**: Now passing consistently without warnings.
   - **Fix**: The Jupiter swap fee fix resolved the intermittent failures in the multi-step test as well.

### Issue 304: LLM Not Properly Handling "all" Transfers - FIXED ✅

**Description:**
LLM was receiving wallet balance context but not properly converting "all" to calculated transferable amount (balance minus gas fees). This caused transfers to fail with insufficient funds errors.

**Fix Applied:**
Implemented structured YML approach for handling "all" keyword transfers as per PLAN_ALL.md:

1. Created `TransferAmountRefinementRequest` structure to provide structured YML prompts to LLM
2. Updated `PromptProcessor` to use structured YML approach for "all" transfers
3. Removed fallback logic that was causing inconsistencies
4. Updated error handling to return errors directly
5. Modified LLM prompts to expect JSON response with `refined_prompt` field for structured requests

**Changes Made:**
- Added `TransferAmountRefinementRequest` with `original_prompt`, `usable_amount`, and `instruction` fields
- Modified `process_prompt` to detect "all" keyword and create structured YML request
- Updated `send_refine_request` to handle different response formats (JSON for YML, text for regular)
- Enhanced system prompt to handle both structured YML and regular text prompts
- Added proper error handling for structured YML responses

**Testing:**
- Both test cases in `e2e_transfer.rs` now pass:
  - `send 1 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq` ✅
  - `send all sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq` ✅

**Priority:** High
**Status:** Resolved

### Issue 302: JupiterProtocol Implementation Is Placeholder

**Description:**
The JupiterProtocol wrapper currently returns hardcoded values rather than executing real operations. It needs to be integrated with actual Jupiter handlers for Swap, Lend, and Earn operations.

Current implementation just returns placeholder signatures and calculations:
```rust
let swap_result = super::JupiterSwapResult {
    success: true,
    signature: Some("jupiter_swap_placeholder".to_string()),
    input_amount: parse_amount(amount)?,
    output_amount: parse_amount(amount)? * 2, // Placeholder calculation
    price_impact: Some(0.05),
};
```

**Priority:** High
**Status:** Open
**Assigned:** Unassigned

### Issue 305: Implemented Structured YML Prompts for "all" Keyword Transfers - FIXED ✅

**Description:**
Implemented structured YML approach for handling "all" keyword transfers as specified in PLAN_ALL.md. This replaces the unstructured prompt approach with a more reliable system for converting "all" to specific transferable amounts.

**Implementation Details:**
1. Created `TransferAmountRefinementRequest` structure with fields for original_prompt, usable_amount, and instruction
2. Modified `PromptProcessor` to detect "all" keyword and create structured YML requests
3. Updated `send_refine_request` to handle different response formats (JSON for YML, text for regular)
4. Enhanced system prompt to handle both structured YML and regular text prompts
5. Removed fallback logic to ensure consistency
6. Updated error handling to return errors directly

**Files Modified:**
- `/reev/crates/reev-core/src/prompt_processor/mod.rs` - Main implementation
- `/reev/crates/reev-core/src/prompts/prompt_processor.rs` - System prompt updates

**Testing Results:**
Both test cases in `e2e_transfer.rs` now pass:
- `send 1 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq` ✅
- `send all sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq` ✅

**Priority:** High
**Status:** Resolved

### Issue 306: Jupiter Swap Fee Mismatch - FIXED ✅

**Description:**
Jupiter swaps were using a 0.01 SOL fee reserve in multiple places, but PromptProcessor was only calculating with 0.001 SOL for all operations. This mismatch caused the "swap all sol" test to fail with transient Jupiter errors (0x6).

**Root Cause:**
1. PromptProcessor calculated usable_amount with only 0.001 SOL fee reserve
2. Jupiter swap implementations were trying to use 0.01 SOL fee reserve
3. This resulted in trying to swap more SOL than was calculated as usable

**Fix Applied:**
1. Added a new constant `JUPITER_SWAP_FEE_RESERVE` (0.01 SOL) to `reev-lib/src/constants/amounts.rs`
2. Updated PromptProcessor to use 0.001 SOL fee for transfers and 0.01 SOL for swaps (using the new constant)
3. Updated all Jupiter swap implementations to use the centralized constant instead of hardcoded values

**Files Modified:**
- `/reev/crates/reev-lib/src/constants/amounts.rs` - Added JUPITER_SWAP_FEE_RESERVE constant
- `/reev/crates/reev-core/src/prompt_processor/mod.rs` - Updated to use different fees for different operations
- `/reev/crates/reev-core/src/execution/handlers/swap/jupiter_swap.rs` - Updated to use centralized constant
- `/reev/crates/reev-core/src/execution/rig_agent/tool_execution.rs` - Updated to use centralized constant
- `/reev/crates/reev-tools/src/tools/jupiter_swap.rs` - Updated to use centralized constant

**Testing Results:**
All e2e tests now pass consistently:
- e2e_swap.rs: Both test cases pass ✅
- e2e_transfer.rs: Both test cases pass ✅
- e2e_multi_step.rs: Test case passes ✅

**Priority:** High
**Status:** Resolved
**Implemented:** 2024-01-15

### Issue 303: Build Warning in Workspace

**Description:**
There's a build warning that should be addressed:
```
warning: /Users/katopz/git/gist/reev/Cargo.toml: unused manifest key: workspace.dev-dependencies
```

**Priority:** Low
**Status:** Open
**Assigned:** Unassigned