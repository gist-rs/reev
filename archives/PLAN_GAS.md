# Plan for Standardizing Gas Reserve Calculation

## Implementation Status: ✅ COMPLETED

This document outlines a comprehensive plan to standardize gas reserve calculation across all components of the Reev system. The implementation has been completed as of December 2024, with all phases successfully implemented.

## Executive Summary

This document outlines a comprehensive plan to standardize gas reserve calculation across all components of the Reev system. Currently, gas reserve calculations are implemented inconsistently across multiple components, leading to potential user experience issues and difficult maintenance.

## Problem Analysis

### Current Issues

1. **Inconsistent Gas Reserve Values**:
   - `sol_transfer.rs` (execution tool): 5,000,000 lamports (0.05 SOL)
   - `MaxAmountCalculator`: 1,000,000 lamports (0.001 SOL) for transfer
   - `planner.rs`: 50,000,000 lamports (0.05 SOL) for swap
   - `query_handler.rs`: 1,000,000 lamports (0.001 SOL) default
   - `yml_generator/flow_builders.rs`: 50,000,000 lamports (0.05 SOL) for swap
   - `sol_transfer.rs` (protocol): 1,000,000 lamports (0.001 SOL)
   - Tests: Comments suggest 0.001 SOL (1,000,000 lamports)

2. **Duplicate Logic**:
   - Gas reserve calculation is implemented in multiple places
   - Different components have their own logic for calculating max transferable amounts
   - Some components check if balance > gas reserve, others do different logic

3. **Inconsistent Error Handling**:
   - Some components return half the balance if it's less than gas reserve
   - Others return 0
   - Some have special error messages, others don't

4. **Centralized Calculation Not Used Everywhere**:
   - `MaxAmountCalculator` exists and has standardized fees
   - `MaxAmountCalculator::calculate_max_transferable_amount` exists
   - `transfer_utils::calculate_max_transferable_amount` exists
   - But many components still implement their own logic

### Why This Matters

1. **User Experience**: Inconsistent gas reserves can lead to:
   - Failed transactions when expected transfer amount is too high
   - Surprising "all" keyword results that don't transfer maximum possible
   - Inconsistent behavior between different operation types

2. **Maintenance Burden**:
   - Updating gas reserve values requires changes in multiple files
   - Code duplication increases bug potential
   - Different logic paths need separate testing and debugging

3. **System Reliability**:
   - Inconsistent error handling makes the system unpredictable
   - Different components have different edge case handling
   - Potential security issues if some components don't properly validate

## Proposed Solution

### Phase 1: Create Standardized Gas Reserve Module

**Where**: `crates/reev-core/src/gas_reserve.rs`

**Why**: Create a single source of truth for gas reserve calculations that all components can use

**How**:

1. **Define gas reserve constants based on actual transaction costs**:
   ```rust
   pub const TRANSFER_GAS_RESERVE: u64 = 1_000_000; // 0.001 SOL
   pub const SWAP_GAS_RESERVE: u64 = 5_000_000; // 0.005 SOL
   pub const DEFAULT_GAS_RESERVE: u64 = 2_000_000; // 0.003 SOL
   ```

2. **Create utility functions**:
   ```rust
   /// Get gas reserve for a specific action
   pub fn get_gas_reserve_for_action(action: PromptAction) -> u64
   
   /// Calculate max transferable amount for SOL
   pub fn calculate_max_sol_transferable(balance: u64, gas_reserve: Option<u64>) -> u64
   
   /// Calculate max transferable amount for SPL tokens
   pub fn calculate_max_spl_transferable(balance: u64, gas_reserve: Option<u64>) -> u64
   
   /// Calculate max amount for "all" keyword handling
   pub fn calculate_amount_for_all_keyword(
       action: PromptAction, 
       sol_balance: u64, 
       token_balances: &HashMap<String, TokenBalance>
   ) -> HashMap<String, f64>
   ```

3. **Implement consistent error handling**:
   ```rust
   /// Standardized error message for insufficient balance
   pub fn insufficient_balance_error(balance: u64, gas_reserve: u64) -> String
   ```

### Phase 2: Update MaxAmountCalculator

**Where**: `crates/reev-core/src/prompt_processor/max_amount_calculator.rs`

**Why**: Ensure MaxAmountCalculator uses the same values as other components

**How**:
1. Import the gas reserve constants
2. Update default fees to match the constants
3. Ensure consistency between MaxAmountCalculator and the new gas reserve module
4. Update existing functions to use the new utility functions

### Phase 3: Update Execution Tools

**Where**: `crates/reev-core/src/execution/rig_agent/tools/sol_transfer.rs` and `jupiter_swap.rs`

**Why**: Remove inline gas reserve calculations and use standardized values

**How**:

1. **For sol_transfer.rs**:
   - Remove inline gas reserve calculation (lines 28-39)
   - Import and use `calculate_max_sol_transferable` from gas_reserve module
   - Update error handling to use standardized error message

2. **For jupiter_swap.rs**:
   - Ensure gas reserve is accounted for in "all" keyword cases
   - Use `get_gas_reserve_for_action(PromptAction::Swap)` instead of any hard-coded values
   - Add proper logging for gas reserve deduction

### Phase 4: Update Planning Components

**Where**: `crates/reev-core/src/planner.rs` and `crates/reev-core/src/yml_generator/flow_builders.rs`

**Why**: Remove hard-coded gas reserve values and use standardized constants

**How**:

1. **For planner.rs**:
   - Replace `let gas_reserve_lamports = 50_000_000u64` with `get_gas_reserve_for_action(PromptAction::Swap)`
   - Update calculations in `create_swap_flow` function
   - Ensure ground truth calculations use standardized values

2. **For flow_builders.rs**:
   - Replace hard-coded `gas_reserve_lamports = 50_000_000u64` with constant import
   - Update `build_swap_flow` function
   - Ensure all calculations use the same gas reserve value

### Phase 5: Update Protocol Implementations

**Where**: `crates/reev-protocols/src/native/sol_transfer.rs` and `crates/reev-core/src/query_handler/mod.rs`

**Why**: Ensure protocol layer uses consistent gas reserve values

**How**:

1. **For sol_transfer.rs** (protocol):
   - Replace `let gas_reserve = 1_000_000u64` with `TRANSFER_GAS_RESERVE` constant
   - Use `insufficient_balance_error` for consistent error messages

2. **For query_handler.rs**:
   - Replace `gas_reserve.unwrap_or(1_000_000)` with proper default based on action
   - Use `get_gas_reserve_for_action` to determine appropriate gas reserve

### Phase 6: Fix Test Cases and Comments

**Where**: All test files that reference gas reserves

**Why**: Ensure tests use standardized values and are consistent with implementation

**How**:

1. **Update test files**:
   - Fix hardcoded values in comments (e.g., 0.05 SOL → 0.001 SOL)
   - Update test expectations to match new standardized values
   - Ensure tests cover edge cases handled by new utility functions

2. **Update documentation**:
   - Fix comments that reference outdated gas reserve values
   - Add references to new gas_reserve module
   - Document the new approach

### Phase 7: Add Comprehensive Tests

**Where**: `crates/reev-core/tests/gas_reserve_tests.rs`

**Why**: Ensure gas reserve calculations work correctly in all scenarios

**How**:

1. **Create tests for utility functions**:
   - Test all gas reserve constants
   - Verify `get_gas_reserve_for_action` returns correct values
   - Test `calculate_max_sol_transferable` and `calculate_max_spl_transferable`

2. **Test edge cases**:
   - Zero balance
   - Balance less than gas reserve
   - Balance exactly equal to gas reserve
   - Very large balances

3. **Integration tests**:
   - Test that all components use the new standardized values
   - Verify consistent behavior across different operation types
   - Test error handling for insufficient balance scenarios

## Implementation Benefits

1. **Consistency**: All components will use the same gas reserve values
2. **Maintainability**: Single source of truth for gas reserve calculations
3. **Reliability**: Standardized error handling for edge cases
4. **Extensibility**: Easy to update gas reserves for all operations in one place
5. **Testing**: Centralized logic is easier to test thoroughly

## Success Criteria

1. All components use gas reserve constants from the centralized module
2. No duplicate gas reserve calculation logic in the codebase
3. Consistent error handling for insufficient balance scenarios
4. All existing tests pass with updated values
5. New comprehensive tests cover edge cases
6. Documentation explains the new approach

## Potential Challenges and Mitigations

1. **Challenge**: Some components might need different gas reserves for different scenarios
   **Mitigation**: Include optional parameters in utility functions to allow customization

2. **Challenge**: Gas requirements might change based on network conditions
   **Mitigation**: Design module to be easily updatable with dynamic values in the future

3. **Challenge**: Existing tests might fail due to changed values
   **Mitigation**: Carefully update all test expectations and add clear documentation

## Implementation Order

1. ✅ Create `gas_reserve.rs` module with constants and utility functions
2. ✅ Update `MaxAmountCalculator` to use the new module
3. ✅ Update execution tools (`sol_transfer.rs`)
4. ✅ Update planning and generation components
5. ✅ Update protocol implementations
6. ✅ Fix test cases and comments
7. ✅ Add comprehensive tests for new module

## Implementation Details

### Created Files:
- `crates/reev-core/src/gas_reserve/mod.rs` - Centralized gas reserve module with constants and utility functions
- `crates/reev-core/tests/gas_reserve_tests.rs` - Comprehensive test suite with 9 tests

### Updated Files:
- `crates/reev-core/src/prompt_processor/max_amount_calculator.rs` - Now uses standardized constants
- `crates/reev-core/src/execution/rig_agent/tools/sol_transfer.rs` - Now uses standardized gas reserve calculation
- `crates/reev-core/src/planner.rs` - Now uses standardized gas reserve for swaps
- `crates/reev-core/src/yml_generator/flow_builders.rs` - Now uses standardized gas reserve
- `crates/reev-protocols/src/native/sol_transfer.rs` - Now uses standardized gas reserve value
- `crates/reev-core/src/query_handler/mod.rs` - Now uses standardized default gas reserve
- `crates/reev-core/src/utils/transfer_utils.rs` - Now delegates to gas_reserve module functions
- `crates/reev-core/src/lib.rs` - Added gas_reserve module export

## Future Enhancements

1. **Dynamic Gas Reserve Calculation**:
   - Based on current network congestion
   - Different values for different transaction types
   - Automatic adjustment based on historical transaction costs

2. **Gas Reserve Configuration**:
   - Allow users to set custom gas reserve preferences
   - Support for different risk levels (conservative, normal, aggressive)
   - API endpoints to update gas reserve values without code deployment

3. **Advanced Gas Optimization**:
   - Priority fee estimation
   - Dynamic fee calculation based on transaction urgency
   - Gas reserve recommendations for different operation types

## Test Coverage

The implementation includes comprehensive test coverage in `crates/reev-core/tests/gas_reserve_tests.rs`:

1. **Constants Tests**: Verify all gas reserve constants are correctly defined
2. **Function Tests**: Test all utility functions with various inputs and edge cases
3. **Integration Tests**: Verify compatibility with existing components
4. **Error Handling Tests**: Test error message generation and edge cases
5. **"All" Keyword Tests**: Test calculation for "all" keyword handling
6. **SOL vs SPL Token Tests**: Verify different behavior for SOL and SPL tokens

All tests are passing, confirming the implementation meets the requirements.
