# Jupiter SDK

A Rust SDK for interacting with the Jupiter Swap and Lend APIs on Solana. This library provides a flexible, layered API designed for two primary use cases:

1.  **Production:** Building unsigned transactions that can be passed to any wallet for signing.
2.  **Simulation:** Running end-to-end tests and simulations against a local `surfpool` mainnet fork.

## Features

-   **Builder Pattern:** A clean, composable API for constructing swap, deposit, and withdraw operations.
-   **Separation of Concerns:** Clear distinction between transaction *building* and transaction *simulation*.
-   **Jupiter Swap API:** Fetches quotes and creates swap transactions.
-   **Jupiter Lend API:** Creates deposit and withdraw instructions for lending.
-   **Surfpool Integration:** Powerful simulation capabilities, including wallet funding and account pre-loading via `surfpool` cheat codes.

## Library Usage

Add `jup-sdk` to your `Cargo.toml`:

```toml
[dependencies]
jup-sdk = { path = "path/to/jup-sdk" }
solana-client = "2.2"
solana-sdk = "2.2"
# ... other dependencies
```

### Use Case 1: Generating an Unsigned Transaction (Production)

This flow is ideal when you need to prepare a transaction in a backend and send it to a frontend or a separate signer.

```rust
use jup_sdk::{Jupiter, models::SwapParams};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::str::FromStr;

async fn build_production_swap() -> anyhow::Result<()> {
    // A keypair whose public key will represent the user.
    // The private key is NOT used for signing here.
    let user_wallet = Keypair::new();

    // Use a real RPC endpoint.
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    let jupiter_client = Jupiter::new(rpc_client).with_signer(&user_wallet);

    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;

    let swap_params = SwapParams {
        input_mint: usdc_mint,
        output_mint: sol_mint,
        amount: 50_000_000, // 50 USDC
        slippage_bps: 500,  // 0.5%
    };

    // This method only builds the transaction; it does not sign or send it.
    let unsigned_tx = jupiter_client
        .swap(swap_params)
        .build_unsigned_transaction()
        .await?;

    // Serialize the unsigned transaction to be sent to the signing entity.
    let serialized_tx = bincode::serialize(&unsigned_tx.transaction)?;
    let tx_base64 = base64::engine::general_purpose::STANDARD.encode(serialized_tx);

    println!("Unsigned Base64 Transaction: {}", tx_base64);

    Ok(())
}
```

### Use Case 2: Running a Full Simulation (Testing)

This flow uses `surfpool` to run a complete, end-to-end operation, including funding a temporary wallet, signing, and executing the transaction.

```rust
use jup_sdk::{Jupiter, models::SwapParams};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::str::FromStr;

async fn run_simulation() -> anyhow::Result<()> {
    // A temporary keypair for the simulation.
    let signer = Keypair::new();

    // The client must point to a local surfpool instance.
    let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
    let jupiter_client = Jupiter::surfpool(rpc_client).with_signer(&signer);

    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;

    let swap_params = SwapParams {
        input_mint: usdc_mint,
        output_mint: sol_mint,
        amount: 50_000_000, // 50 USDC
        slippage_bps: 500,  // 0.5%
    };

    // The .commit() method orchestrates the entire simulation.
    let result = jupiter_client.swap(swap_params).commit().await?;

    println!("✅ Swap simulation successful!");
    println!("   Signature: {}", result.signature);

    Ok(())
}
```

## Examples

The provided examples demonstrate how to run simulations against a local `surfpool` instance.

### Prerequisites

-   Run `surfpool` locally on its default port: `http://127.0.0.1:8899`.
-   Ensure you have an internet connection for Jupiter API calls.

### Running Examples

```bash
# Simulate swapping 50 USDC to SOL
cargo run --example swap

# Simulate depositing 0.1 USDC into a lending protocol
cargo run --example deposit

# Simulate a full deposit and then withdraw cycle
cargo run --example withdraw
```
