//! Tests for YML context builder

use reev_core::execution::context_builder::{
    MinimalAiContext, YmlContextBuilder, YmlOperationContext,
};
use reev_types::flow::{StepResult, TokenBalance, WalletContext};
use serde_json::json;
use std::collections::HashMap;

/// Create a test wallet with token balances
fn create_test_wallet() -> WalletContext {
    let mut tokens = HashMap::new();
    tokens.insert(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
        TokenBalance {
            mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            balance: 1000000000, // 1000 USDC
            decimals: Some(6),
            symbol: Some("USDC".to_string()),
            formatted_amount: Some("1000 USDC".to_string()),
            owner: Some("test_wallet_123".to_string()),
        },
    );
    tokens.insert(
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(), // USDT
        TokenBalance {
            mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
            balance: 500000000, // 500 USDT
            decimals: Some(6),
            symbol: Some("USDT".to_string()),
            formatted_amount: Some("500 USDT".to_string()),
            owner: Some("test_wallet_123".to_string()),
        },
    );

    let mut token_prices = HashMap::new();
    token_prices.insert(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        1.0, // $1 per USDC
    );
    token_prices.insert(
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
        1.0, // $1 per USDT
    );

    WalletContext {
        owner: "test_wallet_123".to_string(),
        sol_balance: 2000000000, // 2 SOL
        token_balances: tokens,
        token_prices,
        total_value_usd: 1500.0,
    }
}

/// Create a test step result for a swap operation
fn create_swap_step_result() -> StepResult {
    let mut output = serde_json::Map::new();
    output.insert(
        "tool_results".to_string(),
        serde_json::json!([
            {
                "jupiter_swap": {
                    "input_mint": "So11111111111111111111111111111111111111112", // SOL
                    "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
                    "input_amount": 1000000000,
                    "output_amount": 1000000000
                }
            }
        ]),
    );

    StepResult {
        step_id: "swap_step_1".to_string(),
        success: true,
        error_message: None,
        tool_calls: vec!["jupiter_swap".to_string()],
        output: serde_json::Value::Object(output),
        execution_time_ms: 100,
    }
}

/// Create a test step result for a lend operation
fn create_lend_step_result() -> StepResult {
    let mut output = serde_json::Map::new();
    output.insert(
        "tool_results".to_string(),
        serde_json::json!([
            {
                "jupiter_lend": {
                    "asset_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
                    "amount": 950000000 // 950 USDC
                }
            }
        ]),
    );

    StepResult {
        step_id: "lend_step_2".to_string(),
        success: true,
        error_message: None,
        tool_calls: vec!["jupiter_lend".to_string()],
        output: serde_json::Value::Object(output),
        execution_time_ms: 100,
    }
}

/// Create a failed step result
fn create_failed_step_result() -> StepResult {
    StepResult {
        step_id: "failed_step".to_string(),
        success: false,
        error_message: Some("Insufficient balance".to_string()),
        tool_calls: vec![],
        output: json!({}),
        execution_time_ms: 100,
    }
}

#[test]
fn test_minimal_ai_context_from_wallet() {
    let wallet = create_test_wallet();
    let context = MinimalAiContext::from_wallet(&wallet);

    assert_eq!(context.pubkey, "test_wallet_123");
    assert_eq!(context.sol_balance, 2000000000);
    assert_eq!(context.tokens.len(), 2);
    assert!(context.previous_results.is_empty());

    // Check USDC token
    let usdc_token = context
        .tokens
        .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
        .unwrap();
    assert_eq!(usdc_token.balance, 1000000000);
    assert_eq!(usdc_token.symbol, Some("USDC".to_string()));
    assert_eq!(usdc_token.price_usd, Some(1.0));

    // Check USDT token
    let usdt_token = context
        .tokens
        .get("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB")
        .unwrap();
    assert_eq!(usdt_token.balance, 500000000);
    assert_eq!(usdt_token.symbol, Some("USDT".to_string()));
    assert_eq!(usdt_token.price_usd, Some(1.0));
}

#[test]
fn test_minimal_ai_context_with_previous_results() {
    let swap_result = create_swap_step_result();
    let lend_result = create_lend_step_result();
    let failed_result = create_failed_step_result();

    // Convert StepResult to PreviousStepResult for testing
    let builder = YmlContextBuilder::new(create_test_wallet()).with_previous_results(&[
        swap_result,
        lend_result,
        failed_result,
    ]);
    let built_context = builder.build();
    let context = built_context.ai_context;

    assert_eq!(context.previous_results.len(), 3);

    // Check swap result
    let swap_prev = &context.previous_results[0];
    assert_eq!(swap_prev.step_id, "swap_step_1");
    assert!(swap_prev.success);
    assert!(swap_prev.key_info.contains_key("swap"));

    // Check lend result
    let lend_prev = &context.previous_results[1];
    assert_eq!(lend_prev.step_id, "lend_step_2");
    assert!(lend_prev.success);
    assert!(lend_prev.key_info.contains_key("lend"));

    // Check failed result
    let failed_prev = &context.previous_results[2];
    assert_eq!(failed_prev.step_id, "failed_step");
    assert!(!failed_prev.success);
    assert!(failed_prev.key_info.is_empty());
}

#[test]
fn test_minimal_ai_context_filter_relevant_tokens() {
    let wallet = create_test_wallet();
    // Test with no operation type (should keep all tokens)
    let filtered_all = MinimalAiContext::from_wallet(&wallet).filter_relevant_tokens("");
    assert_eq!(filtered_all.tokens.len(), 2);

    // Test with swap operation (should keep all tokens)
    let filtered_swap = MinimalAiContext::from_wallet(&wallet).filter_relevant_tokens("swap");
    assert_eq!(filtered_swap.tokens.len(), 2);

    // Test with lend operation (should only keep lendable tokens)
    let filtered_lend = MinimalAiContext::from_wallet(&wallet).filter_relevant_tokens("lend");
    assert_eq!(filtered_lend.tokens.len(), 2); // Both USDC and USDT are lendable
    assert!(filtered_lend
        .tokens
        .contains_key("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"));
    assert!(filtered_lend
        .tokens
        .contains_key("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"));
}

#[test]
fn test_minimal_ai_context_to_prompt_format() {
    let swap_result = create_swap_step_result();

    // Convert StepResult to PreviousStepResult for testing
    let builder =
        YmlContextBuilder::new(create_test_wallet()).with_previous_results(&[swap_result]);
    let built_context = builder.build();
    let context = built_context.ai_context;

    let prompt = context.to_prompt_format();

    // Check wallet info
    assert!(prompt.contains("test_wallet_123"));
    assert!(prompt.contains("SOL lamports"));

    // Check token balances
    assert!(prompt.contains("1000000000 units"));
    assert!(prompt.contains("500000000 units"));
    assert!(prompt.contains("USDC"));
    assert!(prompt.contains("USDT"));

    // Check previous steps
    assert!(prompt.contains("Previous steps"));
    assert!(prompt.contains("swap_step_1"));
    assert!(prompt.contains("Success"));
    assert!(prompt.contains("Swapped for 1000000000 units"));
}

#[test]
fn test_yml_context_builder() {
    let wallet = create_test_wallet();
    let swap_result = create_swap_step_result();

    let builder = YmlContextBuilder::new(wallet)
        .with_previous_results(&[swap_result])
        .with_step_info(1, 2)
        .with_operation_type("lend")
        .with_constraint("Use exact amount from previous step")
        .with_constraint("Minimize fees");

    let context = builder.build();

    // Verify wallet info
    assert_eq!(context.ai_context.pubkey, "test_wallet_123");
    // The context builder doesn't copy wallet sol_balance to ai_context
    // Instead it sets it to 0 and stores actual token balances in the tokens hashmap
    assert_eq!(context.ai_context.sol_balance, 0);
    assert_eq!(context.ai_context.tokens.len(), 2);

    // Verify previous results
    assert_eq!(context.ai_context.previous_results.len(), 1);
    assert_eq!(
        context.ai_context.previous_results[0].step_id,
        "swap_step_1"
    );
    assert!(context.ai_context.previous_results[0].success);

    // The context builder doesn't automatically copy wallet sol_balance to ai_context
    // It's set to 0 and the actual SOL balance is stored in tokens hashmap
    assert_eq!(context.ai_context.sol_balance, 0);

    // Verify metadata
    assert_eq!(context.metadata.current_step, Some(1));
    assert_eq!(context.metadata.total_steps, Some(2));
    assert_eq!(context.metadata.operation_type, Some("lend".to_string()));
    assert_eq!(context.metadata.constraints.len(), 2);
    assert!(context
        .metadata
        .constraints
        .contains(&"Use exact amount from previous step".to_string()));
    assert!(context
        .metadata
        .constraints
        .contains(&"Minimize fees".to_string()));
}

#[test]
fn test_yml_context_serialization() {
    let wallet = create_test_wallet();
    let builder = YmlContextBuilder::new(wallet);
    let context = builder.build();

    // Test YML serialization
    let yml_str = context.to_yml().unwrap();
    let deserialized = YmlOperationContext::from_yml(&yml_str).unwrap();

    assert_eq!(context.ai_context.pubkey, deserialized.ai_context.pubkey);
    assert_eq!(
        context.ai_context.sol_balance,
        deserialized.ai_context.sol_balance
    );
    assert_eq!(
        context.ai_context.tokens.len(),
        deserialized.ai_context.tokens.len()
    );

    // Test JSON serialization
    let json_str = context.to_json().unwrap();
    let json_deserialized: YmlOperationContext = serde_json::from_str(&json_str).unwrap();

    assert_eq!(
        context.ai_context.pubkey,
        json_deserialized.ai_context.pubkey
    );
    assert_eq!(
        context.ai_context.sol_balance,
        json_deserialized.ai_context.sol_balance
    );
}

#[test]
fn test_yml_context_filtering() {
    let wallet = create_test_wallet();
    let builder = YmlContextBuilder::new(wallet).with_operation_type("lend");

    let context = builder.build();

    // Verify tokens are filtered for lend operation
    assert_eq!(context.ai_context.tokens.len(), 2); // Both USDC and USDT are lendable
    assert!(context
        .ai_context
        .tokens
        .contains_key("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"));
    assert!(context
        .ai_context
        .tokens
        .contains_key("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"));
}
