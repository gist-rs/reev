use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, transaction::VersionedTransaction};

// --- Input Parameter Structs ---

/// Parameters required for a swap operation.
#[derive(Debug, Clone)]
pub struct SwapParams {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount: u64,
    pub slippage_bps: u16,
}

/// Parameters for a lending deposit.
#[derive(Debug, Clone)]
pub struct DepositParams {
    pub asset_mint: Pubkey,
    pub amount: u64,
}

/// Parameters for a lending withdrawal.
#[derive(Debug, Clone)]
pub struct WithdrawParams {
    pub asset_mint: Pubkey,
    pub amount: u64,
}

// --- Layer 1: Production Output Structs ---

/// Represents an unsigned transaction ready to be sent to a wallet for signing.
#[derive(Debug, Serialize, Deserialize)]
pub struct UnsignedTransaction {
    /// The unsigned, versioned transaction.
    pub transaction: VersionedTransaction,
    /// The blockheight that the transaction's blockhash is valid for.
    pub last_valid_block_height: u64,
}

// --- Layer 2: Simulation Output Structs ---

/// The detailed result of a `surfpool` simulation.
#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub signature: String,
    pub debug_info: DebugInfo,
}

/// Detailed debug information collected during a simulation.
#[derive(Debug, Clone)]
pub struct DebugInfo {
    /// Human-readable list of unique accounts involved in the transaction.
    pub readable_accounts: Vec<String>,
    /// The error message if the transaction failed.
    pub tx_error: Option<String>,
    /// The final result of the transaction.
    pub tx_result: TransactionResult,
    pub initial_source_token_balance: Option<u64>,
    pub final_source_token_balance: Option<u64>,
    pub initial_destination_token_balance: Option<u64>,
    pub final_destination_token_balance: Option<u64>,
}

/// The outcome of a simulated transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionResult {
    Success,
    Failure,
}

// --- Jupiter API Data Structures ---

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionData {
    pub program_id: String,
    pub accounts: Vec<Key>,
    pub data: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Key {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse {
    pub instructions: Vec<InstructionData>,
    #[serde(rename = "addressLookupTableAddresses")]
    pub address_lookup_table_addresses: Option<Vec<String>>,
}
