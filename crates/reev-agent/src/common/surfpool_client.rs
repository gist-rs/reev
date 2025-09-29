use anyhow::{Context, Result};
use solana_sdk::{account::Account, pubkey::Pubkey};

// A simple client for making RPC "cheat code" calls to surfpool.
pub struct SurfpoolClient {
    client: reqwest::Client,
    url: String,
}

impl Default for SurfpoolClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SurfpoolClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            url: "http://127.0.0.1:8899".to_string(),
        }
    }

    pub async fn set_token_account(&self, owner: &str, mint: &str, amount: u64) -> Result<()> {
        let request_body = serde_json::json!({
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

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            anyhow::bail!("Failed to set token account. Status: {status}, Body: {error_body}");
        }
        Ok(())
    }

    pub async fn set_account(&self, pubkey: &str, lamports: u64) -> Result<()> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "surfnet_setAccount",
            "params": [
                pubkey,
                { "lamports": lamports }
            ]
        });

        let response = self
            .client
            .post(&self.url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send RPC request to set account")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            anyhow::bail!("Failed to set account. Status: {status}, Body: {error_body}");
        }
        Ok(())
    }

    pub async fn set_account_from_account(&self, pubkey: &Pubkey, account: Account) -> Result<()> {
        let request_body = serde_json::json!({
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

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            anyhow::bail!(
                "Failed to set account from account. Status: {status}, Body: {error_body}"
            );
        }
        Ok(())
    }

    pub async fn time_travel_to_now(&self) -> Result<()> {
        let request_body = serde_json::json!({
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

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            anyhow::bail!("Failed to time travel. Status: {status}, Body: {error_body}");
        }
        Ok(())
    }

    pub async fn reset_account(&self, pubkey: &str) -> Result<()> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "surfnet_resetAccount",
            "params": [pubkey]
        });

        let response = self
            .client
            .post(&self.url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send RPC request to reset account")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            anyhow::bail!("Failed to reset account. Status: {status}, Body: {error_body}");
        }
        Ok(())
    }
}
