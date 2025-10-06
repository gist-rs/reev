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

/// Handle Jupiter lend deposit operation using the jup-sdk.
/// This is the real protocol handler that contains the actual Jupiter API logic.
pub async fn handle_jupiter_lend_deposit(
    user_pubkey: Pubkey,
    asset_mint: Pubkey,
    amount: u64,
    _key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
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

    let deposit_params = DepositParams { asset_mint, amount };

    // The sdk's deposit builder will handle instruction generation
    // against the local surfpool instance.
    let (jupiter_sdk_instructions, _alt_accounts) = jupiter_client
        .deposit(deposit_params)
        .prepare_transaction_components()
        .await?;

    // Combine setup instructions with Jupiter instructions and convert them to the agent's format.
    let all_sdk_instructions = [setup_instructions, jupiter_sdk_instructions].concat();

    let raw_instructions = all_sdk_instructions
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

    Ok(raw_instructions)
}
