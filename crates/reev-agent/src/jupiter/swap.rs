use anyhow::{Context, Result};
use jupiter_swap_api_client::{
    quote::QuoteRequest, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use tracing::info;

/// Handles the logic for a Jupiter swap using addresses from the surfpool environment.
///
/// This function calls the public Jupiter API with the provided addresses, assuming they
/// are valid mainnet addresses available in the surfpool fork.
///
/// # Arguments
/// * `user_pubkey` - The public key of the user initiating the swap.
/// * `input_mint` - The mint address of the token to be swapped from.
/// * `output_mint` - The mint address of the token to be swapped to.
/// * `amount` - The amount of the input token to swap, in its smallest unit.
/// * `_slippage_bps` - The slippage tolerance from the caller (ignored).
/// * `_key_map` - The on-chain context map (ignored, kept for signature compatibility).
///
/// # Returns
/// A `Result` containing the `RawInstruction` for the swap, or an error if the API call fails.
pub async fn handle_jupiter_swap(
    user_pubkey: Pubkey,
    input_mint: Pubkey,
    output_mint: Pubkey,
    amount: u64,
    _slippage_bps: u16, // This parameter is ignored to ensure a high slippage tolerance for tests.
    _key_map: &HashMap<String, String>, // Kept for compatibility, but unused.
) -> Result<RawInstruction> {
    info!(
        "[reev-agent] Handling Jupiter swap for surfpool: IN={}, OUT={}, amount={}",
        input_mint, output_mint, amount
    );

    // Proceed with the Jupiter API call using the provided mints directly.
    let jupiter_client = JupiterSwapApiClient::new("https://quote-api.jup.ag/v6".to_string());

    // Use a high slippage tolerance to account for potential state differences between mainnet
    // (where the quote is from) and the local surfpool fork (where the tx is executed).
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

    info!(
        "[reev-agent] Successfully generated Jupiter swap instruction for program: {}",
        raw_instruction.program_id
    );

    Ok(raw_instruction)
}
