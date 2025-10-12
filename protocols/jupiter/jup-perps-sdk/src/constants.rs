use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::str::FromStr;

pub const RPC_URL: &str = "https://api.mainnet-beta.solana.com";

pub const DOVES_PROGRAM_ID: &str = "DoVEsk76QybCEHQGzkvYPWLQu9gzNoZZZt3TPiL597e";

pub const JUPITER_PERPETUALS_PROGRAM_ID: &str = "PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu";

pub const JUPITER_PERPETUALS_EVENT_AUTHORITY_PUBKEY: &str =
    "37hJBDnntwqhGbK7L6M1bLyvccj4u55CCUiLPdYkiqBN";

pub const JLP_POOL_ACCOUNT_PUBKEY: &str = "5BUwFW4nRbftYTDMbgxykoFWqWHPzahFSNAaaaJtVKsq";

pub const JLP_MINT_PUBKEY: &str = "27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4";

#[derive(Debug, Clone, Copy)]
pub enum CustodyPubkey {
    Sol,
    Eth,
    Btc,
    Usdc,
    Usdt,
}

impl CustodyPubkey {
    pub const fn pubkey(self) -> &'static str {
        match self {
            CustodyPubkey::Sol => "7xS2gz2bTp3fwCC7knJvUWTEU9Tycczu6VhJYKgi1wdz",
            CustodyPubkey::Eth => "AQCGyheWPLeo6Qp9WpYS9m3Qj479t7R636N9ey1rEjEn",
            CustodyPubkey::Btc => "5Pv3gM9JrFFH883SWAhvJC9RPYmo8UNxuFtv5bMMALkm",
            CustodyPubkey::Usdc => "G18jKKXQwBbrHeiK3C9MRXhkHsLHf7XgCSisykV46EZa",
            CustodyPubkey::Usdt => "4vkNeXiYEUizLdrpdPS1eC2mccyM4NUPRtERrk6ZETkk",
        }
    }
}

pub fn get_custody_pubkeys() -> Vec<Pubkey> {
    vec![
        Pubkey::from_str(CustodyPubkey::Sol.pubkey()).expect("Invalid SOL custody pubkey"),
        Pubkey::from_str(CustodyPubkey::Btc.pubkey()).expect("Invalid BTC custody pubkey"),
        Pubkey::from_str(CustodyPubkey::Eth.pubkey()).expect("Invalid ETH custody pubkey"),
        Pubkey::from_str(CustodyPubkey::Usdc.pubkey()).expect("Invalid USDC custody pubkey"),
        Pubkey::from_str(CustodyPubkey::Usdt.pubkey()).expect("Invalid USDT custody pubkey"),
    ]
}

pub const USDC_DECIMALS: u8 = 6;
pub const BPS_POWER: u64 = 10_000;
pub const DBPS_POWER: u64 = 100_000;
pub const RATE_POWER: u64 = 1_000_000_000;
pub const DEBT_POWER: u64 = RATE_POWER;
pub const BORROW_SIZE_PRECISION: u64 = 1000;
pub const JLP_DECIMALS: u8 = 6;

pub fn get_rpc_client() -> RpcClient {
    let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| RPC_URL.to_string());
    RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed())
}

pub fn get_doves_program_id() -> Pubkey {
    Pubkey::from_str(DOVES_PROGRAM_ID).expect("Invalid DOVES program ID")
}

pub fn get_jupiter_perpetuals_program_id() -> Pubkey {
    Pubkey::from_str(JUPITER_PERPETUALS_PROGRAM_ID).expect("Invalid Jupiter Perpetuals program ID")
}

pub fn get_jupiter_perpetuals_event_authority() -> Pubkey {
    Pubkey::from_str(JUPITER_PERPETUALS_EVENT_AUTHORITY_PUBKEY)
        .expect("Invalid Jupiter Perpetuals event authority pubkey")
}

pub fn get_jlp_pool_account() -> Pubkey {
    Pubkey::from_str(JLP_POOL_ACCOUNT_PUBKEY).expect("Invalid JLP pool account pubkey")
}

pub fn get_jlp_mint() -> Pubkey {
    Pubkey::from_str(JLP_MINT_PUBKEY).expect("Invalid JLP mint pubkey")
}

pub fn get_system_program_id() -> Pubkey {
    solana_sdk::system_program::id()
}
