use anyhow::Result;
use solana_sdk::pubkey::Pubkey;

use crate::constants::{get_jlp_pool_account, get_jupiter_perpetuals_program_id};
use crate::types::{PositionSide, RequestChange};

/// Generate a Position PDA for storing position data
/// The `Position` PDA stores the position data for a trader's positions (both open and closed).
/// https://station.jup.ag/guides/perpetual-exchange/onchain-accounts#position-account
pub fn generate_position_pda(
    custody: &Pubkey,
    collateral_custody: &Pubkey,
    wallet_address: &Pubkey,
    side: PositionSide,
) -> Result<(Pubkey, u8)> {
    let jlp_pool = get_jlp_pool_account();
    let seeds: &[&[u8]] = &[
        b"position",
        wallet_address.as_ref(),
        jlp_pool.as_ref(),
        custody.as_ref(),
        collateral_custody.as_ref(),
        &[side.as_byte()],
    ];

    Ok(Pubkey::find_program_address(
        seeds,
        &get_jupiter_perpetuals_program_id(),
    ))
}

/// Generate a PositionRequest PDA for trade requests
/// The `positionRequest` PDA holds the requests for all the perpetuals actions. Once the `positionRequest`
/// is submitted on chain, the keeper(s) will pick them up and execute the requests (hence the request
/// fulfillment model)
/// https://station.jup.ag/guides/perpetual-exchange/onchain-accounts#positionrequest-account
pub fn generate_position_request_pda(
    position_pubkey: &Pubkey,
    request_change: RequestChange,
    counter: Option<u64>,
) -> Result<(Pubkey, u8, u64)> {
    // The `counter` constant acts a random seed so we can generate a unique PDA every time the user
    // creates a position request
    let counter_value = counter.unwrap_or_else(|| {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut hasher = DefaultHasher::new();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        timestamp.hash(&mut hasher);
        hasher.finish() % 1_000_000_000
    });

    let counter_bytes = counter_value.to_le_bytes();
    let seeds: &[&[u8]] = &[
        b"position_request",
        position_pubkey.as_ref(),
        &counter_bytes,
        &[request_change.as_byte()],
    ];

    let (position_request, bump) =
        Pubkey::find_program_address(seeds, &get_jupiter_perpetuals_program_id());

    Ok((position_request, bump, counter_value))
}

/// Generate the perpetuals account PDA
pub fn generate_perpetuals_pda() -> Result<(Pubkey, u8)> {
    let seeds: &[&[u8]] = &[b"perpetuals"];
    Ok(Pubkey::find_program_address(
        seeds,
        &get_jupiter_perpetuals_program_id(),
    ))
}

/// Generate custody account PDA
pub fn generate_custody_pda(pool: &Pubkey, mint: &Pubkey) -> Result<(Pubkey, u8)> {
    let seeds: &[&[u8]] = &[b"custody", pool.as_ref(), mint.as_ref()];
    Ok(Pubkey::find_program_address(
        seeds,
        &get_jupiter_perpetuals_program_id(),
    ))
}

/// Generate event authority PDA
pub fn generate_event_authority_pda() -> Result<(Pubkey, u8)> {
    let seeds: &[&[u8]] = &[b"__event_authority"];
    Ok(Pubkey::find_program_address(
        seeds,
        &get_jupiter_perpetuals_program_id(),
    ))
}

/// Generate associated token account address
pub fn get_associated_token_address(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    spl_associated_token_account::get_associated_token_address(wallet, mint)
}

/// Generate associated token account address with allow owner offcurve flag
pub fn get_associated_token_address_with_program_id(
    wallet: &Pubkey,
    mint: &Pubkey,
    token_program_id: &Pubkey,
) -> Pubkey {
    spl_associated_token_account::get_associated_token_address_with_program_id(
        wallet,
        mint,
        token_program_id,
    )
}
