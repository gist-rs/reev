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

### #002: Balance Context Missing Issue - FIXED ✅
**Problem**: Prompt says "Avoid unnecessary balance checks since information is already provided" but no balance info is actually provided
- **Error**: Context only shows account keys, not actual SOL/token holdings
- **Root Cause**: Context builder was parsing benchmark YAML instead of querying surfpool RPC state
- **Impact**: Was breaking all operations - agents made blind decisions, violating prompt instructions
- **Symptoms**: 
  - Agent told to avoid balance checks but had no balance information
  - Context showed 0.0000 SOL instead of expected 1.0 SOL from benchmark
  - Forces agent to make blind decisions or violate prompt instructions
  - Benchmark has balance data but context builder ignored it
  - Will cause failures in token swaps, lending operations, insufficient funds scenarios
- **Status**: ✅ FIXED - Context builder now uses real observation state from surfpool
- **Priority**: ✅ RESOLVED
- **Solution**: Added observation-based context building instead of YAML parsing
- **Implementation**:
  - ✅ Added `build_context_from_observation()` method to ContextBuilder
  - ✅ Added `build_enhanced_prompt_from_observation()` to ContextIntegration
  - ✅ Updated AgentHelper to use account_states when available (falls back to initial_state)
  - ✅ Modified LlmRequest to include account_states field
  - ✅ Updated llm_agent to pass observation.account_states in request payload
  - ✅ Added unit test verifying real balances appear in context (1.0000 SOL, 50 USDC)
  - ✅ Context now shows actual surfpool state instead of parsed YAML values
- **Files Modified**:
  - `crates/reev-agent/src/context/mod.rs` - Added observation-based context building
  - `crates/reev-agent/src/context/builder.rs` - Added observation context methods
  - `crates/reev-agent/src/context/integration.rs` - Added observation prompt building
  - `crates/reev-agent/src/enhanced/common/mod.rs` - Updated to use observation when available
  - `crates/reev-agent/src/lib.rs` - Added account_states to LlmRequest
  - `crates/reev-lib/src/llm_agent.rs` - Pass observation account_states to agent
- **Result**: Context now shows real balances (1.0000 SOL, 50 USDC) from surfpool state

### #005: Recipient Account Missing Issue - FIXED ✅
**Problem**: Recipient accounts with 0 lamports not appearing in context, causing agent confusion
- **Error**: Agent sees RECIPIENT_WALLET_PUBKEY as placeholder instead of resolved address
- **Root Cause**: Accounts with 0 lamports don't exist on-chain, so get_account() fails and they're excluded from observation
- **Symptoms**:
  - Context only shows USER_WALLET_PUBKEY, missing RECIPIENT_WALLET_PUBKEY
  - Agent asks for "actual recipient address" instead of using resolved address
  - Benchmarks 001-004 fail with "Agent returned no actions to execute"
- **Status**: ✅ FIXED - Context now includes resolved addresses and clarity notes
- **Priority**: High - Affects basic transfer benchmarks
- **Solution**: 
  - Add missing accounts from initial_state to observation even if they don't exist on-chain
  - Include resolved addresses section in context for clarity
  - Add explicit note about placeholder resolution in agent prompt
- **Implementation**:
  - ✅ Updated observation.rs to include non-existent accounts with 0 lamports
  - ✅ Added resolved addresses section to formatted context
  - ✅ Added clarity note about placeholder resolution in agent prompt
  - ✅ Fixed System Program constant reference
  - ✅ Context now shows: "📋 RESOLVED ADDRESSES: RECIPIENT_WALLET_PUBKEY → 9SEmW...ifcp"
- **Files Modified**:
  - `crates/reev-lib/src/solana_env/observation.rs` - Include missing accounts
  - `crates/reev-agent/src/context/builder.rs` - Improve context building
  - `crates/reev-agent/src/context/mod.rs` - Add resolved addresses section
  - `crates/reev-agent/src/context/integration.rs` - Add clarity note in prompt
- **Result**: Agent should now understand placeholders are resolved and execute transfers properly

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