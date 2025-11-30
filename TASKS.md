# Task: Implement calculate_max_transferable_amount Function

## Current State Analysis

The transfer functionality has been successfully implemented with the following components:

1. `calculate_max_transferable_amount` function is fully implemented in `crates/reev-core/src/utils/transfer_utils.rs`
2. `PromptProcessor` (renamed from LanguageRefiner) handles "all" keyword detection and processing
3. A new `QueryHandler` provides a high-level interface for processing user queries end-to-end
4. Test cases have been updated to use the new QueryHandler for testing transfer functionality

## Implementation Status

### ✅ Completed Components

1. **calculate_max_transferable_amount Function**
   - Location: `crates/reev-core/src/utils/transfer_utils.rs`
   - Handles SOL transfers with gas reserve calculation
   - Returns 0 if insufficient balance
   - Includes unit tests for various scenarios

2. **PromptProcessor Module**
   - Location: `crates/reev-core/src/prompt_processor/mod.rs`
   - Detects "all" keyword (and typos like "alll", "allll")
   - Calculates max transferable amount when "all" is detected
   - Sends calculated amount as context to LLM for processing
   - LLM handles both typo correction and amount replacement

3. **QueryHandler Module**
   - Location: `crates/reev-core/src/query_handler/mod.rs`
   - Provides high-level interface for processing user queries
   - Can be used by tests, API endpoints, and CLI tools
   - Handles complete flow from prompt to execution

4. **Updated Test Cases**
   - Location: `crates/reev-core/tests/e2e_transfer.rs`
   - Tests specific amounts (e.g., "send 1.5 sol")
   - Tests "all" keyword processing
   - Verifies correct amount is transferred

## How the System Works

When a user submits a query like "send allll sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq":

1. QueryHandler receives the prompt and wallet pubkey
2. PromptProcessor detects "all" keyword (or typos)
3. PromptProcessor calls `calculate_max_transferable_amount` (e.g., 5 SOL - 0.001 SOL = 4.999 SOL)
4. PromptProcessor sends context: "Transferable amount: 4.999 SOL. send allll sol to..."
5. LLM processes and refines to: "send 4.999 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
6. YmlGenerator creates a transfer step with the calculated amount
7. ToolExecutor executes the transfer with the specified amount

## Benefits of This Implementation

1. ✅ Clean separation of concerns: Calculation logic is isolated in transfer_utils
2. ✅ Reusable across different parts of the system
3. ✅ Properly handles typos like "alll", "allll" through LLM processing
4. ✅ Consistent with V3 architecture where PromptProcessor handles calculations
5. ✅ Simplifies ToolExecutor to just execute transfers with specified amounts
6. ✅ Provides a high-level QueryHandler interface for easy integration

## Current Status

- ✅ **COMPLETED**: "send all sol" transfers now work correctly
- ✅ **COMPLETED**: System handles typos like "alll", "allll"
- ✅ **COMPLETED**: Tests verify end-to-end functionality
- ✅ **COMPLETED**: Added reset_wallet_balance method to TestRunner for test consistency
- ✅ **COMPLETED**: Fixed test balance verification to use direct blockchain queries
- 🔄 **IN PROGRESS**: SPL token support (currently uses same logic as SOL)
- ⏸️ **TODO**: Dynamic gas reserve calculation based on transaction type

## Recent Fixes (2025-11-30)

1. **Added reset_wallet_balance method**: 
   - Added to TestRunner in `common/framework/mod.rs`
   - Uses SurfpoolClient.set_account() to reset wallet balance to 5 SOL
   - Ensures consistent test state across test runs

2. **Fixed test balance verification**:
   - Replaced QueryHandler.get_wallet_balance() with direct blockchain queries
   - This ensures accurate balance tracking before and after transfers
   - Fixed issue where test showed 0 SOL transferred instead of 4.999 SOL

## Next Steps

1. Add support for different SPL token types with appropriate gas reserves
2. Implement dynamic gas reserve calculation based on transaction complexity
3. Optimize prompt context for better LLM processing of edge cases
