//! Core types for reev-core architecture

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use std::collections::HashMap;

// Constants from the plan
pub const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

/// Token prices by mint address
pub type TokenPrices = HashMap<String, f64>;

/// Wallet state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletState {
    /// SOL amount in lamports
    pub sol_amount: u64,
    /// USDC amount in smallest units
    pub usdc_amount: u64,
    /// SOL value in USD
    pub sol_usd_value: f64,
    /// USDC value in USD
    pub usdc_usd_value: f64,
    /// Total USD value
    pub total_usd_value: f64,
}

impl Default for WalletState {
    fn default() -> Self {
        Self::new()
    }
}

impl WalletState {
    pub fn new() -> Self {
        Self {
            sol_amount: 0,
            usdc_amount: 0,
            sol_usd_value: 0.0,
            usdc_usd_value: 0.0,
            total_usd_value: 0.0,
        }
    }

    pub fn calculate_total_value(&mut self) {
        self.total_usd_value = self.sol_usd_value + self.usdc_usd_value;
    }

    pub fn sol_balance_sol(&self) -> f64 {
        self.sol_amount as f64 / 1_000_000_000.0
    }

    pub fn usdc_balance_usdc(&self) -> f64 {
        self.usdc_amount as f64 / 1_000_000.0
    }
}

/// Generated wallet with keypair (not cloneable due to Keypair)
#[derive(Debug)]
pub struct GeneratedWallet {
    /// Wallet keypair
    pub keypair: Keypair,
    /// Public key
    pub pubkey: Pubkey,
    /// SOL balance
    pub sol_balance: u64,
    /// USDC balance
    pub usdc_balance: u64,
}

impl GeneratedWallet {
    pub fn new(keypair: Keypair) -> Self {
        let pubkey = keypair.pubkey();
        Self {
            keypair,
            pubkey,
            sol_balance: 0,
            usdc_balance: 0,
        }
    }

    pub fn pubkey_string(&self) -> String {
        self.pubkey.to_string()
    }
}

/// Cached API service configuration
#[derive(Debug, Clone)]
pub struct CachedApiService {
    /// Cache directory path
    pub cache_dir: String,
    /// Real Jupiter client
    pub real_jupiter_client: bool,
    /// Mock mode flag
    pub mock_mode: bool,
}

impl CachedApiService {
    pub fn new(cache_dir: String, real_jupiter_client: bool, mock_mode: bool) -> Self {
        Self {
            cache_dir,
            real_jupiter_client,
            mock_mode,
        }
    }
}

/// Refined prompt with step information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinedPrompt {
    /// Step number
    pub step: u32,
    /// Refined prompt text
    pub prompt: String,
    /// Reasoning for this step
    pub reasoning: String,
}

impl RefinedPrompt {
    pub fn new(step: u32, prompt: String, reasoning: String) -> Self {
        Self {
            step,
            prompt,
            reasoning,
        }
    }
}

/// Execution result from tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Unique execution ID
    pub execution_id: String,
    /// Tool name that was executed
    pub tool_name: String,
    /// Success status
    pub success: bool,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Updated context after execution
    pub updated_context: Option<WalletState>,
}

impl ExecutionResult {
    pub fn new(execution_id: String, tool_name: String) -> Self {
        Self {
            execution_id,
            tool_name,
            success: false,
            execution_time_ms: 0,
            updated_context: None,
        }
    }
}

/// Request context for the 18-step flow
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Unique request ID
    pub request_id: String,
    /// User prompt
    pub user_prompt: String,
    /// Wallet address
    pub wallet_address: String,
    /// Entry wallet state
    pub entry_wallet_state: Option<WalletState>,
    /// Exit wallet state
    pub exit_wallet_state: Option<WalletState>,
    /// Refined prompt series
    pub prompt_series: Vec<RefinedPrompt>,
    /// Execution results
    pub execution_results: Vec<ExecutionResult>,
    /// Token prices
    pub token_prices: TokenPrices,
    /// API service configuration
    pub api_service: CachedApiService,
    /// Current step number
    pub current_step: u32,
    /// Error information
    pub error: Option<String>,
}

impl RequestContext {
    pub fn new(
        request_id: String,
        user_prompt: String,
        wallet_address: String,
        api_service: CachedApiService,
    ) -> Self {
        Self {
            request_id,
            user_prompt,
            wallet_address,
            entry_wallet_state: None,
            exit_wallet_state: None,
            prompt_series: Vec::new(),
            execution_results: Vec::new(),
            token_prices: HashMap::new(),
            api_service,
            current_step: 0,
            error: None,
        }
    }

    pub fn add_execution_result(&mut self, result: ExecutionResult) {
        self.execution_results.push(result);
    }

    pub fn add_refined_prompt(&mut self, prompt: RefinedPrompt) {
        self.prompt_series.push(prompt);
    }

    pub fn increment_step(&mut self) {
        self.current_step += 1;
    }
}

/// Tool execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionRequest {
    /// Tool name to execute
    pub tool_name: String,
    /// Tool parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Execution context
    pub context: WalletState,
    /// Expected output format
    pub expected_format: Option<String>,
}

impl ToolExecutionRequest {
    pub fn new(tool_name: String, context: WalletState) -> Self {
        Self {
            tool_name,
            parameters: HashMap::new(),
            context,
            expected_format: None,
        }
    }
}

/// Jupiter transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterTransaction {
    /// Transaction signature
    pub signature: String,
    /// Input mint
    pub input_mint: String,
    /// Output mint
    pub output_mint: String,
    /// Input amount
    pub input_amount: u64,
    /// Output amount
    pub output_amount: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl JupiterTransaction {
    pub fn new(
        signature: String,
        input_mint: String,
        output_mint: String,
        input_amount: u64,
        output_amount: u64,
    ) -> Self {
        Self {
            signature,
            input_mint,
            output_mint,
            input_amount,
            output_amount,
            timestamp: Utc::now(),
        }
    }
}

/// Error classification for recovery
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ErrorClassification {
    /// Retry with exponential backoff
    Retryable,
    /// User input required
    UserInput,
    /// Fatal error, cannot recover
    Fatal,
    /// Temporary network issue
    Network,
    /// Insufficient funds
    InsufficientFunds,
}

/// Error recovery strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecovery {
    /// Error classification
    pub classification: ErrorClassification,
    /// Recovery action
    pub action: String,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Current retry count
    pub retry_count: u32,
}

impl ErrorRecovery {
    pub fn new(classification: ErrorClassification, action: String, max_retries: u32) -> Self {
        Self {
            classification,
            action,
            max_retries,
            retry_count: 0,
        }
    }

    pub fn can_retry(&self) -> bool {
        matches!(
            self.classification,
            ErrorClassification::Retryable | ErrorClassification::Network
        ) && self.retry_count < self.max_retries
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}
