use anyhow::Result;
use bs58;
use jup_sdk::{models::SwapParams, Jupiter};
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use tracing::{debug, info};

/// Handle Jupiter swap operation using the jup-sdk.
/// This is the real protocol handler that contains the actual Jupiter API logic.
pub async fn handle_jupiter_swap(
    user_pubkey: Pubkey,
    input_mint: Pubkey,
    output_mint: Pubkey,
    amount: u64,
    slippage_bps: u16,
    _key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    let config = super::get_jupiter_config();

    // Log configuration if debug mode is enabled
    config.log_config();

    // Validate slippage against configuration limits
    let validated_slippage = config.validate_slippage(slippage_bps)?;

    info!(
        "Executing Jupiter swap: {} -> {} (amount: {}, slippage: {} bps)",
        input_mint, output_mint, amount, validated_slippage
    );

    // The jup-sdk's client is designed to work with a local validator.
    let jupiter_client = Jupiter::surfpool().with_user_pubkey(user_pubkey);

    // Apply custom RPC URL if configured
    if let Some(ref rpc_url) = config.surfpool_rpc_url {
        debug!("Using custom RPC URL for surfpool: {}", rpc_url);
        // Note: jup-sdk would need to support custom RPC URLs
        // This is a placeholder for when that functionality is available
    }

    let swap_params = SwapParams {
        input_mint,
        output_mint,
        amount,
        slippage_bps: validated_slippage,
    };

    debug!("Swap params: {:?}", swap_params);

    // The sdk's swap builder will handle quoting and instruction generation
    // against the local surfpool instance.
    let (instructions, _alt_accounts) = jupiter_client
        .swap(swap_params)
        .prepare_transaction_components()
        .await?;

    debug!("Generated {} instructions from Jupiter", instructions.len());

    // The sdk returns instructions in its own format, so we need to convert them.
    let raw_instructions: Vec<RawInstruction> = instructions
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
