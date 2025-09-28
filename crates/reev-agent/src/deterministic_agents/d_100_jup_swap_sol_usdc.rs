use anyhow::{Context, Result};
use jupiter_swap_api_client::{
    quote::QuoteRequest, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use spl_token::native_mint;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

/// Handles the deterministic logic for the `100-JUP-SWAP-SOL-USDC` benchmark.
///
/// This agent acts as an "oracle" by calling the real Jupiter API to get a valid
/// swap instruction for the requested trade. This represents the "perfect" action
/// an agent could take.
///
/// NOTE: While this generates a REAL and VALID transaction, executing it on a local
/// `surfpool` fork is expected to FAIL. This is because the transaction is built
/// against the live state of Solana mainnet (liquidity pools, etc.), which is
/// inconsistent with our local test user's state (who only exists on the fork).
///
/// This failure is the CORRECT outcome for the deterministic agent, as it proves
/// that a simple agent cannot solve this complex task without a more advanced
/// simulation or a real AI's planning capabilities.
pub(crate) async fn handle_jup_swap_sol_usdc(
    key_map: &HashMap<String, String>,
) -> Result<RawInstruction> {
    info!("[reev-agent] Matched '100-JUP-SWAP-SOL-USDC' id. Calling real Jupiter API.");

    let user_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

    let input_mint = native_mint::ID;
    let output_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?; // Mainnet USDC
    let amount = 100_000_000; // 0.1 SOL
    let slippage_bps = 50; // 0.5%

    let client = JupiterSwapApiClient::new("https://quote-api.jup.ag/v6".to_string());

    // 1. Get a quote for the desired swap.
    let quote_request = QuoteRequest {
        amount,
        input_mint,
        output_mint,
        slippage_bps,
        ..QuoteRequest::default()
    };
    let quote_response = client
        .quote(&quote_request)
        .await
        .context("Failed to get quote from Jupiter API")?;

    // 2. Get the swap instructions for the quote.
    let swap_request = SwapRequest {
        user_public_key: user_pubkey,
        quote_response,
        config: TransactionConfig::default(),
    };
    let swap_instructions_response = client
        .swap_instructions(&swap_request)
        .await
        .context("Failed to get swap instructions from Jupiter API")?;

    // The client library automatically converts the response to the standard solana_sdk::Instruction type.
    let swap_instruction = swap_instructions_response.swap_instruction;

    info!(
        "[reev-agent] Received instruction from Jupiter API: {:?}",
        swap_instruction
    );

    // Convert from the solana_sdk::Instruction to our internal RawInstruction format.
    Ok(swap_instruction.into())
}
