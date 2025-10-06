# Reev Agent Refactoring Plan

## 🎯 Objectives

1. **Modular Architecture**: Separate protocol handlers from AI tools
2. **Extensibility**: Easy addition of new protocols (Drift, Kamino, etc.)
3. **Feature Flags**: Compile-time protocol selection
4. **Consistency**: Unified error handling and configuration

## 🏗️ Current Directory Structure (After Phase 1)

```
crates/reev-agent/src/
├── protocols/              # ✅ Protocol-specific API handlers
│   ├── mod.rs
│   ├── jupiter/            # ✅ Complete Jupiter protocol implementation
│   │   ├── mod.rs          # ✅ Configuration and utilities
│   │   ├── earnings.rs     # ✅ Jupiter earn API (positions + earnings)
│   │   ├── lend_deposit.rs # ✅ Jupiter lend deposit API
│   │   ├── lend_withdraw.rs# ✅ Jupiter lend withdraw API
│   │   ├── positions.rs    # ✅ Jupiter positions API
│   │   └── swap.rs         # ✅ Jupiter swap API (uses jup-sdk)
│   ├── drift/              # 🔄 Future: Drift protocol
│   ├── kamino/             # 🔄 Future: Kamino protocol
│   └── native/             # 🔄 Future: Native Solana operations
├── tools/                  # ✅ AI tool wrappers (thin layer on top of protocols)
│   ├── mod.rs
│   ├── jupiter_earn.rs     # ✅ Wraps protocols::jupiter::earnings
│   ├── jupiter_lend_deposit.rs # ✅ Wraps protocols::jupiter::lend_deposit
│   ├── jupiter_lend_withdraw.rs# ✅ Wraps protocols::jupiter::lend_withdraw
│   ├── jupiter_swap.rs     # ✅ Wraps protocols::jupiter::swap
│   ├── native.rs           # ✅ Native SOL/SPL transfer tools
│   └── flow/               # ✅ Flow orchestration tools
├── agents/                 # ✅ Agent implementations
│   ├── coding/             # ✅ Deterministic/coding agents
│   │   ├── d_001_sol_transfer.rs
│   │   ├── d_002_spl_transfer.rs
│   │   ├── d_100_jup_swap_sol_usdc.rs
│   │   ├── d_110_jup_lend_deposit_sol.rs
│   │   ├── d_111_jup_lend_deposit_usdc.rs
│   │   ├── d_112_jup_lend_withdraw_sol.rs
│   │   ├── d_113_jup_lend_withdraw_usdc.rs
│   │   └── d_114_jup_positions_and_earnings.rs
│   └── flow/               # ✅ Flow orchestration agents
├── config/                 # 🔄 Future: Configuration management
└── lib.rs
```

## ✅ Phase 1 Complete: Jupiter Protocol Refactoring

### What Was Accomplished:

1. **✅ Separated Protocol Logic from Tools:**
   - Moved real Jupiter API logic from `tools/jupiter_swap.rs` to `protocols/jupiter/swap.rs`
   - Created dedicated protocol handlers for lend operations: `protocols/jupiter/lend_deposit.rs` and `protocols/jupiter/lend_withdraw.rs`
   - All protocol handlers now use the custom `jup-sdk` implementation

2. **✅ Refactored Tools to Thin Wrappers:**
   - `tools/jupiter_swap.rs` → thin wrapper around `protocols::jupiter::swap::handle_jupiter_swap`
   - `tools/jupiter_lend_deposit.rs` → thin wrapper around `protocols::jupiter::lend_deposit::handle_jupiter_lend_deposit`
   - `tools/jupiter_lend_withdraw.rs` → thin wrapper around `protocols::jupiter::lend_withdraw::handle_jupiter_lend_withdraw`

3. **✅ Updated All References:**
   - Fixed all coding agents to use new protocol function names
   - Updated flow agent imports
   - Cleaned up module declarations
   - Removed duplicate implementations

4. **✅ Established Working Architecture:**
   - **Protocols Layer**: Centralized Jupiter API integration using jup-sdk
   - **Tools Layer**: AI argument parsing and protocol delegation
   - **Agents Layer**: Direct protocol usage for deterministic flows

### Current Architecture Pattern:

```rust
// Protocol Handler (uses jup-sdk)
protocols/jupiter/swap.rs → handle_jupiter_swap() → jup_sdk::Jupiter::surfpool()

// AI Tool (thin wrapper)
tools/jupiter_swap.rs → JupiterSwapTool::call() → handle_jupiter_swap()

// Coding Agent (direct protocol usage)
agents/coding/d_100_jup_swap_sol_usdc.rs → handle_jupiter_swap()
```

## 🔄 Remaining Implementation Plan

### Phase 2: Native Protocol Implementation
**Status**: ✅ **COMPLETED**
- ✅ Moved native SOL/SPL transfer logic from `tools/native.rs` to `protocols/native/`
- ✅ Created `protocols/native/sol_transfer.rs` and `protocols/native/spl_transfer.rs`
- ✅ Refactored `tools/native.rs` to use protocol handlers
- ✅ Updated coding agents to use protocol handlers directly
- ✅ Fixed all module declarations and imports

### Phase 3: Jupiter Configuration Enhancement
**Status**: 🔄 Not Started  
- Enhance existing `protocols/jupiter/mod.rs` configuration
- Add environment variable support with dotenvy
- Integrate config with jup-sdk initialization

### Phase 4: Protocol Abstraction Layer
**Status**: 🔄 Not Started
- Create common protocol traits for consistent interfaces
- Standardize error handling across all protocols
- Add protocol health checks and metrics

### Phase 5: Feature Flags Implementation
**Status**: 🔄 Not Started
```toml
# Cargo.toml
[features]
default = ["jupiter", "native"]
jupiter = []          # Jupiter protocol support
drift = []            # Future: Drift protocol support  
kamino = []           # Future: Kamino protocol support
native = []           # Native Solana operations
all-protocols = ["jupiter", "drift", "kamino", "native"]
```

```rust
// protocols/mod.rs
#[cfg(feature = "jupiter")]
pub mod jupiter;
#[cfg(feature = "native")]
pub mod native;
#[cfg(feature = "drift")]  
pub mod drift;
#[cfg(feature = "kamino")]
pub mod kamino;
```

### Phase 6: Future Protocol Support
**Status**: 🔄 Not Started
- Add Drift protocol structure
- Add Kamino protocol structure
- Follow established pattern from Jupiter + Native implementations

## 🔧 Implementation Details (Current State)

### 1. Protocol Handlers Layer ✅

**Purpose**: Real API integration using jup-sdk and Solana instructions
**Returns**: `Vec<RawInstruction>` for instruction-based operations
**Error Handling**: `anyhow::Result<T>` propagated to tools

```rust
// protocols/jupiter/swap.rs (IMPLEMENTED)
pub async fn handle_jupiter_swap(
    user_pubkey: Pubkey,
    input_mint: Pubkey,
    output_mint: Pubkey,
    amount: u64,
    slippage_bps: u16,
    _key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    let jupiter_client = Jupiter::surfpool().with_user_pubkey(user_pubkey);
    // ... jup_sdk integration
}

// protocols/native/sol_transfer.rs (IMPLEMENTED)
pub async fn handle_sol_transfer(
    from_pubkey: Pubkey,
    to_pubkey: Pubkey,
    lamports: u64,
    _key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    let instruction = solana_system_interface::instruction::transfer(&from_pubkey, &to_pubkey, lamports);
    // Convert to RawInstruction format
}
```

### 2. AI Tools Layer ✅

**Purpose**: Thin wrappers for AI agent usage
**Responsibility**: Argument parsing, validation, protocol delegation

```rust
// tools/jupiter_swap.rs (IMPLEMENTED)
impl Tool for JupiterSwapTool {
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate arguments
        let user_pubkey = Pubkey::from_str(&args.user_pubkey)?;
        
        // Call protocol handler
        let raw_instructions = handle_jupiter_swap(
            user_pubkey, input_mint, output_mint, amount, slippage_bps, &self.key_map
        ).await?;
        
        // Serialize response
        Ok(serde_json::to_string(&raw_instructions)?)
    }
}

// tools/native.rs (IMPLEMENTED)
impl Tool for SolTransferTool {
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate arguments
        let from_pubkey = Pubkey::from_str(&args.from_pubkey)?;
        let to_pubkey = Pubkey::from_str(&args.to_pubkey)?;
        
        // Call protocol handler
        let instructions = handle_sol_transfer(from_pubkey, to_pubkey, args.lamports, &self.key_map).await?;
        
        // Serialize response
        Ok(serde_json::to_string(&instructions)?)
    }
}
```

### 3. Coding Agents Layer ✅

**Purpose**: Deterministic agents using protocols directly
**Responsibility**: Direct protocol handler usage

```rust
// agents/coding/d_100_jup_swap_sol_usdc.rs (IMPLEMENTED)
pub async fn handle_jup_swap_sol_usdc(
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    let instructions = handle_jupiter_swap(user_pubkey, sol_mint, usdc_mint, amount, slippage, key_map).await?;
    Ok(instructions)
}

// agents/coding/d_001_sol_transfer.rs (IMPLEMENTED)
pub async fn handle_sol_transfer(
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    let instructions = protocol_handle_sol_transfer(from, to, lamports, key_map).await?;
    Ok(instructions)
}
```

## 🧪 Testing Strategy

### Completed ✅:
- **Compilation Tests**: All refactored code compiles successfully
- **Import Tests**: All module imports resolve correctly
- **Integration Tests**: Tools → Protocols → jup-sdk flow works
- **Native Protocol Tests**: SOL/SPL transfer protocols working correctly
- **Agent Integration**: Coding agents using protocols directly

### Remaining 🔄:
- **Unit Tests**: Individual protocol handler testing
- **E2E Tests**: Complete transaction flow testing
- **Feature Flag Tests**: Compile with different feature combinations

## 🎯 Success Criteria

### Completed ✅:
1. ✅ All existing functionality preserved
2. ✅ Clear separation of concerns achieved
3. ✅ Protocol logic centralized
4. ✅ Tools act as thin wrappers
5. ✅ Coding agents use protocols directly
6. ✅ Module structure is clean and extensible
7. ✅ Native protocol moved to protocols layer
8. ✅ Two complete protocol examples (Jupiter + Native)

### Remaining 🔄:
1. 🔄 Feature flags implemented
2. 🔄 Configuration enhanced with environment variables
3. 🔄 Future protocols (Drift, Kamino) structure ready
4. 🔄 All tests passing with comprehensive coverage

## 🚀 Benefits Achieved

### ✅ Current Benefits:
1. **Modularity**: Clear separation between protocols, tools, and agents
2. **Maintainability**: Jupiter logic centralized in protocols layer
3. **Reusability**: Same protocol handlers used by both tools and agents
4. **Testability**: Each layer can be tested independently
5. **Consistency**: Established pattern for future protocol additions

### 🔄 Future Benefits:
1. **Extensibility**: Easy protocol addition following established pattern
2. **Flexibility**: Feature flag configuration for compile-time selection
3. **Performance**: Optimized through protocol centralization
4. **Scalability**: Architecture supports many protocols without bloat

## 📊 Progress Summary

- **Phase 1 (Jupiter Refactoring)**: ✅ **COMPLETED**
- **Phase 2 (Native Protocol)**: ✅ **COMPLETED**
- **Phase 3 (Configuration)**: 🔄 **NOT STARTED**  
- **Phase 4 (Abstraction)**: 🔄 **NOT STARTED**
- **Phase 5 (Feature Flags)**: 🔄 **NOT STARTED**
- **Phase 6 (Future Protocols)**: 🔄 **NOT STARTED**

**Overall Progress**: 33% Complete (2 of 6 phases)

The foundation is now solid for the complete modular architecture. Both Jupiter and Native protocols serve as templates for all future protocol implementations, demonstrating the complete pattern from protocol handlers → AI tools → coding agents.