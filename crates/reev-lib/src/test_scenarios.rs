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
    benchmark::{InitialAccountState, TestCase},
    solana_env::environment::SolanaEnv,
};
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;
use tracing::info;

/// A simplified client for interacting with the `surfpool` RPC "cheat codes".
struct SurfpoolClient {
    client: Client,
    url: String,
}

impl SurfpoolClient {
    fn new() -> Self {
        Self {
            client: Client::new(),
            url: "http://127.0.0.1:8899".to_string(),
        }
    }

    /// Calls the `surfnet_setTokenAccount` cheat code to create or update an SPL token account.
    async fn set_token_account(&self, owner: &str, mint: &str, amount: u64) -> Result<()> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "surfnet_setTokenAccount",
            "params": [
                owner,
                mint,
                { "amount": amount },
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            ]
        });

        let response = self
            .client
            .post(&self.url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send RPC request to set token account")?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await?;
            anyhow::bail!("Failed to set token account. Status: {status}, Body: {error_body}");
        }
        Ok(())
    }
}

/// A struct for deserializing the `data` field of an SPL Token account state.
#[derive(Debug, Deserialize)]
struct TokenAccountData {
    mint: String,
    owner: String,
    amount: String,
}

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
    let client = SurfpoolClient::new();
    let token_program_id = spl_token::id();

    // Collect all token account states first to avoid borrowing issues.
    let token_account_states: Vec<&InitialAccountState> = test_case
        .initial_state
        .iter()
        .filter(|s| s.owner == token_program_id.to_string())
        .collect();

    if token_account_states.is_empty() {
        return Ok(()); // Not an SPL benchmark, nothing to do.
    }

    info!("[setup_spl_scenario] Found SPL accounts to configure.");

    for account_state in token_account_states {
        let data: TokenAccountData = serde_json::from_value(account_state.data.clone().unwrap())
            .context("Failed to deserialize token account data")?;

        let mint_pubkey =
            Pubkey::from_str(&data.mint).context("Failed to parse mint pubkey from data")?;
        let owner_wallet_pubkey_str = observation
            .key_map
            .get(&data.owner)
            .with_context(|| format!("Owner placeholder '{}' not found in key_map", data.owner))?;
        let owner_wallet_pubkey = Pubkey::from_str(owner_wallet_pubkey_str)?;

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

    // Give the validator a moment to process the state changes.
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    Ok(())
}
