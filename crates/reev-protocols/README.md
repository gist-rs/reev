# reev-protocols: Low-Level Solana Protocol Implementations

The `reev-protocols` crate provides low-level, protocol-specific implementations for Solana blockchain interactions. It serves as the foundation layer between high-level agent tools and raw Solana program interactions.

## 🎯 Core Features

### Protocol Categories
- **Native Protocols**: Core Solana operations (SOL transfers, system instructions)
- **Jupiter Protocols**: Jupiter DEX, lending, and earning protocols
- **Token Protocols**: SPL token standard implementations
- **Common Utilities**: Shared types and error handling across protocols

### Protocol Capabilities
- **Instruction Building**: Create Solana instructions from structured data
- **Transaction Construction**: Build complete transactions with proper signing
- **Account Management**: Handle address resolution and account states
- **Error Handling**: Protocol-specific error types and recovery
- **Type Safety**: Strongly-typed data structures for all protocols

## 🏗️ Architecture

### Core Components

```rust
// Protocol-specific instruction builders
pub struct SolTransferInstruction {
    pub from_pubkey: Pubkey,
    pub to_pubkey: Pubkey,
    pub lamports: u64,
}

// Jupiter protocol implementations
pub struct JupiterSwapInstruction {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub input_amount: u64,
    pub slippage_bps: u64,
}

// Common error types
pub enum ProtocolError {
    InvalidInstruction,
    AccountNotFound,
    InsufficientFunds,
    SerializationError,
}
```

### Protocol Registry
- **Dynamic Discovery**: Protocol registration for tool systems
- **Type Safety**: Compile-time validation of protocol usage
- **Extensibility**: Easy addition of new protocols
- **Standardization**: Consistent interfaces across all protocols

## 🛠️ Available Protocols

### Native Solana Operations

#### SOL Transfer Protocol
```rust
// Native SOL transfer implementation
SolTransferInstruction {
    from_pubkey: user_wallet,
    to_pubkey: recipient,
    lamports: 1000000, // 0.001 SOL
}
```

#### SPL Transfer Protocol
```rust
// SPL token transfer implementation
SplTransferInstruction {
    mint: usdc_mint,
    from_account: user_token_account,
    to_account: recipient_token_account,
    authority: user_wallet,
    amount: 1000000, // 1 USDC (6 decimals)
}
```

### Jupiter DeFi Protocols

#### Jupiter Swap Protocol
```rust
// Jupiter DEX aggregation swap
JupiterSwapInstruction {
    input_mint: sol_mint,
    output_mint: usdc_mint,
    input_amount: 1000000000, // 1 SOL
    slippage_bps: 100, // 1% slippage
}
```

#### Jupiter Lending Protocol
```rust
// Jupiter lending protocol operations
JupiterLendDepositInstruction {
    mint: usdc_mint,
    amount: 1000000,
    user_wallet: owner_pubkey,
}
```

## 📁 Project Structure

```
src/
├── lib.rs                    # Main protocol exports
├── common/                   # Shared utilities and types
│   ├── mod.rs
│   └── types.rs
├── native/                   # Native Solana protocols
│   ├── mod.rs
│   ├── sol_transfer.rs
│   └── spl_transfer.rs
└── jupiter/                  # Jupiter protocol implementations
    ├── mod.rs
    ├── swap.rs
    ├── lend_deposit.rs
    └── lend_withdraw.rs
```

## 🔧 Dependencies

```toml
[dependencies]
solana-sdk = { workspace = true }
spl-token = { workspace = true }
spl-associated-token-account = { workspace = true }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
thiserror = { workspace = true }
anyhow = { workspace = true }
bs58 = { workspace = true }
```

## 🚀 Usage Examples

### Basic Protocol Usage
```rust
use reev_protocols::{SolTransferInstruction, SplTransferInstruction};

// Create native SOL transfer
let sol_transfer = SolTransferInstruction::new(
    user_wallet,
    recipient_wallet,
    1000000
)?;

// Create SPL token transfer
let spl_transfer = SplTransferInstruction::new(
    usdc_mint,
    user_token_account,
    recipient_token_account,
    user_wallet,
    1000000
)?;
```

### Jupiter Protocol Integration
```rust
use reev_protocols::jupiter::{JupiterSwapInstruction, JupiterLendDepositInstruction};

// Create Jupiter swap
let swap_instruction = JupiterSwapInstruction::new(
    sol_mint,
    usdc_mint,
    1000000000,
    100 // 1% slippage
)?;

// Create Jupiter lend deposit
let lend_instruction = JupiterLendDepositInstruction::new(
    usdc_mint,
    1000000,
    user_wallet
)?;
```

### Transaction Building
```rust
use reev_protocols::{ProtocolBuilder, SolanaTransaction};

// Build complete transaction from instructions
let mut builder = ProtocolBuilder::new(user_wallet);
builder.add_instruction(sol_transfer);
builder.add_instruction(spl_transfer);

let transaction = builder.build_and_sign()?;
println!("Transaction: {}", transaction.signature());
```

## 🎮 Integration with Reev Architecture

Protocols fit into the Reev evaluation flow:

```
Tools (high-level operations)
    ↓
Protocols (instruction builders)
    ↓
SolanaEnv (transaction execution)
    ↓
Surfpool (on-chain processing)
```

## 🔍 Advanced Features

### Error Handling
- **Protocol-Specific Errors**: Custom error types for each protocol
- **Recovery Strategies**: Automatic retry and fallback mechanisms
- **Validation**: Input validation before instruction creation
- **Debug Information**: Detailed error context for troubleshooting

### Type Safety
- **Compile-Time Checks**: Strong typing for all protocol operations
- **Generic Implementations**: Support for different token types
- **Address Validation**: Built-in pubkey validation and parsing
- **Amount Validation**: Proper handling of token decimals

### Performance Optimization
- **Instruction Caching**: Reusable instruction patterns
- **Batch Operations**: Multiple instructions in single transaction
- **Fee Estimation**: Accurate fee calculation for complex operations
- **Address Resolution**: Efficient caching of derived addresses

## 📋 API Reference

### Protocol Builder
```rust
pub struct ProtocolBuilder {
    payer: Pubkey,
    instructions: Vec<Instruction>,
    recent_blockhash: Option<Hash>,
}
```

#### Methods
- `new(payer: Pubkey) -> Self`
- `add_instruction(&mut self, instruction: Instruction)`
- `set_recent_blockhash(&mut self, hash: Hash)`
- `build_and_sign(&mut self) -> Result<SolanaTransaction>`
- `estimate_fees(&self) -> Result<u64>`

### Native Instructions
```rust
// SOL transfer
SolTransferInstruction::new(from: Pubkey, to: Pubkey, lamports: u64) -> Result<Self>

// SPL transfer
SplTransferInstruction::new(
    mint: Pubkey,
    from: Pubkey,
    to: Pubkey,
    authority: Pubkey,
    amount: u64
) -> Result<Self>
```

### Jupiter Instructions
```rust
// Jupiter swap
JupiterSwapInstruction::new(
    input_mint: Pubkey,
    output_mint: Pubkey,
    input_amount: u64,
    slippage_bps: u64
) -> Result<Self>

// Jupiter lend deposit
JupiterLendDepositInstruction::new(
    mint: Pubkey,
    amount: u64,
    user_wallet: Pubkey
) -> Result<Self>
```

## 🔄 Design Principles

1. **Protocol Abstraction**: Clean separation between protocol logic and execution
2. **Type Safety**: Compile-time validation of all protocol operations
3. **Extensibility**: Easy addition of new protocols and use cases
4. **Performance**: Optimized for high-frequency operations
5. **Reliability**: Comprehensive error handling and recovery
6. **Standards Compliance**: Adherence to Solana and SPL standards

## 🔧 Troubleshooting

### Common Protocol Issues
- **Invalid Address**: Ensure all pubkeys are valid base58 strings
- **Insufficient Balance**: Check account states before operations
- **Wrong Token Mint**: Verify token mint addresses and decimals
- **Instruction Limits**: Monitor transaction size and instruction limits

### Debugging Protocol Execution
```rust
// Enable detailed logging
RUST_LOG=debug cargo test -p reev-protocols

// Validate instruction structure
println!("Instruction data: {:?}", instruction.serialize());

// Check account relationships
assert!(instruction.accounts.len() > 0, "No accounts provided");

// Verify transaction building
let tx = builder.build_and_sign()?;
println!("Transaction: {}", tx.signature());
```

### Performance Optimization
- Use instruction builders for complex operations
- Batch related operations into single transactions
- Optimize account ordering for better efficiency
- Monitor instruction sizes to avoid limits

## 📊 Protocol Statistics

| Protocol Category | Number of Protocols | Use Cases |
|------------------|---------------------|-------------|
| Native | 2 | SOL/SPL transfers |
| Jupiter | 2 | DEX aggregation, lending |
| Common | N | Shared utilities and types |
| Error Types | 1 | Comprehensive error handling |

## 🎯 Future Roadmap

### Planned Enhancements
- **More DeFi Protocols**: Additional protocols beyond Jupiter
- **Cross-Chain Protocols**: Multi-chain operation support
- **Advanced DeFi**: Yield farming, liquidity provision
- **Security Protocols**: Multi-signature and timelock operations
- **Metaplex Protocols**: NFT and marketplace operations

### Protocol Development Guide
1. Implement instruction builders for new protocols
2. Add comprehensive error handling and validation
3. Include unit tests with mock data
4. Update protocol registry and exports
5. Document with examples and use cases
6. Add integration tests with surfpool

## 🔄 Version History

- **v0.1.0**: Initial release with native and Jupiter protocols
- **v0.1.1**: Added comprehensive error handling
- **v0.1.2**: Enhanced instruction builders and validation
- **v0.1.3**: Added performance optimizations
- **v0.1.4**: Improved type safety and documentation
- **v0.1.5**: Added common utilities and shared types