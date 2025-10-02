use anyhow::{Context, Result};
use solana_sdk::{account::Account, pubkey::Pubkey};

// A simple client for making RPC "cheat code" calls to surfpool.
#[derive(Clone)]
pub enum RpcProvider {
    Surfpool { url: String },
    Mainnet { url: String },
}

pub struct SurfpoolClient {
    client: reqwest::Client,
    provider: RpcProvider,
}

impl SurfpoolClient {
    pub fn new() -> Self {
        Self::with_provider(RpcProvider::Surfpool {
            url: "http://127.0.0.1:8899".to_string(),
        })
    }

    pub fn with_provider(provider: RpcProvider) -> Self {
        Self {
            client: reqwest::Client::new(),
            provider,
        }
    }

    fn rpc_url(&self) -> &str {
        match &self.provider {
            RpcProvider::Surfpool { url } => url,
            RpcProvider::Mainnet { url } => url,
        }
    }

    fn is_surfpool(&self) -> bool {
        matches!(self.provider, RpcProvider::Surfpool { .. })
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

        if !self.is_surfpool() {
            // For mainnet, skip cheat codes
            return Ok(());
        }

        let response = self
            .client
            .post(self.rpc_url())
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

        if !self.is_surfpool() {
            return Ok(());
        }

        let response = self
            .client
            .post(self.rpc_url())
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

        if !self.is_surfpool() {
            return Ok(());
        }

        let response = self
            .client
            .post(self.rpc_url())
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

        if !self.is_surfpool() {
            return Ok(());
        }

        let response = self
            .client
            .post(self.rpc_url())
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

        if !self.is_surfpool() {
            return Ok(());
        }

        let response = self
            .client
            .post(self.rpc_url())
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

impl Default for SurfpoolClient {
    fn default() -> Self {
        Self::new()
    }
}

pub trait RpcOperations {
    fn set_account(
        &self,
        pubkey: &str,
        lamports: u64,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
    fn set_token_account(
        &self,
        owner: &str,
        mint: &str,
        amount: u64,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
    fn set_account_from_account(
        &self,
        pubkey: &Pubkey,
        account: Account,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
    fn time_travel_to_now(&self) -> impl std::future::Future<Output = Result<()>> + Send;
    fn reset_account(&self, pubkey: &str) -> impl std::future::Future<Output = Result<()>> + Send;
}

impl RpcOperations for SurfpoolClient {
    async fn set_account(&self, pubkey: &str, lamports: u64) -> Result<()> {
        self.set_account(pubkey, lamports).await
    }

    async fn set_token_account(&self, owner: &str, mint: &str, amount: u64) -> Result<()> {
        self.set_token_account(owner, mint, amount).await
    }

    async fn set_account_from_account(&self, pubkey: &Pubkey, account: Account) -> Result<()> {
        self.set_account_from_account(pubkey, account).await
    }

    async fn time_travel_to_now(&self) -> Result<()> {
        self.time_travel_to_now().await
    }

    async fn reset_account(&self, pubkey: &str) -> Result<()> {
        self.reset_account(pubkey).await
    }
}

// Dummy impl for real RPC (no-op cheat codes)
impl RpcOperations for () {
    async fn set_account(&self, _pubkey: &str, _lamports: u64) -> Result<()> {
        Ok(())
    }
    async fn set_token_account(&self, _owner: &str, _mint: &str, _amount: u64) -> Result<()> {
        Ok(())
    }
    async fn set_account_from_account(&self, _pubkey: &Pubkey, _account: Account) -> Result<()> {
        Ok(())
    }
    async fn time_travel_to_now(&self) -> Result<()> {
        Ok(())
    }
    async fn reset_account(&self, _pubkey: &str) -> Result<()> {
        Ok(())
    }
}
