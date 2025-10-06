use anyhow::Result;
use jup_sdk::{models::SwapParams, Jupiter};
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// Handle Jupiter swap operation using the jup-sdk.
pub async fn handle_jupiter_swap(
    user_pubkey: Pubkey,
    input_mint: Pubkey,
    output_mint: Pubkey,
    amount: u64,
    slippage_bps: u16,
    _key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    // The jup-sdk's client is designed to work with a local validator.
    let jupiter_client = Jupiter::surfpool().with_user_pubkey(user_pubkey);

    let swap_params = SwapParams {
        input_mint,
        output_mint,
        amount,
        slippage_bps,
    };

    // The sdk's swap builder will handle quoting and instruction generation
    // against the local surfpool instance.
    let (instructions, _alt_accounts) = jupiter_client
        .swap(swap_params)
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
