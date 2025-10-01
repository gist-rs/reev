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

pub async fn execute_lend_withdraw() -> Result<()> {
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

    // Deposit
    let client = reqwest::Client::new();
    let deposit_data = serde_json::json!({
        "asset": usdc_mint.to_string(),
        "signer": user_wallet.pubkey().to_string(),
        "amount": "100000",
        "cluster": "mainnet"
    });

    let deposit_response = client
        .post("https://lite-api.jup.ag/lend/v1/earn/deposit-instructions")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&deposit_data)
        .send()
        .await?
        .error_for_status()?;
    let deposit_body = deposit_response.text().await?;
    info!("deposit_body: {}", deposit_body);
    let deposit_resp: LendResponse = serde_json::from_str(&deposit_body)?;
    let deposit_instr = deposit_resp
        .instructions
        .into_iter()
        .next()
        .ok_or(anyhow!("No deposit instructions"))?;
    info!("✅ Got deposit instruction from Jupiter Lend API.");

    // Assume the second account is the lend token ATA
    let lend_ata = Pubkey::from_str(&deposit_instr.accounts[2].pubkey)?;

    let InstructionData {
        program_id: deposit_program_id,
        accounts: deposit_accounts,
        data: deposit_instruction_data,
    } = deposit_instr;

    let deposit_instruction = Instruction {
        program_id: Pubkey::from_str(&deposit_program_id)?,
        accounts: deposit_accounts
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
        data: STANDARD.decode(&deposit_instruction_data)?,
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
        info!("✅ Pre-loaded all missing accounts for deposit.");
    } else {
        info!(
            "✅ All {} unique accounts already exist locally.",
            all_keys.len()
        );
    }
    info!("--- Deposit account verification complete ---");

    // Send deposit
    let async_rpc_client = AsyncRpcClient::new("http://127.0.0.1:8899".to_string());
    let deposit_signature = async_rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .context("Failed to send and confirm deposit transaction")?;
    info!(
        "✅ DEPOSIT TRANSACTION CONFIRMED! Signature: {}",
        deposit_signature
    );

    // Check balance after deposit
    let balance_after_deposit = rpc_client.get_token_account_balance(&user_usdc_ata)?;
    info!(
        "✅ USDC balance after deposit: {}",
        balance_after_deposit.ui_amount_string
    );

    // Get lend token balance for withdraw amount
    let lend_balance = rpc_client.get_token_account_balance(&lend_ata)?;
    let withdraw_amount = lend_balance.amount;
    info!("Lend token balance: {}", withdraw_amount);

    // Now withdraw
    let withdraw_data = serde_json::json!({
        "asset": usdc_mint.to_string(),
        "signer": user_wallet.pubkey().to_string(),
        "amount": withdraw_amount,
        "cluster": "mainnet"
    });
    let withdraw_response = client
        .post("https://lite-api.jup.ag/lend/v1/earn/withdraw-instructions")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&withdraw_data)
        .send()
        .await?
        .error_for_status()?;
    let withdraw_body = withdraw_response.text().await?;
    info!("withdraw_body: {}", withdraw_body);
    let withdraw_resp: LendResponse = serde_json::from_str(&withdraw_body)?;
    let withdraw_instr = withdraw_resp
        .instructions
        .into_iter()
        .next()
        .ok_or(anyhow!("No withdraw instructions"))?;
    info!("✅ Got withdraw instruction from Jupiter Lend API.");

    let InstructionData {
        program_id: withdraw_program_id,
        accounts: withdraw_accounts,
        data: withdraw_data,
    } = withdraw_instr;

    let withdraw_instruction = Instruction {
        program_id: Pubkey::from_str(&withdraw_program_id)?,
        accounts: withdraw_accounts
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
        data: STANDARD.decode(&withdraw_data)?,
    };

    // Build withdraw transaction
    surfpool_client.time_travel_to_now().await?;
    info!("✅ Time traveled to now for withdraw.");
    let latest_blockhash2 = rpc_client.get_latest_blockhash()?;
    let message2 = v0::Message::try_compile(
        &user_wallet.pubkey(),
        &[withdraw_instruction],
        &[],
        latest_blockhash2,
    )?;
    info!("✅ Compiled withdraw transaction message with local blockhash.");

    let transaction2 =
        VersionedTransaction::try_new(VersionedMessage::V0(message2.clone()), &[&user_wallet])?;
    info!("✅ Signed withdraw transaction locally.");

    // Diagnostic for withdraw
    info!("--- Verifying all withdraw transaction accounts exist ---");
    let static_keys2 = &message2.account_keys;
    let mut all_keys2: Vec<Pubkey> = static_keys2.to_vec();
    all_keys2.sort();
    all_keys2.dedup();
    info!(
        "Found {} static keys. Total unique accounts to verify: {}.",
        static_keys2.len(),
        all_keys2.len()
    );

    let mut missing_accounts2 = Vec::new();
    for chunk in all_keys2.chunks(100) {
        let accounts_from_rpc = rpc_client.get_multiple_accounts(chunk)?;
        for (key, account_option) in chunk.iter().zip(accounts_from_rpc.iter()) {
            if account_option.is_none() {
                missing_accounts2.push(*key);
            }
        }
    }

    missing_accounts2.retain(|&pk| pk != user_wallet.pubkey());

    if !missing_accounts2.is_empty() {
        info!(
            "🚨 Found {} missing accounts for withdraw. Pre-loading them into surfpool...",
            missing_accounts2.len()
        );
        let public_rpc_client = RpcClient::new(PUBLIC_RPC_URL.to_string());
        let accounts_to_load2 = public_rpc_client
            .get_multiple_accounts(&missing_accounts2)
            .context("Failed to fetch missing accounts from mainnet RPC for withdraw")?;

        for (pubkey, account_option) in missing_accounts2.iter().zip(accounts_to_load2.iter()) {
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
        info!("✅ Pre-loaded all missing accounts for withdraw.");
    } else {
        info!(
            "✅ All {} unique accounts already exist locally.",
            all_keys2.len()
        );
    }
    info!("--- Withdraw account verification complete ---");

    // Send withdraw
    let withdraw_signature = async_rpc_client
        .send_and_confirm_transaction(&transaction2)
        .await
        .context("Failed to send and confirm withdraw transaction")?;
    info!(
        "✅ WITHDRAW TRANSACTION CONFIRMED! Signature: {}",
        withdraw_signature
    );

    // Verify final balance
    let final_balance = rpc_client.get_token_account_balance(&user_usdc_ata)?;
    if final_balance.amount.parse::<u64>()? >= amount_to_set - 2000000 {
        info!(
            "✅ Final USDC balance verified: {}. Withdraw successful!",
            final_balance.ui_amount_string
        );
    } else {
        info!(
            "⚠️ Final USDC balance: {}. Expected close to initial.",
            final_balance.ui_amount_string
        );
    }

    Ok(())
}
