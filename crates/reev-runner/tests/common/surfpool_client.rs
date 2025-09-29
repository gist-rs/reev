//! # Surfpool RPC HTTP Client (for Tests)
//!
//! This module provides a simple asynchronous HTTP client for interacting with the
//! `surfpool` RPC server's "cheat code" endpoints. It is designed specifically
//! for test setups to programmatically manipulate the on-chain state, such as
//! pre-loading accounts from mainnet into the local fork.

use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use solana_sdk::{account::Account, pubkey::Pubkey};
use tracing::info;

/// A client for making RPC "cheat code" calls to a local `surfpool` instance.
pub struct SurfpoolClient {
    client: Client,
    url: String,
}

impl SurfpoolClient {
    /// Creates a new client targeting `http://127.0.0.1:8899`.
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            url: "http://127.0.0.1:8899".to_string(),
        }
    }

    /// Sets the full account data for a given pubkey using the `surfnet_setAccount` cheat code.
    ///
    /// This is used to pre-load accounts fetched from mainnet into the local fork.
    pub async fn set_account_from_account(&self, pubkey: &Pubkey, account: Account) -> Result<()> {
        info!(
            "   -> [SurfpoolClient] Loading account {} ({} lamports) into surfpool",
            pubkey, account.lamports
        );
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "surfnet_setAccount",
            "params": [
                pubkey.to_string(),
                {
                    "lamports": account.lamports,
                    "owner": account.owner.to_string(),
                    "executable": account.executable,
                    "rent_epoch": account.rent_epoch,
                    // Account data must be hex-encoded for the RPC call.
                    "data": hex::encode(&account.data),
                }
            ]
        });

        let response = self
            .client
            .post(&self.url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send RPC request to set account from account")?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await?;
            anyhow::bail!(
                "Failed to set account from account. Status: {status}, Body: {error_body}"
            );
        }

        Ok(())
    }

    /// Calls the `surfnet_timeTravel` cheat code to align the validator's clock.
    ///
    /// This is crucial for swaps involving oracles like Pyth, which rely on recent timestamps.
    pub async fn time_travel_to_now(&self) -> Result<()> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "surfnet_timeTravel",
            "params": [{ "unix_timestamp": chrono::Utc::now().timestamp() }]
        });

        let response = self
            .client
            .post(&self.url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send RPC request to time travel")?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await?;
            anyhow::bail!("Failed to time travel. Status: {status}, Body: {error_body}");
        }

        Ok(())
    }
}
