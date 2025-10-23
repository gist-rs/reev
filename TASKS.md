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

### #003: Balance Context Missing Issue - CRITICAL
**Problem**: Prompt says "Avoid unnecessary balance checks since information is already provided" but no balance info is actually provided
- **Error**: Context only shows account keys, not actual SOL/token holdings
- **Root Cause**: Runner not passing `initial_state` from benchmark to agent context
- **Impact**: BREAKS ALL OPERATIONS - agents make blind decisions, violate prompt instructions
- **Symptoms**: 
  - Agent told to avoid balance checks but has no balance information
  - Context only shows wallet pubkeys, no amounts/token holdings
  - Forces agent to make blind decisions or violate prompt instructions
  - Benchmark has balance data but context builder ignores it
  - Will cause failures in token swaps, lending operations, insufficient funds scenarios
- **Status**: 🚨 CRITICAL - SOL transfers work by workaround, but ALL other operations will fail
- **Priority**: 🔥 URGENT - Must fix before any other benchmarks
- **Solution**: Fix runner to pass `initial_state` data to agent for proper context building
- **Implementation**:
  - Find where runner creates LlmRequest payload
  - Ensure benchmark.initial_state is properly passed to payload.initial_state
  - Test with SOL transfers to verify balance context appears
  - Validate token operations work with proper balance context

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