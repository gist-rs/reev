# Reev Project Issues

## Current Issues

### Issue #308: Swap Regression with "all" Keyword
**Status**: Fixed  
**Priority**: High  
**Description**: When executing "swap all sol for usdc", the system was failing with "insufficient lamports" error. The issue was that the Jupiter swap tool was checking if the amount string was literally "all" and deducting the gas reserve again, even though the prompt processor had already calculated the max transferable amount.

**Root Cause**: The execution flow was not properly using the structured response from the prompt processor, which includes the extracted parameters with the calculated amount. Instead, it was still parsing the refined prompt text and checking for "all" keyword.

**Solution Implemented**:
1. Removed the "backward compatibility" approach that incorrectly tried to convert StructuredRefinedPrompt to RefinedPrompt
2. Updated the execution flow to use structured data directly:
   - Added `structured_prompt` field to `YmlStep` to carry structured data through the pipeline
   - Modified `execute_step_with_rig_and_history` to use structured action and parameters when available
   - Added `create_tool_calls_from_structured_data` method to directly create tool calls from structured data without parsing
3. Updated the `YmlGenerator` to:
   - Set the `structured_prompt` field in `YmlStep`
   - Create appropriate expected tool calls based on action type and parameters
4. Fixed the Jupiter swap tool to properly handle the calculated amount instead of checking for "all" keyword

**Files Modified**:
- `crates/reev-core/src/planner.rs` - Updated to use structured response directly
- `crates/reev-core/src/prompt_processor/mod.rs` - Removed incorrect `structured_to_refined` method
- `crates/reev-core/src/yml_generator/mod.rs` - Updated to handle all action types and set structured_prompt
- `crates/reev-core/src/execution/rig_agent/mod.rs` - Added method to create tool calls from structured data
- `crates/reev-core/src/yml_schema.rs` - Added `structured_prompt` field to YmlStep
- `crates/reev-core/src/execution/rig_agent/tools/jupiter_swap.rs` - Fixed to handle calculated amount correctly

**Testing**:
- Fixed test cases to handle scenario where wallet balance is insufficient for swaps (usable_amount = 0)
- All prompt processor tests are now passing, including structured response tests

**Result**: The "all" keyword now works correctly for swaps when sufficient balance is available. When balance is insufficient, the system correctly calculates a zero usable amount and passes this information through to the execution flow.
