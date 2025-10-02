//! # Centralized Benchmark Scenario Setup
//!
//! This module provides the core logic for setting up the on-chain state for SPL Token
//! related benchmarks. It acts as a bridge between the declarative benchmark definitions
//! in the `.yml` files and the `surfpool` test validator's state-manipulation RPCs.
//!
//! The key problem this solves is that Associated Token Account (ATA) addresses are
//! *derived*, not pre-determined. A benchmark file can only contain a placeholder name
//! (e.g., `USER_USDC_ATA`). This module contains the logic to:
//!
//! 1.  Read the initial state defined in the benchmark.
//! 2.  Derive the correct ATA addresses based on the user's wallet (which gets a real,
//!     random keypair for the test run) and the token mint.
//! 3.  Update the environment's internal `key_map` so the placeholder name now points to
//!     the correct, derived address.
//! 4.  Use `surfpool`'s `surfnet_setTokenAccount` RPC "cheat code" to create and fund
//!     the token account at the correct derived address with the amount specified in the
//!     benchmark file.
//!
//! By centralizing this logic in `reev-lib`, we ensure that the `reev-runner` CLI,
//! the `reev-tui`, and the `cargo test` suite all share the exact same setup process,
//! guaranteeing consistent behavior across all parts of the framework.

use crate::{
    agent::AgentObservation,
    benchmark::{InitialStateItem, TestCase},
    solana_env::environment::SolanaEnv,
};
use anyhow::{Context, Result};
use jup_sdk::surfpool::SurfpoolClient;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;
use tracing::info;

/// Corrects the on-chain state for SPL benchmarks after an initial `reset`.
///
/// This is the core function that ensures SPL benchmarks are set up correctly. It
/// iterates through the benchmark's defined initial state, finds all SPL token accounts,
/// derives their true ATA addresses, updates the environment's key mapping, and then
/// uses the `surfpool` client to create and fund these accounts on the local validator.
pub async fn setup_spl_scenario(
    env: &mut SolanaEnv,
    test_case: &TestCase,
    observation: &mut AgentObservation,
) -> Result<()> {
    // Use the consolidated SurfpoolClient from the jup-sdk.
    let client = SurfpoolClient::new(&env.rpc_client.url());
    let token_program_id = spl_token::id();

    // Collect all token account states first to avoid borrowing issues.
    let token_account_states: Vec<&InitialStateItem> = test_case
        .initial_state
        .iter()
        .filter(|s| s.owner == token_program_id.to_string() && s.data.is_some())
        .collect();

    if token_account_states.is_empty() {
        return Ok(()); // Not an SPL benchmark, nothing to do.
    }

    info!("[setup_spl_scenario] Found SPL accounts to configure.");

    for account_state in token_account_states {
        if let Some(data) = &account_state.data {
            let mint_pubkey =
                Pubkey::from_str(&data.mint).context("Failed to parse mint pubkey from data")?;

            // The owner can be a placeholder or a literal program ID.
            // Check the key_map first. If it's not there, assume it's a literal pubkey.
            let owner_wallet_pubkey_str = match observation.key_map.get(&data.owner) {
                Some(pubkey_str) => {
                    info!(
                        "[setup_spl_scenario] Resolved owner placeholder '{}' -> '{}'",
                        data.owner, pubkey_str
                    );
                    pubkey_str.clone()
                }
                None => {
                    info!(
                        "[setup_spl_scenario] Owner '{}' not in key_map, assuming literal pubkey.",
                        data.owner
                    );
                    data.owner.clone()
                }
            };
            let owner_wallet_pubkey =
                Pubkey::from_str(&owner_wallet_pubkey_str).with_context(|| {
                    format!("Failed to parse owner pubkey: {owner_wallet_pubkey_str}")
                })?;

            // Derive the *correct* ATA address.
            let derived_ata = get_associated_token_address(&owner_wallet_pubkey, &mint_pubkey);
            info!(
                "[setup_spl_scenario] Placeholder '{}' -> Derived ATA: {}",
                account_state.pubkey, derived_ata
            );

            // Update the environment's maps to replace the placeholder with the real derived ATA.
            let placeholder = account_state.pubkey.clone();
            env.pubkey_map.insert(placeholder.clone(), derived_ata);
            observation
                .key_map
                .insert(placeholder, derived_ata.to_string());

            // Use the cheat code to create and fund the account at the correct address.
            let amount = data
                .amount
                .parse::<u64>()
                .context("Failed to parse token amount")?;

            client
                .set_token_account(&owner_wallet_pubkey.to_string(), &data.mint, amount)
                .await?;

            info!(
                "[setup_spl_scenario] Set state for {} with owner {} and amount {}",
                derived_ata, owner_wallet_pubkey, amount
            );
        }
    }

    // Give the validator a moment to process the state changes.
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    Ok(())
}
