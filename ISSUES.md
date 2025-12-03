# Reev Project Issues

## Current Issues

### Issue #311: SPL Lending Tests Implementation
**Status**: Completed  
**Priority**: Medium  
**Description**: End-to-end lending tests were only covering native SOL deposits/withdrawals. We needed to add comprehensive SPL token lending tests to ensure the system works correctly with Jupiter's lending protocol for SPL tokens.

**Solution Implemented**:
1. Added a new test function `test_spl_lend` to `crates/reev-core/tests/e2e_lend.rs`
2. The test covers both deposit and withdraw operations with USDC tokens
3. Implemented proper token balance setup using `set_token_balance` with USDC mint address
4. For withdraw tests, added logic to deposit first to ensure sufficient lending balance
5. Added comprehensive error handling similar to the existing SOL lending tests
6. Included detailed logging for debugging and monitoring

**Files Modified**:
- `crates/reev-core/tests/e2e_lend.rs` - Added `test_spl_lend` function for USDC lending tests

**Testing**:
- All tests pass successfully with "4 passed; 0 failed"
- Tests cover both "deposit 50 usdc to jupiter lend" and "withdraw 25 usdc from jupiter lend"
- Verified proper token balance setup before operations
- Confirmed error handling works correctly for both successful and failed operations

**Result**: SPL token lending operations are now fully tested with proper end-to-end coverage. This ensures the system correctly handles Jupiter lending protocol operations for SPL tokens, not just native SOL.

### Issue #310: Missing ATA Creation for Recipient in SPL Transfer
**Status**: Fixed  
**Priority**: High  
**Description**: When transferring SPL tokens (like USDT) to a recipient address that doesn't have an associated token account (ATA), the system was failing with an "InvalidAccountData" error. This was happening because the system was trying to transfer to an ATA that didn't exist.

**Root Cause**: The `get_or_create_token_accounts` function in `crates/reev-core/src/execution/rig_agent/tools/spl_transfer.rs` was only calculating the recipient's ATA address but not actually creating it if it didn't exist. This worked for USDC transfers because the recipient's ATA had already been created in previous tests, but failed for new recipients like in the USDT test case.

**Solution Implemented**:
1. Added logic to check if the recipient's ATA exists using `rpc_client.get_account(&recipient_ata).await.is_ok()`
2. If the ATA doesn't exist, create it using:
   - `spl_associated_token_account::instruction::create_associated_token_account`
   - Execute the instruction with `reev_lib::execute_transaction(vec![create_ata_ix], keypair.pubkey(), &keypair)`
3. Fixed type conversion issues between `Instruction` and `RawInstruction` by using `.into()`
4. Added the `spl-token` dependency to the `Cargo.toml` file for `reev-core`

**Files Modified**:
- `crates/reev-core/src/execution/rig_agent/tools/spl_transfer.rs` - Added ATA creation logic
- `crates/reev-core/Cargo.toml` - Added `spl-token` dependency

**Testing**:
- Verified that "send 1 usdc to [address]" works correctly
- Verified that "send all usdc to [address]" works correctly
- Verified that "transfer 0.1 usdt to [address]" now works correctly after creating the recipient's ATA
- All tests are now passing

**Result**: SPL transfers now work correctly for any recipient address, automatically creating the necessary ATA if it doesn't exist. This ensures a smooth user experience without requiring manual ATA creation steps.
