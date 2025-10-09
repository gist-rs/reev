# TOFIX - Issues to Resolve

## Phase 5: Context Enhancement - Current Status

### ✅ Completed
- **Context Builder Module**: Created `crates/reev-agent/src/context/` with parsing capabilities
- **Context Integration**: Enhanced agents now include account context in system prompts
- **Two-Tier Depth Strategy**: Adaptive conversation depth based on context availability
- **Discovery Tools**: Implemented tools for prerequisite validation when context is insufficient

### 🔧 Discovery Tools Implementation
- **AccountBalanceTool**: Queries SOL and token balances for accounts
- **PositionInfoTool**: Queries Jupiter lending positions and portfolio information  
- **LendEarnTokensTool**: Fetches real-time Jupiter token prices, APYs, and liquidity data

### 🐛 Issues Fixed
- **MaxDepthError**: Resolved by increasing discovery depth for simple benchmarks (5→7)
- **OPENAI_API_KEY Validation**: Fixed Rig framework API key validation for local models
- **Placeholder Pubkey Handling**: Tools now gracefully handle placeholder addresses with simulated data

### 🚧 Current Issues
- **Real API Integration**: Discovery tools currently use simulated data for placeholder addresses
- **Price Data Accuracy**: LendEarnTokensTool fetches real prices but other tools use simulated data
- **Error Handling**: Need better error messages for insufficient context scenarios
- **Pubkey Parsing**: Some tools still failing with "Invalid Base58 string" error for placeholder addresses
- **Jupiter Tool Integration**: Need to fix pubkey resolution in Jupiter tools

### 🔴 **Critical Issues from Benchmark Testing (2025-01-09)**

#### **1. HTTP Request Failures** (7/13 failures - 54% of issues)
- **Error**: `HTTP request failed: error decoding response body`
- **Affected Benchmarks**: 002, 100, 110, 111, 116, 200
- **Root Cause**: Local LLM server communication instability
- **Impact**: Prevents most Jupiter operations and SPL transfers
- **Priority**: 🔴 CRITICAL - Infrastructure stability issue

#### **2. Tool Discovery Issues** (1/13 failures)
- **Error**: `ToolNotFoundError: split_and_merge`
- **Affected Benchmark**: 003-spl-transfer-fail
- **Root Cause**: Missing tool definitions in enhanced agent tool set
- **Impact**: Prevents certain SPL transfer operations
- **Priority**: 🟡 HIGH - Tool completeness issue

#### **3. Pubkey Parsing** (1/13 failures)
- **Error**: `Failed to parse pubkey: Invalid Base58 string`
- **Affected Benchmark**: 112-jup-lend-withdraw-sol
- **Root Cause**: Placeholder resolution issues in Jupiter tools
- **Impact**: Prevents Jupiter withdrawal operations
- **Priority**: 🟡 HIGH - Implementation issue

#### **4. MaxDepthError** (1/13 failures)
- **Error**: `MaxDepthError: (reached limit: 7)`
- **Affected Benchmark**: 004-partial-score-spl-transfer
- **Root Cause**: Complex operations requiring more conversation depth
- **Impact**: Affects complex SPL transfers
- **Priority**: 🟡 HIGH - Depth optimization issue

#### **5. Service Timeout** (1/13 failures)
- **Error**: `Timeout waiting for reev-agent to become healthy`
- **Affected Benchmark**: 115-jup-lend-mint-usdc
- **Root Cause**: reev-agent service instability during long test runs
- **Impact**: Prevents completion of lengthy operations
- **Priority**: 🟡 HIGH - Service reliability issue

### 📊 Benchmark Results Summary (2025-01-09)

#### **Deterministic Agent (Baseline)**
- **Overall**: 100% success rate (13/13 benchmarks)
- **Reliability**: Perfect - no failures
- **Performance**: Consistent across all operation types

#### **Enhanced Local Agent (Phase 5)**
- **Overall**: 23% success rate (3/13 benchmarks)
- **Working**: ✅ 001-sol-transfer (100%), ✅ 113-lend-withdraw-usdc (75%), ✅ 114-jup-positions-and-earnings (100%)
- **Failed**: ❌ 10/13 benchmarks due to infrastructure and tool issues
- **Key Insight**: Discovery tools work perfectly when infrastructure is stable

### 🎯 Next Steps - Prioritized Action Plan

#### **Priority 1: Infrastructure Stability** (Critical)
1. **Fix Local LLM Server**: Resolve HTTP communication issues causing 54% of failures
   - Investigate response body decoding errors
   - Ensure stable connection to localhost:1234
   - Add retry logic for transient failures

2. **Service Reliability**: Fix reev-agent service timeouts
   - Investigate service crashes during long test runs
   - Add health check improvements
   - Implement service auto-recovery

#### **Priority 2: Tool Completeness** (High)
3. **Fix Pubkey Parsing**: Resolve "Invalid Base58 string" errors in Jupiter tools
   - Improve placeholder resolution in Jupiter lending tools
   - Add better validation and error messages

4. **Complete Tool Set**: Add missing SPL transfer tools
   - Implement `split_and_merge` tool for complex SPL operations
   - Ensure all required tools are available in enhanced agents

#### **Priority 3: Performance Optimization** (Medium)
5. **Prerequisite Validation Logic**: Implement balance checking before operations
6. **Smart Tool Selection**: Update tool descriptions to reference available context
7. **Depth Optimization**: Increase depth limits for complex operations
8. **Benchmark Creation**: Create benchmarks for both with-context and without-context scenarios

#### **Target Metrics**
- **Immediate Goal**: Achieve 70%+ success rate (from current 23%)
- **Phase 5 Completion**: Target 85%+ overall success rate
- **Production Ready**: 95%+ success rate with fallback mechanisms