# Jupiter SDK

A Rust library for simulating Jupiter DeFi operations on Solana using surfpool (mainnet fork).

## Features

- **Swap**: Perform token swaps using Jupiter API.
- **Deposit**: Deposit tokens into Jupiter lending protocol.
- **Withdraw**: Withdraw tokens from Jupiter lending protocol.

## Examples

Run the examples to simulate operations on a local surfpool instance.

### Prerequisites

- Run surfpool locally on `http://127.0.0.1:8899`
- Internet connection for Jupiter API calls

### Running Examples

```bash
# Swap 50 USDC to SOL
cargo run --example swap

# Deposit 0.1 USDC
cargo run --example deposit

# Deposit 0.1 USDC then withdraw
cargo run --example withdraw
```

## Library Usage

Add to Cargo.toml:

```toml
[dependencies]
jup-sdk = { path = "path/to/jup-sdk" }
```

Then use:

```rust
use jup_sdk::{swap, lend};
use solana_sdk::{signature::Keypair, pubkey::Pubkey};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let usdc = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let sol = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
    let signer = Keypair::new();

    swap::swap(usdc, sol, 50000000, 500).await?;
    lend::deposit(signer.clone(), usdc, 100000).await?;
    lend::withdraw(signer, usdc, 100000).await?;
    Ok(())
}
```

## API

- `pub async fn swap(input_mint: Pubkey, output_mint: Pubkey, amount: u64, slippage_bps: u16) -> Result<()>`
- `pub async fn deposit(signer: Keypair, asset: Pubkey, amount: u64) -> Result<()>`
- `pub async fn withdraw(signer: Keypair, asset: Pubkey, amount: u64) -> Result<()>`

## Implementation Details

- Uses Jupiter Lite API for instructions
- Builds transactions locally with fresh blockhashes
- Pre-loads missing accounts from mainnet into surfpool
- Verifies balances before and after operations

## Dependencies

- solana-sdk, solana-client for blockchain interaction
- reqwest for HTTP API calls
- surfpool for local simulation
- Jupiter API clients for swap and lend

Note: Ensure surfpool is running for simulations to work.
