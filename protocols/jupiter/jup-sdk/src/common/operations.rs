use anyhow::{Context, Result};
use solana_client::{nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_client::RpcClient};
use solana_sdk::{
    address_lookup_table::AddressLookupTableAccount,
    instruction::Instruction,
    message::{v0, VersionedMessage},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use spl_associated_token_account::get_associated_token_address;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

use super::config;
use super::surfpool_client::SurfpoolClient;

/// Setup wallet with SOL and tokens, verify balance
pub async fn setup_wallet(
    rpc_client: &RpcClient,
    surfpool_client: &SurfpoolClient,
    user_wallet: &Keypair,
    asset: &Pubkey,
    amount_to_set: u64,
) -> Result<()> {
    surfpool_client
        .set_account(&user_wallet.pubkey().to_string(), 1_000_000_000)
        .await?;
    info!("✅ Funded wallet with 1 SOL via cheat code.");
    surfpool_client
        .set_token_account(
            &user_wallet.pubkey().to_string(),
            &asset.to_string(),
            amount_to_set,
        )
        .await?;
    info!(
        "✅ Funded wallet with {} tokens via cheat code.",
        amount_to_set
    );

    // Verify balance
    let ata = get_associated_token_address(&user_wallet.pubkey(), asset);
    for _ in 0..10 {
        if let Ok(balance) = rpc_client.get_token_account_balance(&ata) {
            if balance.amount.parse::<u64>()? == amount_to_set {
                info!("✅ Initial balance verified: {}", balance.ui_amount_string);
                break;
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

/// Execute transaction with diagnostics and account pre-loading
pub async fn execute_transaction(
    rpc_client: &RpcClient,
    surfpool_client: &SurfpoolClient,
    user_wallet: &Keypair,
    instructions: &[Instruction],
    alt_accounts: &[AddressLookupTableAccount],
) -> Result<()> {
    // Time travel
    surfpool_client.time_travel_to_now().await?;
    info!("✅ Time traveled to now.");

    // Get blockhash
    let latest_blockhash = rpc_client.get_latest_blockhash()?;

    // Compile message
    let message = v0::Message::try_compile(
        &user_wallet.pubkey(),
        instructions,
        alt_accounts,
        latest_blockhash,
    )?;
    info!("✅ Compiled transaction message with local blockhash.");

    // Sign
    let transaction =
        VersionedTransaction::try_new(VersionedMessage::V0(message.clone()), &[user_wallet])?;
    info!("✅ Signed transaction locally.");

    // Diagnostics
    let static_keys = &message.account_keys;
    let alt_keys: Vec<Pubkey> = alt_accounts
        .iter()
        .flat_map(|table| table.addresses.clone())
        .collect();
    let mut all_keys: Vec<Pubkey> = static_keys
        .iter()
        .cloned()
        .chain(alt_keys.into_iter())
        .collect();
    all_keys.sort();
    all_keys.dedup();

    let alt_key_count = alt_accounts
        .iter()
        .map(|table| table.addresses.len())
        .sum::<usize>();
    info!(
        "Found {} static keys and {} keys in {} ALTs. Total unique accounts to verify: {}.",
        static_keys.len(),
        alt_key_count,
        alt_accounts.len(),
        all_keys.len()
    );

    // Pre-load missing accounts
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
        let public_rpc_url = config::public_rpc_url();
        let public_rpc_client = RpcClient::new(public_rpc_url);
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
                info!("⚠️ Could not fetch account {} from mainnet RPC. Assuming it's created by the transaction.", pubkey);
            }
        }
        info!("✅ Pre-loaded all missing accounts.");
    } else {
        info!(
            "✅ All {} unique accounts already exist locally.",
            all_keys.len()
        );
    }
    info!("--- Account verification complete ---");

    // Send transaction
    let async_rpc_client = AsyncRpcClient::new(rpc_client.url().to_string());
    let signature = async_rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .context("Failed to send and confirm transaction")?;
    info!("✅ TRANSACTION CONFIRMED! Signature: {}", signature);

    Ok(())
}
