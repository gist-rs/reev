use anchor_client::anchor_lang::AccountDeserialize;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum PositionSide {
    Long,
    Short,
}

impl PositionSide {
    pub fn as_byte(&self) -> u8 {
        match self {
            PositionSide::Long => 1,
            PositionSide::Short => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RequestChange {
    Increase,
    Decrease,
}

impl RequestChange {
    pub fn as_byte(&self) -> u8 {
        match self {
            RequestChange::Increase => 1,
            RequestChange::Decrease => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub pubkey: Pubkey,
    pub owner: Pubkey,
    pub custody: Pubkey,
    pub collateral_custody: Pubkey,
    pub pool: Pubkey,
    pub side: PositionSide,
    pub size_usd: u64,
    pub collateral_usd: u64,
    pub open_time: i64,
    pub borrow_index: u64,
    pub cumulative_interest_snapshot: u64,
}

#[derive(Debug, Clone)]
pub struct PositionRequest {
    pub pubkey: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub counter: u64,
    pub request_change: RequestChange,
    pub size_usd_delta: u64,
    pub collateral_token_delta: u64,
    pub collateral_usd_delta: u64,
    pub price_slippage: u64,
    pub jupiter_minimum_out: Option<u64>,
    pub entire_position: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct Custody {
    pub pubkey: Pubkey,
    pub pool: Pubkey,
    pub mint: Pubkey,
    pub is_asset: bool,
    pub total_amount: u64,
    pub borrow_rate: u64,
    pub cumulative_interest: u64,
    pub total_borrow: u64,
    pub oracle_price: u64,
    pub decimals: u8,
    pub pricing: Pricings,
}

#[derive(Debug, Clone)]
pub struct Pricings {
    pub trade_fee_long: u64,
    pub trade_fee_short: u64,
    pub oracle_price_buffer: u64,
    pub max_position_size_usd: u64,
    pub funding_rate_conflict: u64,
    pub pnl_conflict: u64,
    pub borrow_rate_conflict: u64,
}

#[derive(Debug, Clone)]
pub struct Pool {
    pub pubkey: Pubkey,
    pub authority: Pubkey,
    pub lp_mint: Pubkey,
    pub total_lp_shares: u64,
    pub aum: u64,
    pub assets: HashMap<Pubkey, Custody>,
}

#[derive(Debug, Clone)]
pub struct OraclePrice {
    pub price: u64,
    pub expo: i32,
    pub confidence: u64,
    pub publish_time: i64,
}

#[derive(Debug, Clone)]
pub struct CreatePositionRequestParams {
    pub custody: Pubkey,
    pub collateral_custody: Pubkey,
    pub collateral_token_delta: u64,
    pub input_mint: Pubkey,
    pub jupiter_minimum_out: Option<u64>,
    pub owner: Pubkey,
    pub price_slippage: u64,
    pub side: PositionSide,
    pub size_usd_delta: u64,
    pub position_pubkey: Pubkey,
}

#[derive(Debug, Clone)]
pub struct ClosePositionRequestParams {
    pub position_pubkey: Pubkey,
    pub desired_mint: Pubkey,
    pub price_slippage: u64,
}

#[derive(Debug, Clone)]
pub struct PositionAccount {
    pub pubkey: Pubkey,
    pub account: Position,
}

#[derive(Debug, Clone)]
pub struct PositionRequestAccount {
    pub pubkey: Pubkey,
    pub account: PositionRequest,
}

#[derive(Debug, Clone)]
pub struct CustodyAccount {
    pub pubkey: Pubkey,
    pub account: Custody,
}

#[derive(Debug, Clone)]
pub struct BorrowPosition {
    pub pubkey: Pubkey,
    pub owner: Pubkey,
    pub custody: Pubkey,
    pub pool: Pubkey,
    pub borrow_amount: u64,
    pub cumulative_interest: u64,
    pub last_update: i64,
}

pub type PoolApr = u64;
pub type ContractTypes = HashMap<String, serde_json::Value>;

// Helper functions for decoding accounts from raw data
pub trait JupiterPerpsAccount: AccountDeserialize + Sized {
    fn deserialize_account(data: &[u8]) -> Result<Self>;
}

impl<T: AccountDeserialize> JupiterPerpsAccount for T {
    fn deserialize_account(data: &[u8]) -> Result<Self> {
        let mut disc = [0u8; 8];
        disc.copy_from_slice(&data[..8]);
        Self::try_deserialize(&mut &data[8..])
            .map_err(|e| anyhow::anyhow!("Failed to deserialize account: {e}"))
    }
}

#[derive(Debug, Clone)]
pub struct MarketTradeContext {
    pub custody: CustodyAccount,
    pub collateral_custody: CustodyAccount,
    pub input_mint: Pubkey,
    pub owner: Pubkey,
    pub position_side: PositionSide,
    pub size_change: u64,
    pub collateral_token_change: u64,
    pub price_slippage: u64,
    pub jupiter_minimum_out: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct MarketTradeContextBuilder {
    custody: Option<CustodyAccount>,
    collateral_custody: Option<CustodyAccount>,
    input_mint: Option<Pubkey>,
    owner: Option<Pubkey>,
    size_change: Option<u64>,
    collateral_token_change: Option<u64>,
    price_slippage: Option<u64>,
    jupiter_minimum_out: Option<u64>,
}

impl MarketTradeContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn custody(mut self, custody: CustodyAccount) -> Self {
        self.custody = Some(custody);
        self
    }

    pub fn collateral_custody(mut self, collateral_custody: CustodyAccount) -> Self {
        self.collateral_custody = Some(collateral_custody);
        self
    }

    pub fn input_mint(mut self, input_mint: Pubkey) -> Self {
        self.input_mint = Some(input_mint);
        self
    }

    pub fn owner(mut self, owner: Pubkey) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn size_change(mut self, size_change: u64) -> Self {
        self.size_change = Some(size_change);
        self
    }

    pub fn collateral_token_change(mut self, collateral_token_change: u64) -> Self {
        self.collateral_token_change = Some(collateral_token_change);
        self
    }

    pub fn price_slippage(mut self, price_slippage: u64) -> Self {
        self.price_slippage = Some(price_slippage);
        self
    }

    pub fn jupiter_minimum_out(mut self, jupiter_minimum_out: Option<u64>) -> Self {
        self.jupiter_minimum_out = jupiter_minimum_out;
        self
    }

    pub fn build_long(self) -> MarketTradeContext {
        MarketTradeContext {
            custody: self.custody.expect("custody is required"),
            collateral_custody: self
                .collateral_custody
                .expect("collateral_custody is required"),
            input_mint: self.input_mint.expect("input_mint is required"),
            owner: self.owner.expect("owner is required"),
            position_side: PositionSide::Long,
            size_change: self.size_change.expect("size_change is required"),
            collateral_token_change: self
                .collateral_token_change
                .expect("collateral_token_change is required"),
            price_slippage: self.price_slippage.expect("price_slippage is required"),
            jupiter_minimum_out: self.jupiter_minimum_out,
        }
    }

    pub fn build_short(self) -> MarketTradeContext {
        MarketTradeContext {
            custody: self.custody.expect("custody is required"),
            collateral_custody: self
                .collateral_custody
                .expect("collateral_custody is required"),
            input_mint: self.input_mint.expect("input_mint is required"),
            owner: self.owner.expect("owner is required"),
            position_side: PositionSide::Short,
            size_change: self.size_change.expect("size_change is required"),
            collateral_token_change: self
                .collateral_token_change
                .expect("collateral_token_change is required"),
            price_slippage: self.price_slippage.expect("price_slippage is required"),
            jupiter_minimum_out: self.jupiter_minimum_out,
        }
    }
}

impl MarketTradeContext {
    pub fn builder() -> MarketTradeContextBuilder {
        MarketTradeContextBuilder::new()
    }

    #[deprecated(note = "Use MarketTradeContext::builder() instead")]
    pub fn new_open_long(
        custody: CustodyAccount,
        collateral_custody: CustodyAccount,
        input_mint: Pubkey,
        owner: Pubkey,
        size_change: u64,
        collateral_token_change: u64,
        price_slippage: u64,
    ) -> Self {
        Self::builder()
            .custody(custody)
            .collateral_custody(collateral_custody)
            .input_mint(input_mint)
            .owner(owner)
            .size_change(size_change)
            .collateral_token_change(collateral_token_change)
            .price_slippage(price_slippage)
            .build_long()
    }

    #[deprecated(note = "Use MarketTradeContext::builder() instead")]
    pub fn new_open_short(
        custody: CustodyAccount,
        collateral_custody: CustodyAccount,
        input_mint: Pubkey,
        owner: Pubkey,
        size_change: u64,
        collateral_token_change: u64,
        price_slippage: u64,
    ) -> Self {
        Self::builder()
            .custody(custody)
            .collateral_custody(collateral_custody)
            .input_mint(input_mint)
            .owner(owner)
            .size_change(size_change)
            .collateral_token_change(collateral_token_change)
            .price_slippage(price_slippage)
            .build_short()
    }
}
