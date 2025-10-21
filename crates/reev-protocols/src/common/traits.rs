//! Protocol trait definitions
//!
//! This module contains the core trait definitions for all blockchain protocols.

use crate::common::{HealthStatus, ProtocolError, ProtocolMetrics};
use async_trait::async_trait;
use reev_lib::agent::RawInstruction;
use serde::{Deserialize, Serialize};

/// Core protocol trait that all protocols must implement
#[async_trait]
pub trait Protocol: Send + Sync {
    /// Protocol name for identification
    fn name(&self) -> &'static str;

    /// Protocol version
    fn version(&self) -> &'static str {
        "1.0.0"
    }

    /// Check if the protocol is healthy and operational
    async fn health_check(&self) -> Result<HealthStatus, ProtocolError>;

    /// Get current protocol metrics
    fn metrics(&self) -> &ProtocolMetrics;

    /// Reset protocol metrics
    fn reset_metrics(&mut self);

    /// Validate configuration
    fn validate_config(&self) -> Result<(), ProtocolError> {
        // Default implementation - override if needed
        Ok(())
    }

    /// Get supported operations
    fn supported_operations(&self) -> Vec<ProtocolOperation> {
        vec![]
    }

    /// Initialize the protocol
    async fn initialize(&mut self) -> Result<(), ProtocolError> {
        // Default implementation - override if needed
        Ok(())
    }

    /// Shutdown the protocol
    async fn shutdown(&mut self) -> Result<(), ProtocolError> {
        // Default implementation - override if needed
        Ok(())
    }
}

/// Operations supported by protocols
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProtocolOperation {
    Swap,
    Deposit,
    Withdraw,
    Transfer,
    Stake,
    Unstake,
    Borrow,
    Repay,
    GetPositions,
    GetEarnings,
    GetBalance,
    Custom(String),
}

/// Trait for protocols that support token swaps
#[async_trait]
pub trait SwapProtocol: Protocol {
    /// Execute a token swap
    async fn swap(
        &self,
        user_pubkey: &str,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<Vec<RawInstruction>, ProtocolError>;

    /// Get a quote for a potential swap
    async fn get_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<SwapQuote, ProtocolError>;

    /// Get supported token pairs
    async fn supported_pairs(&self) -> Result<Vec<TokenPair>, ProtocolError>;

    /// Get minimum swap amount for a token pair
    async fn minimum_amount(
        &self,
        input_mint: &str,
        output_mint: &str,
    ) -> Result<u64, ProtocolError>;
}

/// Swap quote information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuote {
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact_pct: f64,
    pub slippage_bps: u16,
    pub routes: Vec<SwapRoute>,
    pub fee_amount: u64,
    pub fee_pct: f64,
    pub valid_until: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRoute {
    pub protocol: String,
    pub percentage: f64,
    pub steps: Vec<SwapStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapStep {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub input_mint: String,
    pub output_mint: String,
    pub liquidity_usd: f64,
    pub volume_24h: f64,
}

/// Trait for protocols that support lending operations
#[async_trait]
pub trait LendProtocol: Protocol {
    /// Deposit tokens into lending protocol
    async fn deposit(
        &self,
        user_pubkey: &str,
        token_mint: &str,
        amount: u64,
    ) -> Result<Vec<RawInstruction>, ProtocolError>;

    /// Withdraw tokens from lending protocol
    async fn withdraw(
        &self,
        user_pubkey: &str,
        token_mint: &str,
        amount: u64,
    ) -> Result<Vec<RawInstruction>, ProtocolError>;

    /// Get user's lending positions
    async fn get_positions(&self, user_pubkey: &str)
        -> Result<Vec<LendingPosition>, ProtocolError>;

    /// Get earnings for a position
    async fn get_earnings(
        &self,
        user_pubkey: &str,
        position_id: Option<String>,
    ) -> Result<Vec<EarningInfo>, ProtocolError>;

    /// Get available lending markets
    async fn available_markets(&self) -> Result<Vec<LendingMarket>, ProtocolError>;

    /// Get APY for a specific market
    async fn get_apy(&self, token_mint: &str) -> Result<f64, ProtocolError>;
}

/// Lending position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LendingPosition {
    pub position_id: String,
    pub token_mint: String,
    pub deposited_amount: u64,
    pub current_value: u64,
    pub apy: f64,
    pub protocol: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Earning information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningInfo {
    pub position_id: String,
    pub token_mint: String,
    pub earned_amount: u64,
    pub usd_value: f64,
    pub earned_at: chrono::DateTime<chrono::Utc>,
    pub apy: f64,
}

/// Lending market information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LendingMarket {
    pub token_mint: String,
    pub token_symbol: String,
    pub total_supply: u64,
    pub total_borrowed: u64,
    pub supply_apy: f64,
    pub borrow_apy: f64,
    pub utilization_rate: f64,
    pub protocol: String,
}

/// Trait for protocols that support native transfers
#[async_trait]
pub trait TransferProtocol: Protocol {
    /// Transfer native SOL
    async fn transfer_sol(
        &self,
        from_pubkey: &str,
        to_pubkey: &str,
        lamports: u64,
    ) -> Result<Vec<RawInstruction>, ProtocolError>;

    /// Transfer SPL tokens
    async fn transfer_spl(
        &self,
        source_pubkey: &str,
        destination_pubkey: &str,
        authority_pubkey: &str,
        token_mint: &str,
        amount: u64,
    ) -> Result<Vec<RawInstruction>, ProtocolError>;

    /// Get balance for an account
    async fn get_balance(
        &self,
        pubkey: &str,
        token_mint: Option<String>,
    ) -> Result<u64, ProtocolError>;

    /// Validate address format
    fn validate_address(&self, address: &str) -> Result<(), ProtocolError>;
}

/// Trait for protocols that support staking operations
#[async_trait]
pub trait StakeProtocol: Protocol {
    /// Stake tokens
    async fn stake(
        &self,
        user_pubkey: &str,
        token_mint: &str,
        amount: u64,
    ) -> Result<Vec<RawInstruction>, ProtocolError>;

    /// Unstake tokens
    async fn unstake(
        &self,
        user_pubkey: &str,
        token_mint: &str,
        amount: u64,
    ) -> Result<Vec<RawInstruction>, ProtocolError>;

    /// Get staking positions
    async fn get_staking_positions(
        &self,
        user_pubkey: &str,
    ) -> Result<Vec<StakingPosition>, ProtocolError>;

    /// Get staking rewards
    async fn get_rewards(&self, user_pubkey: &str) -> Result<Vec<RewardInfo>, ProtocolError>;
}

/// Staking position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPosition {
    pub position_id: String,
    pub token_mint: String,
    pub staked_amount: u64,
    pub rewards_earned: u64,
    pub apy: f64,
    pub protocol: String,
    pub lock_period: Option<chrono::Duration>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Reward information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardInfo {
    pub position_id: String,
    pub token_mint: String,
    pub reward_amount: u64,
    pub usd_value: f64,
    pub earned_at: chrono::DateTime<chrono::Utc>,
}

/// Trait for protocols that support borrowing operations
#[async_trait]
pub trait BorrowProtocol: Protocol {
    /// Borrow tokens
    async fn borrow(
        &self,
        user_pubkey: &str,
        token_mint: &str,
        amount: u64,
    ) -> Result<Vec<RawInstruction>, ProtocolError>;

    /// Repay borrowed tokens
    async fn repay(
        &self,
        user_pubkey: &str,
        token_mint: &str,
        amount: u64,
    ) -> Result<Vec<RawInstruction>, ProtocolError>;

    /// Get borrowing positions
    async fn get_borrowing_positions(
        &self,
        user_pubkey: &str,
    ) -> Result<Vec<BorrowingPosition>, ProtocolError>;

    /// Get collateral information
    async fn get_collateral_info(&self, user_pubkey: &str)
        -> Result<CollateralInfo, ProtocolError>;
}

/// Borrowing position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowingPosition {
    pub position_id: String,
    pub token_mint: String,
    pub borrowed_amount: u64,
    pub interest_rate: f64,
    pub collateral_required: u64,
    pub protocol: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Collateral information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollateralInfo {
    pub total_collateral_usd: f64,
    pub total_borrowed_usd: f64,
    pub health_factor: f64,
    pub liquidation_threshold: f64,
    pub collateral_assets: Vec<CollateralAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollateralAsset {
    pub token_mint: String,
    pub amount: u64,
    pub usd_value: f64,
    pub collateral_factor: f64,
}
