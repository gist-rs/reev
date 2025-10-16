//! Tests for address constants

use reev_lib::constants::addresses::{network, programs, tokens};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[test]
fn test_valid_addresses() {
    // Test that all hardcoded addresses are valid
    assert!(Pubkey::from_str(tokens::USDC_MINT).is_ok());
    assert!(Pubkey::from_str(tokens::SOL_MINT).is_ok());
    assert!(Pubkey::from_str(tokens::JUSDC_MINT).is_ok());
    assert!(Pubkey::from_str(programs::SYSTEM_PROGRAM).is_ok());
    assert!(Pubkey::from_str(programs::TOKEN_PROGRAM).is_ok());
    assert!(Pubkey::from_str(programs::A_TOKEN_PROGRAM).is_ok());
    assert!(Pubkey::from_str(programs::JUPITER_PROGRAM).is_ok());
}

#[test]
fn test_url_generation() {
    assert_eq!(network::surfpool_url(), "http://127.0.0.1:8899");
    assert_eq!(network::reev_agent_url(), "http://127.0.0.1:9090");
}
