# Plan to Implement Structured Max Amount Calculation System

## IMPORTANT NOTES
- DO NOT ADD ANY MORE CODE TO THE ALREADY HUGE mod.rs FILE (1200+ lines)
- No backward compatibility needed in development - tests must follow code changes
- No migration strategies, mocks, or fallback mechanisms
- Code quality is priority over preserving existing tests
- BREAKING CHANGES ARE ACCEPTABLE - tests should be updated to match implementation



## Executive Summary

This document outlines a comprehensive plan to refactor the max_amount_decimal calculation system from a rule-based approach to a structured, scalable solution. The current implementation has extensive rule-based logic that is brittle, hard to maintain, and won't scale as we add new operation types.

## Problem Analysis

### Current State
- The system has extensive rule-based logic in `process_prompt_legacy` and `process_prompt_structured`
- Multiple code paths to detect operation type and calculate max amounts
- Hardcoded token addresses (USDC, USDT) scattered throughout the code
- Inconsistent handling between different operation types (transfer vs swap)
- Double gas deduction for Jupiter swap operations

### Key Issues Identified
1. **Brittle parsing**: Current system relies on string matching that breaks with minor variations
2. **Poor scalability**: Adding new operation types requires updating multiple code paths
3. **Code duplication**: Similar logic scattered across different functions
4. **Maintenance burden**: Any change to fees or tokens requires updates in multiple places
5. **Inconsistent validation**: Different operations have different parameter extraction logic

## Solution Overview

Implement a structured max amount calculation system where:
1. All max amounts are calculated upfront for all operation types
2. Calculations are grouped by action type (transfer, swap, lend, etc.)
3. Each action type has its own fixed fee structure
4. The complete set of max amounts is included in the structured YML prompt

This approach eliminates the need for rule-based detection and provides a more reliable, extensible foundation.



## Phase 1: Create Max Amount Calculator Module

### Purpose
Create a centralized module for calculating max amounts across all action types.

### Tasks
1. Define `MaxAmountCalculation` struct to hold max amounts for an action type:
   ```rust
   pub struct MaxAmountCalculation {
       pub action: PromptAction,
       pub max_amounts: HashMap<String, f64>, // token_symbol -> max_amount
   }
   ```

2. Define `MaxAmountCalculator` struct with fixed fees for each action type:
   ```rust
   pub struct MaxAmountCalculator {
       // Fixed fees for each action type
       fees: HashMap<PromptAction, f64>,
       // Common token mint addresses
       token_mints: HashMap<String, String>,
   }
   ```

3. Implement method to calculate max amounts for all action types:
   ```rust
   impl MaxAmountCalculator {
       pub async fn calculate_all_max_amounts(
           wallet_address: &str,
       ) -> Result<Vec<MaxAmountCalculation>>
   }
   ```

4. Create method to format max amounts as YML for LLM prompt:
   ```rust
   impl MaxAmountCalculator {
       pub fn format_as_yml_prompt(
           max_amounts: &Vec<MaxAmountCalculation>
       ) -> Result<String>
   }
   ```

### Implementation Location
- Primary: `crates/reev-core/src/prompt_processor/max_amount_calculator.rs`

## Phase 2: Update Prompt Processor

### Purpose
Modify the prompt processor to use the new structured max amount calculation.

### Tasks
1. Update `process_prompt_structured` to:
   - Use `MaxAmountCalculator` to get all max amounts upfront
   - Pass this information to the LLM in the structured prompt
   - Remove all rule-based detection logic for operation types

2. Remove all if/else logic that detects operation type based on string matching

3. Simplify the prompt processing flow:
   - Always calculate max amounts for all operation types
   - Format max amounts as YML and include in the prompt sent to LLM
   - Let LLM determine the appropriate max amount based on action type
   - LLM must respond in JSON format (not YML) as specified in PLAN_ALL.md

### Implementation Location
- Primary: `crates/reev-core/src/prompt_processor/types.rs`
- Secondary: Create `crates/reev-core/src/prompt_processor/refine_request.rs` for generic request handling
- CRITICAL: TRY TO NOT ADD ANY MORE CODE TO THE ALREADY HUGE mod.rs FILE (1200+ lines) ONLY REMOVE OR MOVE OUT TO MAKE IT SMALLER
## Phase 3: Update Structured Prompt Format

### Purpose
Include all max amounts in the structured prompt sent to the LLM.

### Tasks
1. Modify the structured prompt to include a comprehensive max amounts section:
   ```yaml
   max_amounts:
     transfer:
       SOL: 10.5
       USDC: 1000.0
       USDT: 1000.0
     swap:
       SOL: 10.3
       USDC: 1000.0
       USDT: 1000.0
     lend:
       SOL: 10.4
       USDC: 1000.0
       USDT: 1000.0
     borrow:
       SOL: 10.2
       USDC: 1000.0
       USDT: 1000.0
   ```

2. Ensure token symbols are standardized and consistent

### Implementation Location
- Primary: `crates/reev-core/src/prompts/prompt_processor.rs`

## Phase 4: Update LLM System Prompt

### Purpose
Instruct the LLM to use the structured max amounts appropriately.

### Tasks
1. Update the system prompt to instruct the LLM to:
   - Identify the action type from the user's prompt
   - Use the appropriate max amount from the YML structure provided in the prompt
   - Include the calculated amount in the refined prompt
   - Clearly indicate when "all" keyword was used
   - RESPOND IN JSON FORMAT (not YML) as specified in PLAN_ALL.md

2. Add YML examples in the system prompt (sent to LLM) with validation rules:
  - Include validation section with check/expected/result fields
  - Each example must validate amount doesn't exceed max for the action type
  - Examples should cover different action types (transfer, swap, lend, borrow)
  - All structured data in the PROMPT must be in valid YML format
  - LLM response must be in JSON format as specified in PLAN_ALL.md

3. Provide YML examples in the system prompt showing how max amounts are structured:
      ```yaml
      # Example 1: Swap all SOL to USDC
      prompt: "Swap all SOL to USDC"
      max_amounts:
         swap:
           SOL: 10.3
      # NOTE: This YML block is part of the PROMPT sent to LLM
      # LLM should extract max_amounts.swap.SOL (10.3) and use it in JSON response
      ```
   
      ```yaml
      # Example 2: Transfer all USDC
      prompt: "Transfer all USDC to address"
      max_amounts:
        transfer:
          USDC: 1000.0
      # NOTE: This YML block is part of the PROMPT sent to LLM
      # LLM should extract max_amounts.transfer.USDC (1000.0) and use it in JSON response
      ```
   
      LLM RESPONSE FORMAT (JSON, following PLAN_ALL.md):
      ```json
      {
        "refined_prompt": "Swap 10.3 SOL to USDC",
        "action": "swap",
        "parameters": {
          "amount": "10.3",
          "input_mint": "So11111111111111111111111111111111111111112",
          "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        }
      }
      ```

### Implementation Location
- Primary: `crates/reev-core/src/prompts/prompt_processor.rs`

## Phase 5: Update Validation Logic

### Purpose
Ensure the structured responses use appropriate max amounts.

### Tasks
1. Update `validate_structured_response` to:
   - Check that the amount in JSON response doesn't exceed the max for the action type from the YML prompt
   - Validate the action type in JSON response matches the max amounts used from YML
   - Ensure consistent use of max amounts when "all" keyword was detected
   - Note: We're validating that JSON response properly uses max amounts from YML prompt

2. Add validation functions for specific scenarios:
   ```rust
   fn validate_amount_for_action(
       amount: f64,
       action: PromptAction,
       max_amounts: &HashMap<String, f64>,
   ) -> ValidationResult
   ```

3. Add comprehensive error messages for validation failures:
   - "Amount X exceeds max Y for action Z"
   - "Action type mismatch in max amount usage"
   - "Invalid max amount calculation detected"

### Implementation Location
- Primary: `crates/reev-core/src/prompt_processor/validation.rs`

## Phase 6: Refactor Prompt Processor Module Structure

### Purpose
Refactor the oversized prompt_processor/mod.rs file and eliminate legacy components.

### Tasks
1. Break down the oversized mod.rs file:
   - Create `max_amount_calculator.rs` (Phase 1 implementation)
   - Create `legacy_processor.rs` for legacy methods (process_prompt_legacy)
   - Create `structured_processor.rs` for structured methods (process_prompt_structured)
   - Create `llm_client.rs` for LLM communication methods
   - Move utility functions to `utils.rs` module
   - Keep only the main PromptProcessor struct definition and public API in mod.rs

2. Remove TransferAmountRefinementRequest:
   - The TransferAmountRefinementRequest is a legacy component specific to transfers
   - It was created when only transfers had special "all" keyword handling
   - No corresponding components exist for swap, lend, borrow, etc.
   - Eliminate this entirely since `StructuredRefineRequest` already serves this purpose
   - The `build_refinement_prompt` function that works with TransferAmountRefinementRequest should also be eliminated
   - ALL of this functionality should be moved to new modules, not added to mod.rs

3. Update import statements and module declarations:
   - Update mod.rs to include new submodules
   - Update imports in files that use moved functions
   - Ensure proper visibility (pub(crate) for internal functions)

3. Update module documentation:
   - Document the new module structure
   - Explain the separation of concerns
   - Provide examples of which module handles which functionality

4. Add implementation location for refactoring phase:
   - Primary: `crates/reev-core/src/prompt_processor/mod.rs` (breakdown and updates)
   - Secondary: All new modules in `crates/reev-core/src/prompt_processor/` directory
   - Tertiary: Update import statements across the codebase



## Phase 7: Testing and Verification

### Purpose
Ensure the new structured max amount calculation system works correctly.

### Tasks
1. Update existing tests in `prompt_processor_tests.rs` to match new implementation:
   - Update `test_all_keyword_processing` to verify structured max amounts
   - Extend `test_structured_response_all_keyword` to validate correct max amounts for each action type
   - Update `test_all_keyword_with_typos` to ensure max amounts are calculated correctly despite typos
   - Enhance `test_spl_token_extraction` to verify max amounts for SPL tokens
   - Tests that fail due to implementation changes should be updated, not worked around

2. Add new test cases for structured max amount calculation:
   - Test max amount calculation for each action type (transfer, swap, lend, borrow)
   - Verify different fees are applied correctly for each action type
   - Test max amounts with multiple token types
   - Validate proper handling of zero balance tokens

3. Test refactored module structure:
   - Verify all moved functions still work correctly
   - Ensure public API remains unchanged
   - Test visibility boundaries between modules
   - Validate imports are correct in all files

4. Add specific test examples to validate the structured approach:
   ```rust
   #[rstest]
   #[case("transfer all SOL", PromptAction::Transfer)]
   #[case("swap all SOL for USDC", PromptAction::Swap)]
   #[case("lend all USDC", PromptAction::Lend)]
   #[case("borrow all USDT", PromptAction::Borrow)]
   async fn test_structured_max_amounts_by_action(
       #[case] prompt: &str,
       #[case] expected_action: PromptAction,
   ) -> Result<()>
   ```
   
   ```rust
   #[rstest]
   #[case("transfer all SOL", "transfer", "SOL")]
   #[case("swap all USDC for SOL", "swap", "USDC")]
   #[case("lend all SOL", "lend", "SOL")]
   async fn test_correct_fee_application(
       #[case] prompt: &str,
       #[case] action: &str,
       #[case] token: &str,
   ) -> Result<()>
   ```

4. Test edge cases:
    - Insufficient balance scenarios
    - Tokens not in wallet
    - Multiple tokens with same symbol
    - Verify max amounts are calculated correctly for all supported tokens

5. Verify performance improvements:
    - Reduced execution time by eliminating string matching
    - Lower memory usage with structured approach
    - Measure before/after metrics for prompt processing

6. Implementation-first approach:
    - Tests must follow code changes, not the other way around
    - No need to preserve legacy behavior
    - Focus on correct implementation, not compatibility
    - Tests that fail due to implementation changes should be updated, not worked around
    - Tests that validate incorrect behavior should be fixed or removed

### Implementation Location
- Primary: `crates/reev-core/tests/prompt_processor_tests.rs` (extend existing tests)
- Secondary: Create new module `crates/reev-core/tests/prompt_processor_max_tests.rs` for new specific tests
- Tertiary: Create `crates/reev-core/tests/prompt_processor_refactor_tests.rs` for refactoring validation

## Success Criteria

1. ✅ Elimination of rule-based max amount calculation
2. ✅ Reduced code complexity in prompt processor
3. ✅ Centralized fee structure management
4. ✅ Improved extensibility for new operation types
5. ✅ Consistent max amount calculation across all operations
6. ✅ Comprehensive test coverage for all operation types
7. ✅ Performance improvements in prompt processing
8. ✅ Prompt processor mod.rs file size reduced to under 512 lines
9. ✅ Clear separation of concerns with dedicated modules
10. ✅ Removal of TransferAmountRefinementRequest and other legacy components
11. ✅ All tests updated to match new implementation where needed
12. ✅ No new code added to the already oversized mod.rs file
13. ✅ Tests validating incorrect legacy behavior have been fixed or removed

## Timeline

- Phase 1-2: 2-3 days (Max Amount Calculator)
- Phase 3-4: 2-3 days (Update Prompts)
- Phase 5: 2-3 days (Validation Logic)
- Phase 6: 3-4 days (Refactor Module Structure)
- Phase 7: 2-3 days (Testing and Verification)
- Total: 11-16 days

## Dependencies

- Access to wallet balance API for testing
- Completion of current structured response implementation
- Production environment for monitoring setup

## Next Steps

1. Implement the MaxAmountCalculator module
2. Update prompt processor to use structured calculations
3. Modify system prompts to include max amounts
4. Add comprehensive validation
5. Refactor prompt_processor module structure
6. Remove TransferAmountRefinementRequest and legacy components
7. Create test cases for all operation types
8. Update documentation

## Conclusion

The structured max amount calculation system provides a more reliable, extensible foundation for handling "all" keyword operations. By calculating all max amounts upfront and including them in the structured prompt, we eliminate brittle rule-based parsing and ensure consistent behavior across all operation types. This approach aligns with the V3 architecture and provides a solid foundation for future enhancements.
