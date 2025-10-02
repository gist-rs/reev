use crate::{
    api::{self},
    models::{DepositParams, SimulationResult, SwapParams, UnsignedTransaction, WithdrawParams},
    surfpool::{self, SurfpoolClient},
    transaction,
};
use anyhow::{Context, Result, anyhow};
use serde_json::from_value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    address_lookup_table::AddressLookupTableAccount, instruction::Instruction, signature::Keypair,
    signer::Signer,
};
use tracing::info;

/// The main Jupiter client, acting as a builder for swap and lend operations.
pub struct Jupiter<'a> {
    rpc_client: RpcClient,
    signer: Option<&'a Keypair>,
    is_surfpool: bool,
}

impl<'a> Jupiter<'a> {
    /// Creates a new Jupiter client for building real-world transactions.
    pub fn new(rpc_client: RpcClient) -> Self {
        Self {
            rpc_client,
            signer: None,
            is_surfpool: false,
        }
    }

    /// Creates a new Jupiter client configured for `surfpool` simulations using the default URL (`http://127.0.0.1:8899`).
    pub fn surfpool() -> Self {
        let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
        Self::surfpool_with_rpc(rpc_client)
    }

    /// Creates a new Jupiter client configured for `surfpool` simulations with a custom RPC endpoint.
    pub fn surfpool_with_rpc(rpc_client: RpcClient) -> Self {
        Self {
            rpc_client,
            signer: None,
            is_surfpool: true,
        }
    }

    /// (For simulations and instruction fetching) Sets the keypair whose public key will be used as the user account.
    /// This is required for all builder methods.
    pub fn with_signer(mut self, signer: &'a Keypair) -> Self {
        self.signer = Some(signer);
        self
    }

    /// Prepares a swap operation.
    pub fn swap(&self, params: SwapParams) -> SwapBuilder {
        SwapBuilder {
            client: self,
            params,
        }
    }

    /// Prepares a deposit operation.
    pub fn deposit(&self, params: DepositParams) -> DepositBuilder {
        DepositBuilder {
            client: self,
            params,
        }
    }

    /// Prepares a withdraw operation.
    pub fn withdraw(&self, params: WithdrawParams) -> WithdrawBuilder {
        WithdrawBuilder {
            client: self,
            params,
        }
    }
}

// --- Swap Builder ---

pub struct SwapBuilder<'a> {
    client: &'a Jupiter<'a>,
    params: SwapParams,
}

impl<'a> SwapBuilder<'a> {
    /// Private helper to fetch and prepare all components needed for a swap transaction.
    async fn prepare_transaction_components(
        &self,
    ) -> Result<(Vec<Instruction>, Vec<AddressLookupTableAccount>)> {
        let signer_pubkey = self
            .client
            .signer
            .ok_or_else(|| anyhow!("A signer must be provided via .with_signer()"))?
            .pubkey();

        // 1. Fetch quote and instructions from API
        let quote = api::swap::get_quote(&self.params).await?;
        let instructions_response =
            api::swap::get_swap_instructions(&signer_pubkey, &quote).await?;

        // 2. Parse all instructions from the API response
        let setup_instructions: Vec<crate::models::InstructionData> = instructions_response
            .get("setupInstructions")
            .and_then(|v| from_value(v.clone()).ok())
            .unwrap_or_default();
        let swap_instruction: crate::models::InstructionData =
            from_value(instructions_response["swapInstruction"].clone())
                .context("Failed to parse swapInstruction")?;
        let cleanup_instruction: Option<crate::models::InstructionData> = instructions_response
            .get("cleanupInstruction")
            .and_then(|v| from_value(v.clone()).ok());

        let mut all_instructions_data = Vec::new();
        all_instructions_data.extend(setup_instructions);
        all_instructions_data.push(swap_instruction);
        if let Some(ci) = cleanup_instruction {
            all_instructions_data.push(ci);
        }

        let instructions = transaction::convert_instructions(all_instructions_data)?;

        // 3. Fetch Address Lookup Tables
        let alt_addresses: Vec<String> = instructions_response
            .get("addressLookupTableAddresses")
            .and_then(|v| from_value(v.clone()).ok())
            .unwrap_or_default();
        let alt_accounts =
            transaction::fetch_address_lookup_tables(&self.client.rpc_client, alt_addresses)?;

        Ok((instructions, alt_accounts))
    }

    /// Builds an unsigned transaction for the swap, ready to be signed by a wallet.
    pub async fn build_unsigned_transaction(&self) -> Result<UnsignedTransaction> {
        let (instructions, alt_accounts) = self.prepare_transaction_components().await?;
        let signer_pubkey = self.client.signer.unwrap().pubkey();

        transaction::compile_transaction(
            &self.client.rpc_client,
            &signer_pubkey,
            instructions,
            alt_accounts,
        )
    }

    /// Executes the full swap simulation against a `surfpool` instance.
    pub async fn commit(&self) -> Result<SimulationResult> {
        if !self.client.is_surfpool {
            return Err(anyhow!("`.commit()` is only available in surfpool mode."));
        }
        let signer = self
            .client
            .signer
            .ok_or_else(|| anyhow!("A signer is required for `.commit()`."))?;
        let surfpool_client = SurfpoolClient::new(&self.client.rpc_client.url());

        // 1. Setup the simulation environment
        surfpool::setup_wallet(
            &self.client.rpc_client,
            &surfpool_client,
            signer,
            &self.params.input_mint,
            self.params.amount * 2, // Fund with double the amount
        )
        .await?;

        // 2. Prepare transaction components
        let (instructions, alt_accounts) = self.prepare_transaction_components().await?;

        // 3. Execute the simulation
        surfpool::execute_simulation(
            &self.client.rpc_client,
            &surfpool_client,
            signer,
            instructions,
            alt_accounts,
        )
        .await
    }
}

// --- Deposit Builder ---

pub struct DepositBuilder<'a> {
    client: &'a Jupiter<'a>,
    params: DepositParams,
}

impl<'a> DepositBuilder<'a> {
    /// Private helper to fetch and prepare deposit transaction components.
    async fn prepare_transaction_components(
        &self,
    ) -> Result<(Vec<Instruction>, Vec<AddressLookupTableAccount>)> {
        let signer_pubkey = self
            .client
            .signer
            .ok_or_else(|| anyhow!("A signer must be provided via .with_signer()"))?
            .pubkey();
        let api_response =
            api::lend::get_deposit_instructions(&signer_pubkey, &self.params).await?;
        let instructions = transaction::convert_instructions(api_response.instructions)?;
        Ok((instructions, vec![])) // Lend API does not use ALTs
    }

    /// Builds an unsigned transaction for the deposit.
    pub async fn build_unsigned_transaction(&self) -> Result<UnsignedTransaction> {
        let (instructions, alt_accounts) = self.prepare_transaction_components().await?;
        let signer_pubkey = self.client.signer.unwrap().pubkey();
        transaction::compile_transaction(
            &self.client.rpc_client,
            &signer_pubkey,
            instructions,
            alt_accounts,
        )
    }

    /// Executes the full deposit simulation against a `surfpool` instance.
    pub async fn commit(&self) -> Result<SimulationResult> {
        if !self.client.is_surfpool {
            return Err(anyhow!("`.commit()` is only available in surfpool mode."));
        }
        let signer = self
            .client
            .signer
            .ok_or_else(|| anyhow!("A signer is required for `.commit()`."))?;
        let surfpool_client = SurfpoolClient::new(&self.client.rpc_client.url());

        surfpool::setup_wallet(
            &self.client.rpc_client,
            &surfpool_client,
            signer,
            &self.params.asset_mint,
            self.params.amount * 2,
        )
        .await?;

        let (instructions, alt_accounts) = self.prepare_transaction_components().await?;

        surfpool::execute_simulation(
            &self.client.rpc_client,
            &surfpool_client,
            signer,
            instructions,
            alt_accounts,
        )
        .await
    }
}

// --- Withdraw Builder ---

pub struct WithdrawBuilder<'a> {
    client: &'a Jupiter<'a>,
    params: WithdrawParams,
}

impl<'a> WithdrawBuilder<'a> {
    /// Private helper to fetch and prepare withdraw transaction components.
    async fn prepare_transaction_components(
        &self,
    ) -> Result<(Vec<Instruction>, Vec<AddressLookupTableAccount>)> {
        let signer_pubkey = self
            .client
            .signer
            .ok_or_else(|| anyhow!("A signer must be provided via .with_signer()"))?
            .pubkey();
        let api_response =
            api::lend::get_withdraw_instructions(&signer_pubkey, &self.params).await?;
        let instructions = transaction::convert_instructions(api_response.instructions)?;
        Ok((instructions, vec![])) // Lend API does not use ALTs
    }

    /// Builds an unsigned transaction for the withdrawal.
    pub async fn build_unsigned_transaction(&self) -> Result<UnsignedTransaction> {
        let (instructions, alt_accounts) = self.prepare_transaction_components().await?;
        let signer_pubkey = self.client.signer.unwrap().pubkey();
        transaction::compile_transaction(
            &self.client.rpc_client,
            &signer_pubkey,
            instructions,
            alt_accounts,
        )
    }

    /// Executes the full withdrawal simulation against a `surfpool` instance.
    pub async fn commit(&self) -> Result<SimulationResult> {
        if !self.client.is_surfpool {
            return Err(anyhow!("`.commit()` is only available in surfpool mode."));
        }
        let signer = self
            .client
            .signer
            .ok_or_else(|| anyhow!("A signer is required for `.commit()`."))?;
        let surfpool_client = SurfpoolClient::new(&self.client.rpc_client.url());

        // For a withdraw simulation, we need to deposit first to have L-tokens to withdraw.
        info!("[SIM] Staging: Performing a deposit to enable withdrawal...");
        let deposit_params = DepositParams {
            asset_mint: self.params.asset_mint,
            amount: self.params.amount,
        };
        let (deposit_ixs, _) = DepositBuilder {
            client: self.client,
            params: deposit_params,
        }
        .prepare_transaction_components()
        .await?;
        surfpool::setup_wallet(
            &self.client.rpc_client,
            &surfpool_client,
            signer,
            &self.params.asset_mint,
            self.params.amount * 2,
        )
        .await?;
        surfpool::execute_simulation(
            &self.client.rpc_client,
            &surfpool_client,
            signer,
            deposit_ixs,
            vec![],
        )
        .await
        .context("Staging deposit failed before withdrawal attempt")?;
        info!("[SIM] Staging: Deposit complete. Proceeding to withdraw.");

        // Now, prepare and execute the actual withdrawal
        let (withdraw_ixs, alt_accounts) = self.prepare_transaction_components().await?;
        surfpool::execute_simulation(
            &self.client.rpc_client,
            &surfpool_client,
            signer,
            withdraw_ixs,
            alt_accounts,
        )
        .await
    }
}
