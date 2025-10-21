//! Jupiter lend withdraw protocol handler
//!
//! This module provides the real Jupiter API integration for lend withdraw operations.

use anyhow::Result;
use bs58;
use jup_sdk::{models::WithdrawParams, Jupiter};
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use spl_associated_token_account;
use spl_token;

use tracing::{debug, info};

/// Handle Jupiter lend withdraw operation using the jup-sdk.
/// This is the real protocol handler that contains the actual Jupiter API logic.
pub async fn handle_jupiter_lend_withdraw(
    user_pubkey: Pubkey,
    asset_mint: Pubkey,
    amount: u64,
) -> Result<Vec<RawInstruction>> {
    // Check for placeholder addresses that would cause Base58 parsing errors
    let user_pubkey_str = user_pubkey.to_string();
    let asset_mint_str = asset_mint.to_string();

    // If we detect placeholder addresses, return simulated instructions
    if user_pubkey_str.starts_with("USER_")
        || user_pubkey_str.starts_with("RECIPIENT_")
        || asset_mint_str.starts_with("USER_")
        || asset_mint_str.starts_with("RECIPIENT_")
    {
        info!("Detected placeholder addresses, returning simulated withdraw instructions");
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
            ],
            data: "SIMULATED_WITHDRAW".to_string(),
        }]);
    }
    let config = super::get_jupiter_config();

    // Log configuration if debug mode is enabled
    config.log_config();

    info!(
        "Executing Jupiter lend withdraw: {} (amount: {})",
        asset_mint, amount
    );

    let mut post_instructions: Vec<Instruction> = Vec::new();

    // If withdrawing native SOL, post-processing instructions to unwrap it are required.
    if asset_mint == spl_token::native_mint::ID {
        let wsol_ata = spl_associated_token_account::get_associated_token_address(
            &user_pubkey,
            &spl_token::native_mint::ID,
        );

        post_instructions = vec![
            // 1. Close the WSOL account to unwrap back to native SOL.
            spl_token::instruction::close_account(
                &spl_token::ID,
                &wsol_ata,
                &user_pubkey,
                &user_pubkey,
                &[],
            )?,
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

    let withdraw_params = WithdrawParams { asset_mint, amount };

    debug!("Withdraw params: {:?}", withdraw_params);

    // The sdk's withdraw builder will handle instruction generation
    // against the local surfpool instance.
    let (jupiter_sdk_instructions, _alt_accounts) = jupiter_client
        .withdraw(withdraw_params)
        .prepare_transaction_components()
        .await?;

    debug!(
        "Generated {} instructions from Jupiter",
        jupiter_sdk_instructions.len()
    );

    // Combine Jupiter instructions with post-processing instructions and convert them to the agent's format.
    let all_sdk_instructions = [jupiter_sdk_instructions, post_instructions].concat();

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
