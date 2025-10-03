//! # Debugging Helpers for Integration Tests
//!
//! This module contains utility functions that are not part of the core test
//! logic but are useful for debugging test failures or discovering on-chain
//! state during development.

#![cfg(test)]

use anyhow::{Context, Result};
use reev_lib::solana_env::environment::SolanaEnv;
use solana_sdk::pubkey::Pubkey;
use spl_token;
use tracing::info;

/// A debug helper to log all token accounts and their mints for a given user.
///
/// This function connects to the test validator's RPC endpoint, fetches all
/// SPL Token accounts owned by the specified `user_pubkey`, and logs their
/// pubkey, mint address, and current token balance. It is particularly useful
/// for dynamic discovery of addresses like L-Tokens generated during DeFi
/// interactions.
#[allow(dead_code)] // This is a debug helper, not always used in tests.
pub fn log_user_token_accounts(env: &SolanaEnv, user_pubkey: &Pubkey) -> Result<()> {
    info!(
        "--- START DEBUG: Token Account Discovery for {} ---",
        user_pubkey
    );

    let token_accounts = env
        .rpc_client
        .get_token_accounts_by_owner(
            user_pubkey,
            solana_client::rpc_request::TokenAccountsFilter::ProgramId(spl_token::ID),
        )
        .context("Failed to get token accounts by owner")?;

    if token_accounts.is_empty() {
        info!("No SPL Token accounts found for this user.");
    } else {
        info!("Found {} token accounts:", token_accounts.len());
        for rpc_token_account in token_accounts {
            info!("rpc_token_account: {rpc_token_account:?}");
        }
    }

    info!("--- END DEBUG: Token Account Discovery ---");
    Ok(())
}
