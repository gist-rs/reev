# reev-context: Context Resolution & Validation

The `reev-context` crate provides centralized context resolution and validation for the Reev evaluation framework. It handles placeholder resolution, account state consolidation, and multi-step flow context management.

## 🎯 Core Features

### Context Resolution
- **Placeholder Resolution**: Maps placeholder names (e.g., `USER_WALLET_PUBKEY`) to real addresses
- **State Consolidation**: Combines YAML initial state with surfpool blockchain data
- **Multi-Step Support**: Tracks context changes across complex workflows
- **YAML Validation**: Ensures benchmark structure compliance

### Ground Truth Separation
- **Deterministic Mode**: Allows ground truth access for reproducible tests
- **LLM Mode**: Blocks ground truth to prevent information leakage
- **Mode Detection**: Configurable via agent type, environment, or tags

## 🏗️ Architecture

### Core Components

```rust
/// Complete context for agent execution
pub struct AgentContext {
    pub key_map: HashMap<String, String>,
    pub account_states: HashMap<String, serde_json::Value>,
    pub fee_payer_placeholder: Option<String>,
    pub current_step: Option<u32>,
    pub step_results: HashMap<String, serde_json::Value>,
}

/// Context resolver for placeholder resolution
pub struct ContextResolver {
    rpc_client: solana_client::rpc_client::RpcClient,
}
```

### Key Functions

- `ContextResolver::new()` - Create resolver with RPC client
- `resolve_initial_context()` - Build context from initial state + ground truth
- `validate_resolved_context()` - Validate context structure
- `context_to_yaml_with_comments()` - Generate LLM-friendly format

## 🧪 Testing Strategy

### Test Files (3 tests)
- `benchmark_context_validation.rs` - Context preparation testing (no external deps)
- `benchmark_yaml_validation.rs` - End-to-end YAML validation (ground truth only)
- `context_validation_test.rs` - Comprehensive validation with surfpool integration

### Running Tests

```bash
# Run all context tests
cargo test -p reev-context

# Run specific test with output
cargo test -p reev-context --test benchmark_yaml_validation -- --nocapture

# Context format validation
cargo test -p reev-context --test benchmark_context_validation -- --nocapture
```

## 📁 Project Structure

```
src/
├── lib.rs              # Main context resolver implementation
└── (no submodules - single-file design)

tests/
├── benchmark_context_validation.rs      # Context preparation tests
├── benchmark_yaml_validation.rs        # YAML structure tests  
└── context_validation_test.rs          # Integration tests
```

## 🔧 Dependencies

```toml
[dependencies]
reev-tools = { path = "../reev-tools" }
anyhow = { workspace = true }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
serde_yaml = { workspace = true }
solana-sdk = { workspace = true }
solana-client = { workspace = true }
tracing = { workspace = true }
bs58 = { workspace = true }
```

## 🚀 Usage Examples

### Basic Context Resolution
```rust
use reev_context::{ContextResolver, InitialState};

let resolver = ContextResolver::new(rpc_client);
let initial_state = vec![
    InitialState {
        pubkey: "USER_WALLET_PUBKEY".to_string(),
        owner: "11111111111111111111111111111111111".to_string(),
        lamports: 1000000000,
        data: None,
    }
];

let context = resolver.resolve_initial_context(
    &initial_state, 
    &ground_truth,  // Use None for LLM mode
    None
).await?;
```

### Mock Context for Testing
```rust
let mock_context = create_mock_context_from_initial_state(&initial_state);
resolver.validate_resolved_context(&mock_context)?;
```

### YAML Export for LLM
```rust
let yaml_output = resolver.context_to_yaml_with_comments(&context)?;
println!("LLM will see:\n{}", yaml_output);
```

## 🎮 Integration with Reev Architecture

The context resolver fits into the Reev flow as:

```
Runner (loads YAML) 
    ↓
ContextResolver (resolves placeholders) 
    ↓
Agent (receives resolved context) 
    ↓
Tools (execute with real addresses) 
    ↓
Surfpool (processes transactions)
```

## 🔍 Validation & Troubleshooting

### Common Validation Errors
- **Invalid Initial State**: Missing `pubkey`, `owner`, or `lamports` fields
- **Placeholder Resolution**: Unknown placeholders not in key_map
- **Ground Truth Mismatch**: Final state assertions don't match initial state
- **Type Mismatches**: YAML value types incompatible with expectations

### Debugging Context Issues
```rust
// Enable detailed logging
RUST_LOG=debug cargo test -p reev-context

// Check context structure
println!("Key map: {:?}", context.key_map);
println!("Account states: {:?}", context.account_states);

// Validate specific placeholder
assert!(context.key_map.contains_key("USER_WALLET_PUBKEY"));
```

### Performance Considerations
- Context resolution is O(n) in number of initial state accounts
- Placeholder resolution uses efficient HashMap lookups
- YAML export optimized for large account sets

## 📋 API Reference

### ContextResolver

#### Methods
- `new(rpc_client: RpcClient) -> Self`
- `resolve_initial_context(initial_state, ground_truth, existing_key_map) -> Result<AgentContext>`
- `validate_resolved_context(context) -> Result<()>`
- `context_to_yaml_with_comments(context) -> Result<String>`

### AgentContext

#### Fields
- `key_map: HashMap<String, String>` - Placeholder to address mapping
- `account_states: HashMap<String, serde_json::Value>` - Account data
- `fee_payer_placeholder: Option<String>` - Fee payer identifier
- `current_step: Option<u32>` - Multi-step flow position
- `step_results: HashMap<String, serde_json::Value>` - Previous results

### InitialState

#### Fields
- `pubkey: String` - Account identifier or placeholder
- `owner: String` - Account owner program ID
- `lamports: u64` - SOL balance
- `data: Option<String>` - Token account data

## 🎯 Design Principles

1. **Separation of Concerns**: Context resolution separate from business logic
2. **No Information Leakage**: Ground truth properly separated from LLM context
3. **Deterministic Behavior**: Same inputs always produce same outputs
4. **Error Handling**: Comprehensive error messages for debugging
5. **Testability**: All components testable without external dependencies

## 🔄 Version History

- **v0.1.0**: Initial release with core context resolution
- **v0.1.1**: Added multi-step flow support
- **v0.1.2**: Enhanced ground truth separation
- **v0.1.3**: Added comprehensive YAML validation