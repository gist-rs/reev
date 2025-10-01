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

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use solana_client::{nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_client::RpcClient};
use solana_sdk::{
    address_lookup_table::{state::AddressLookupTable, AddressLookupTableAccount},
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

use crate::common::{
    api_client as http, surfpool_client::SurfpoolClient, types::InstructionData,
    utils::hex_to_base58,
};

pub async fn swap(
    input_mint: Pubkey,
    output_mint: Pubkey,
    amount: u64,
    slippage_bps: u16,
) -> Result<()> {
    const PUBLIC_RPC_URL: &str = "https://api.mainnet-beta.solana.com";

    // 1. Create a new wallet and fund it.
    let user_wallet = Keypair::new();
    info!("✅ Created user wallet: {}", user_wallet.pubkey());
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let amount_to_set = amount * 2; // Fund double the swap amount

    let surfpool_client = SurfpoolClient::new();
    surfpool_client
        .set_account(&user_wallet.pubkey().to_string(), 1_000_000_000) // 1 SOL
        .await?;
    info!("✅ Funded wallet with 1 SOL via cheat code.");
    surfpool_client
        .set_token_account(
            &user_wallet.pubkey().to_string(),
            &input_mint.to_string(),
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

    // 3. Use the Jupiter API to get a quote and then swap INSTRUCTIONS.
    // We reset the Pyth oracle account to ensure it's not stale, which can cause errors.
    let sol_usdc_hex_feed_id = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
    let sol_usdc_pyth_oracle = hex_to_base58(sol_usdc_hex_feed_id)?;
    surfpool_client.reset_account(&sol_usdc_pyth_oracle).await?;
    info!(
        "✅ Reset SOL/USDC Pyth oracle account: {}",
        sol_usdc_pyth_oracle
    );

    let client = http::api_client();
    let quote_url = format!(
        "https://lite-api.jup.ag/swap/v1/quote?inputMint={input_mint}&outputMint={output_mint}&amount={amount}&slippageBps={slippage_bps}&onlyDirectRoutes=true"
    );
    let quote_resp: Value = client
        .get(&quote_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    info!("✅ Got quote from Jupiter API.");

    let swap_req = json!({
        "userPublicKey": user_wallet.pubkey().to_string(),
        "quoteResponse": quote_resp,
        "config": {
            "computeUnitLimit": 600000,
            "computeUnitPriceMicroLamports": "auto",
            "asLegacyTransaction": false
        }
    });
    let instructions_resp: Value = client
        .post("https://lite-api.jup.ag/swap/v1/swap-instructions")
        .headers(http::json_headers())
        .json(&swap_req)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    info!("✅ Got swap instructions from Jupiter API.");

    // 4. Build the transaction locally using a fresh blockhash and ALTs.
    let mut lookup_table_keys: Vec<Pubkey> = Vec::new();
    if let Some(addrs) = instructions_resp
        .get("addressLookupTableAddresses")
        .and_then(|v| v.as_array())
    {
        for addr in addrs {
            if let Some(s) = addr.as_str() {
                lookup_table_keys.push(Pubkey::from_str(s).map_err(anyhow::Error::from)?);
            }
        }
    }

    let alt_accounts = rpc_client
        .get_multiple_accounts(&lookup_table_keys)?
        .into_iter()
        .zip(lookup_table_keys.clone())
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

    // Add setup instructions
    if let Some(setup_arr) = instructions_resp
        .get("setupInstructions")
        .and_then(|v| v.as_array())
    {
        for instr in setup_arr {
            let ix_data: InstructionData = serde_json::from_value(instr.clone())?;
            let ix = Instruction {
                program_id: Pubkey::from_str(&ix_data.program_id)?,
                accounts: ix_data
                    .accounts
                    .iter()
                    .map(|k| -> Result<AccountMeta> {
                        Ok(AccountMeta {
                            pubkey: Pubkey::from_str(&k.pubkey).map_err(anyhow::Error::from)?,
                            is_signer: k.is_signer,
                            is_writable: k.is_writable,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?,
                data: STANDARD.decode(&ix_data.data)?,
            };
            instructions.push(ix);
        }
    }

    // Add swap instruction
    if let Some(swap_obj) = instructions_resp.get("swapInstruction") {
        let ix_data: InstructionData = serde_json::from_value(swap_obj.clone())?;
        let ix = Instruction {
            program_id: Pubkey::from_str(&ix_data.program_id)?,
            accounts: ix_data
                .accounts
                .iter()
                .map(|k| -> Result<AccountMeta> {
                    Ok(AccountMeta {
                        pubkey: Pubkey::from_str(&k.pubkey).map_err(anyhow::Error::from)?,
                        is_signer: k.is_signer,
                        is_writable: k.is_writable,
                    })
                })
                .collect::<Result<Vec<_>>>()?,
            data: STANDARD.decode(&ix_data.data)?,
        };
        instructions.push(ix);
    }

    // Add cleanup instruction
    if let Some(cleanup_obj) = instructions_resp.get("cleanupInstruction") {
        let ix_data: InstructionData = serde_json::from_value(cleanup_obj.clone())?;
        let ix = Instruction {
            program_id: Pubkey::from_str(&ix_data.program_id)?,
            accounts: ix_data
                .accounts
                .iter()
                .map(|k| -> Result<AccountMeta> {
                    Ok(AccountMeta {
                        pubkey: Pubkey::from_str(&k.pubkey).map_err(anyhow::Error::from)?,
                        is_signer: k.is_signer,
                        is_writable: k.is_writable,
                    })
                })
                .collect::<Result<Vec<_>>>()?,
            data: STANDARD.decode(&ix_data.data)?,
        };
        instructions.push(ix);
    }

    // Add compute budget instructions
    if let Some(compute_arr) = instructions_resp
        .get("computeBudgetInstructions")
        .and_then(|v| v.as_array())
    {
        for instr in compute_arr {
            let ix_data: InstructionData = serde_json::from_value(instr.clone())?;
            let ix = Instruction {
                program_id: Pubkey::from_str(&ix_data.program_id)?,
                accounts: vec![], // Assuming empty for compute budget
                data: STANDARD.decode(&ix_data.data)?,
            };
            instructions.push(ix);
        }
    }

    if instructions.is_empty() {
        return Err(anyhow!("No instructions in swap response"));
    }

    // Align the fork's time with the oracle and get a fresh blockhash.
    surfpool_client.time_travel_to_now().await?;
    info!("✅ Time traveled to now.");
    // Align the fork's time with the oracle and get a fresh blockhash.
    surfpool_client.time_travel_to_now().await?;
    info!("✅ Time traveled to now.");
    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    let message = v0::Message::try_compile(
        &user_wallet.pubkey(),
        &instructions,
        &alt_accounts,
        latest_blockhash,
    )?;
    info!("✅ Compiled transaction message with local blockhash.");

    let transaction =
        VersionedTransaction::try_new(VersionedMessage::V0(message.clone()), &[&user_wallet])?;
    info!("✅ Signed transaction locally.");

    // --- START DIAGNOSTIC CODE ---
    info!("--- Verifying all transaction accounts exist ---");
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

    // Find all accounts that don't exist in the local fork.
    let mut missing_accounts = Vec::new();
    for chunk in all_keys.chunks(100) {
        let accounts_from_rpc = rpc_client.get_multiple_accounts(chunk)?;
        for (key, account_option) in chunk.iter().zip(accounts_from_rpc.iter()) {
            if account_option.is_none() {
                missing_accounts.push(*key);
            }
        }
    }

    // Filter out our own wallet, which we know doesn't exist on mainnet yet.
    missing_accounts.retain(|&pk| pk != user_wallet.pubkey());

    // Proactively fetch and set the missing accounts from mainnet into the local fork.
    // This pre-populates the cache so the transaction simulation doesn't fail.
    if !missing_accounts.is_empty() {
        info!(
            "🚨 Found {} missing accounts. Pre-loading them into surfpool...",
            missing_accounts.len()
        );

        // Fetch the full account data for all missing accounts from a public RPC.
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
                // Use the new client method to set the full account data in surfpool.
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
    info!("--- Account verification complete ---");
    // --- END DIAGNOSTIC CODE ---

    // 5. Send and confirm the transaction.
    let async_rpc_client = AsyncRpcClient::new("http://127.0.0.1:8899".to_string());
    let signature = async_rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .context("Failed to send and confirm swap transaction")?;
    info!("✅ SWAP TRANSACTION CONFIRMED! Signature: {}", signature);

    // 6. Verify the final USDC balance.
    let final_balance = rpc_client.get_token_account_balance(&user_usdc_ata)?;
    assert!(
        final_balance.amount.parse::<u64>()? < amount_to_set,
        "Final input balance should be less than initial balance."
    );
    info!(
        "✅ Final input balance verified: {}. Swap successful!",
        final_balance.ui_amount_string
    );

    Ok(())
}
