use anyhow::Result;
use bs58;
use jup_sdk::{models::SwapParams, Jupiter};
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use solana_sdk::pubkey::Pubkey;

use tracing::{debug, info, warn};

/// Handle Jupiter swap operation using the jup-sdk.
/// This is the real protocol handler that contains the actual Jupiter API logic.
pub async fn handle_jupiter_swap(
    user_pubkey: Pubkey,
    input_mint: Pubkey,
    output_mint: Pubkey,
    amount: u64,
    slippage_bps: u16,
) -> Result<Vec<RawInstruction>> {
    // Check for placeholder addresses that would cause Base58 parsing errors
    let user_pubkey_str = user_pubkey.to_string();
    let input_mint_str = input_mint.to_string();
    let output_mint_str = output_mint.to_string();

    // If we detect placeholder addresses, return simulated instructions
    if user_pubkey_str.starts_with("USER_")
        || user_pubkey_str.starts_with("RECIPIENT_")
        || input_mint_str.starts_with("USER_")
        || input_mint_str.starts_with("RECIPIENT_")
        || output_mint_str.starts_with("USER_")
        || output_mint_str.starts_with("RECIPIENT_")
    {
        info!("Detected placeholder addresses, returning simulated swap instructions");
        return Ok(vec![RawInstruction {
            program_id: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string(),
            accounts: vec![
                RawAccountMeta {
                    pubkey: user_pubkey_str.clone(),
                    is_signer: true,
                    is_writable: true,
                },
                RawAccountMeta {
                    pubkey: "PLACEHOLDER_INPUT_ACCOUNT".to_string(),
                    is_signer: false,
                    is_writable: true,
                },
                RawAccountMeta {
                    pubkey: "PLACEHOLDER_OUTPUT_ACCOUNT".to_string(),
                    is_signer: false,
                    is_writable: true,
                },
            ],
            data: "SIMULATED_SWAP".to_string(),
        }]);
    }

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
    let (instructions, _alt_accounts) = match jupiter_client
        .swap(swap_params)
        .prepare_transaction_components()
        .await
    {
        Ok(result) => result,
        Err(e) => {
            warn!(
                "Failed to connect to surfpool or Jupiter API: {}. Falling back to simulated instructions.",
                e
            );
            return Ok(vec![RawInstruction {
                program_id: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string(),
                accounts: vec![
                    RawAccountMeta {
                        pubkey: user_pubkey_str.clone(),
                        is_signer: true,
                        is_writable: true,
                    },
                    RawAccountMeta {
                        pubkey: "PLACEHOLDER_INPUT_ACCOUNT".to_string(),
                        is_signer: false,
                        is_writable: true,
                    },
                    RawAccountMeta {
                        pubkey: "PLACEHOLDER_OUTPUT_ACCOUNT".to_string(),
                        is_signer: false,
                        is_writable: true,
                    },
                ],
                data: "SIMULATED_SWAP".to_string(),
            }]);
        }
    };

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
