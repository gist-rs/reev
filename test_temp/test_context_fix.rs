#!/usr/bin/env rust-script
//! This is a test script to verify the context builder fix
//!
//! ```cargo
//! [dependencies]
//! reev-agent = { path = "../crates/reev-agent" }
//! serde_json = "1.0"
//! tokio = { version = "1.0", features = ["full"] }
//! ```

use reev_agent::context::{ContextBuilder, AccountContext};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a context builder
    let builder = ContextBuilder::new();

    // Create mock account states simulating surfpool observation
    let mut account_states = HashMap::new();

    // Add a SOL account (like USER_WALLET_PUBKEY)
    account_states.insert("USER_WALLET_PUBKEY".to_string(), json!({
        "lamports": 1000000000, // 1 SOL
        "owner": "11111111111111111111111111111111", // System Program
        "executable": false,
        "data_len": 0
    }));

    // Add a USDC token account
    account_states.insert("USER_USDC_ATA".to_string(), json!({
        "lamports": 2039280,
        "owner": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", // Token Program
        "executable": false,
        "data_len": 165,
        "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "token_account_owner": "USER_WALLET_PUBKEY_RESOLVED",
        "amount": "50000000" // 50 USDC
    }));

    // Create key map
    let mut key_map = HashMap::new();
    key_map.insert("USER_WALLET_PUBKEY".to_string(), "USER_WALLET_PUBKEY_RESOLVED".to_string());
    key_map.insert("USER_USDC_ATA".to_string(), "USER_USDC_ATA_RESOLVED".to_string());

    // Build context from observation (NEW METHOD)
    let context = builder.build_context_from_observation(
        &account_states,
        &key_map,
        "001-sol-transfer"
    )?;

    println!("=== Context from Observation (REAL balances) ===");
    println!("{}", context.formatted_context);

    // Verify the context contains real balances
    assert!(context.formatted_context.contains("1.0000 SOL"));
    assert!(context.formatted_context.contains("50.000000 USDC"));
    println!("✅ SUCCESS: Context shows real balances from observation!");

    Ok(())
}
