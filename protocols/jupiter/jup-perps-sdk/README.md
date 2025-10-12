# Jupiter Perps Rust SDK

A Rust SDK for interacting with Jupiter Perpetuals on Solana. This SDK provides a clean, type-safe interface for opening, closing, and managing perpetual positions on Jupiter's decentralized exchange.

## Features

- 🦀 Pure Rust implementation with type safety
- 📊 Get open positions across the protocol or for specific wallets
- 🚀 Open long/short positions with various collateral types
- 🛑 Close positions with stop-loss and take-profit support
- 💱 Built-in Jupiter swap integration for token conversion
- 🔧 PDA generation for positions and requests
- 📈 Position management and PnL calculations
- ⚡ Async/await support with tokio
- 🎣 Request-fulfillment model compatible with Jupiter keepers

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
jup-perps-sdk = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
dotenv = "0.15"
```

## Quick Start

### 1. Environment Setup

Create a `.env` file with your configuration:

```env
# RPC endpoint (optional, defaults to mainnet)
RPC_URL=https://api.mainnet-beta.solana.com

# Your wallet private key (base58 or array format)
PRIVATE_KEY=your_base58_private_key_here

# Optional: Specific wallet to query
WALLET_PUBKEY=your_wallet_pubkey_here

# Optional: Specific position to close
POSITION_PUBKEY=your_position_pubkey_here
```

### 2. Basic Usage

```rust
use jupiter_perps_rs::{JupiterPerpsClient, PositionSide};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize client
    let client = JupiterPerpsClient::from_env()?;

    // Get all open positions
    let positions = client.get_open_positions().await?;
    println!("Found {} open positions", positions.len());

    // Open a long position
    let sol_custody = Pubkey::from_str("7xS2gz2bTp3fwCC7knJvUWTEU9Tycczu6VhJYKgi1wdz")?;
    let usdc_custody = Pubkey::from_str("G18jKKXQwBbrHeiK3C9MRXhkHsLHf7XgCSisykV46EZa")?;

    let (position_pda, _) = client.generate_position_pda(
        &sol_custody,
        &usdc_custody,
        PositionSide::Long
    )?;

    // Create position request
    let params = CreatePositionRequestParams {
        custody: sol_custody,
        collateral_custody: usdc_custody,
        collateral_token_delta: 10_000_000, // 0.01 SOL
        input_mint: spl_token::native_mint::ID,
        jupiter_minimum_out: None,
        owner: client.keypair.pubkey(),
        price_slippage: 95_000_000, // $95 with 6 decimals
        side: PositionSide::Long,
        size_usd_delta: 1_000_000_000, // $1000 with 6 decimals
        position_pubkey: position_pda,
    };

    let transaction = client.create_market_open_position_request(params).await?;
    let signature = client.sign_and_submit_transaction(transaction).await?;

    println!("Position opened: {}", signature);
    Ok(())
}
```

## Examples

The SDK includes comprehensive examples:

### Get Positions
```bash
cargo run --example get_positions
```
Retrieves all open positions, wallet-specific positions, custody information, and generates position PDAs.

### Open Position
```bash
cargo run --example open_position
```
Demonstrates opening long/short positions with different collateral types and Jupiter swap integration.

### Close Position
```bash
cargo run --example close_position
```
Shows how to close positions with stop-loss, take-profit, and emergency close scenarios.

## Core Concepts

### Request-Fulfillment Model

Jupiter Perps uses a request-fulfillment model where:
1. You submit a position request (open/close)
2. Keepers monitor and fulfill these requests
3. Position execution is not immediate

### Position Types

- **Long Positions**: Profit when asset price increases
- **Short Positions**: Profit when asset price decreases

### Collateral Options

- **Native Collateral**: Direct token deposits (USDC, SOL, etc.)
- **Swapped Collateral**: Use Jupiter to swap tokens before depositing

## API Reference

### JupiterPerpsClient

Main client for interacting with Jupiter Perps:

```rust
let client = JupiterPerpsClient::from_env()?;
```

#### Methods

- `get_open_positions()` - Get all open positions
- `get_open_positions_for_wallet(wallet)` - Get positions for specific wallet
- `get_custody(custody_pubkey)` - Get custody account data
- `create_market_open_position_request(params)` - Create open position transaction
- `create_market_close_position_request(params)` - Create close position transaction
- `sign_and_submit_transaction(transaction)` - Sign and submit transaction
- `generate_position_pda(custody, collateral_custody, side)` - Generate position PDA

### Data Types

#### PositionSide
```rust
pub enum PositionSide {
    Long,
    Short,
}
```

#### CreatePositionRequestParams
```rust
pub struct CreatePositionRequestParams {
    pub custody: Pubkey,
    pub collateral_custody: Pubkey,
    pub collateral_token_delta: u64,
    pub input_mint: Pubkey,
    pub jupiter_minimum_out: Option<u64>,
    pub owner: Pubkey,
    pub price_slippage: u64,
    pub side: PositionSide,
    pub size_usd_delta: u64,
    pub position_pubkey: Pubkey,
}
```

#### ClosePositionRequestParams
```rust
pub struct ClosePositionRequestParams {
    pub position_pubkey: Pubkey,
    pub desired_mint: Pubkey,
    pub price_slippage: u64,
}
```

## Constants

Important program addresses and custodies:

```rust
// Program IDs
pub const JUPITER_PERPETUALS_PROGRAM_ID: &str = "PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu";
pub const JLP_POOL_ACCOUNT_PUBKEY: &str = "5BUwFW4nRbftYTDMbgxykoFWqWHPzahFSNAaaaJtVKsq";

// Custody Accounts
pub enum CustodyPubkey {
    Sol = "7xS2gz2bTp3fwCC7knJvUWTEU9Tycczu6VhJYKgi1wdz",
    Eth = "AQCGyheWPLeo6Qp9WpYS9m3Qj479t7R636N9ey1rEjEn",
    Btc = "5Pv3gM9JrFFH883SWAhvJC9RPYmo8UNxuFtv5bMMALkm",
    Usdc = "G18jKKXQwBbrHeiK3C9MRXhkHsLHf7XgCSisykV46EZa",
    Usdt = "4vkNeXiYEUizLdrpdPS1eC2mccyM4NUPRtERrk6ZETkk",
}
```

## Error Handling

The SDK uses `anyhow::Result` for error handling:

```rust
match client.get_open_positions().await {
    Ok(positions) => println!("Found {} positions", positions.len()),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Security Considerations

- 🔐 Keep your private keys secure and never commit them to version control
- ⚠️ Always verify transaction details before signing
- 🛡️ Use appropriate slippage tolerance based on market conditions
- 💰 Maintain sufficient SOL for gas fees
- 🧪 Test on devnet before using on mainnet

## Jupiter Integration

For token swaps, integrate with Jupiter Quote API:

```rust
// Get quote from Jupiter API
let quote = jupiter_quote_api.get_quote(
    input_mint: usdc_mint,
    output_mint: sol_mint,
    amount: usdc_amount,
    slippage: 5
);

let jupiter_minimum_out = quote.out_amount;
```

## Development

### Running Tests

```bash
cargo test
```

### Building

```bash
cargo build --release
```

### Examples with Custom RPC

```bash
RPC_URL=https://your-rpc-endpoint cargo run --example get_positions
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Resources

- [Jupiter Station - Perpetuals Guide](https://station.jup.ag/guides/perpetual-exchange)
- [Jupiter Perps Documentation](https://docs.jup.ag/perpetuals)
- [Solana Documentation](https://docs.solana.com/)
- [Anchor Framework](https://www.anchor-lang.com/)

## Support

- 📚 Check the examples for common use cases
- 🐛 Open issues for bugs or feature requests
- 💬 Join the Jupiter Discord for community support
