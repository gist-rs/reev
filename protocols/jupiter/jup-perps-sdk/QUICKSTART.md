# Jupiter Perps Rust SDK - Quick Start Guide

## 🚀 Getting Started

### 1. Setup Environment

Create a `.env` file with your credentials:

```env
# RPC endpoint (optional, defaults to mainnet)
RPC_URL=https://api.mainnet-beta.solana.com

# Your wallet private key (base58 format)
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
    
    println!("Position PDA: {}", position_pda);
    Ok(())
}
```

## 📊 Examples

### Run Examples

```bash
# Get positions
cargo run --example get_positions

# Open position
cargo run --example open_position

# Close position  
cargo run --example close_position
```

### Key Features

- **Get Positions**: Fetch all open positions or wallet-specific positions
- **Open Positions**: Create long/short positions with various collateral types
- **Close Positions**: Close positions with stop-loss and take-profit support
- **PDA Generation**: Generate position and request PDAs
- **Price Calculations**: Convert between token amounts and USD sizes

## 🏗️ Architecture

### Core Components

1. **JupiterPerpsClient**: Main client for all operations
2. **Types**: Position, Custody, Request data structures
3. **PDA**: Program-derived address generation
4. **Constants**: Program IDs and custody addresses

### Request-Fulfillment Model

Jupiter Perps uses a request-fulfillment model:
1. Submit position request (open/close)
2. Keepers fulfill requests
3. Position execution is not immediate

## 🔧 Common Operations

### Get Open Positions

```rust
let positions = client.get_open_positions().await?;
for position in positions {
    println!("Position: {} | Size: ${:.2}", 
        position.pubkey, 
        position.account.size_usd as f64 / 1_000_000.0
    );
}
```

### Open Long Position

```rust
use jupiter_perps_rs::CreatePositionRequestParams;

let params = CreatePositionRequestParams {
    custody: sol_custody,
    collateral_custody: usdc_custody,
    collateral_token_delta: 10_000_000, // 0.01 SOL
    input_mint: solana_sdk::native_mint::id(),
    jupiter_minimum_out: None,
    owner: client.keypair.pubkey(),
    price_slippage: 95_000_000, // $95 with 6 decimals
    side: PositionSide::Long,
    size_usd_delta: 1_000_000_000, // $1000 with 6 decimals
    position_pubkey: position_pda,
};

let transaction = client.create_market_open_position_request(params).await?;
let signature = client.sign_and_submit_transaction(transaction).await?;
```

### Close Position

```rust
use jupiter_perps_rs::ClosePositionRequestParams;

let params = ClosePositionRequestParams {
    position_pubkey: position_to_close,
    desired_mint: usdc_mint,
    price_slippage: 100_000_000, // Current price
};

let transaction = client.create_market_close_position_request(params).await?;
let signature = client.sign_and_submit_transaction(transaction).await?;
```

## 📝 Important Notes

### Security
- Keep private keys secure - never commit to version control
- Always verify transaction details before signing
- Use appropriate slippage tolerance

### Gas Fees
- Maintain sufficient SOL for gas fees
- Consider gas fees when opening/closing small positions

### Market Conditions
- Set appropriate slippage based on volatility
- Monitor position requests after submission
- Requests are fulfilled by keepers, not immediately

### Testing
- Test on devnet before mainnet
- Use small position sizes initially
- Monitor transactions carefully

## 🛠️ Development

### Build Project

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Examples with Custom RPC

```bash
RPC_URL=https://your-rpc-endpoint cargo run --example get_positions
```

## 📚 Resources

- [Jupiter Station - Perpetuals Guide](https://station.jup.ag/guides/perpetual-exchange)
- [Jupiter Perps Documentation](https://docs.jup.ag/perpetuals)
- [Solana Documentation](https://docs.solana.com/)
- [Anchor Framework](https://www.anchor-lang.com/)

## 🤝 Support

- 📚 Check examples for common use cases
- 🐛 Open issues for bugs or features
- 💬 Join Jupiter Discord for community support
- 📖 Read the comprehensive [README.md](./README.md)

---

⚡ **Happy Trading with Jupiter Perps!** 🚀