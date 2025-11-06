# Issues

## Issue #39 - Production Mock Behavior Missing Feature Flag
**Status**: RESOLVED ✅
**Priority**: HIGH
**Component**: Build Configuration (Cargo.toml, feature flags)
**Description**: Mock/deterministic behaviors properly feature-flagged for clean production deployment

### **Problem Analysis**
**Production Risk**: Mock behaviors leak into production deployment
- Deterministic agent responses enabled in production
- Mock tool responses bypass real Jupiter execution
- Test fixtures interfere with live scoring
- No clean separation between development/testing modes

### **Root Cause RESOLVED**
Feature flag architecture implemented to control mock behaviors:
```rust
// ✅ IMPLEMENTED: Compile-time separation
#[cfg(feature = "mock_behaviors")]
fn run_deterministic_agent() -> Result<Json<LlmResponse>> { ... }

#[cfg(not(feature = "mock_behaviors"))]
if payload.mock {
    return Err(anyhow::anyhow!("Mock behaviors are disabled in production mode"));
}
```

### **Implementation Completed**
#### Feature Flag Architecture ✅
```toml
# ✅ IMPLEMENTED in individual crates
[features]
default = ["production"]
production = []                    # Clean LLM orchestration
mock_behaviors = []                  # Mock for development
```

#### Code Separation ✅
```rust
// ✅ IMPLEMENTED: Compile-time separation
#[cfg(feature = "mock_behaviors")]
fn run_deterministic_agent(payload: LlmRequest) -> Result<Json<LlmResponse>>

#[cfg(not(feature = "mock_behaviors"))]
fn generate_transaction(...) -> Response {
    if mock_enabled {
        return Err(anyhow::anyhow!("Mock behaviors are disabled in production mode"));
    }
    // Production: Route to LLM-only execution
}
```

### **Files to Modify**
- `Cargo.toml` - Add feature flag definitions
- `crates/reev-agent/src/lib.rs` - Agent routing with feature gates
- `crates/reev-runner/src/lib.rs` - Deterministic fallback control
- `crates/reev-orchestrator/src/gateway.rs` - Mock behavior flags
- All test files - Use `#[cfg(feature = "mock_behaviors")]`

### **Build Commands**
```bash
# Production: Clean LLM orchestration only
cargo build --release --features production

# Development: Include mock behaviors
cargo build --features mock_behaviors
```

### **Validation Criteria**
- Production build excludes all mock/deterministic code
- Development build retains testing capabilities
- No mock behaviors can accidentally reach production
- Clear compile-time separation enforced

---

## Issue #38 - Incomplete Multi-Step Flow Visualization
**Status**: IN PROGRESS 🔄
**Priority**: HIGH
**Component**: Flow Visualization (reev-api handlers/flow_diagram)
**Description**: 300 benchmark generates 4-step complex strategy but Mermaid diagrams only show single tool calls

### **Implementation Progress**
✅ **Enhanced Tool Call Tracking**: Implemented ToolCallSummary with parameter extraction
✅ **Improved Ping-Pong Executor**: Enhanced parsing and OTEL storage
✅ **Parameter Context**: Regex-based extraction of amounts, percentages, APY
✅ **Session Parser**: Supports enhanced OTEL tool call format
✅ **Dynamic Flow Generator**: Multi-step diagram with enhanced notes

### **Problem Analysis**
**Expected Behavior**:
```mermaid
stateDiagram
    [*] --> AccountDiscovery
    AccountDiscovery --> ContextAnalysis : "Extract 50% SOL requirement"
    ContextAnalysis --> BalanceCheck : "Current: 4 SOL, 20 USDC"
    BalanceCheck --> JupiterSwap : "Swap 2 SOL → ~300 USDC"
    JupiterSwap --> JupiterLend : "Deposit USDC for yield"
    JupiterLend --> PositionValidation : "Verify 1.5x target"
    PositionValidation --> [*] : "Final: 336 USDC achieved"
```

**Current Behavior**:
```mermaid
stateDiagram
    [*] --> Prompt
    Prompt --> Agent : |
    Agent --> jupiter_swap : 2.000 SOL → USDC
    jupiter_swap --> [*]
```

### **Root Cause PARTIALLY RESOLVED**
- ✅ **Flow Generation**: 4-step plan created correctly in `gateway.rs:352-363`
- ✅ **Tool Execution**: All 4 steps execute successfully (score: 1.0)
- 🔄 **Tool Call Tracking**: Enhanced ToolCallSummary captures all steps with parameters
- 🔄 **Visualization**: Enhanced diagram generation supports AccountDiscovery → JupiterSwap → JupiterLend → PositionValidation

### **Fix Required**
1. **Enhanced Tool Call Logging**: Capture all 4 execution steps in OpenTelemetry traces
2. **Improved Session Parsing**: Parse complete tool sequence from execution logs
3. **Parameter Context**: Display amounts, wallets, and calculations in diagram
4. **Step Validation**: Show success/failure status for each step

**Files Modified**:
- ✅ `reev-orchestrator/src/execution/ping_pong_executor.rs` - Enhanced tool call tracking with ToolCallSummary
- ✅ `reev-api/src/handlers/flow_diagram/session_parser.rs` - Enhanced OTEL parsing support
- ✅ `reev-api/src/handlers/flow_diagram/state_diagram_generator.rs` - Multi-step diagram generation
- ✅ `reev/tests/scripts/test_flow_visualization.sh` - Validation script for 4-step flows

---

## Issue #37 - ToolName Enum Mismatch - FIXED ✅
**Status**: RESOLVED ✅
**Progress**: Comprehensive string-to-constants refactor completed
**Description**: ToolName enum inconsistencies resolved, all hardcoded strings eliminated

### **Resolution Summary**
✅ Added missing `spl_transfer` and `ExecuteTransaction` tools
✅ Fixed serialization names (`account_balance` → `get_account_balance`)
✅ Created `reev-constants` crate for centralized tool management
✅ Replaced all hardcoded strings with type-safe constants
✅ Updated all tests and documentation

---

## Issue #29 - USER_WALLET_PUBKEY Auto-Generation - IMPLEMENTED ✅
**Status**: RESOLVED ✅
**Component**: ContextResolver (reev-orchestrator)
**Description**: Placeholders automatically resolved to unique keypairs during execution

### **Implementation Summary**
✅ `ContextResolver::resolve_placeholder()` generates unique keypairs
✅ Consistent mapping in `SolanaEnv` for execution lifetime
✅ Zero user confusion - documentation placeholders work automatically

---

## Issue #10 - Orchestrator-Agent Ping-Pong - RESOLVED ✅
**Status**: RESOLVED ✅
**Component**: PingPongExecutor (reev-orchestrator)
**Description**: Sequential step execution with validation and recovery implemented

### **Implementation Summary**
✅ Multi-step flow coordination working
✅ Progress tracking and partial scoring implemented
✅ Enhanced OpenTelemetry logging with parameters
✅ Recovery mechanisms for critical failures

---

**Last Updated**: 2025-11-06
**Total Issues**: 1 Active, 3 Resolved
**Next Review**: After Issue #38 resolution