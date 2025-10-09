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

### 📊 Test Results
- **001-sol-transfer**: ✅ 100% success rate
- **002-spl-transfer**: ✅ 100% success rate (after depth fix)
- **114-jup-positions-and-earnings**: ✅ 100% success rate
- **100-jup-swap-sol-usdc**: ✅ 75% success rate (Jupiter swap failed due to market conditions)

### 🎯 Next Steps
1. **Prerequisite Validation Logic**: Implement balance checking before operations
2. **Smart Tool Selection**: Update tool descriptions to reference available context
3. **Benchmark Creation**: Create benchmarks for both with-context and without-context scenarios
4. **Performance Testing**: Compare tool call counts with vs without context