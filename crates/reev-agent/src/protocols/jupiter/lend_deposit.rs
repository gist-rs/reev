use anyhow::Result;
use jup_sdk::{models::DepositParams, Jupiter};
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// Handle Jupiter lend deposit operation
pub async fn handle_jupiter_deposit(
    user_pubkey: Pubkey,
    asset_mint: Pubkey,
    amount: u64,
    _key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    let jupiter_client = Jupiter::surfpool().with_user_pubkey(user_pubkey);

    let deposit_params = DepositParams { asset_mint, amount };

    let (instructions, _alt_accounts) = jupiter_client
        .deposit(deposit_params)
        .prepare_transaction_components()
        .await?;

    // The sdk returns instructions in its own format, so we need to convert them.
    let raw_instructions = instructions
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
