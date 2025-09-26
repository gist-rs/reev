use anyhow::{Context, Result};
use jupiter_swap_api_client::{
    quote::QuoteRequest, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const NATIVE_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

/// Handles the deterministic logic for a Jupiter swap.
/// This function calls the Jupiter API to get a quote and the swap instruction.
pub async fn handle_deterministic_swap(
    key_map: &HashMap<String, String>,
) -> Result<RawInstruction> {
    info!("[reev-agent] Matched deterministic 'jupiter-swap'.");
    let user_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

    let jupiter_client = JupiterSwapApiClient::new("https://quote-api.jup.ag/v6".to_string());

    // Hardcoded for the specific benchmark: swap 0.1 SOL for USDC
    let quote_request = QuoteRequest {
        amount: 100_000_000, // 0.1 SOL
        input_mint: NATIVE_MINT,
        output_mint: USDC_MINT,
        slippage_bps: 50,
        ..Default::default()
    };

    info!("[reev-agent] Getting Jupiter quote...");
    let quote_response = jupiter_client.quote(&quote_request).await?;

    info!("[reev-agent] Getting Jupiter swap instructions...");
    let swap_instructions = jupiter_client
        .swap_instructions(&SwapRequest {
            user_public_key: user_pubkey,
            quote_response,
            config: TransactionConfig::default(),
        })
        .await?;

    // Convert the main swap instruction to the RawInstruction format our framework uses.
    let raw_instruction: RawInstruction = swap_instructions.swap_instruction.into();
    Ok(raw_instruction)
}
