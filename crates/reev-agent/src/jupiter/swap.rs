use crate::common::surfpool_client::SurfpoolClient;
use anyhow::{Context, Result};
use jupiter_swap_api_client::{
    quote::QuoteRequest, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use reev_lib::agent::RawInstruction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    address_lookup_table::{state::AddressLookupTable, AddressLookupTableAccount},
    instruction::Instruction,
    message::v0,
    pubkey::Pubkey,
};
use std::collections::HashMap;
use tracing::{info, warn};

const LOCAL_RPC_URL: &str = "http://127.0.0.1:8899";
const PUBLIC_RPC_URL: &str = "https://api.mainnet-beta.solana.com";

/// Handles the logic for a Jupiter swap, including pre-loading all necessary
/// accounts into a local `surfpool` mainnet fork.
///
/// This function performs the following steps:
/// 1. Calls the public Jupiter API to get a quote and a full set of swap instructions
///    (setup, swap, and cleanup).
/// 2. Compiles a transaction message locally to identify all required accounts (static and in ALTs).
/// 3. Checks the local `surfpool` instance for any of these accounts that are missing.
/// 4. Fetches the missing accounts from a public mainnet RPC.
/// 5. Uses `surfpool` "cheat codes" to load the mainnet account data into the local fork.
/// 6. Returns the complete set of instructions, ready to be executed as a single transaction
///    against the prepared forked environment.
pub async fn handle_jupiter_swap(
    user_pubkey: Pubkey,
    input_mint: Pubkey,
    output_mint: Pubkey,
    amount: u64,
    slippage_bps: u16,
    _key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    info!(
        "[reev-agent] Handling Jupiter swap for surfpool: IN={}, OUT={}, amount={}",
        input_mint, output_mint, amount
    );

    // 1. Get quote and instructions from Jupiter API
    let jupiter_client = JupiterSwapApiClient::new("https://quote-api.jup.ag/v6".to_string());

    let quote_request = QuoteRequest {
        amount,
        input_mint,
        output_mint,
        slippage_bps,
        ..Default::default()
    };
    info!("[reev-agent] Getting Jupiter quote...");
    let quote_response = jupiter_client
        .quote(&quote_request)
        .await
        .context("Failed to get Jupiter quote from API")?;

    info!("[reev-agent] Getting Jupiter swap instructions...");
    let instructions_response = jupiter_client
        .swap_instructions(&SwapRequest {
            user_public_key: user_pubkey,
            quote_response,
            config: TransactionConfig::default(),
        })
        .await
        .context("Failed to get Jupiter swap instructions from API")?;

    // 2. Identify and preload all required accounts into surfpool
    info!("[reev-agent] Starting account pre-loading process for surfpool...");
    let surfpool_client = SurfpoolClient::new();
    let local_rpc_client = RpcClient::new(LOCAL_RPC_URL.to_string());

    // Collect ALL instructions to compile a complete message for account discovery.
    let mut all_instructions: Vec<Instruction> = instructions_response.setup_instructions;
    all_instructions.push(instructions_response.swap_instruction);
    if let Some(cleanup_instruction) = instructions_response.cleanup_instruction {
        all_instructions.push(cleanup_instruction);
    }

    // Fetch Address Lookup Tables (ALTs) from the local fork.
    let lookup_table_keys: Vec<Pubkey> = instructions_response
        .address_lookup_table_addresses
        .to_vec();

    // Check if the ALT accounts themselves exist. If not, load them. This is a critical step.
    if !lookup_table_keys.is_empty() {
        let existing_alts = local_rpc_client.get_multiple_accounts(&lookup_table_keys)?;
        let mut missing_alt_keys = Vec::new();
        for (i, acc_opt) in existing_alts.iter().enumerate() {
            if acc_opt.is_none() {
                missing_alt_keys.push(lookup_table_keys[i]);
            }
        }

        if !missing_alt_keys.is_empty() {
            info!(
                "[reev-agent] Found {} missing ALT accounts. Pre-loading them...",
                missing_alt_keys.len()
            );
            let public_rpc_client = RpcClient::new(PUBLIC_RPC_URL.to_string());
            let accounts_to_load = public_rpc_client
                .get_multiple_accounts(&missing_alt_keys)
                .context("Failed to fetch missing ALTs from mainnet RPC")?;

            for (pubkey, account_option) in missing_alt_keys.iter().zip(accounts_to_load.iter()) {
                if let Some(account) = account_option {
                    surfpool_client
                        .set_account_from_account(pubkey, account.clone())
                        .await?;
                }
            }
        }
    }

    // Now that ALTs are loaded, fetch their contents.
    let alt_accounts = if !lookup_table_keys.is_empty() {
        local_rpc_client
            .get_multiple_accounts(&lookup_table_keys)?
            .into_iter()
            .zip(lookup_table_keys)
            .filter_map(|(acc_opt, key)| acc_opt.map(|acc| (key, acc)))
            .map(|(key, acc)| {
                let table = AddressLookupTable::deserialize(&acc.data)?;
                Ok(AddressLookupTableAccount {
                    key,
                    addresses: table.addresses.to_vec(),
                })
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        Vec::new()
    };
    info!(
        "[reev-agent] Fetched {} address lookup tables from local fork.",
        alt_accounts.len()
    );

    // Align fork time and get a fresh blockhash from the local fork.
    surfpool_client.time_travel_to_now().await?;
    let latest_blockhash = local_rpc_client.get_latest_blockhash()?;

    // Compile the message to get the full list of account keys.
    let message = v0::Message::try_compile(
        &user_pubkey,
        &all_instructions,
        &alt_accounts,
        latest_blockhash,
    )?;

    let static_keys = &message.account_keys;
    let alt_keys: Vec<Pubkey> = alt_accounts
        .iter()
        .flat_map(|table| table.addresses.clone())
        .collect();

    let mut all_unique_keys: Vec<Pubkey> = static_keys
        .iter()
        .cloned()
        .chain(alt_keys.into_iter())
        .collect();
    all_unique_keys.sort();
    all_unique_keys.dedup();

    info!(
        "[reev-agent] Transaction requires {} unique accounts. Verifying their existence locally...",
        all_unique_keys.len()
    );

    // Find which accounts are missing from the local fork.
    let mut missing_keys = Vec::new();
    for chunk in all_unique_keys.chunks(100) {
        let accounts_from_rpc = local_rpc_client.get_multiple_accounts(chunk)?;
        for (key, account_option) in chunk.iter().zip(accounts_from_rpc.iter()) {
            if account_option.is_none() {
                missing_keys.push(*key);
            }
        }
    }

    // Filter out our own wallet, which is expected to be missing from mainnet.
    missing_keys.retain(|&pk| pk != user_pubkey);

    // Fetch and load the missing accounts from mainnet into the local fork.
    if !missing_keys.is_empty() {
        info!(
            "[reev-agent] Found {} missing accounts. Pre-loading them into surfpool...",
            missing_keys.len()
        );

        let public_rpc_client = RpcClient::new(PUBLIC_RPC_URL.to_string());
        let accounts_to_load = public_rpc_client
            .get_multiple_accounts(&missing_keys)
            .context("Failed to fetch missing accounts from mainnet RPC")?;

        for (pubkey, account_option) in missing_keys.iter().zip(accounts_to_load.iter()) {
            if let Some(account) = account_option {
                info!(
                    "   -> Loading account {} ({} lamports) into surfpool",
                    pubkey, account.lamports
                );
                surfpool_client
                    .set_account_from_account(pubkey, account.clone())
                    .await?;
            } else {
                warn!(
                    "   -> WARNING: Could not fetch account {} from mainnet. Assuming it's created by the transaction.",
                    pubkey
                );
            }
        }
        info!("[reev-agent] Finished pre-loading missing accounts.");
    } else {
        info!("[reev-agent] All required accounts already exist in local fork.");
    }

    // 3. Return the complete set of instructions.
    let raw_instructions: Vec<RawInstruction> =
        all_instructions.into_iter().map(|ix| ix.into()).collect();
    info!(
        "[reev-agent] Successfully generated and prepared {} Jupiter swap instructions.",
        raw_instructions.len()
    );

    Ok(raw_instructions)
}
