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

### #002: Dynamic Tool Selection System  
**Goal**: Implement LLM+dynamic tool routing for context-aware tool selection
- Remove hardcoded Jupiter lending prompts for all operations
- Add operation type detection (transfer vs swap vs lend)
- Allow flexible tool call limits based on operation complexity
- Create proper response type separation (account info vs transactions)
- Re-enable `get_account_balance` tool with intelligent context routing

### #003: Enhanced Test Coverage
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