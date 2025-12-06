//! Tests for AvailableTokens struct

use reev_core::execution::context_builder::AvailableTokens;

#[test]
fn test_available_tokens_new() {
    let tokens = AvailableTokens::new();
    assert!(
        tokens.tokens.is_empty(),
        "New AvailableTokens should be empty"
    );
}

#[test]
fn test_available_tokens_add_token() {
    let tokens = AvailableTokens::new()
        .add_token(
            "So11111111111111111111111111111111111111112".to_string(),
            1000000,
        )
        .add_token(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            2000000,
        );

    assert_eq!(tokens.tokens.len(), 2, "Should have 2 tokens");

    assert_eq!(
        tokens.get_amount("So11111111111111111111111111111111111111112"),
        Some(1000000),
        "Should return correct SOL amount"
    );

    assert_eq!(
        tokens.get_amount("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        Some(2000000),
        "Should return correct USDC amount"
    );

    assert_eq!(
        tokens.get_amount("NonExistentToken"),
        None,
        "Should return None for non-existent token"
    );
}

#[test]
fn test_available_tokens_to_hashmap() {
    let tokens = AvailableTokens::new()
        .add_token(
            "So11111111111111111111111111111111111111112".to_string(),
            1000000,
        )
        .add_token(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            2000000,
        );

    let hashmap = tokens.to_hashmap();
    assert_eq!(hashmap.len(), 2, "Hashmap should have 2 entries");

    assert_eq!(
        hashmap.get("So11111111111111111111111111111111111111112"),
        Some(&1000000),
        "Hashmap should contain correct SOL amount"
    );

    assert_eq!(
        hashmap.get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        Some(&2000000),
        "Hashmap should contain correct USDC amount"
    );
}

#[test]
fn test_available_tokens_serialization() {
    let tokens = AvailableTokens::new()
        .add_token(
            "So11111111111111111111111111111111111111112".to_string(),
            1000000,
        )
        .add_token(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            2000000,
        );

    // Test serialization to JSON
    let json_str = serde_json::to_string(&tokens).expect("Failed to serialize");

    // Test deserialization from JSON
    let deserialized: AvailableTokens =
        serde_json::from_str(&json_str).expect("Failed to deserialize");

    // Verify the deserialized object has the same data
    assert_eq!(
        deserialized.get_amount("So11111111111111111111111111111111111111112"),
        Some(1000000),
        "Deserialized object should have correct SOL amount"
    );

    assert_eq!(
        deserialized.get_amount("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        Some(2000000),
        "Deserialized object should have correct USDC amount"
    );
}

#[test]
fn test_available_tokens_default() {
    let tokens: AvailableTokens = Default::default();
    assert!(
        tokens.tokens.is_empty(),
        "Default AvailableTokens should be empty"
    );
}

#[test]
fn test_available_tokens_chaining() {
    let tokens = AvailableTokens::default()
        .add_token("Token1".to_string(), 100)
        .add_token("Token2".to_string(), 200)
        .add_token("Token3".to_string(), 300);

    assert_eq!(tokens.tokens.len(), 3, "Should have 3 tokens");
    assert_eq!(tokens.get_amount("Token1"), Some(100));
    assert_eq!(tokens.get_amount("Token2"), Some(200));
    assert_eq!(tokens.get_amount("Token3"), Some(300));
}
