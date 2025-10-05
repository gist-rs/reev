# Reev Agent Refactoring Plan

## 🎯 Objectives

1. **Modular Architecture**: Separate protocol handlers from AI tools
2. **Extensibility**: Easy addition of new protocols (Drift, Kamino, etc.)
3. **Feature Flags**: Compile-time protocol selection
4. **Consistency**: Unified error handling and configuration

## 🏗️ New Directory Structure

```
crates/reev-agent/src/
├── protocols/              # Protocol-specific API handlers
│   ├── mod.rs
│   ├── jupiter/
│   │   ├── mod.rs
│   │   ├── earn.rs         # Real Jupiter earn API (positions + earnings)
│   │   ├── lend.rs         # Real Jupiter lend API (deposit + withdraw)
│   │   └── swap.rs         # Real Jupiter swap API
│   ├── drift/              # Future: Drift protocol
│   │   ├── mod.rs
│   │   └── perp.rs
│   ├── kamino/             # Future: Kamino protocol
│   │   ├── mod.rs
│   │   └── lending.rs
│   └── native/             # Native Solana operations
│       ├── mod.rs
│       ├── sol_transfer.rs
│       └── spl_transfer.rs
├── tools/                  # AI tool wrappers (layer on top of protocols)
│   ├── mod.rs
│   ├── jupiter_earn.rs     # Wraps protocols::jupiter::earn
│   ├── jupiter_lend.rs     # Wraps protocols::jupiter::lend
│   ├── jupiter_swap.rs     # Wraps protocols::jupiter::swap
│   ├── drift_perp.rs       # Future: Wraps protocols::drift::perp
│   ├── kamino_lending.rs   # Future: Wraps protocols::kamino::lending
│   ├── sol_transfer.rs     # Wraps native::sol_transfer
│   └── spl_transfer.rs     # Wraps native::spl_transfer
├── agents/
│   ├── coding/             # Renamed from deterministic_agents
│   │   ├── mod.rs
│   │   ├── d_001_sol_transfer.rs
│   │   ├── d_002_spl_transfer.rs
│   │   ├── d_100_jup_swap_sol_usdc.rs
│   │   ├── d_110_jup_lend_deposit_sol.rs
│   │   ├── d_111_jup_lend_deposit_usdc.rs
│   │   ├── d_112_jup_lend_withdraw_sol.rs
│   │   ├── d_113_jup_lend_withdraw_usdc.rs
│   │   └── d_114_jup_positions_and_earnings.rs
│   └── flow/
│       ├── agent.rs
│       ├── benchmark.rs
│       ├── state.rs
│       └── mod.rs
├── config/                 # Configuration management
│   ├── mod.rs
│   ├── jupiter.rs
│   ├── drift.rs
│   └── native.rs
└── lib.rs
```

## 🔧 Implementation Details

### 1. Protocol Handlers Layer

**Purpose**: Real API integration with protocols
**Returns**: Same return types as current implementation
**Error Handling**: `anyhow::Result<T>` propagated to main `thiserror`

```rust
// protocols/jupiter/earn.rs
use anyhow::Result;
use serde_json::Value;

pub async fn get_positions(user_pubkey: String) -> Result<Value> {
    // Real Jupiter API call to lite-api.jup.ag/lend/v1/earn/positions
}

pub async fn get_earnings(user_pubkey: String, position: Option<String>) -> Result<Value> {
    // Real Jupiter API call to lite-api.jup.ag/lend/v1/earn/earnings
}

// protocols/jupiter/lend.rs
use anyhow::Result;
use reev_lib::agent::RawInstruction;

pub async fn deposit(user_pubkey: String, mint: String, amount: u64) -> Result<Vec<RawInstruction>> {
    // Real Jupiter lend deposit API
}

pub async fn withdraw(user_pubkey: String, mint: String, amount: u64) -> Result<Vec<RawInstruction>> {
    // Real Jupiter lend withdraw API
}
```

### 2. AI Tools Layer

**Purpose**: Thin wrappers around protocol handlers for AI agent usage
**Responsibility**: Argument parsing, AI-specific logic, protocol delegation

```rust
// tools/jupiter_earn.rs
use crate::protocols::jupiter::earn;
use rig::tool::Tool;

pub struct JupiterEarnTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for JupiterEarnTool {
    async fn call(&self, args: JupiterEarnArgs) -> Result<String> {
        let user_pubkey = self.key_map.get("USER_WALLET_PUBKEY").unwrap();
        let positions = earn::get_positions(user_pubkey.clone()).await?;
        let earnings = earn::get_earnings(user_pubkey.clone(), args.position_address).await?;
        // Combine and format for AI response
    }
}
```

### 3. Coding Agents Layer

**Purpose**: Deterministic/coding agents that call protocols directly
**Responsibility**: No tool layer, direct protocol handler usage

```rust
// agents/coding/d_114_jup_positions_and_earnings.rs
use crate::protocols::jupiter::earn;

pub async fn handle_jup_positions_and_earnings(key_map: &HashMap<String, String>) -> Result<serde_json::Value> {
    let user_pubkey = key_map.get("USER_WALLET_PUBKEY")?;
    let positions = earn::get_positions(user_pubkey.clone()).await?;
    let earnings = earn::get_earnings(user_pubkey.clone(), None).await?;
    // Combine into flow response
}
```

### 4. Configuration Layer

**Purpose**: Environment-based configuration with dotenvy
**Default Values**: Fallback to current working values

```rust
// config/jupiter.rs
use std::env;

pub struct JupiterConfig {
    pub api_base_url: String,
    pub timeout_seconds: u64,
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            api_base_url: "https://lite-api.jup.ag".to_string(),
            timeout_seconds: 30,
        }
    }
}

impl JupiterConfig {
    pub fn from_env() -> Self {
        Self {
            api_base_url: env::var("JUPITER_API_BASE_URL")
                .unwrap_or_else(|_| Self::default().api_base_url),
            timeout_seconds: env::var("JUPITER_TIMEOUT_SECONDS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Self::default().timeout_seconds),
        }
    }
}
```

### 5. Feature Flags

```toml
# Cargo.toml
[features]
default = ["jupiter", "native"]
jupiter = []          # Jupiter protocol support
drift = []            # Drift protocol support  
kamino = []           # Kamino protocol support
native = []           # Native Solana operations
all-protocols = ["jupiter", "drift", "kamino"]
```

```rust
// protocols/mod.rs
#[cfg(feature = "jupiter")]
pub mod jupiter;
#[cfg(feature = "drift")]  
pub mod drift;
#[cfg(feature = "kamino")]
pub mod kamino;
#[cfg(feature = "native")]
pub mod native;
```

## 📋 Migration Steps

### Phase 1: Directory Restructuring
1. Create new directory structure
2. Move existing files to appropriate locations
3. Update module declarations

### Phase 2: Protocol Implementation
1. Implement real Jupiter APIs in `protocols/jupiter/`
2. Move real API logic from tools to protocol handlers
3. Replace placeholder implementations

### Phase 3: Tool Layer Refactoring
1. Update tools to use protocol handlers
2. Keep tool-specific logic (AI argument parsing)
3. Ensure thin wrapper pattern

### Phase 4: Agent Updates
1. Rename `deterministic_agents` to `coding_agents`
2. Update agents to use protocol handlers directly
3. Update imports and module declarations

### Phase 5: Configuration
1. Create config layer with dotenvy support
2. Add default values
3. Update protocol handlers to use config

### Phase 6: Feature Flags
1. Add feature flags to Cargo.toml
2. Update module declarations with cfg attributes
3. Test compilation with different feature combinations

## 🧪 Testing Strategy

1. **Unit Tests**: Each protocol handler tested independently
2. **Integration Tests**: Tool layer with mocked protocols
3. **Agent Tests**: Coding agents with real protocol calls
4. **Feature Flag Tests**: Compile with different feature combinations

## 🎯 Success Criteria

1. ✅ All existing functionality preserved
2. ✅ New protocols can be added easily
3. ✅ Feature flags work correctly
4. ✅ Configuration is environment-based
5. ✅ Error handling is consistent
6. ✅ All tests pass

## 🚀 Benefits

1. **Modularity**: Clear separation of concerns
2. **Extensibility**: Easy protocol addition
3. **Maintainability**: Centralized protocol logic
4. **Testability**: Independent layer testing
5. **Flexibility**: Feature flag configuration
6. **Performance**: Compile-time protocol selection
```

Now let me start implementing the refactoring:
