//! Native Solana operations implementation
//!
//! This module provides native Solana operations including SOL transfers,
//! SPL token transfers, and other low-level blockchain interactions.

// Inline config helpers
fn get_env_string(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn get_env_var<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}
use anyhow::Result;

use solana_client::rpc_config::RpcSimulateTransactionConfig;

use solana_sdk::{
    commitment_config::CommitmentConfig, compute_budget::ComputeBudgetInstruction,
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, transaction::Transaction,
};
use std::time::Duration;

pub mod sol_transfer;
pub mod spl_transfer;

// Re-export all native functions
pub use sol_transfer::{handle_sol_transfer, instruction_to_raw as sol_instruction_to_raw};
pub use spl_transfer::{handle_spl_transfer, instruction_to_raw as spl_instruction_to_raw};

/// Native Solana configuration
#[derive(Debug, Clone)]
pub struct NativeConfig {
    /// Solana RPC endpoint URL
    pub rpc_url: String,
    /// WebSocket endpoint URL for subscriptions
    pub ws_url: Option<String>,
    /// Timeout for RPC requests
    pub timeout: Duration,
    /// Maximum number of retries for failed requests
    pub max_retries: u32,
    /// Confirmations required for transaction finality
    pub confirmations: u64,
    /// Default compute units per transaction
    pub compute_units: u32,
    /// Default priority fee in lamports
    pub priority_fee_lamports: u64,
    /// User agent string for RPC requests
    pub user_agent: String,
}

impl Default for NativeConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            ws_url: Some("wss://api.mainnet-beta.solana.com".to_string()),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            confirmations: 1,
            compute_units: 200_000,
            priority_fee_lamports: 10_000,
            user_agent: "reev-agent/0.1.0".to_string(),
        }
    }
}

impl NativeConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            rpc_url: get_env_string("SOLANA_RPC_URL", "https://api.mainnet-beta.solana.com"),
            ws_url: Some(get_env_string(
                "SOLANA_WS_URL",
                "wss://api.mainnet-beta.solana.com",
            )),
            timeout: Duration::from_secs(get_env_var("SOLANA_TIMEOUT_SECONDS", 30)),
            max_retries: get_env_var("SOLANA_MAX_RETRIES", 3),
            confirmations: get_env_var("SOLANA_CONFIRMATIONS", 1),
            compute_units: get_env_var("SOLANA_COMPUTE_UNITS", 200_000),
            priority_fee_lamports: get_env_var("SOLANA_PRIORITY_FEE_LAMPORTS", 10_000),
            user_agent: get_env_string("SOLANA_USER_AGENT", "reev-agent/0.1.0"),
        }
    }

    /// Get commitment configuration
    pub fn commitment(&self) -> CommitmentConfig {
        CommitmentConfig::confirmed()
    }

    /// Create RPC client with this configuration
    pub async fn create_client(&self) -> Result<solana_client::rpc_client::RpcClient> {
        let client = solana_client::rpc_client::RpcClient::new_with_commitment(
            self.rpc_url.clone(),
            self.commitment(),
        );
        Ok(client)
    }

    /// Create WebSocket client if URL is configured
    pub fn create_ws_client(&self) -> Option<solana_client::nonblocking::rpc_client::RpcClient> {
        self.ws_url.as_ref().map(|ws_url| {
            solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment(
                ws_url.clone(),
                self.commitment(),
            )
        })
    }
}

use std::sync::OnceLock;

/// Global native configuration
static NATIVE_CONFIG: OnceLock<NativeConfig> = OnceLock::new();

/// Initialize global native configuration
pub fn init_native_config(config: NativeConfig) {
    NATIVE_CONFIG
        .set(config)
        .expect("Native config already initialized");
}

/// Get global native configuration
pub fn get_native_config() -> &'static NativeConfig {
    NATIVE_CONFIG.get_or_init(NativeConfig::from_env)
}

/// Create a compute budget instruction
pub fn create_compute_budget_instruction(compute_units: u32) -> Instruction {
    ComputeBudgetInstruction::set_compute_unit_limit(compute_units)
}

/// Create a priority fee instruction
pub fn create_priority_fee_instruction(lamports: u64) -> Instruction {
    ComputeBudgetInstruction::set_compute_unit_price(lamports)
}

/// Build a transaction with instructions
pub fn build_transaction(
    instructions: Vec<Instruction>,
    payer: &Pubkey,
    recent_blockhash: solana_sdk::hash::Hash,
) -> Transaction {
    let mut transaction = Transaction::new_with_payer(&instructions, Some(payer));
    transaction.message.recent_blockhash = recent_blockhash;
    transaction
}

/// Sign a transaction
pub fn sign_transaction(transaction: &mut Transaction, keypairs: &[&Keypair]) -> Result<()> {
    transaction.try_sign(keypairs, transaction.message.recent_blockhash)?;
    Ok(())
}

/// Estimate transaction cost in lamports
pub async fn estimate_transaction_cost(
    client: &solana_client::rpc_client::RpcClient,
    transaction: &Transaction,
) -> Result<u64> {
    let simulation = client.simulate_transaction_with_config(
        transaction,
        RpcSimulateTransactionConfig {
            sig_verify: false,
            replace_recent_blockhash: true,
            commitment: Some(CommitmentConfig::processed()),
            ..Default::default()
        },
    )?;

    if let Some(err) = simulation.value.err {
        return Err(anyhow::anyhow!("Simulation error: {err}"));
    }

    if let Some(units_consumed) = simulation.value.units_consumed {
        Ok(units_consumed)
    } else {
        Ok(0)
    }
}
