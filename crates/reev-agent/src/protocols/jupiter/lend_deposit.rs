//! Jupiter lend deposit protocol handler
//!
//! This module provides the real Jupiter API integration for lend deposit operations.

use anyhow::Result;
use bs58;
use jup_sdk::{models::DepositParams, Jupiter};
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use solana_system_interface::instruction as system_instruction;
use spl_associated_token_account;
use spl_token;
use std::collections::HashMap;
use tracing::{debug, info};

/// Handle Jupiter lend deposit operation using the jup-sdk.
/// This is the real protocol handler that contains the actual Jupiter API logic.
pub async fn handle_jupiter_lend_deposit(
    user_pubkey: Pubkey,
    asset_mint: Pubkey,
    amount: u64,
    _key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    // Check for placeholder addresses that would cause Base58 parsing errors
    let user_pubkey_str = user_pubkey.to_string();
    let asset_mint_str = asset_mint.to_string();

    info!(
        "DEBUG: Jupiter lend deposit called with user_pubkey={}, asset_mint={}, amount={}",
        user_pubkey_str, asset_mint_str, amount
    );

    // If we detect placeholder addresses, return simulated instructions
    if user_pubkey_str.starts_with("USER_")
        || user_pubkey_str.starts_with("RECIPIENT_")
        || asset_mint_str.starts_with("USER_")
        || asset_mint_str.starts_with("RECIPIENT_")
    {
        info!("Detected placeholder addresses, returning simulated deposit instructions");
        return Ok(vec![RawInstruction {
            program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
            accounts: vec![
                RawAccountMeta {
                    pubkey: user_pubkey_str.clone(),
                    is_signer: true,
                    is_writable: true,
                },
                RawAccountMeta {
                    pubkey: "PLACEHOLDER_TOKEN_ACCOUNT".to_string(),
                    is_signer: false,
                    is_writable: true,
                },
                RawAccountMeta {
                    pubkey: "PLACEHOLDER_PROGRAM_ID".to_string(),
                    is_signer: false,
                    is_writable: false,
                },
            ],
            data: "SIMULATED_DEPOSIT".to_string(),
        }]);
    }

    let config = super::get_jupiter_config();

    // Log configuration if debug mode is enabled
    config.log_config();

    info!(
        "Executing Jupiter lend deposit: {} (amount: {})",
        asset_mint, amount
    );

    let mut setup_instructions: Vec<Instruction> = Vec::new();

    // If depositing native SOL, prerequisite instructions to wrap it are required.
    if asset_mint == spl_token::native_mint::ID {
        let wsol_ata = spl_associated_token_account::get_associated_token_address(
            &user_pubkey,
            &spl_token::native_mint::ID,
        );

        setup_instructions = vec![
            // 1. Create the associated token account for WSOL if it doesn't exist.
            spl_associated_token_account::instruction::create_associated_token_account(
                &user_pubkey,
                &user_pubkey,
                &spl_token::native_mint::ID,
                &spl_token::ID,
            ),
            // 2. Transfer native SOL to the ATA, which wraps it.
            system_instruction::transfer(&user_pubkey, &wsol_ata, amount),
            // 3. Sync the ATA to ensure the WSOL balance is recognized.
            spl_token::instruction::sync_native(&spl_token::ID, &wsol_ata)?,
        ];
    }

    // The jup-sdk's client is designed to work with a local validator.
    let jupiter_client = Jupiter::surfpool().with_user_pubkey(user_pubkey);
    // Apply custom RPC URL if configured
    if let Some(ref rpc_url) = config.surfpool_rpc_url {
        debug!("Using custom RPC URL for surfpool: {}", rpc_url);
        // Note: jup-sdk would need to support custom RPC URLs
        // This is a placeholder for when that functionality is available
    }

    let deposit_params = DepositParams { asset_mint, amount };

    debug!("Deposit params: {:?}", deposit_params);

    // The sdk's deposit builder will handle instruction generation
    // against the local surfpool instance.
    let (jupiter_sdk_instructions, _alt_accounts) = jupiter_client
        .deposit(deposit_params)
        .prepare_transaction_components()
        .await?;

    debug!(
        "Generated {} instructions from Jupiter",
        jupiter_sdk_instructions.len()
    );

    // Combine setup instructions with Jupiter instructions and convert them to the agent's format.
    let all_sdk_instructions = [setup_instructions, jupiter_sdk_instructions].concat();

    let raw_instructions: Vec<RawInstruction> = all_sdk_instructions
        .into_iter()
        .map(|inst| {
            let accounts = inst
                .accounts
                .into_iter()
                .map(|acc| RawAccountMeta {
                    pubkey: acc.pubkey.to_string(),
                    is_signer: acc.is_signer,
                    is_writable: acc.is_writable,
                })
                .collect();

            RawInstruction {
                program_id: inst.program_id.to_string(),
                accounts,
                data: bs58::encode(inst.data).into_string(),
            }
        })
        .collect();

    info!(
        "Successfully converted {} instructions to RawInstruction format",
        raw_instructions.len()
    );

    Ok(raw_instructions)
}
