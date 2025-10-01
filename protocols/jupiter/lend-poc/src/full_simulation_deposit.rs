//! # Jupiter Lend Deposit Proof-of-Concept
//!
//! This demonstrates a full end-to-end Jupiter lend deposit
//! against a local surfpool (mainnet fork) validator.

use crate::common::surfpool_client::SurfpoolClient;
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use solana_client::{nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_client::RpcClient};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::{v0, VersionedMessage},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use spl_associated_token_account::get_associated_token_address;
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;
use tracing::info;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LendResponse {
    instructions: Vec<InstructionData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstructionData {
    program_id: String,
    accounts: Vec<Key>,
    data: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Key {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

pub async fn execute_lend_deposit() -> Result<()> {
    const PUBLIC_RPC_URL: &str = "https://api.mainnet-beta.solana.com";

    // 1. Create a new wallet and fund it.
    let user_wallet = Keypair::new();
    info!("✅ Created user wallet: {}", user_wallet.pubkey());
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let amount_to_set = 100_000_000; // 100 USDC

    let surfpool_client = SurfpoolClient::new();
    surfpool_client
        .set_account(&user_wallet.pubkey().to_string(), 1_000_000_000)
        .await?;
    info!("✅ Funded wallet with 1 SOL via cheat code.");
    surfpool_client
        .set_token_account(
            &user_wallet.pubkey().to_string(),
            &usdc_mint.to_string(),
            amount_to_set,
        )
        .await?;
    info!("✅ Funded wallet with 100 USDC via cheat code.");

    // 2. Verify the initial USDC balance.
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

    // Get deposit instruction
    let client = reqwest::Client::new();
    let data = serde_json::json!({
        "asset": usdc_mint.to_string(),
        "signer": user_wallet.pubkey().to_string(),
        "amount": "100000",
        "cluster": "mainnet"
    });
    let response = client
        .post("https://lite-api.jup.ag/lend/v1/earn/deposit-instructions")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&data)
        .send()
        .await?
        .error_for_status()?;
    let body = response.text().await?;
    info!("API response body: {}", body);
    let response: LendResponse = serde_json::from_str(&body)?;
    let instr = response
        .instructions
        .into_iter()
        .next()
        .ok_or(anyhow!("No instructions"))?;
    info!("✅ Got deposit instruction from Jupiter Lend API.");

    let InstructionData {
        program_id,
        accounts,
        data,
    } = instr;

    let deposit_instruction = Instruction {
        program_id: Pubkey::from_str(&program_id)?,
        accounts: accounts
            .into_iter()
            .map(|k| -> Result<AccountMeta> {
                let pubkey = Pubkey::from_str(&k.pubkey).context("Invalid pubkey")?;
                Ok(AccountMeta {
                    pubkey,
                    is_signer: k.is_signer,
                    is_writable: k.is_writable,
                })
            })
            .collect::<Result<Vec<_>>>()?,

        data: STANDARD.decode(&data)?,
    };

    // Build deposit transaction
    surfpool_client.time_travel_to_now().await?;
    info!("✅ Time traveled to now.");
    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    let message = v0::Message::try_compile(
        &user_wallet.pubkey(),
        &[deposit_instruction],
        &[],
        latest_blockhash,
    )?;
    info!("✅ Compiled deposit transaction message with local blockhash.");

    let transaction =
        VersionedTransaction::try_new(VersionedMessage::V0(message.clone()), &[&user_wallet])?;
    info!("✅ Signed deposit transaction locally.");

    // Diagnostic for deposit
    info!("--- Verifying all deposit transaction accounts exist ---");
    let static_keys = &message.account_keys;
    let mut all_keys: Vec<Pubkey> = static_keys.to_vec();
    all_keys.sort();
    all_keys.dedup();

    info!(
        "Found {} static keys. Total unique accounts to verify: {}.",
        static_keys.len(),
        all_keys.len()
    );

    let mut missing_accounts = Vec::new();
    for chunk in all_keys.chunks(100) {
        let accounts_from_rpc = rpc_client.get_multiple_accounts(chunk)?;
        for (key, account_option) in chunk.iter().zip(accounts_from_rpc.iter()) {
            if account_option.is_none() {
                missing_accounts.push(*key);
            }
        }
    }

    missing_accounts.retain(|&pk| pk != user_wallet.pubkey());

    if !missing_accounts.is_empty() {
        info!(
            "🚨 Found {} missing accounts. Pre-loading them into surfpool...",
            missing_accounts.len()
        );
        let public_rpc_client = RpcClient::new(PUBLIC_RPC_URL.to_string());
        let accounts_to_load = public_rpc_client
            .get_multiple_accounts(&missing_accounts)
            .context("Failed to fetch missing accounts from mainnet RPC")?;

        for (pubkey, account_option) in missing_accounts.iter().zip(accounts_to_load.iter()) {
            if let Some(account) = account_option {
                info!(
                    "   -> Loading account {} with {} lamports",
                    pubkey, account.lamports
                );
                surfpool_client
                    .set_account_from_account(pubkey, account.clone())
                    .await?;
            } else {
                info!(
                    "⚠️ Could not fetch account {} from mainnet RPC. Assuming it's created by the transaction.",
                    pubkey
                );
            }
        }
        info!("✅ Pre-loaded all missing accounts.");
    } else {
        info!(
            "✅ All {} unique accounts already exist locally.",
            all_keys.len()
        );
    }
    info!("--- Deposit account verification complete ---");

    // Send deposit
    let async_rpc_client = AsyncRpcClient::new("http://127.0.0.1:8899".to_string());
    let signature = async_rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .context("Failed to send and confirm deposit transaction")?;
    info!("✅ DEPOSIT TRANSACTION CONFIRMED! Signature: {}", signature);

    // Check balance after deposit
    let final_balance = rpc_client.get_token_account_balance(&user_usdc_ata)?;
    assert!(
        final_balance.amount.parse::<u64>()? < amount_to_set,
        "Final balance should be less than initial balance."
    );
    info!(
        "✅ Final USDC balance verified: {}. Deposit successful!",
        final_balance.ui_amount_string
    );

    Ok(())
}
