//! This module contains all logic specific to running simulations against a `surfpool` instance.
//! It includes the cheat code client, environment setup, and transaction execution for testing.

use crate::{
    config,
    models::{DebugInfo, SimulationResult, TransactionResult},
};
use anyhow::{Context, Result};
use solana_client::{nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_client::RpcClient};
use solana_sdk::{
    account::Account,
    address_lookup_table::AddressLookupTableAccount,
    hash::Hash,
    instruction::Instruction,
    message::{VersionedMessage, v0},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use spl_associated_token_account::get_associated_token_address;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

// --- SurfpoolClient for cheat codes ---

/// A client for making RPC "cheat code" calls to a surfpool instance.
pub struct SurfpoolClient {
    client: reqwest::Client,
    url: String,
}

impl SurfpoolClient {
    pub fn new(url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: url.to_string(),
        }
    }

    /// Sets the balance of an SPL token account for a given owner.
    pub async fn set_token_account(&self, owner: &str, mint: &str, amount: u64) -> Result<()> {
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

    /// Sets the lamport balance of a given account.
    pub async fn set_account(&self, pubkey: &str, lamports: u64) -> Result<()> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "surfnet_setAccount",
            "params": [
                pubkey,
                { "lamports": lamports }
            ]
        });

        let response = self
            .client
            .post(&self.url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send RPC request to set account")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            anyhow::bail!("Failed to set account. Status: {status}, Body: {error_body}");
        }
        Ok(())
    }

    /// Sets the entire state of an account from an `Account` struct.
    pub async fn set_account_from_account(&self, pubkey: &Pubkey, account: Account) -> Result<()> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "surfnet_setAccount",
            "params": [
                pubkey.to_string(),
                {
                    "lamports": account.lamports,
                    "owner": account.owner.to_string(),
                    "executable": account.executable,
                    "rent_epoch": account.rent_epoch,
                    "data": hex::encode(&account.data),
                }
            ]
        });

        let response = self
            .client
            .post(&self.url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send RPC request to set account from account")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            anyhow::bail!(
                "Failed to set account from account. Status: {status}, Body: {error_body}"
            );
        }
        Ok(())
    }

    /// Advances the validator's clock to the current real-world time.
    pub async fn time_travel_to_now(&self) -> Result<()> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "surfnet_timeTravel",
            "params": [{ "unix_timestamp": chrono::Utc::now().timestamp() }]
        });

        let response = self
            .client
            .post(&self.url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send RPC request to time travel")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            anyhow::bail!("Failed to time travel. Status: {status}, Body: {error_body}");
        }
        Ok(())
    }
}

// --- Simulation Orchestration ---

/// Sets up a wallet with SOL and a specific SPL token for simulation.
pub async fn setup_wallet(
    rpc_client: &RpcClient,
    surfpool_client: &SurfpoolClient,
    user_wallet: &Keypair,
    asset_mint: &Pubkey,
    amount_to_set: u64,
) -> Result<()> {
    surfpool_client
        .set_account(&user_wallet.pubkey().to_string(), 1_000_000_000) // 1 SOL
        .await?;
    info!("✅ [SIM] Funded wallet with 1 SOL.");

    if amount_to_set > 0 {
        surfpool_client
            .set_token_account(
                &user_wallet.pubkey().to_string(),
                &asset_mint.to_string(),
                amount_to_set,
            )
            .await?;
        info!(
            "✅ [SIM] Funded wallet with {} of token {}.",
            amount_to_set, asset_mint
        );

        let ata = get_associated_token_address(&user_wallet.pubkey(), asset_mint);
        for _ in 0..10 {
            if let Ok(balance) = rpc_client.get_token_account_balance(&ata) {
                if balance.amount.parse::<u64>()? == amount_to_set {
                    info!(
                        "✅ [SIM] Initial balance verified: {}",
                        balance.ui_amount_string
                    );
                    return Ok(());
                }
            }
            sleep(Duration::from_millis(500)).await;
        }
        anyhow::bail!("Failed to verify initial token balance after setup.");
    }
    Ok(())
}

/// Pre-loads all accounts required for a transaction from mainnet into the local surfpool fork.
pub async fn preload_accounts(
    local_rpc_client: &RpcClient,
    surfpool_client: &SurfpoolClient,
    user_pubkey: &Pubkey,
    instructions: &[Instruction],
    alt_accounts: &[AddressLookupTableAccount],
) -> Result<()> {
    // Compile a temporary message just to get all the static account keys.
    let temp_message =
        v0::Message::try_compile(user_pubkey, instructions, alt_accounts, Hash::new_unique())?;

    let static_keys = &temp_message.account_keys;
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

    info!(
        "[SIM] Verifying {} unique accounts for transaction.",
        all_keys.len()
    );

    let mut missing_accounts = Vec::new();
    for chunk in all_keys.chunks(100) {
        let accounts_from_rpc = local_rpc_client.get_multiple_accounts(chunk)?;
        for (key, account_option) in chunk.iter().zip(accounts_from_rpc.iter()) {
            if account_option.is_none() {
                missing_accounts.push(*key);
            }
        }
    }

    missing_accounts.retain(|&pk| &pk != user_pubkey);

    if !missing_accounts.is_empty() {
        info!(
            "🚨 [SIM] Found {} missing accounts. Pre-loading...",
            missing_accounts.len()
        );

        let public_rpc_client = RpcClient::new(config::public_rpc_url());
        let accounts_to_load = public_rpc_client
            .get_multiple_accounts(&missing_accounts)
            .context("Failed to fetch missing accounts from mainnet RPC")?;

        for (pubkey, account_option) in missing_accounts.iter().zip(accounts_to_load.iter()) {
            if let Some(account) = account_option {
                surfpool_client
                    .set_account_from_account(pubkey, account.clone())
                    .await?;
            } else {
                info!(
                    "⚠️ [SIM] Could not fetch account {} from mainnet. Assuming it is created by the tx.",
                    pubkey
                );
            }
        }
        info!("✅ [SIM] Pre-loaded all missing accounts.");
    } else {
        info!("✅ [SIM] All unique accounts already exist locally.");
    }

    Ok(())
}

/// Executes a transaction against the `surfpool` environment, handling all simulation-specific logic.
pub async fn execute_simulation(
    rpc_client: &RpcClient,
    surfpool_client: &SurfpoolClient,
    user_wallet: &Keypair,
    instructions: Vec<Instruction>,
    alt_accounts: Vec<AddressLookupTableAccount>,
) -> Result<SimulationResult> {
    // 1. Pre-load accounts before doing anything state-modifying.
    preload_accounts(
        rpc_client,
        surfpool_client,
        &user_wallet.pubkey(),
        &instructions,
        &alt_accounts,
    )
    .await?;

    // 2. Time-travel to now to ensure oracles are fresh.
    surfpool_client.time_travel_to_now().await?;
    info!("✅ [SIM] Time traveled to now.");

    // 3. Get a fresh blockhash from the local (time-traveled) surfpool instance.
    let latest_blockhash = rpc_client
        .get_latest_blockhash()
        .context("Failed to get latest blockhash for simulation")?;

    // 4. Compile the message with the fresh blockhash.
    let message = v0::Message::try_compile(
        &user_wallet.pubkey(),
        &instructions,
        &alt_accounts,
        latest_blockhash,
    )?;
    info!("✅ [SIM] Compiled transaction message with local blockhash.");

    // 5. Create a new, signed transaction using `try_new`. This is the correct way.
    let transaction = VersionedTransaction::try_new(VersionedMessage::V0(message), &[user_wallet])
        .context("Failed to sign transaction for simulation")?;
    info!("✅ [SIM] Signed transaction locally.");

    // 6. Send and confirm.
    let async_rpc_client = AsyncRpcClient::new(rpc_client.url());
    let signature = async_rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .context("Failed to send and confirm transaction in simulation")?;

    info!("✅ [SIM] TRANSACTION CONFIRMED! Signature: {}", signature);

    // TODO: Populate DebugInfo with pre/post balances and other metrics.
    Ok(SimulationResult {
        signature: signature.to_string(),
        debug_info: DebugInfo {
            readable_accounts: vec![],
            tx_error: None,
            tx_result: TransactionResult::Success,
            initial_source_token_balance: None,
            final_source_token_balance: None,
            initial_destination_token_balance: None,
            final_destination_token_balance: None,
        },
    })
}
