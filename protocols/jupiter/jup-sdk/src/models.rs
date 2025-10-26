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

// --- Token API Data Structures ---

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub decimals: u8,
    #[serde(rename = "circSupply")]
    pub circ_supply: Option<f64>,
    #[serde(rename = "totalSupply")]
    pub total_supply: Option<f64>,
    #[serde(rename = "tokenProgram")]
    pub token_program: String,
    #[serde(rename = "mintAuthority")]
    pub mint_authority: Option<String>,
    #[serde(rename = "freezeAuthority")]
    pub freeze_authority: Option<String>,
    #[serde(rename = "firstPool")]
    pub first_pool: Option<FirstPool>,
    #[serde(rename = "holderCount")]
    pub holder_count: Option<u64>,
    pub audit: Option<AuditInfo>,
    pub apy: Option<ApyInfo>,
    #[serde(rename = "organicScore")]
    pub organic_score: Option<f64>,
    #[serde(rename = "organicScoreLabel")]
    pub organic_score_label: Option<String>,
    #[serde(rename = "isVerified")]
    pub is_verified: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub fdv: Option<f64>,
    pub mcap: Option<f64>,
    #[serde(rename = "usdPrice")]
    pub usd_price: Option<f64>,
    #[serde(rename = "priceBlockId")]
    pub price_block_id: Option<u64>,
    pub liquidity: Option<f64>,
    #[serde(rename = "stats5m")]
    pub stats_5m: Option<TokenStats>,
    #[serde(rename = "stats1h")]
    pub stats_1h: Option<TokenStats>,
    #[serde(rename = "stats6h")]
    pub stats_6h: Option<TokenStats>,
    #[serde(rename = "stats24h")]
    pub stats_24h: Option<TokenStats>,
    #[serde(rename = "ctLikes")]
    pub ct_likes: Option<u32>,
    #[serde(rename = "smartCtLikes")]
    pub smart_ct_likes: Option<u32>,
    pub twitter: Option<String>,
    pub website: Option<String>,
    pub dev: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstPool {
    pub id: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuditInfo {
    #[serde(rename = "mintAuthorityDisabled")]
    pub mint_authority_disabled: Option<bool>,
    #[serde(rename = "freezeAuthorityDisabled")]
    pub freeze_authority_disabled: Option<bool>,
    #[serde(rename = "topHoldersPercentage")]
    pub top_holders_percentage: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApyInfo {
    #[serde(rename = "jupEarn")]
    pub jup_earn: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenStats {
    #[serde(rename = "priceChange")]
    pub price_change: Option<f64>,
    #[serde(rename = "liquidityChange")]
    pub liquidity_change: Option<f64>,
    #[serde(rename = "volumeChange")]
    pub volume_change: Option<f64>,
    #[serde(rename = "buyVolume")]
    pub buy_volume: Option<f64>,
    #[serde(rename = "sellVolume")]
    pub sell_volume: Option<f64>,
    #[serde(rename = "buyOrganicVolume")]
    pub buy_organic_volume: Option<f64>,
    #[serde(rename = "sellOrganicVolume")]
    pub sell_organic_volume: Option<f64>,
    #[serde(rename = "numBuys")]
    pub num_buys: Option<u64>,
    #[serde(rename = "numSells")]
    pub num_sells: Option<u64>,
    #[serde(rename = "numTraders")]
    pub num_traders: Option<u64>,
    #[serde(rename = "numOrganicBuyers")]
    pub num_organic_buyers: Option<u64>,
    #[serde(rename = "numNetBuyers")]
    pub num_net_buyers: Option<u64>,
}

/// Parameters for searching tokens
#[derive(Debug, Clone)]
pub struct TokenSearchParams {
    pub query: String,
}
