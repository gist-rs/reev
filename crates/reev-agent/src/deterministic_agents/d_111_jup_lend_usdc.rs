use crate::jupiter;
use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

pub(crate) async fn handle_jup_lend_usdc(
    key_map: &HashMap<String, String>,
) -> Result<RawInstruction> {
    info!("[reev-agent] Matched '111-JUP-LEND-USDC' id. Generating instruction with code.");
    let user_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

    let input_mint_str = key_map
        .get("MOCK_USDC_MINT")
        .context("MOCK_USDC_MINT not found in key_map")?;
    let input_mint = Pubkey::from_str(input_mint_str)?;
    let output_mint = Pubkey::from_str("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB")?;
    let amount = 100_000_000;
    let slippage_bps = 50;

    jupiter::swap::handle_jupiter_swap(
        user_pubkey,
        input_mint,
        output_mint,
        amount,
        slippage_bps,
        key_map,
    )
    .await
}
