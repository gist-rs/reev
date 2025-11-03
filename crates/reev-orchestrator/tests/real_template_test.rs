//! Integration test for real template usage with token price helpers
//!
//! This test validates that the fix works with actual template files
//! from the templates directory.

use reev_orchestrator::templates::TemplateRenderer;
use reev_types::flow::WalletContext;
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_real_swap_template_with_prices() {
    // Create template renderer with real templates
    let renderer = TemplateRenderer::new("../../templates").unwrap();

    // Initialize and register all templates
    renderer.initialize().await.unwrap();

    // Create wallet context with token prices
    let mut wallet_context = WalletContext::new("test_user_wallet".to_string());
    wallet_context.add_token_price(
        "So11111111111111111111111111111111111112".to_string(),
        150.42,
    );
    wallet_context.add_token_price(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        1.00,
    );

    // Render swap template
    let result = renderer
        .render_swap(
            &wallet_context,
            10.0,
            "So11111111111111111111111111111111111112",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            None,
        )
        .await
        .unwrap();

    println!("Real swap template result:\n{}", result.rendered);

    // Verify the rendered content contains correct prices
    assert!(result.rendered.contains("test_user_wallet"));

    // These should now work with the fix
    assert!(result.rendered.contains("$150.420000"));
    assert!(result.rendered.contains("$1.000000"));

    // Should not contain $0.0 prices
    assert!(!result.rendered.contains("$0.0"));
}

#[tokio::test]
async fn test_real_lend_template_with_prices() {
    // Create template renderer with real templates
    let renderer = TemplateRenderer::new("../../templates").unwrap();

    // Initialize and register all templates
    renderer.initialize().await.unwrap();

    // Create wallet context with token prices
    let mut wallet_context = WalletContext::new("lender_wallet".to_string());
    wallet_context.add_token_price(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        1.00,
    );

    // Render lend template
    let result = renderer
        .render_lend(
            &wallet_context,
            1000.0,
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            None,
            Some(5.5),
        )
        .await
        .unwrap();

    println!("Real lend template result:\n{}", result.rendered);

    // Verify the rendered content contains correct prices
    assert!(result.rendered.contains("lender_wallet"));

    // Should contain USDC price
    assert!(result.rendered.contains("$1.000000"));

    // Should not contain $0.0 prices
    assert!(!result.rendered.contains("$0.0"));
}

#[tokio::test]
async fn test_jupiter_swap_template_with_prices() {
    // Create template renderer with real templates
    let renderer = TemplateRenderer::new("../../templates").unwrap();

    // Initialize and register all templates
    renderer.initialize().await.unwrap();

    // Create wallet context with token prices
    let mut wallet_context = WalletContext::new("jupiter_user".to_string());
    wallet_context.add_token_price(
        "So11111111111111111111111111111111111112".to_string(),
        156.78,
    );
    wallet_context.add_token_price(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        1.00,
    );

    // Render Jupiter swap template
    let result = renderer
        .render_swap(
            &wallet_context,
            5.0,
            "So11111111111111111111111111111111111112",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            Some("jupiter"),
        )
        .await
        .unwrap();

    println!("Jupiter swap template result:\n{}", result.rendered);

    // Verify the rendered content contains correct prices
    assert!(result.rendered.contains("jupiter_user"));

    // Should contain correct SOL price
    assert!(result.rendered.contains("$156.780000"));
    assert!(result.rendered.contains("$1.000000"));

    // Should not contain $0.0 prices
    assert!(!result.rendered.contains("$0.0"));
}

#[tokio::test]
async fn test_custom_template_with_token_helpers() {
    // Create template renderer with real templates
    let renderer = TemplateRenderer::new("../../templates").unwrap();

    // Initialize and register all templates
    renderer.initialize().await.unwrap();

    // Create wallet context with token prices and balances
    let mut wallet_context = WalletContext::new("advanced_user".to_string());
    wallet_context.add_token_price(
        "So11111111111111111111111111111111111112".to_string(),
        145.32,
    );
    wallet_context.add_token_price(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        1.00,
    );

    use reev_types::benchmark::TokenBalance;

    let sol_balance = TokenBalance::new(
        "So11111111111111111111111111111111111112".to_string(),
        2000000000, // 2 SOL in lamports
    )
    .with_decimals(9)
    .with_symbol("SOL".to_string())
    .with_owner("advanced_user".to_string());

    wallet_context.add_token_balance(
        "So11111111111111111111111111111111111112".to_string(),
        sol_balance,
    );

    // Render custom template
    let mut variables = HashMap::new();
    variables.insert("amount".to_string(), json!(0.5));
    variables.insert(
        "from_token".to_string(),
        json!("So11111111111111111111111111111111111112"),
    );
    variables.insert(
        "to_token".to_string(),
        json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    );

    let result = renderer
        .render_custom("swap", &wallet_context, &variables)
        .await
        .unwrap();

    println!("Custom template result:\n{}", result.rendered);

    // Verify the rendered content contains correct prices
    assert!(result.rendered.contains("advanced_user"));

    // Should contain correct prices and balances
    assert!(result.rendered.contains("$145.320000"));
    assert!(result.rendered.contains("$1.000000"));

    // Should not contain $0.0 prices
    assert!(!result.rendered.contains("$0.0"));
}
