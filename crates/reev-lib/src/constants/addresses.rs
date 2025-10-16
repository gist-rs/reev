//! Blockchain address constants for Solana mainnet

use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Token mint addresses
pub mod tokens {
    use super::*;

    /// USDC mint address on Solana mainnet
    pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

    /// Wrapped SOL mint address
    pub const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

    /// Jupiter USDC mint address (jUSDC)
    pub const JUSDC_MINT: &str = "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D";

    /// Get USDC mint as Pubkey
    pub fn usdc_mint() -> Pubkey {
        Pubkey::from_str(USDC_MINT).expect("Invalid USDC mint address")
    }

    /// Get SOL mint as Pubkey
    pub fn sol_mint() -> Pubkey {
        Pubkey::from_str(SOL_MINT).expect("Invalid SOL mint address")
    }

    /// Get Jupiter USDC mint as Pubkey
    pub fn jusdc_mint() -> Pubkey {
        Pubkey::from_str(JUSDC_MINT).expect("Invalid JUSDC mint address")
    }
}

/// Program IDs
pub mod programs {
    use super::*;

    /// Solana System Program
    pub const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";

    /// SPL Token Program
    pub const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

    /// SPL Associated Token Account Program
    pub const A_TOKEN_PROGRAM: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

    /// Jupiter Program
    pub const JUPITER_PROGRAM: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";

    /// Get System Program as Pubkey
    pub fn system_program() -> Pubkey {
        Pubkey::from_str(SYSTEM_PROGRAM).expect("Invalid System Program address")
    }

    /// Get Token Program as Pubkey
    pub fn token_program() -> Pubkey {
        Pubkey::from_str(TOKEN_PROGRAM).expect("Invalid Token Program address")
    }

    /// Get Associated Token Program as Pubkey
    pub fn associated_token_program() -> Pubkey {
        Pubkey::from_str(A_TOKEN_PROGRAM).expect("Invalid Associated Token Program address")
    }

    /// Get Jupiter Program as Pubkey
    pub fn jupiter_program() -> Pubkey {
        Pubkey::from_str(JUPITER_PROGRAM).expect("Invalid Jupiter Program address")
    }
}

/// Network and service addresses
pub mod network {
    /// Default RPC host for local development
    pub const LOCALHOST: &str = "127.0.0.1";

    /// Default surfpool port
    pub const SURFPOOL_PORT: u16 = 8899;

    /// Default reev-agent port
    pub const REEV_AGENT_PORT: u16 = 9090;

    /// Get surfpool RPC URL
    pub fn surfpool_url() -> String {
        format!("http://{LOCALHOST}:{SURFPOOL_PORT}")
    }

    /// Get reev-agent URL
    pub fn reev_agent_url() -> String {
        format!("http://{LOCALHOST}:{REEV_AGENT_PORT}")
    }
}
