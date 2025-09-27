//! # Surfpool RPC HTTP Client
//!
//! This module provides a simple asynchronous HTTP client for interacting with the
//! `surfpool` RPC server's "cheat code" endpoints. It is designed specifically
//! for test setups to programmatically manipulate the on-chain state.

use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;

/// A client for making RPC calls to a local `surfpool` instance.
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

    /// Calls the `surfnet_setTokenAccount` cheat code to create or update an SPL token account.
    ///
    /// This is a convenience method for tests to programmatically set the balance of an SPL
    /// token account, simulating a pre-funded wallet for benchmarks.
    pub async fn set_token_account(&self, owner: &str, mint: &str, amount: u64) -> Result<()> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "surfnet_setTokenAccount",
            "params": [
                owner,
                mint,
                {
                    "amount": amount,
                },
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" // spl-token program id
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
