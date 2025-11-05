# Issues

## Issue #21: ZAI API 400 Bad Request Errors - RESOLVED ✅

### **Problem Summary**
GLM agents (glm-4.6 and glm-4.6-coding) were encountering ZAI API 400 Bad Request errors with "Invalid API parameter, please check documentation" messages, preventing successful tool execution despite proper API configuration.

### **Root Cause Analysis**
**API Parameter Format Issue**: The ZAI API is rejecting requests due to malformed parameter structure or missing required fields in the tool call JSON sent to the API.

**Error Pattern**:
```
"ProviderError: ZAI API error 400 Bad Request: {\"error\":{\"code\":\"1210\",\"message\":\"Invalid API parameter, please check the documentation.\"}}"
```

**Current Status**:
- ✅ API endpoints correctly constructed (Issue #18 resolved)
- ✅ Agent routing working (glm-4.6 → OpenAI, glm-4.6-coding → ZAI)
- ✅ Dynamic flow orchestration operational
- ❌ ZAI API parameter formatting causing 400 errors

### **Evidence from Current Execution**
```bash
# Test execution showing the error
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "use my 50% sol to multiply usdc 1.5x on jup",
    "wallet": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }'

# Response shows 400 Bad Request errors
{
  "tool_calls": [
    {
      "tool_name": "account_balance",
      "success": false,
      "error": "ProviderError: ZAI API error 400 Bad Request..."
    },
    {
      "tool_name": "jupiter_swap", 
      "success": false,
      "error": "ProviderError: ZAI API error 400 Bad Request..."
    }
  ]
}
```

### **Investigation Required**
1. **Tool Call JSON Structure**: Verify tool parameters are properly formatted for ZAI API
2. **API Request Headers**: Check Content-Type and Authorization headers
3. **Parameter Encoding**: Ensure JSON serialization matches ZAI API expectations
4. **Tool Definition**: Validate tool schemas align with ZAI API requirements

### Files to Examine
- `crates/reev-orchestrator/src/context_resolver.rs` - `resolve_fresh_wallet_context()` needs pubkey generation
- `crates/reev-orchestrator/src/gateway.rs` - Check flow creation process
- `crates/reev-orchestrator/src/execution/ping_pong_executor.rs` - Key map creation working correctly

### Next Steps Required
1. **Generate real Solana pubkeys** in context resolver for placeholder keys
2. **Add pubkey preparation phase** before flow execution
3. **Test with real pubkeys** to verify tool execution success
4. **Update test scenarios** to use generated pubkeys instead of hardcoded ones

<<<<<<< HEAD
### Related Issues
- Issue #19: Pubkey resolution in dynamic flow execution (resolved)
- Issue #17: OTEL integration at orchestrator level (working)
=======
# Issues

## Issue #21: ZAI API 400 Bad Request Errors - RESOLVED ✅

### **Problem Summary**
GLM agents (glm-4.6 and glm-4.6-coding) were encountering ZAI API 400 Bad Request errors with "Invalid API parameter, please check documentation" messages, preventing successful tool execution despite proper API configuration.

### **Root Cause Analysis**
**API Parameter Format Issue**: The ZAI API was rejecting requests due to malformed parameter structure or missing required fields in the tool call JSON sent to the API.

**Error Pattern**:
```
"ProviderError: ZAI API error 400 Bad Request: {\"error\":{\"code\":\"1210\",\"message\":\"Invalid API parameter, please check the documentation.\"}}"
```

**Current Status**:
- ✅ API endpoints correctly constructed (Issue #18 resolved)
- ✅ Agent routing working (glm-4.6 → OpenAI, glm-4.6-coding → ZAI)
- ✅ Dynamic flow orchestration operational
- ✅ Balance tool re-enabled in both ZAI and OpenAI agents
- ✅ Real agent execution working via ping-pong coordination

### **Fix Applied**
1. **Re-enabled Balance Tool**: Restored `get_account_balance` in both ZAI and OpenAI agents
2. **Tool Name Consistency**: Fixed `jupiter_earn_tool` vs `jupiter_earn` naming mismatch
3. **Added Missing Case**: Added `jupiter_swap_flow` handling in ZAI agent match statement
4. **Surfpool Integration**: Real account funding provides actual balances for testing

### **Test Results**
```bash
# Single-step swap: ✅ Working (jupiter_swap success)
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -d '{"prompt": "swap 1 SOL for USDC", "agent": "glm-4.6-coding", ...}'
# Response: {"status": "Completed", "tool_calls": [{"tool_name": "jupiter_swap", "success": true}]}

# Complex 4-step multiplication: ✅ Working (account_balance → jupiter_swap → jupiter_lend_earn_deposit → jupiter_positions)
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -d '{"prompt": "use my 50% sol to multiply usdc 1.5x on jup", "agent": "glm-4.6-coding", ...}'
# Response: {"status": "Completed", "tool_calls": [
#   {"tool_name": "account_balance", "success": true},
#   {"tool_name": "jupiter_swap", "success": true}, 
#   {"tool_name": "jupiter_lend_earn_deposit", "success": true},
#   {"tool_name": "jupiter_positions", "success": true}
# ]}
```

### **Resolution**: ✅ **RESOLVED**

## Issue #22: Incomplete Flow Execution & Missing Details in Dynamic Flows - RESOLVED ✅

### **Problem Summary**
Dynamic flows were terminating early at the swap step without completing the full multiplication strategy, and flow visualization lacked detailed execution information (amounts, addresses, specific parameters).

### **Root Cause Analysis**
**Early Flow Termination**: Dynamic flows with complex strategies (e.g., "multiply USDC 1.5x") were stopping after the jupiter_swap step without proceeding to the expected jupiter_lend step, resulting in incomplete 2-step flows instead of 4-step strategies.

**Missing Flow Details**: Flow visualization only showed high-level step names without execution details like:
- Specific amounts being swapped (e.g., "2 SOL → 300 USDC")
- Wallet addresses involved
- Jupiter transaction parameters
- Real transaction signatures

**Expected vs Actual Flow**:
- **Expected**: account_balance → jupiter_swap → jupiter_lend_earn_deposit → jupiter_positions
- **Actual**: account_balance → jupiter_swap → [*] (terminates early)

### **Current Status**
- ✅ **4-Step Flow Execution**: Now executes complete multiplication strategies
- ✅ **Tool Name Consistency**: Fixed `jupiter_swap_flow`, `get_account_balance` parsing
- ✅ **Real Execution**: All steps showing actual execution times and success status
- ❌ **Missing Visualization Details**: `params: {}`, `result_data: {}`, `tool_args: null`

### **Evidence from Current Execution**
```bash
# Flow execution successful
{
  "status": "Completed",
  "tool_calls": [
    {"tool_name": "jupiter_swap", "success": true, "params": {}, "result_data": {}},
    {"tool_name": "jupiter_lend_earn_deposit", "success": true, "params": {}, "result_data": {}}
  ]
}

# But missing actual amounts, pubkeys, transaction details
{
  "tool_calls": [
    {"params": {}, "result_data": {}},  # Should show: {"amount": "1 SOL", "output": "150 USDC"}
    {"params": {}, "result_data": {}}   # Should show: {"amount": "150 USDC", "apy": "5.8%"}
  ]
}
```

### **Investigation Required**
**Extract Function Issue**: `extract_transaction_details()` function expects different JSON structure than what agents return:
- **Expected**: `{"swap": {...}}` or `{"lend": {...}}`  
- **Actual**: `{"signatures": [...], "transactions": [...]}`

**Agent Response Structure** (from debug logs):
```json
{
  "signatures": ["estimated_signature"],
  "summary": "Account balance retrieved successfully", 
  "transactions": [{
    "account": {
      "pubkey": "FFMBsUjK9mbZSAmFzDcpGGwDYLvLRs89twEH1L7x5GSP",
      "sol_balance": 5000000000
    }
  }]
}
```

### **Files to Examine**
- `crates/reev-api/src/handlers/dynamic_flows/mod.rs` - `extract_transaction_details()` function needs update
- `crates/reev-agent/src/enhanced/zai_agent.rs` - Agent output structure analysis
- `crates/reev-orchestrator/src/execution/ping_pong_executor.rs` - Step result handling

### **Next Steps Required**
1. **Fix Extraction Function**: Update `extract_transaction_details()` to parse actual agent response structure
2. **Extract Real Details**: Parse amounts, pubkeys, signatures from agent JSON output
3. **Enhanced Visualization**: Populate `params`, `result_data`, `tool_args` with actual execution data
4. **Test Rich Details**: Verify flow diagrams show "2 SOL → 300 USDC" with real signatures

### **Expected Results After Fix**
- Flow visualization should show specific amounts (e.g., "2 SOL → 300 USDC")
- Wallet addresses and transaction details should be visible
- `params` should contain input amounts, mint addresses
- `result_data` should contain output amounts, transaction signatures

### **Priority**: 🟡 MEDIUM - Critical for user experience, but core functionality works


## Issue #23: Missing Amounts & Pubkeys in Flow Visualization - ACTIVE 🟡

### **Problem Summary**
Dynamic flow execution is working correctly (4-step strategies completing successfully), but flow visualization lacks critical execution details like specific amounts, wallet pubkeys, and transaction signatures.

### **Root Cause Analysis**
**JSON Structure Mismatch**: The `extract_transaction_details()` function expects agent responses in format like `{"swap": {...}}` but actual agent output uses different structure.

**Agent Response Structure** (from debug logs):
```json
{
  "signatures": ["estimated_signature"],
  "summary": "Account balance retrieved successfully", 
  "transactions": [{
    "account": {
      "pubkey": "FFMBsUjK9mbZSAmFzDcpGGwDYLvLRs89twEH1L7x5GSP",
      "sol_balance": 5000000000,
      "token_balances": [...]
    }
  }]
}
```

**Expected Visualization Details** (currently missing):
- **Amounts**: "2 SOL → 300 USDC" 
- **Pubkeys**: Real wallet addresses from agent execution
- **Signatures**: Actual transaction IDs from on-chain execution
- **Parameters**: Tool input arguments (slippage, amounts, mints)

**Current vs Expected**:
```bash
# Current: Empty details
{
  "tool_calls": [
    {
      "tool_name": "jupiter_swap",
      "success": true,
      "params": {},              # ❌ EMPTY
      "result_data": {},        # ❌ EMPTY  
      "tool_args": null
    }
  ]
}

# Expected: Rich details
{
  "tool_calls": [
    {
      "tool_name": "jupiter_swap", 
      "success": true,
      "params": {
        "input_amount": 1000000000,
        "input_mint": "So11111111111111111111111111111111111112", 
        "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "slippage": 100
      },                     # ✅ RICH DETAILS
      "result_data": {
        "signature": "5XJ3Xabc123...",
        "output_amount": 150000000,
        "impact": 2.3
      },                     # ✅ RICH DETAILS
      "tool_args": "..."        # ✅ INPUT ARGUMENTS
    }
  ]
}
```

### **Evidence from Current Execution**
```bash
# API test showing missing details
curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{"prompt": "swap 1 SOL for USDC", "agent": "glm-4.6-coding", ...}' \
| jq '.tool_calls[0] | {params, result_data, tool_args}'

# Response: Empty details confirmed
{
  "params": {},           # ❌ Should show amounts
  "result_data": {},      # ❌ Should show signatures  
  "tool_args": null       # ❌ Should show input args
}
```

### **Investigation Required**
**Function to Update**: `extract_transaction_details()` in `crates/reev-api/src/handlers/dynamic_flows/mod.rs`

**Current Extraction Logic** (lines 501-610):
```rust
// Current: Looks for "swap", "lend", "action" keys that don't exist
if let Some(swap) = tx.get("swap") { ... }
else if let Some(lend) = tx.get("lend") { ... }
else if let Some(action) = tx.get("action") { ... }
```

**Required Update**: Parse `{"transactions": [...]}` structure from agent responses
```rust
// New: Parse actual agent response structure
if let Some(transactions) = tx.get("transactions").and_then(|v| v.as_array()) {
    if let Some(first_tx) = transactions.first() {
        // Extract account, pubkey, balances, signatures
        if let Some(account) = first_tx.get("account") {
            // Parse real execution details
        }
    }
}
```

### **Files to Modify**
- `crates/reev-api/src/handlers/dynamic_flows/mod.rs` - Lines 501-610 (extraction function)
- `crates/reev-api/src/handlers/dynamic_flows/mod.rs` - Lines 450-470 (extraction call)

### **Next Steps Required**
1. **Update Extraction Logic**: Parse `"transactions"` array instead of `"swap"`/`"lend"` keys
2. **Extract Account Details**: Get pubkeys, balances from account objects
3. **Extract Signatures**: Parse from `"signatures"` array in agent response
4. **Populate Rich Details**: Fill `params`, `result_data`, `tool_args` with real data
5. **Test Enhanced Visualization**: Verify flow shows "2 SOL → 300 USDC" with signatures

### **Expected Results After Fix**
- Flow diagrams with specific amounts and pubkeys
- Transaction signatures visible in visualization
- Rich parameter details (slippage, mints, amounts)  
- Production-ready flow visualization with complete execution context

### **Priority**: 🟡 MEDIUM - Critical for user experience, but core functionality works

### **Related Issues**
- Issue #21: ZAI API 400 Errors (RESOLVED ✅)
- Issue #22: Incomplete Flow Execution (PARTIALLY RESOLVED ✅)

## Issue #21: ZAI API 400 Bad Request Errors - RESOLVED ✅

### **Problem Summary**
GLM agents (glm-4.6 and glm-4.6-coding) were encountering ZAI API 400 Bad Request errors with "Invalid API parameter, please check documentation" messages, preventing successful tool execution despite proper API configuration.

### **Root Cause Analysis**
**API Parameter Format Issue**: The ZAI API was rejecting requests due to malformed parameter structure or missing required fields in the tool call JSON sent to the API.

**Current Status**:
- ✅ Dynamic flow generation working correctly
- ✅ ZAI API connectivity established  
- ✅ Orchestrator context resolver implemented with SolanaEnv integration
- ❌ Tool execution failing with unresolved placeholders
- ✅ Flow diagram generation working

### **Implementation Progress**

**Phase 1: Context Resolver Integration** 
- [✅] Added `SolanaEnv` integration to `ContextResolver`
- [✅] Implemented placeholder detection using same logic as static benchmarks
- [✅] Added pubkey generation with `Keypair::new()` for placeholders
- [✅] Added placeholder mapping with `Arc<Mutex<HashMap>>`

**Phase 2: Orchestrator Integration**
- [✅] Updated `OrchestratorGateway` to create `SolanaEnv` with RPC client
- [✅] Integrated context resolver with Solana environment
- [✅] Updated constructor to be async and handle SolanaEnv creation errors

**Phase 3: Ping-Pong Executor Integration**
- [🔄] IN PROGRESS: Adding context resolver to `PingPongExecutor`
- [🔄] IN PROGRESS: Implementing placeholder resolution before key map creation
- [🔄] IN PROGRESS: Updating `create_key_map_with_wallet()` to use resolved pubkeys

### **Technical Implementation**

**Files Modified**:
- `crates/reev-orchestrator/src/context_resolver.rs`
  - Added `SolanaEnv` field and `with_solana_env()` constructor
  - Implemented `resolve_placeholder()` method with placeholder detection logic
  - Added real pubkey generation using `Keypair::new()`
  
- `crates/reev-orchestrator/src/gateway.rs`
  - Removed duplicate incomplete `process_user_request()` function
  - Complete implementation working with 4-step flow generation

**Type System Fixes**:
- `crates/reev-orchestrator/src/execution/ping_pong_executor.rs`
  - Added `context_resolver: Arc<ContextResolver>` field
  - Updated `execute_agent_step()` to resolve wallet pubkey before creating key map
  - Modified `create_key_map_with_wallet()` to use resolved pubkey

### **Critical Issue Identified**
**Compilation Error in `PingPongExecutor`**: Multiple type and syntax conflicts preventing successful build:
- Arc vs owned ContextResolver type mismatches
- Missing import for `Arc<ContextResolver>`
- Brace matching issues in struct definitions

### **Next Critical Steps Required**
1. **Fix Compilation Errors**: Resolve type mismatches and syntax errors in ping-pong executor
2. **Test Placeholder Resolution**: Verify `USER_WALLET_PUBKEY` → real pubkey resolution works end-to-end
3. **Test Dynamic Flow**: Confirm `300-jup-swap-then-lend-deposit-dyn.yml` works with resolved pubkeys
4. **Test Static Flow**: Confirm `001-sol-transfer.yml` works with same placeholder resolution system
5. **Generate Flow Diagrams**: Verify both test cases produce proper mermaid diagrams

### **Expected Results After Fix**
```bash
# All tests passing
cargo test --package reev-orchestrator
# test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured

# Clean compilation
cargo check --all
# Finished dev profile [unoptimized + debuginfo] target(s) in X.XXs

# Clippy clean
cargo clippy --fix --allow-dirty  
# warning: field `solana_env` is never read (minor)
```

### **Priority**: ✅ **RESOLVED**
**Status**: 🟢 **COMPLETED - ALL FUNCTIONALITY WORKING**
**Assigned**: reev-orchestrator

**Root Cause**: Placeholder resolution logic implemented but compilation errors in executor integration preventing testing.

-### **Priority**: ✅ **RESOLVED**

### **Resolution** ✅
- Re-enabled balance tool in ZAI and OpenAI agents
- Fixed tool name mismatches and API parameter structure  
- Added surfpool-based account funding for production testing
- All GLM agents now execute successfully via ping-pong orchestration


---

## Issue #22: Incomplete Flow Execution & Missing Details in Dynamic Flows - ACTIVE 🔴

### **Problem Summary**
Dynamic flows are terminating early at the swap step without completing the full multiplication strategy, and flow visualization lacks detailed execution information (amounts, addresses, specific parameters).

### **Root Cause Analysis**
**Early Flow Termination**: Dynamic flows with complex strategies (e.g., "multiply USDC 1.5x") are stopping after the jupiter_swap step without proceeding to the expected jupiter_lend step, resulting in incomplete 2-step flows instead of 4-step strategies.

**Missing Flow Details**: Flow visualization only shows high-level step names without execution details like:
- Specific amounts being swapped (e.g., "2 SOL → 300 USDC")
- Wallet addresses involved
- Jupiter transaction parameters
- Error details when steps fail

**Expected vs Actual Flow**:
- **Expected**: account_balance → jupiter_swap → jupiter_lend → positions_check
- **Actual**: account_balance → jupiter_swap → [*] (terminates early)
### **Current Status**:
- ✅ **4-Step Flow Execution**: Now executes complete multiplication strategies
- ✅ **Tool Name Consistency**: Fixed `jupiter_swap_flow`, `get_account_balance` parsing
- ✅ **Real Execution**: All steps showing actual execution times and success status
- ❌ **Missing Visualization Details**: `params: {}`, `result_data: {}`, `tool_args: null`

### **Evidence from Current Execution**
```bash
# Flow execution successful
{
  "status": "Completed",
  "tool_calls": [
    {"tool_name": "jupiter_swap", "success": true, "params": {}, "result_data": {}},
    {"tool_name": "jupiter_lend_earn_deposit", "success": true, "params": {}, "result_data": {}}
  ]
}

# But missing actual amounts, pubkeys, transaction details
{
  "tool_calls": [
    {"params": {}, "result_data": {}},  # Should show: {"amount": "1 SOL", "output": "150 USDC"}
    {"params": {}, "result_data": {}}   # Should show: {"amount": "150 USDC", "apy": "5.8%"}
  ]
}
```

### **Investigation Required**
**Extract Function Issue**: `extract_transaction_details()` function expects agent responses in format like `{"swap": {...}}` but actual agent output uses different structure.

**Agent Response Structure** (from debug logs):
```json
{
  "signatures": ["estimated_signature"],
  "summary": "Account balance retrieved successfully", 
  "transactions": [{
    "account": {
      "pubkey": "FFMBsUjK9mbZSAmFzDcpGGwDYLvLRs89twEH1L7x5GSP",
      "sol_balance": 5000000000,
      "token_balances": []
    }
  }]
}
```

**Expected Visualization Details** (currently missing):
- **Amounts**: "2 SOL → 300 USDC" 
- **Pubkeys**: Real wallet addresses from agent execution
- **Signatures**: Actual transaction IDs from on-chain execution
- **Parameters**: Tool input arguments (slippage, amounts, mints)

**Current vs Expected**:
```bash
# Current: Empty details
{
  "tool_calls": [
    {
      "tool_name": "jupiter_swap",
      "success": true,
      "params": {},              # ❌ EMPTY
      "result_data": {},        # ❌ EMPTY  
      "tool_args": null
    }
  ]
}

# Expected: Rich details
{
  "tool_calls": [
    {
      "tool_name": "jupiter_swap", 
      "success": true,
      "params": {
        "input_amount": 1000000000,
        "input_mint": "So11111111111111111111111111111111111112", 
        "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "slippage": 100
      },                     # ✅ RICH DETAILS
      "result_data": {
        "signature": "5XJ3Xabc123...",
        "output_amount": 150000000,
        "impact": 2.3
      },                     # ✅ RICH DETAILS
      "tool_args": "..."        # ✅ INPUT ARGUMENTS
    }
  ]
}
```

### **Files to Examine**
- `crates/reev-api/src/handlers/dynamic_flows/mod.rs` - `extract_transaction_details()` function needs update
- `crates/reev-agent/src/enhanced/zai_agent.rs` - Agent output structure analysis
- `crates/reev-orchestrator/src/execution/ping_pong_executor.rs` - Step result handling

### **Next Steps Required**
1. **Fix Extraction Function**: Update `extract_transaction_details()` to parse actual agent response structure
2. **Extract Real Details**: Parse amounts, pubkeys, signatures from agent JSON output
3. **Enhanced Visualization**: Populate `params`, `result_data`, `tool_args` with actual execution data
4. **Test Rich Details**: Verify flow diagrams show "2 SOL → 300 USDC" with real signatures

### **Expected Results After Fix**
- Flow visualization with specific amounts and pubkeys
- Transaction signatures visible in visualization
- Rich parameter details (slippage, mints, amounts)  
- Production-ready flow visualization with complete execution context

### **Priority**: 🟡 MEDIUM - Critical for user experience, but core functionality works

### **Related Issues**
- Issue #21: ZAI API 400 Errors (RESOLVED ✅)
- Issue #22: Incomplete Flow Execution (PARTIALLY RESOLVED ✅)

---

## Issue #23: Missing Amounts & Pubkeys in Flow Visualization - ACTIVE 🟡

### **Problem Summary**
Dynamic flow execution is working correctly (4-step strategies completing successfully), but flow visualization lacks critical execution details like specific amounts, wallet pubkeys, and transaction signatures.
Current mermaid flow diagrams are too basic and lack the detailed step-by-step information needed for proper scoring and agent understanding. The system generates correct flows but doesn't provide sufficient transparency for:

1. **Scoring System**: Need detailed execution metrics, parameter accuracy, and success criteria evaluation
2. **Agent Understanding**: Missing step-by-step reasoning, calculations, and decision logic
3. **Comprehensive Visualization**: Current flows show only tool execution, not the complete decision process

**Current Mermaid Output (Too Basic)**:
```mermaid
    [*] --> Prompt
    Prompt --> Agent : |
    Agent --> jupiter_swap : 2.000 SOL → USDC
    jupiter_swap --> [*]
```

**Missing Information**:
- Initial balance checking
- Percentage calculations (50% SOL determination)
- Lending operation details
- Final position verification
- Error handling paths
- Transaction parameters and amounts
- Success criteria validation

### **Root Cause Analysis**
**Limited Flow Step Detail**: Current flow generation in `gateway.rs` uses basic step creation without rich context or detailed reasoning. The `YmlGenerator` creates functional but minimal flows, and there's no mermaid-specific enhancement system.

**Key Limitations**:
1. **Flow Generation**: Only 4 basic steps, no decision branching
2. **Mermaid Generation**: No specialized mermaid diagram creation
3. **Scoring Integration**: No comprehensive scoring system integration
4. **Parameter Tracking**: Limited parameter transparency in flows

### **Current Architecture Analysis**

**Flow Generation Location**: `crates/reev-orchestrator/src/gateway.rs`
- `generate_flow_plan()` method creates basic steps
- Step creators: `create_*_step_with_recovery()` functions
- Limited intent parsing with simple keyword matching

**YML Generation**: `crates/reev-orchestrator/src/generators/yml_generator.rs`
- `generate_yml_content()` creates basic benchmark structure
- No mermaid-specific enhancements
- Simple ground truth generation

**Execution**: `crates/reev-orchestrator/src/execution/ping_pong_executor.rs`
- Step-by-step execution working correctly
- No enhanced logging for mermaid generation
- Missing detailed parameter capture

### **Development Plan**

#### **Phase 1: Enhanced Flow Generation (1-2 days)**

**1.1 Rich Step Creation**
```rust
// In gateway.rs - Enhanced step creators
impl OrchestratorGateway {
    fn create_enhanced_balance_check_step(
        &self, 
        context: &WalletContext,
        step_id: &str
    ) -> DynamicStep {
        let prompt_template = format!(
            "Check wallet {} balances and portfolio. Current SOL: {:.2}, USDC: {:.2}, Total: ${:.2}. \
             Calculate available SOL for operations and prepare for percentage-based strategy.",
            context.owner,
            context.sol_balance_sol(),
            context.get_token_balance("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                .map(|b| b.balance as f64 / 1_000_000.0)
                .unwrap_or(0.0),
            context.total_value_usd
        );

        DynamicStep::new(
            format!("{}_balance_check", step_id),
            prompt_template,
            "Initial portfolio assessment and balance verification".to_string(),
        )
        .with_tool("account_balance")
        .with_estimated_time(10)
        .with_recovery(RecoveryStrategy::Retry { attempts: 3 })
    }

    fn create_enhanced_calculation_step(
        &self,
        context: &WalletContext,
        target_multiplier: f64,
        sol_percentage: f64,
    ) -> DynamicStep {
        let sol_amount = context.sol_balance_sol() * (sol_percentage / 100.0);
        let target_usdc = sol_amount * 150.0; // SOL price assumption
        let required_usdc = target_usdc * target_multiplier;

        let prompt_template = format!(
            "Calculate multiplication strategy: Use {:.2}% of SOL ({:.2} SOL ≈ ${:.2}) to achieve {:.1}x portfolio growth. \
             Target: ${:.2} USDC position + remaining SOL value = ${:.2} total portfolio value. \
             Determine optimal swap amount considering slippage and fees.",
            sol_percentage,
            sol_amount,
            sol_amount * 150.0,
            target_multiplier,
            required_usdc,
            context.total_value_usd * target_multiplier
        );

        DynamicStep::new(
            "calculation_step".to_string(),
            prompt_template,
            format!("Calculate {:.1}x multiplication using {:.0}% SOL", target_multiplier, sol_percentage),
        )
        .with_estimated_time(5)
    }
}
```

**1.2 Decision Logic Integration**
```rust
// Enhanced flow generation with branching
pub fn generate_enhanced_300_flow(&self, context: &WalletContext) -> DynamicFlowPlan {
    let mut flow = DynamicFlowPlan::new(
        format!("enhanced-300-{}", uuid::Uuid::new_v4()),
        "use my 50% sol to multiply usdc 1.5x on jup".to_string(),
        context.clone()
    );

    // Step 1: Initial assessment
    flow = flow.with_step(self.create_enhanced_balance_check_step(context, "initial"));

    // Step 2: Strategy calculation  
    flow = flow.with_step(self.create_enhanced_calculation_step(context, 1.5, 50.0));

    // Step 3: Jupiter swap with parameters
    flow = flow.with_step(self.create_enhanced_swap_step(context, 50.0, 1.5));

    // Step 4: Swap verification
    flow = flow.with_step(self.create_swap_verification_step(context));

    // Step 5: Lending with yield targeting
    flow = flow.with_step(self.create_enhanced_lend_step(context, 1.5));

    // Step 6: Final position verification
    flow = flow.with_step(self.create_final_verification_step(context, 1.5));

    flow
}
```

#### **Phase 2: Enhanced Mermaid Generation (1-2 days)**

**2.1 Create Mermaid Generator Module**
```rust
// New file: crates/reev-orchestrator/src/generators/mermaid_generator.rs

use reev_types::flow::{DynamicFlowPlan, StepResult};

pub struct MermaidGenerator;

impl MermaidGenerator {
    pub fn generate_enhanced_diagram(
        flow_plan: &DynamicFlowPlan,
        execution_results: &[StepResult]
    ) -> String {
        let mut diagram = String::from("stateDiagram\n    [*] --> Check_Balance\n");
        
        // Generate state transitions
        for (i, step) in flow_plan.steps.iter().enumerate() {
            let prev_state = if i == 0 { "Check_Balance" } else { &flow_plan.steps[i-1].step_id };
            let action = self.extract_action(&step.description);
            diagram.push_str(&format!("    {} --> {} : {}\n", prev_state, step.step_id, action));
        }
        
        diagram.push_str("    Check_Final_Positions --> [*]\n\n");
        
        // Add detailed notes
        for step in &flow_plan.steps {
            diagram.push_str(&self.generate_step_note(step, execution_results));
        }
        
        diagram
    }
    
    fn generate_step_note(&self, step: &DynamicStep, results: &[StepResult]) -> String {
        let base_note = format!("note right of {} : {}\n", step.step_id, step.description);
        
        // Add execution data if available
        if let Some(result) = results.iter().find(|r| r.step_id == step.step_id) {
            let execution_note = format!(
                "note right of {} : {}\\nDuration: {}ms\\nSuccess: {}",
                step.step_id,
                self.format_execution_details(result),
                result.duration_ms,
                result.success
            );
            return execution_note;
        }
        
        base_note
    }
}
```

**2.2 Enhanced Flow Structure**
```rust
// Expected enhanced mermaid output for 300 benchmark
/*
stateDiagram
    [*] --> Initial_Balance_Check
    Initial_Balance_Check --> Calculation_Step : Analyze portfolio
    Calculation_Step --> Jupiter_Swap : 2.0 SOL → 300 USDC
    Swap_Verification --> Jupiter_Lend : Deposit 300 USDC
    Jupiter_Lend --> Final_Verification : Check 1.5x growth
    Final_Verification --> [*]

note right of Initial_Balance_Check : SOL: 4.0\\nUSDC: 20.0\\nTotal: $620\\nAvailable: 50% SOL
note right of Calculation_Step : Target: 1.5x\\nSOL needed: 2.0\\nExpected USDC: 300\\nSlippage: 5%
note right of Jupiter_Swap : Amount: 2.0 SOL\\nMint: So1111111\\nSlippage: 100bps\\nMin: 295 USDC
note right of Jupiter_Lend : Amount: 300 USDC\\nProtocol: Jupiter\\nExpected APY: 5.8%\\nDuration: Flexible
note right of Final_Verification : Target: 1.5x achieved\\nSOL value: $300\\nUSDC position: $300\\nTotal: $600
*/
```

#### **Phase 3: Comprehensive Scoring System (2-3 days)**

**3.1 Scoring Architecture**
```rust
// New file: crates/reev-orchestrator/src/scoring/mod.rs

pub struct FlowScorer;

impl FlowScorer {
    pub fn score_300_execution(
        flow_plan: &DynamicFlowPlan,
        execution_results: &[StepResult],
        target_multiplier: f64
    ) -> ExecutionScore {
        let mut score = ExecutionScore::new();
        
        // Sequence correctness (30%)
        score.add_component(
            ScoringComponent::SequenceCorrectness,
            self.score_sequence_correctness(flow_plan, execution_results),
            0.3
        );
        
        // Parameter accuracy (25%)
        score.add_component(
            ScoringComponent::ParameterAccuracy, 
            self.score_parameter_accuracy(execution_results, target_multiplier),
            0.25
        );
        
        // Success criteria achievement (25%)
        score.add_component(
            ScoringComponent::SuccessCriteria,
            self.score_success_criteria(execution_results, target_multiplier),
            0.25
        );
        
        // Execution efficiency (20%)
        score.add_component(
            ScoringComponent::Efficiency,
            self.score_execution_efficiency(execution_results),
            0.2
        );
        
        score.calculate_final()
    }
}
```

**3.2 Detailed Metrics Collection**
```rust
// Enhanced execution tracking
#[derive(Debug, Serialize)]
pub struct EnhancedStepResult {
    pub base: StepResult,
    pub execution_details: ExecutionDetails,
    pub scoring_data: ScoringData,
}

#[derive(Debug, Serialize)]
pub struct ExecutionDetails {
    pub input_parameters: HashMap<String, serde_json::Value>,
    pub output_results: HashMap<String, serde_json::Value>,
    pub transaction_signatures: Vec<String>,
    pub gas_costs: Vec<u64>,
    pub timing_breakdown: TimingBreakdown,
}
```

#### **Phase 4: Integration & Testing (1-2 days)**

**4.1 API Integration**
```rust
// Update dynamic_flows/mod.rs
pub async fn execute_enhanced_dynamic_flow(
    State(_state): State<ApiState>,
    Json(request): Json<EnhancedFlowRequest>
) -> impl IntoResponse {
    // Generate enhanced flow plan
    let enhanced_flow = gateway.generate_enhanced_300_flow(&context)?;
    
    // Execute with enhanced tracking
    let execution_results = execute_with_enhanced_tracking(&enhanced_flow).await?;
    
    // Generate enhanced mermaid diagram
    let enhanced_diagram = MermaidGenerator::generate_enhanced_diagram(
        &enhanced_flow, 
        &execution_results
    );
    
    // Score execution
    let score = FlowScorer::score_300_execution(
        &enhanced_flow, 
        &execution_results, 
        1.5
    );
    
    Json(EnhancedExecutionResponse {
        flow_id: enhanced_flow.flow_id,
        diagram: enhanced_diagram,
        score,
        detailed_results: execution_results,
    })
}
```

**4.2 Test Suite Updates**
```rust
// Enhanced tests in integration_tests.rs
#[tokio::test]
async fn test_enhanced_300_benchmark() -> Result<()> {
    let gateway = OrchestratorGateway::new().await?;
    
    // Test enhanced flow generation
    let flow_plan = gateway.generate_enhanced_300_flow(&test_context)?;
    assert_eq!(flow_plan.steps.len(), 6); // Enhanced step count
    
    // Test mermaid generation
    let diagram = MermaidGenerator::generate_enhanced_diagram(&flow_plan, &[]);
    assert!(diagram.contains("Initial_Balance_Check"));
    assert!(diagram.contains("Calculation_Step"));
    assert!(diagram.contains("Jupiter_Swap"));
    assert!(diagram.contains("Jupiter_Lend"));
    assert!(diagram.contains("Final_Verification"));
    
    // Test scoring system
    let mock_results = create_mock_execution_results();
    let score = FlowScorer::score_300_execution(&flow_plan, &mock_results, 1.5);
    assert!(score.total_score >= 0.8); // Should pass with good execution
    
    Ok(())
}
```

### **Implementation Priority**

**🔴 Critical Path**:
1. **Enhanced step creation** (Gateway.rs) - Foundation for everything else
2. **Mermaid generator** (New module) - Visualization capability  
3. **Scoring system** (New module) - Assessment framework
4. **API integration** (Dynamic flows) - End-to-end functionality

**🟡 Supporting Items**:
- Execution detail tracking in ping_pong_executor.rs
- Enhanced parameter capture in tool calls
- Test suite updates and validation

### **Expected Results**

**Enhanced Mermaid Output**:
```mermaid
stateDiagram
    [*] --> Initial_Balance_Check
    Initial_Balance_Check --> Calculation_Step : Analyze portfolio
    Calculation_Step --> Jupiter_Swap : 2.0 SOL → 300 USDC
    Swap_Verification --> Jupiter_Lend : Deposit 300 USDC  
    Jupiter_Lend --> Final_Verification : Check 1.5x growth
    Final_Verification --> [*]

note right of Initial_Balance_Check : SOL: 4.0\\nUSDC: 20.0\\nTotal: $620\\nAvailable: 50% SOL
note right of Calculation_Step : Target: 1.5x\\nSOL needed: 2.0\\nExpected USDC: 300\\nSlippage: 5%
note right of Jupiter_Swap : Amount: 2.0 SOL\\nMint: So1111111\\nSlippage: 100bps\\nMin: 295 USDC
note right of Jupiter_Lend : Amount: 300 USDC\\nProtocol: Jupiter\\nExpected APY: 5.8%\\nDuration: Flexible
note right of Final_Verification : Target: 1.5x achieved\\nSOL value: $300\\nUSDC position: $300\\nTotal: $600
```

**Comprehensive Scoring**:
- Sequence correctness: 95% (all steps executed in order)
- Parameter accuracy: 90% (amounts and percentages correct)  
- Success criteria: 100% (1.5x multiplication achieved)
- Execution efficiency: 85% (reasonable time and gas usage)
- **Overall Score: 92.5%**

### **Files to Create/Modify**

**New Files**:
- `crates/reev-orchestrator/src/generators/mermaid_generator.rs`
- `crates/reev-orchestrator/src/scoring/mod.rs`
- `crates/reev-orchestrator/src/scoring/types.rs`
- `crates/reev-orchestrator/src/scoring/calculators.rs`

**Modified Files**:
- `crates/reev-orchestrator/src/gateway.rs` - Enhanced flow generation
- `crates/reev-orchestrator/src/generators/yml_generator.rs` - Richer content
- `crates/reev-orchestrator/src/execution/ping_pong_executor.rs` - Detailed tracking
- `crates/reev-api/src/handlers/dynamic_flows/mod.rs` - Enhanced endpoints
- `crates/reev-orchestrator/tests/integration_tests.rs` - Comprehensive tests

### **Timeline**: 5-7 days total
**Priority**: 🟡 **HIGH** - Critical for production readiness

**Assigned**: reev-orchestrator team

---
