use anchor_client::solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::VersionedTransaction,
};
use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use solana_client::rpc_client::RpcClient;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;

use solana_sdk::transaction::Transaction;
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;
use spl_token::instruction::sync_native;

use crate::constants::{
    get_custody_pubkeys, get_jlp_pool_account, get_jupiter_perpetuals_program_id,
};
use crate::pda::{
    generate_position_pda, generate_position_request_pda, get_associated_token_address,
};
use crate::types::{
    ClosePositionRequestParams, CreatePositionRequestParams, Custody, CustodyAccount, Position,
    PositionAccount, PositionSide, RequestChange,
};

/// Main Jupiter Perps client for interacting with the perpetuals program
pub struct JupiterPerpsClient {
    pub rpc_url: String,
    pub program_id: Pubkey,
    pub keypair_bytes: Vec<u8>,
}

impl JupiterPerpsClient {
    /// Create a new Jupiter Perps client
    pub fn new(rpc_url: Option<String>, keypair: Keypair) -> Result<Self> {
        let rpc_url = rpc_url.unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string());
        let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
        let program_id = get_jupiter_perpetuals_program_id();

        info!("Creating Jupiter Perps client with program ID: {program_id}");

        Ok(Self {
            rpc_url: rpc_client.url(),
            program_id,
            keypair_bytes: keypair.to_bytes().to_vec(),
        })
    }

    /// Create a client from environment variables
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let rpc_url = std::env::var("RPC_URL").ok();
        let private_key = std::env::var("PRIVATE_KEY")
            .map_err(|_| anyhow!("PRIVATE_KEY environment variable not set"))?;

        let keypair = if private_key.starts_with('[') {
            // Array format
            let bytes: Vec<u8> = serde_json::from_str(&private_key)
                .map_err(|_| anyhow!("Invalid private key array format"))?;
            Keypair::try_from(&bytes[..])
                .map_err(|_| anyhow!("Failed to convert array to keypair"))?
        } else {
            // Base58 format
            let decoded = bs58::decode(&private_key)
                .into_vec()
                .map_err(|_| anyhow!("Invalid base58 private key"))?;
            Keypair::try_from(&decoded[..])
                .map_err(|_| anyhow!("Failed to create keypair from bytes"))?
        };

        Self::new(rpc_url, keypair)
    }

    /// Get RPC client
    pub fn get_rpc_client(&self) -> RpcClient {
        RpcClient::new_with_commitment(&self.rpc_url, CommitmentConfig::confirmed())
    }

    /// Get keypair
    pub fn get_keypair(&self) -> Result<Keypair> {
        Keypair::try_from(&self.keypair_bytes[..])
            .map_err(|_| anyhow!("Failed to recreate keypair"))
    }

    /// Get all open positions from the program
    /// This returns positions with size_usd > 0
    pub async fn get_open_positions(&self) -> Result<Vec<PositionAccount>> {
        info!("Fetching all open positions");

        // For now, return empty positions as we need to implement proper account fetching
        // In a real implementation, you'd use the program's account filters or get_program_accounts
        let open_positions = Vec::new();
        Ok(open_positions)
    }

    /// Get open positions for a specific wallet
    pub async fn get_open_positions_for_wallet(
        &self,
        wallet: &Pubkey,
    ) -> Result<Vec<PositionAccount>> {
        info!("Fetching open positions for wallet: {wallet}");

        let all_positions = self.get_open_positions().await?;
        let wallet_positions: Vec<_> = all_positions
            .into_iter()
            .filter(|pos| pos.account.owner == *wallet)
            .collect();

        info!("Found {} open positions for wallet", wallet_positions.len());
        Ok(wallet_positions)
    }

    /// Get custody account data
    pub async fn get_custody(&self, custody_pubkey: &Pubkey) -> Result<CustodyAccount> {
        debug!("Fetching custody account: {custody_pubkey}");

        let rpc_client =
            RpcClient::new_with_commitment(&self.rpc_url, CommitmentConfig::confirmed());
        let account_data = rpc_client
            .get_account_data(custody_pubkey)
            .map_err(|e| anyhow!("Failed to get custody account: {e}"))?;

        let custody = self
            .decode_custody_account(&account_data)
            .map_err(|e| anyhow!("Failed to decode custody account: {e}"))?;

        Ok(CustodyAccount {
            pubkey: *custody_pubkey,
            account: custody,
        })
    }

    /// Create a market open position request
    pub async fn create_market_open_position_request(
        &self,
        params: CreatePositionRequestParams,
    ) -> Result<VersionedTransaction> {
        info!(
            "Creating market open position request for custody: {}",
            params.custody
        );

        let (position_request, _bump, counter) =
            generate_position_request_pda(&params.position_pubkey, RequestChange::Increase, None)?;

        // Get associated token addresses
        let position_request_ata =
            get_associated_token_address(&position_request, &params.input_mint);
        let funding_account = get_associated_token_address(&params.owner, &params.input_mint);

        // Build instructions
        let mut instructions = Vec::new();

        // Handle SOL wrapping if needed
        if params.input_mint == spl_token::native_mint::ID {
            // Create associated token account for wrapped SOL
            instructions.push(create_associated_token_account_idempotent(
                &Keypair::try_from(&self.keypair_bytes[..]).unwrap().pubkey(),
                &funding_account,
                &params.owner,
                &spl_token::native_mint::ID,
            ));

            // Transfer SOL to wrapped SOL account
            instructions.push(solana_sdk::system_instruction::transfer(
                &params.owner,
                &funding_account,
                params.collateral_token_delta,
            ));

            // Sync wrapped SOL balance
            instructions.push(sync_native(
                &spl_token::native_mint::id(),
                &funding_account,
            )?);
        }

        // Get perpetuals PDA
        let (perpetuals_pda, _perpetuals_bump) = crate::pda::generate_perpetuals_pda()?;
        let jlp_pool = get_jlp_pool_account();

        // Create the increase position request instruction
        // Note: This would need the actual IDL-generated instruction builders
        // For now, we'll create a placeholder instruction structure
        let instruction_data = self.build_increase_position_request_data(
            counter,
            params.collateral_token_delta,
            params.jupiter_minimum_out,
            params.price_slippage,
            params.side.clone(),
            params.size_usd_delta,
        );

        instructions.push(Instruction {
            program_id: self.program_id,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(params.custody, false),
                solana_sdk::instruction::AccountMeta::new(params.collateral_custody, false),
                solana_sdk::instruction::AccountMeta::new(funding_account, false),
                solana_sdk::instruction::AccountMeta::new_readonly(params.input_mint, false),
                solana_sdk::instruction::AccountMeta::new_readonly(params.owner, true),
                solana_sdk::instruction::AccountMeta::new(perpetuals_pda, false),
                solana_sdk::instruction::AccountMeta::new(jlp_pool, false),
                solana_sdk::instruction::AccountMeta::new(params.position_pubkey, false),
                solana_sdk::instruction::AccountMeta::new(position_request, false),
                solana_sdk::instruction::AccountMeta::new(position_request_ata, false),
            ],
            data: instruction_data,
        });

        // Add compute budget instructions
        instructions.insert(0, ComputeBudgetInstruction::set_compute_unit_price(100_000));

        // Get recent blockhash
        let rpc_client =
            RpcClient::new_with_commitment(&self.rpc_url, CommitmentConfig::confirmed());
        let recent_blockhash = rpc_client.get_latest_blockhash()?;

        // Create and sign transaction
        let keypair = Keypair::try_from(&self.keypair_bytes[..])
            .map_err(|_| anyhow!("Failed to recreate keypair"))?;
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&params.owner),
            &[&keypair],
            recent_blockhash,
        );

        // Convert to VersionedTransaction for simulation
        let versioned_transaction = VersionedTransaction::from(transaction);

        // Simulate transaction to estimate compute units
        match self.simulate_transaction(&versioned_transaction).await {
            Ok(units) => {
                instructions.insert(1, ComputeBudgetInstruction::set_compute_unit_limit(units));
                // Recreate transaction with compute unit limit
                let recent_blockhash = rpc_client.get_latest_blockhash()?;
                let keypair = Keypair::try_from(&self.keypair_bytes[..])
                    .map_err(|_| anyhow!("Failed to recreate keypair"))?;
                let transaction = Transaction::new_signed_with_payer(
                    &instructions,
                    Some(&params.owner),
                    &[&keypair],
                    recent_blockhash,
                );
                Ok(VersionedTransaction::from(transaction))
            }
            Err(_) => {
                warn!("Failed to simulate transaction, using default compute limit");
                instructions.insert(
                    1,
                    ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
                );
                let recent_blockhash = rpc_client.get_latest_blockhash()?;
                let keypair = Keypair::try_from(&self.keypair_bytes[..])
                    .map_err(|_| anyhow!("Failed to recreate keypair"))?;
                let transaction = Transaction::new_signed_with_payer(
                    &instructions,
                    Some(&params.owner),
                    &[&keypair],
                    recent_blockhash,
                );
                Ok(VersionedTransaction::from(transaction))
            }
        }
    }

    /// Create a market close position request
    pub async fn create_market_close_position_request(
        &self,
        params: ClosePositionRequestParams,
    ) -> Result<VersionedTransaction> {
        info!(
            "Creating market close position request for position: {}",
            params.position_pubkey
        );

        // Get position data
        let rpc_client =
            RpcClient::new_with_commitment(&self.rpc_url, CommitmentConfig::confirmed());
        let position_data = rpc_client
            .get_account_data(&params.position_pubkey)
            .map_err(|e| anyhow!("Failed to get position account: {e}"))?;

        let position = self
            .decode_position_account(&position_data)
            .map_err(|e| anyhow!("Failed to decode position account: {e}"))?;

        let (position_request, _bump, counter) =
            generate_position_request_pda(&params.position_pubkey, RequestChange::Decrease, None)?;

        // Get associated token addresses
        let position_request_ata =
            get_associated_token_address(&position_request, &params.desired_mint);
        let receiving_account = get_associated_token_address(&position.owner, &params.desired_mint);

        // Build instructions
        let mut instructions = Vec::new();

        // Create associated token account if it doesn't exist (for non-SOL tokens)
        if params.desired_mint != spl_token::native_mint::ID {
            instructions.push(create_associated_token_account_idempotent(
                &Keypair::try_from(&self.keypair_bytes[..]).unwrap().pubkey(),
                &receiving_account,
                &position.owner,
                &params.desired_mint,
            ));
        }

        // Get perpetuals PDA
        let (perpetuals_pda, _perpetuals_bump) = crate::pda::generate_perpetuals_pda()?;
        let jlp_pool = get_jlp_pool_account();

        // Create the decrease position request instruction
        let instruction_data = self.build_decrease_position_request_data(
            counter,
            0, // collateral_usd_delta (0 for full close)
            0, // size_usd_delta (0 for full close)
            params.price_slippage,
            None, // jupiter_minimum_out
            true, // entire_position
        );

        instructions.push(Instruction {
            program_id: self.program_id,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new_readonly(position.owner, true),
                solana_sdk::instruction::AccountMeta::new(receiving_account, false),
                solana_sdk::instruction::AccountMeta::new(perpetuals_pda, false),
                solana_sdk::instruction::AccountMeta::new(jlp_pool, false),
                solana_sdk::instruction::AccountMeta::new(params.position_pubkey, false),
                solana_sdk::instruction::AccountMeta::new(position_request, false),
                solana_sdk::instruction::AccountMeta::new(position_request_ata, false),
                solana_sdk::instruction::AccountMeta::new(position.custody, false),
                solana_sdk::instruction::AccountMeta::new(position.collateral_custody, false),
                solana_sdk::instruction::AccountMeta::new_readonly(params.desired_mint, false),
            ],
            data: instruction_data,
        });

        // Add compute budget instructions
        instructions.insert(0, ComputeBudgetInstruction::set_compute_unit_price(100_000));

        // Get recent blockhash
        let rpc_client =
            RpcClient::new_with_commitment(&self.rpc_url, CommitmentConfig::confirmed());
        let recent_blockhash = rpc_client.get_latest_blockhash()?;

        // Create and sign transaction
        let keypair = Keypair::try_from(&self.keypair_bytes[..])
            .map_err(|_| anyhow!("Failed to recreate keypair"))?;
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&position.owner),
            &[&keypair],
            recent_blockhash,
        );

        // Convert to VersionedTransaction for simulation
        let versioned_transaction = VersionedTransaction::from(transaction);

        // Simulate transaction to estimate compute units
        match self.simulate_transaction(&versioned_transaction).await {
            Ok(units) => {
                instructions.insert(1, ComputeBudgetInstruction::set_compute_unit_limit(units));
                // Recreate transaction with compute unit limit
                let recent_blockhash = rpc_client.get_latest_blockhash()?;
                let keypair = Keypair::try_from(&self.keypair_bytes[..])
                    .map_err(|_| anyhow!("Failed to recreate keypair"))?;
                let transaction = Transaction::new_signed_with_payer(
                    &instructions,
                    Some(&position.owner),
                    &[&keypair],
                    recent_blockhash,
                );
                Ok(VersionedTransaction::from(transaction))
            }
            Err(_) => {
                warn!("Failed to simulate transaction, using default compute limit");
                instructions.insert(
                    1,
                    ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
                );
                let recent_blockhash = rpc_client.get_latest_blockhash()?;
                let keypair = Keypair::try_from(&self.keypair_bytes[..])
                    .map_err(|_| anyhow!("Failed to recreate keypair"))?;
                let transaction = Transaction::new_signed_with_payer(
                    &instructions,
                    Some(&position.owner),
                    &[&keypair],
                    recent_blockhash,
                );
                Ok(VersionedTransaction::from(transaction))
            }
        }
    }

    /// Sign and submit a transaction
    pub async fn sign_and_submit_transaction(
        &self,
        transaction: VersionedTransaction,
    ) -> Result<String> {
        info!("Submitting transaction");

        let rpc_client =
            RpcClient::new_with_commitment(&self.rpc_url, CommitmentConfig::confirmed());

        // Submit transaction (already signed)
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| anyhow!("Failed to send transaction: {e}"))?;

        info!("Transaction confirmed: {signature}");
        Ok(signature.to_string())
    }

    /// Simulate a transaction to get compute unit usage
    async fn simulate_transaction(&self, transaction: &VersionedTransaction) -> Result<u32> {
        let rpc_client =
            RpcClient::new_with_commitment(&self.rpc_url, CommitmentConfig::confirmed());
        let simulation_result = rpc_client
            .simulate_transaction(transaction)
            .map_err(|e| anyhow!("Failed to simulate transaction: {e}"))?;

        match simulation_result.value.err {
            Some(err) => Err(anyhow!("Simulation failed: {err:?}")),
            None => Ok(simulation_result
                .value
                .units_consumed
                .unwrap_or(1_400_000)
                .try_into()
                .unwrap()),
        }
    }

    // Helper methods for decoding accounts
    fn decode_position_account(&self, data: &[u8]) -> Result<Position> {
        // This is a simplified decoder - in a real implementation,
        // you would use the IDL-generated account deserializers
        if data.len() < 8 + 32 * 6 + 8 * 4 + 8 {
            return Err(anyhow!("Invalid position account data length"));
        }

        let mut offset = 8; // Skip discriminator

        let owner = Pubkey::try_from(&data[offset..offset + 32])?;
        offset += 32;
        let custody = Pubkey::try_from(&data[offset..offset + 32])?;
        offset += 32;
        let collateral_custody = Pubkey::try_from(&data[offset..offset + 32])?;
        offset += 32;
        let pool = Pubkey::try_from(&data[offset..offset + 32])?;
        // Skip more fields for simplicity...

        // This is a simplified implementation
        Ok(Position {
            pubkey: Pubkey::default(),
            owner,
            custody,
            collateral_custody,
            pool,
            side: PositionSide::Long, // Simplified
            size_usd: 0,              // Would be parsed from data
            collateral_usd: 0,
            open_time: 0,
            borrow_index: 0,
            cumulative_interest_snapshot: 0,
        })
    }

    fn decode_custody_account(&self, data: &[u8]) -> Result<Custody> {
        // Simplified custody decoder
        if data.len() < 8 + 32 * 3 + 1 + 8 * 5 {
            return Err(anyhow!("Invalid custody account data length"));
        }

        // This is a simplified implementation
        Ok(Custody {
            pubkey: Pubkey::default(),
            pool: Pubkey::default(),
            mint: Pubkey::default(),
            is_asset: true,
            total_amount: 0,
            borrow_rate: 0,
            cumulative_interest: 0,
            total_borrow: 0,
            oracle_price: 0,
            decimals: 6,
            pricing: crate::types::Pricings {
                trade_fee_long: 0,
                trade_fee_short: 0,
                oracle_price_buffer: 0,
                max_position_size_usd: 0,
                funding_rate_conflict: 0,
                pnl_conflict: 0,
                borrow_rate_conflict: 0,
            },
        })
    }

    // Helper methods for building instruction data
    fn build_increase_position_request_data(
        &self,
        counter: u64,
        collateral_token_delta: u64,
        jupiter_minimum_out: Option<u64>,
        price_slippage: u64,
        side: PositionSide,
        size_usd_delta: u64,
    ) -> Vec<u8> {
        // This is a simplified implementation
        // In a real implementation, you would use the IDL-generated instruction builders
        let mut data = Vec::new();

        // Instruction discriminator (placeholder)
        data.extend_from_slice(&[123, 45, 67, 89, 12, 34, 56, 78]);

        // Arguments
        data.extend_from_slice(&counter.to_le_bytes());
        data.extend_from_slice(&collateral_token_delta.to_le_bytes());
        data.extend_from_slice(&jupiter_minimum_out.unwrap_or(0).to_le_bytes());
        data.extend_from_slice(&price_slippage.to_le_bytes());
        data.push(side.as_byte());
        data.extend_from_slice(&size_usd_delta.to_le_bytes());

        data
    }

    fn build_decrease_position_request_data(
        &self,
        counter: u64,
        collateral_usd_delta: u64,
        size_usd_delta: u64,
        price_slippage: u64,
        jupiter_minimum_out: Option<u64>,
        entire_position: bool,
    ) -> Vec<u8> {
        // This is a simplified implementation
        let mut data = Vec::new();

        // Instruction discriminator (placeholder)
        data.extend_from_slice(&[234, 56, 78, 90, 23, 45, 67, 89]);

        // Arguments
        data.extend_from_slice(&collateral_usd_delta.to_le_bytes());
        data.extend_from_slice(&size_usd_delta.to_le_bytes());
        data.extend_from_slice(&price_slippage.to_le_bytes());
        data.extend_from_slice(&jupiter_minimum_out.unwrap_or(0).to_le_bytes());
        data.extend_from_slice(&counter.to_le_bytes());
        data.push(entire_position as u8);

        data
    }
}

/// Utility functions for common operations
impl JupiterPerpsClient {
    /// Get all custody accounts
    pub async fn get_all_custodies(&self) -> Result<Vec<CustodyAccount>> {
        let mut custodies = Vec::new();

        for custody_pubkey in get_custody_pubkeys() {
            match self.get_custody(&custody_pubkey).await {
                Ok(custody) => custodies.push(custody),
                Err(e) => warn!("Failed to fetch custody {custody_pubkey}: {e}"),
            }
        }

        Ok(custodies)
    }

    /// Generate position PDA for opening a new position
    pub fn generate_position_pda(
        &self,
        custody: &Pubkey,
        collateral_custody: &Pubkey,
        side: PositionSide,
    ) -> Result<(Pubkey, u8)> {
        generate_position_pda(
            custody,
            collateral_custody,
            &Keypair::try_from(&self.keypair_bytes[..]).unwrap().pubkey(),
            side,
        )
    }

    /// Get current SOL price (simplified)
    pub async fn get_sol_price(&self) -> Result<f64> {
        // This is a simplified implementation
        // In a real implementation, you would fetch from Pyth or another oracle
        Ok(100.0) // Placeholder
    }

    /// Calculate position size in USD
    pub fn calculate_position_size_usd(
        &self,
        token_amount: u64,
        token_price: f64,
        decimals: u8,
    ) -> u64 {
        let token_amount_f64 = token_amount as f64 / 10_f64.powi(decimals as i32);
        (token_amount_f64 * token_price * 1_000_000.0) as u64 // 6 decimal places for USD
    }

    /// Calculate token amount from USD size
    pub fn calculate_token_amount_from_usd(
        &self,
        usd_size: u64,
        token_price: f64,
        decimals: u8,
    ) -> u64 {
        let usd_size_f64 = usd_size as f64 / 1_000_000.0; // Convert from 6 decimal places
        let token_amount = usd_size_f64 / token_price * 10_f64.powi(decimals as i32);
        token_amount as u64
    }
}
