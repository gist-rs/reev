//! # Jupiter Swap Proof-of-Concept
//!
//! This is a standalone binary to demonstrate a full end-to-end Jupiter swap
//! against a local `surfpool` (mainnet fork) validator. It proves that the core
//! logic of funding a wallet, fetching instructions from the Jupiter API,
//! building a transaction with a local blockhash, signing it, and executing
//! it against the forked environment is sound.
//!
//! This approach avoids dependency conflicts by using a clean, isolated set of dependencies.
//!
//! ## How to Run:
//! 1. Make sure `reev-agent` is buildable in the parent workspace (`cargo build --package reev-agent`).
//! 2. Run this binary from its own directory: `cd protocols/jupiter/swap-poc && cargo run`.

use anyhow::{Context, Result};
use jupiter_swap_api_client::{
    quote::QuoteRequest, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use solana_client::{nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_client::RpcClient};
use solana_sdk::{
    address_lookup_table::{state::AddressLookupTable, AddressLookupTableAccount},
    instruction::Instruction,
    message::{v0, VersionedMessage},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use spl_associated_token_account::get_associated_token_address;
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;
use tracing::info;

// A simple client for making RPC "cheat code" calls to surfpool.
struct SurfpoolClient {
    client: reqwest::Client,
    url: String,
}

impl SurfpoolClient {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            url: "http://127.0.0.1:8899".to_string(),
        }
    }

    async fn set_token_account(&self, owner: &str, mint: &str, amount: u64) -> Result<()> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "surfnet_setTokenAccount",
            "params": [
                owner,
                mint,
                { "amount": amount },
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            ]
        });

        let response = self
            .client
            .post(&self.url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send RPC request to set token account")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            anyhow::bail!("Failed to set token account. Status: {status}, Body: {error_body}");
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Setup: Initialize logging and start the surfpool agent.
    tracing_subscriber::fmt()
        .with_env_filter("info,swap_poc=info")
        .init();

    // 2. Create a new wallet and fund it with 100 USDC using the RPC cheat code.
    let user_wallet = Keypair::new();
    info!("✅ Created user wallet: {}", user_wallet.pubkey());
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let native_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
    let amount_to_set = 100_000_000; // 100 USDC

    let surfpool_client = SurfpoolClient::new();
    surfpool_client
        .set_token_account(
            &user_wallet.pubkey().to_string(),
            &usdc_mint.to_string(),
            amount_to_set,
        )
        .await?;
    info!("✅ Funded wallet with 100 USDC via cheat code.");

    // 3. Verify the initial USDC balance.
    let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
    let user_usdc_ata = get_associated_token_address(&user_wallet.pubkey(), &usdc_mint);

    for _ in 0..10 {
        if let Ok(balance) = rpc_client.get_token_account_balance(&user_usdc_ata) {
            if balance.amount.parse::<u64>()? == amount_to_set {
                info!(
                    "✅ Initial USDC balance verified: {}",
                    balance.ui_amount_string
                );
                break;
            }
        }
        sleep(Duration::from_millis(500)).await;
    }

    // 4. Use the Jupiter API to get a quote and then swap INSTRUCTIONS.
    let jupiter_client = JupiterSwapApiClient::new("https://quote-api.jup.ag/v6".into());
    let quote_request = QuoteRequest {
        amount: 10_000_000, // 10 USDC
        input_mint: usdc_mint,
        output_mint: native_mint,
        slippage_bps: 100, // 1%
        ..Default::default()
    };

    let quote_response = jupiter_client.quote(&quote_request).await?;
    info!("✅ Got quote from Jupiter API.");

    let instructions_response = jupiter_client
        .swap_instructions(&SwapRequest {
            user_public_key: user_wallet.pubkey(),
            quote_response,
            config: TransactionConfig::default(),
        })
        .await?;
    info!("✅ Got swap instructions from Jupiter API.");

    // 5. Build the transaction locally using a fresh blockhash and ALTs.
    let lookup_table_keys: Vec<Pubkey> = instructions_response
        .address_lookup_table_addresses
        .to_vec();

    let alt_accounts = rpc_client
        .get_multiple_accounts(&lookup_table_keys)?
        .into_iter()
        .zip(lookup_table_keys)
        .filter_map(|(acc, key)| acc.map(|a| (key, a)))
        .map(|(key, acc)| {
            let table = AddressLookupTable::deserialize(&acc.data)?;
            Ok(AddressLookupTableAccount {
                key,
                addresses: table.addresses.to_vec(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    info!("✅ Fetched {} address lookup tables.", alt_accounts.len());

    let mut instructions: Vec<Instruction> = Vec::new();
    for setup_ix in instructions_response.setup_instructions {
        instructions.push(setup_ix);
    }
    instructions.push(instructions_response.swap_instruction);
    if let Some(cleanup_ix) = instructions_response.cleanup_instruction {
        instructions.push(cleanup_ix);
    }

    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    let message = v0::Message::try_compile(
        &user_wallet.pubkey(),
        &instructions,
        &alt_accounts,
        latest_blockhash,
    )?;
    info!("✅ Compiled transaction message with local blockhash.");

    let transaction =
        VersionedTransaction::try_new(VersionedMessage::V0(message), &[&user_wallet])?;
    info!("✅ Signed transaction locally.");

    // 6. Send and confirm the transaction.
    let async_rpc_client = AsyncRpcClient::new("http://127.0.0.1:8899".to_string());
    let signature = async_rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .context("Failed to send and confirm swap transaction")?;
    info!("✅ SWAP TRANSACTION CONFIRMED! Signature: {}", signature);

    // 7. Verify the final USDC balance.
    let final_balance = rpc_client.get_token_account_balance(&user_usdc_ata)?;
    assert!(
        final_balance.amount.parse::<u64>()? < amount_to_set,
        "Final balance should be less than initial balance."
    );
    info!(
        "✅ Final USDC balance verified: {}. Swap successful!",
        final_balance.ui_amount_string
    );

    Ok(())
}
