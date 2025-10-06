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
**Status**: ✅ **COMPLETED**
- ✅ Enhanced existing `protocols/jupiter/mod.rs` configuration with more options
- ✅ Added environment variable support with dotenvy
- ✅ Integrated config with jup_sdk initialization
- ✅ Added configuration validation and debug logging
- ✅ Enhanced tools to use configuration defaults
- ✅ Added global configuration initialization on server startup

### Phase 4: Protocol Abstraction Layer
**Status**: ✅ **COMPLETED**
-- ✅ Created common protocol traits for consistent interfaces (Protocol, SwapProtocol, LendProtocol, TransferProtocol)
-- ✅ Standardized error handling across all protocols (ProtocolError enum with comprehensive error types)
-- ✅ Added protocol health checks and metrics (HealthChecker, MetricsCollector with comprehensive monitoring)
-- ✅ Created Jupiter protocol implementation using traits (JupiterProtocol with full trait implementations)
-- ✅ Established protocol abstraction foundation for future protocols
-- ✅ Added comprehensive metrics collection (request counts, response times, error tracking, volume monitoring)
-- ✅ Implemented health monitoring system (degraded/unhealthy states, auto-recovery, multi-protocol monitoring)

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
    let config = super::get_jupiter_config();
    config.log_config();
    
    // Validate slippage against configuration limits
    let validated_slippage = config.validate_slippage(slippage_bps)?;
    
    let jupiter_client = Jupiter::surfpool().with_user_pubkey(user_pubkey);
    // ... jup_sdk integration with configuration
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

### 4. Protocol Abstraction Layer ✅

**Purpose**: Common traits and utilities for consistent protocol interfaces
**Returns**: Standardized protocol interfaces with health monitoring and metrics

```rust
// protocols/common/traits.rs (IMPLEMENTED)
#[async_trait]
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    async fn health_check(&self) -> Result<HealthStatus, ProtocolError>;
    fn metrics(&self) -> &ProtocolMetrics;
    fn supported_operations(&self) -> Vec<ProtocolOperation>;
}

#[async_trait]
pub trait SwapProtocol: Protocol {
    async fn swap(&self, user_pubkey: &str, input_mint: &str, output_mint: &str, amount: u64, slippage_bps: u16) -> Result<Vec<RawInstruction>, ProtocolError>;
    async fn get_quote(&self, input_mint: &str, output_mint: &str, amount: u64) -> Result<SwapQuote, ProtocolError>;
}

// protocols/jupiter/protocol.rs (IMPLEMENTED)
#[async_trait]
impl Protocol for JupiterProtocol {
    fn name(&self) -> &'static str { "jupiter" }
    async fn health_check(&self) -> Result<HealthStatus, ProtocolError> { /* health check implementation */ }
    fn metrics(&self) -> &ProtocolMetrics { /* metrics access */ }
}

#[async_trait]
impl SwapProtocol for JupiterProtocol {
    async fn swap(&self, user_pubkey: &str, input_mint: &str, output_mint: &str, amount: u64, slippage_bps: u16) -> Result<Vec<RawInstruction>, ProtocolError> {
        // Jupiter swap implementation with metrics and error handling
    }
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

// tools/jupiter_swap.rs (ENHANCED)
impl Tool for JupiterSwapTool {
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Use default slippage from configuration if not provided
        let config = get_jupiter_config();
        let slippage_bps = match args.slippage_bps {
            Some(slippage) => config.validate_slippage(slippage)?,
            None => config.default_slippage(),
        };
        
        // Call protocol handler with validated slippage
        let instructions = handle_jupiter_swap(
            user_pubkey, input_mint, output_mint, args.amount, slippage_bps, &self.key_map
        ).await?;
        
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
3. ✅ Configuration enhanced with environment variables
4. 🔄 Future protocols (Drift, Kamino) structure ready
5. 🔄 All tests passing with comprehensive coverage

## 🚀 Benefits Achieved

### ✅ Current Benefits:
1. **Modularity**: Clear separation between protocols, tools, and agents
2. **Maintainability**: Jupiter logic centralized in protocols layer
3. **Reusability**: Same protocol handlers used by both tools and agents
4. **Testability**: Each layer can be tested independently
5. **Consistency**: Established pattern for future protocol additions
6. **Performance**: Optimized through protocol centralization
7. **Standardized Interfaces**: Common traits ensure consistent protocol behavior
8. **Comprehensive Monitoring**: Health checks and metrics for all protocols
9. **Error Handling**: Standardized error types across all protocol operations
10. **Extensibility**: Trait-based architecture makes adding new protocols straightforward

### 🔄 Future Benefits:
1. **Extensibility**: Easy protocol addition following established pattern
2. **Flexibility**: Feature flag configuration for compile-time selection
3. **Scalability**: Architecture supports many protocols without bloat
4. **Protocol Composition**: Multiple protocols can be combined in complex operations
5. **Runtime Monitoring**: Real-time health and performance metrics for all protocols

## 🔧 Environment Configuration

### Jupiter Configuration Options:
```bash
# .env file
JUPITER_API_BASE_URL=https://lite-api.jup.ag
JUPITER_TIMEOUT_SECONDS=30
JUPITER_MAX_RETRIES=3
JUPITER_USER_AGENT=reev-agent/0.1.0
JUPITER_DEFAULT_SLIPPAGE_BPS=50      # 0.5%
JUPITER_MAX_SLIPPAGE_BPS=1000        # 10%
JUPITER_DEBUG=false
JUPITER_SURFPOOL_RPC_URL=           # Optional custom RPC URL
```

### Native Configuration Options:
```bash
# .env file
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_WS_URL=wss://api.mainnet-beta.solana.com
SOLANA_TIMEOUT_SECONDS=30
SOLANA_MAX_RETRIES=3
SOLANA_CONFIRMATIONS=1
SOLANA_COMPUTE_UNITS=200000
SOLANA_PRIORITY_FEE_LAMPORTS=10000
SOLANA_USER_AGENT=reev-agent/0.1.0
```

### Configuration Features:
- **Environment Variable Support**: All settings can be overridden via environment variables
- **Validation**: Configuration values are validated on startup
- **Default Values**: Sensible defaults provided for all settings
- **Debug Logging**: Optional debug logging for troubleshooting
- **Global State**: Configuration initialized once and shared across the application

## 📊 Progress Summary

- **Phase 1 (Jupiter Refactoring)**: ✅ **COMPLETED**
- **Phase 2 (Native Protocol)**: ✅ **COMPLETED**
- **Phase 3 (Configuration)**: ✅ **COMPLETED**  
- **Phase 4 (Abstraction)**: ✅ **COMPLETED**
- **Phase 5 (Feature Flags)**: 🔄 **NOT STARTED**
- **Phase 6 (Future Protocols)**: 🔄 **NOT STARTED**

**Overall Progress**: 67% Complete (4 of 6 phases)

The foundation is now solid for the complete modular architecture. Both Jupiter and Native protocols serve as templates for all future protocol implementations, demonstrating the complete pattern from protocol handlers → AI tools → coding agents. The configuration system provides robust environment-based customization with validation and debugging capabilities. The protocol abstraction layer establishes consistent interfaces, standardized error handling, and comprehensive health monitoring for all protocols.

## 🎯 Phase 4 Achievements Summary:

### ✅ Protocol Abstraction Layer Complete:
1. **Common Protocol Traits**: Established `Protocol`, `SwapProtocol`, `LendProtocol`, `TransferProtocol` interfaces
2. **Standardized Error Handling**: Comprehensive `ProtocolError` enum covering all protocol failure scenarios
3. **Health Monitoring System**: `HealthChecker` and `HealthMonitor` for real-time protocol status tracking
4. **Metrics Collection**: `ProtocolMetrics` and `MetricsCollector` for performance monitoring and analytics
5. **Jupiter Protocol Implementation**: Full trait-based implementation demonstrating the abstraction pattern
6. **Extensibility Framework**: Clear template for implementing future protocols (Drift, Kamino, etc.)

### 🔧 Technical Achievements:
- **Async Trait System**: Proper async/await support for all protocol operations
- **Type Safety**: Strong typing ensures protocol interface compliance at compile time
- **Performance Tracking**: Request/response times, success rates, error categorization
- **Health States**: Healthy/Degraded/Unhealthy status with automatic recovery
- **Macro Support**: Utility macros for common protocol implementation patterns

The protocol abstraction layer now provides a robust foundation for building and managing multiple blockchain protocols with consistent interfaces, comprehensive monitoring, and standardized error handling.