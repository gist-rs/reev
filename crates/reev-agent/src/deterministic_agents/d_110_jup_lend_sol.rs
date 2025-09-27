use crate::jupiter;
use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use spl_token::native_mint;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

pub(crate) async fn handle_jup_lend_sol(
    key_map: &HashMap<String, String>,
) -> Result<RawInstruction> {
    info!("[reev-agent] Matched '110-JUP-LEND-SOL' id. Generating instruction with code.");
    let user_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

    let input_mint = native_mint::ID;
    let output_mint = Pubkey::from_str("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn")?;
    let amount = 1_000_000_000;
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
