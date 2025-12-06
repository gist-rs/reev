//! Tests for TokenMint implementation

use reev_core::execution::context_builder::{helpers, mints, MintError, TokenMint};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;

#[test]
fn test_token_mint_new() {
    let address = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let token_mint = TokenMint::new(address);

    assert_eq!(token_mint.address, address);
    assert_eq!(token_mint.symbol, None);
    assert_eq!(token_mint.decimals, None);
    assert_eq!(token_mint.price_usd, None);
}

#[test]
fn test_token_mint_with_symbol() {
    let address = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let symbol = "USDC";
    let token_mint = TokenMint::with_symbol(address, symbol);

    assert_eq!(token_mint.address, address);
    assert_eq!(token_mint.symbol, Some(symbol.to_string()));
    assert_eq!(token_mint.decimals, None);
    assert_eq!(token_mint.price_usd, None);
}

#[test]
fn test_token_mint_with_metadata() {
    let address = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let symbol = "USDC";
    let decimals = 6;
    let price_usd = 1.0;
    let token_mint = TokenMint::with_metadata(
        address,
        Some(symbol.to_string()),
        Some(decimals),
        Some(price_usd),
    );

    assert_eq!(token_mint.address, address);
    assert_eq!(token_mint.symbol, Some(symbol.to_string()));
    assert_eq!(token_mint.decimals, Some(decimals));
    assert_eq!(token_mint.price_usd, Some(price_usd));
}

#[test]
fn test_token_mint_to_pubkey() {
    let address = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let token_mint = TokenMint::new(address);

    let pubkey = token_mint.to_pubkey().unwrap();
    assert_eq!(pubkey.to_string(), address);
}

#[test]
fn test_token_mint_to_pubkey_invalid() {
    let invalid_address = "invalid_address";
    let token_mint = TokenMint::new(invalid_address);

    let result = token_mint.to_pubkey();
    assert!(result.is_err());
    match result.unwrap_err() {
        MintError::ConversionError(_) => {} // Expected
        _ => panic!("Expected ConversionError"),
    }
}

#[test]
fn test_token_mint_from_pubkey() {
    let pubkey = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let token_mint = TokenMint::from(pubkey);

    assert_eq!(token_mint.address, pubkey.to_string());
    assert_eq!(token_mint.symbol, None);
    assert_eq!(token_mint.decimals, None);
    assert_eq!(token_mint.price_usd, None);
}

#[test]
fn test_token_mint_from_str() {
    let address = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let token_mint = TokenMint::from_str(address).unwrap();

    assert_eq!(token_mint.address, address);
    assert_eq!(token_mint.symbol, None);
    assert_eq!(token_mint.decimals, None);
    assert_eq!(token_mint.price_usd, None);
}

#[test]
fn test_token_mint_from_str_invalid() {
    let invalid_address = "invalid_address";
    let result = TokenMint::from_str(invalid_address);

    assert!(result.is_err());
    match result.unwrap_err() {
        MintError::InvalidAddress(_) => {} // Expected
        _ => panic!("Expected InvalidAddress"),
    }
}

#[test]
fn test_token_mint_try_from_string() {
    let address = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string();
    let token_mint = TokenMint::try_from(address).unwrap();

    assert_eq!(
        token_mint.address,
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    );
}

#[test]
fn test_token_mint_is_sol() {
    let sol_mint = mints::sol();
    assert!(sol_mint.is_sol());

    let usdc_mint = mints::usdc();
    assert!(!usdc_mint.is_sol());
}

#[test]
fn test_token_mint_is_usdc() {
    let usdc_mint = mints::usdc();
    assert!(usdc_mint.is_usdc());

    let sol_mint = mints::sol();
    assert!(!sol_mint.is_usdc());
}

#[test]
fn test_token_mint_is_usdt() {
    let usdt_mint = mints::usdt();
    assert!(usdt_mint.is_usdt());

    let sol_mint = mints::sol();
    assert!(!sol_mint.is_usdt());
}

#[test]
fn test_mints_constants() {
    assert_eq!(mints::SOL, "So11111111111111111111111111111111111111112");
    assert_eq!(mints::USDC, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    assert_eq!(mints::USDT, "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
    assert_eq!(mints::RAY, "4k3Dyjzvz8SLJJ5aB6qQmYCzpcLQGQdip8JFkQkqP5Cv");

    // Test the getter functions
    let sol_mint = mints::sol();
    assert_eq!(sol_mint.address, mints::SOL);
    assert_eq!(sol_mint.symbol, Some("SOL".to_string()));

    let usdc_mint = mints::usdc();
    assert_eq!(usdc_mint.address, mints::USDC);
    assert_eq!(usdc_mint.symbol, Some("USDC".to_string()));

    let usdt_mint = mints::usdt();
    assert_eq!(usdt_mint.address, mints::USDT);
    assert_eq!(usdt_mint.symbol, Some("USDT".to_string()));

    let ray_mint = mints::ray();
    assert_eq!(ray_mint.address, mints::RAY);
    assert_eq!(ray_mint.symbol, Some("RAY".to_string()));
}

#[test]
fn test_resolve_mint_or_default() {
    let mut key_map = HashMap::new();
    key_map.insert("USER_TOKEN".to_string(), mints::USDC.to_string());

    // Test with a valid mint address
    let result = helpers::resolve_mint_or_default(mints::USDC, &key_map);
    assert_eq!(result, mints::USDC);

    // Test with a key that needs to be resolved
    let result = helpers::resolve_mint_or_default("USER_TOKEN", &key_map);
    assert_eq!(result, mints::USDC);

    // Test with an invalid mint address (should default to SOL)
    let result = helpers::resolve_mint_or_default("invalid_address", &key_map);
    assert_eq!(result, mints::SOL);

    // Test with a key that doesn't exist (should default to SOL)
    let result = helpers::resolve_mint_or_default("NONEXISTENT_KEY", &key_map);
    assert_eq!(result, mints::SOL);
}

#[test]
fn test_get_mint_symbol() {
    assert_eq!(helpers::get_mint_symbol(mints::SOL), "SOL");
    assert_eq!(helpers::get_mint_symbol(mints::USDC), "USDC");
    assert_eq!(helpers::get_mint_symbol(mints::USDT), "USDT");
    assert_eq!(helpers::get_mint_symbol(mints::RAY), "RAY");
    assert_eq!(helpers::get_mint_symbol("unknown_address"), "Unknown");
}

#[test]
fn test_token_mint_serialization() {
    let token_mint =
        TokenMint::with_metadata(mints::USDC, Some("USDC".to_string()), Some(6), Some(1.0));

    // Test serialization to JSON
    let json = serde_json::to_value(&token_mint).unwrap();
    assert_eq!(json["address"], mints::USDC);
    assert_eq!(json["symbol"], "USDC");
    assert_eq!(json["decimals"], 6);
    assert_eq!(json["price_usd"], 1.0);

    // Test deserialization from JSON
    let deserialized: TokenMint = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.address, mints::USDC);
    assert_eq!(deserialized.symbol, Some("USDC".to_string()));
    assert_eq!(deserialized.decimals, Some(6));
    assert_eq!(deserialized.price_usd, Some(1.0));
}

#[test]
fn test_token_mint_partial_metadata() {
    // Test with only symbol
    let token_mint = TokenMint::with_symbol(mints::USDT, "USDT");
    assert_eq!(token_mint.address, mints::USDT);
    assert_eq!(token_mint.symbol, Some("USDT".to_string()));
    assert_eq!(token_mint.decimals, None);
    assert_eq!(token_mint.price_usd, None);

    // Test with only decimals
    let token_mint = TokenMint::with_metadata(mints::RAY, None, Some(9), None);
    assert_eq!(token_mint.address, mints::RAY);
    assert_eq!(token_mint.symbol, None);
    assert_eq!(token_mint.decimals, Some(9));
    assert_eq!(token_mint.price_usd, None);

    // Test with only price
    let token_mint = TokenMint::with_metadata(mints::SOL, None, None, Some(150.0));
    assert_eq!(token_mint.address, mints::SOL);
    assert_eq!(token_mint.symbol, None);
    assert_eq!(token_mint.decimals, None);
    assert_eq!(token_mint.price_usd, Some(150.0));
}

#[test]
fn test_token_mint_clone() {
    let original =
        TokenMint::with_metadata(mints::USDC, Some("USDC".to_string()), Some(6), Some(1.0));

    let mut cloned = original.clone();
    assert_eq!(original.address, cloned.address);
    assert_eq!(original.symbol, cloned.symbol);
    assert_eq!(original.decimals, cloned.decimals);
    assert_eq!(original.price_usd, cloned.price_usd);

    // Verify they are distinct instances
    cloned.symbol = Some("Modified".to_string());
    assert_ne!(original.symbol, cloned.symbol);
}

#[test]
fn test_token_mint_debug_format() {
    let token_mint =
        TokenMint::with_metadata(mints::USDC, Some("USDC".to_string()), Some(6), Some(1.0));

    let debug_str = format!("{token_mint:?}");
    assert!(debug_str.contains("TokenMint"));
    assert!(debug_str.contains("address"));
    assert!(debug_str.contains("symbol"));
    assert!(debug_str.contains("decimals"));
    assert!(debug_str.contains("price_usd"));
}
