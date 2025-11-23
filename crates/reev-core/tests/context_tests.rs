//! Tests for context module

use reev_core::context::{ContextResolver, SolanaEnvironment};
use reev_types::benchmark::TokenBalance;
use reev_types::flow::WalletContext;

#[tokio::test]
async fn test_placeholder_mappings() {
    let mut context = WalletContext::new("test_pubkey".to_string());
    context.sol_balance = 1_000_000_000; // 1 SOL
    context.add_token_balance(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        TokenBalance::new(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1_000_000,
        )
        .with_decimals(6)
        .with_symbol("USDC".to_string()),
    );
    context.add_token_price(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        1.0,
    );
    context.calculate_total_value();

    let resolver = ContextResolver::new(SolanaEnvironment::default());
    let mappings = resolver.get_placeholder_mappings(&context).await;

    assert_eq!(
        mappings.get("WALLET_PUBKEY"),
        Some(&"test_pubkey".to_string())
    );
    assert_eq!(
        mappings.get("SOL_BALANCE"),
        Some(&"1.000000000".to_string())
    );
    assert_eq!(mappings.get("USDC_BALANCE"), Some(&"1.000000".to_string()));
}

#[tokio::test]
async fn test_benchmark_mode_detection() {
    let resolver = ContextResolver::new(SolanaEnvironment::default());

    // Test with USER_WALLET_PUBKEY
    assert!(resolver.is_benchmark_mode("USER_WALLET_PUBKEY"));

    // Test with regular pubkey
    assert!(!resolver.is_benchmark_mode("11111111111111111111111111111112"));

    // Test with BENCHMARK_MODE env variable
    std::env::set_var("BENCHMARK_MODE", "1");
    assert!(resolver.is_benchmark_mode("any_pubkey"));
    std::env::remove_var("BENCHMARK_MODE");
}
