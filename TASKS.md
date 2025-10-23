# TASKS.md

## Current Issues

### #001: SOL Transfer Agent Issue
**Problem**: Agent fails to execute native SOL transfers, stops after balance check
- **Error**: `Agent returned no actions to execute`
- **Root Cause**: Jupiter lending prompt confusion for native transfers
- **Symptoms**: 
  - Agent calls `get_account_balance` (tool 1/2)
  - Stops due to "STOP after discovery tool" rule
  - Returns balance info instead of proceeding with transfer
- **Status**: ✅ FIXED - Commented out `get_account_balance` tool
- **Priority**: High
- **Solution**: Temporarily disabled balance checking tool to allow direct SOL transfers
- **Result**: Score improved from 0% to 100% - SOL transfers now work correctly

## Next Tasks

### #002: Balance Context Missing Issue - CRITICAL
**Problem**: Prompt says "Avoid unnecessary balance checks since information is already provided" but no balance info is actually provided
- **Error**: Context only shows account keys, not actual SOL/token holdings
- **Root Cause**: Wrong approach - should use surfpool RPC to set state then query it back
- **Impact**: BREAKS ALL OPERATIONS - agents make blind decisions, violate prompt instructions
- **Symptoms**: 
  - Agent told to avoid balance checks but has no balance information
  - Context shows 0.0000 SOL instead of expected 1.0 SOL from benchmark
  - Forces agent to make blind decisions or violate prompt instructions
  - Benchmark has balance data but context builder ignores it
  - Will cause failures in token swaps, lending operations, insufficient funds scenarios
- **Status**: 🚨 CRITICAL - Need proper surfpool state setup and query
- **Priority**: 🔥 URGENT - Must fix before any other benchmarks
- **Solution**: Use surfpool RPC cheats to set state, then query real state for context
- **Implementation**:
  - Parse ALL account types from benchmark's `initial_state`:
    - SOL accounts: Use `surfnet_setAccount` with lamports
    - Token accounts: Use `surfnet_setTokenAccount` with mint and amount
    - Program accounts: Use `surfnet_setAccount` with owner and data
    - Custom data accounts: Use `surfnet_setAccount` with full account data
  - Handle both placeholder pubkeys (USER_WALLET_PUBKEY) and literal pubkeys
  - Query current state from surfpool using RPC calls for agent context
  - Ensure context shows real SOL/token balances from surfpool state
  - Test with multiple benchmarks: SOL transfers, SPL transfers, Jupiter operations
  - Verify context matches exactly what's specified in benchmark YAML

### #003: Dynamic Tool Selection System (POSTPONED)
**Goal**: Implement LLM+dynamic tool routing for context-aware tool selection
- **Status**: 🚫 POSTPONED - Will address after context issues are fully resolved
- **Priority**: Low - Not relevant until proper balance context is working
- **Future Implementation**:
  - Remove hardcoded Jupiter lending prompts for all operations
  - Add operation type detection (transfer vs swap vs lend)
  - Allow flexible tool call limits based on operation complexity
  - Create proper response type separation (account info vs transactions)
  - Re-enable `get_account_balance` tool with intelligent context routing
 
### #004: Enhanced Test Coverage
**Goal**: Add comprehensive test cases for balance validation
- Create `010-sol-check-balance-transfer.yml` benchmark
- Test insufficient funds scenarios
- Validate proper balance check → transfer flow
- Cross-agent consistency testing

## Recent Fixes

### ✅ SOL Transfer Issue (#001) - FIXED
**Date**: 2025-10-23
**Changes Made**:
- Commented out `get_account_balance` tool in ZAI and OpenAI agents
- Added TODO comments for future re-enablement with proper context routing
- Verified fix works: SOL transfer score improved from 0% to 100%

**Files Modified**:
- `crates/reev-agent/src/enhanced/zai_agent.rs`
- `crates/reev-agent/src/enhanced/openai.rs`

## Implementation Plan

1. **Quick Fix**: Comment out `get_account_balance` tool to enable immediate SOL transfers
2. **Context Fix**: Separate Jupiter lending prompts from native transfer logic  
3. **Response Fix**: Add proper response type handling (account_info vs transactions)
4. **Dynamic System**: Implement intelligent tool selection based on user intent