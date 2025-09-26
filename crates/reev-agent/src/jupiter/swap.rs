use anyhow::{Context, Result};
use jupiter_swap_api_client::{
    quote::QuoteRequest, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use reev_lib::agent::RawInstruction;
use solana_sdk::{pubkey, pubkey::Pubkey};
use std::{collections::HashMap, str::FromStr};
use tracing::info;

// Define the official, mainnet mint addresses that the public Jupiter API understands.
const MAINNET_USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

/// Handles the logic for a Jupiter swap, transparently replacing mock mints with their
/// real mainnet counterparts before calling the Jupiter API.
///
/// This function is the central point for all Jupiter swap interactions, ensuring that
/// even when the agent is operating in a sandboxed test environment with mock tokens,
/// the calls to the public Jupiter API are made with valid, tradable mint addresses.
///
/// # Arguments
/// * `user_pubkey` - The public key of the user initiating the swap.
/// * `input_mint` - The mint address of the token to be swapped from.
/// * `output_mint` - The mint address of the token to be swapped to.
/// * `amount` - The amount of the input token to swap, in its smallest unit.
/// * `_slippage_bps` - The slippage tolerance from the caller (ignored).
/// * `key_map` - The on-chain context map, used to identify mock mint addresses.
///
/// # Returns
/// A `Result` containing the `RawInstruction` for the swap, or an error if the API call fails.
pub async fn handle_jupiter_swap(
    user_pubkey: Pubkey,
    mut input_mint: Pubkey,
    mut output_mint: Pubkey,
    amount: u64,
    _slippage_bps: u16, // This parameter is ignored to ensure a high slippage tolerance for tests.
    key_map: &HashMap<String, String>,
) -> Result<RawInstruction> {
    info!(
        "[reev-agent] Handling Jupiter swap. Initial mints: IN={}, OUT={}",
        input_mint, output_mint
    );

    // The TUI/runner might be using a mock USDC mint in a local validator.
    // We must replace it with the real mainnet USDC mint before calling the public API.
    if let Some(mock_usdc_mint_str) = key_map.get("MOCK_USDC_MINT") {
        if let Ok(mock_usdc_pubkey) = Pubkey::from_str(mock_usdc_mint_str) {
            // Check if the input mint from the agent matches the mock mint from the context.
            if input_mint == mock_usdc_pubkey {
                info!(
                    "[reev-agent] Replacing mock input mint {} with mainnet USDC mint {}",
                    input_mint, MAINNET_USDC_MINT
                );
                input_mint = MAINNET_USDC_MINT;
            }
            // Check if the output mint from the agent matches the mock mint from the context.
            if output_mint == mock_usdc_pubkey {
                info!(
                    "[reev-agent] Replacing mock output mint {} with mainnet USDC mint {}",
                    output_mint, MAINNET_USDC_MINT
                );
                output_mint = MAINNET_USDC_MINT;
            }
        }
    }

    info!(
        "[reev-agent] Final mints for API call: IN={} OUT={}",
        input_mint, output_mint
    );

    // Proceed with the Jupiter API call using the corrected mints.
    let jupiter_client = JupiterSwapApiClient::new("https://quote-api.jup.ag/v6".to_string());

    // Use a high slippage tolerance to account for state differences between mainnet (where the quote is from)
    // and the local surfpool fork (where the transaction is executed).
    let slippage_bps = 500; // 5%

    let quote_request = QuoteRequest {
        amount,
        input_mint,
        output_mint,
        slippage_bps,
        ..Default::default()
    };

    info!(
        "[reev-agent] Getting Jupiter quote with slippage: {} bps",
        slippage_bps
    );
    let quote_response = jupiter_client
        .quote(&quote_request)
        .await
        .context("Failed to get Jupiter quote from API")?;

    info!("[reev-agent] Getting Jupiter swap instructions...");
    let swap_instructions = jupiter_client
        .swap_instructions(&SwapRequest {
            user_public_key: user_pubkey,
            quote_response,
            config: TransactionConfig::default(),
        })
        .await
        .context("Failed to get Jupiter swap instructions from API")?;

    // Convert the main swap instruction to the RawInstruction format our framework uses.
    let raw_instruction: RawInstruction = swap_instructions.swap_instruction.into();
    Ok(raw_instruction)
}
