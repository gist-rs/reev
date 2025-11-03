//! Test for token price helper functionality
//!
//! This test reproduces Issue #7: Template Token Price Helper Not Working
//! and validates the fix.

use reev_orchestrator::templates::{TemplateEngine, TemplateMetadata, TemplateType};
use reev_types::flow::WalletContext;
use std::collections::HashMap;
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_token_price_helper_with_nested_context() {
    // Create temporary directory for templates
    let temp_dir = TempDir::new().unwrap();
    let templates_dir = temp_dir.path();

    // Create base directory
    let base_dir = templates_dir.join("base");
    fs::create_dir_all(&base_dir).await.unwrap();

    // Create test template that uses get_token_price helper
    let template_content = r#"
Token Price Test
Wallet: {{wallet.owner}}
SOL Price: ${{get_token_price "So11111111111111111111111111111111111112"}}
USDC Price: ${{get_token_price "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"}}
Unknown Price: ${{get_token_price "UnknownMint"}}
"#;

    let template_path = base_dir.join("test_prices.hbs");
    fs::write(&template_path, template_content).await.unwrap();

    // Create template engine
    let engine = TemplateEngine::new(templates_dir).unwrap();

    // Register template
    let metadata = TemplateMetadata::new(
        "test_prices".to_string(),
        TemplateType::Base,
        "Test token price helper".to_string(),
        vec![],
        vec![],
    );

    let registration = engine
        .register_template_file(&template_path, metadata)
        .await
        .unwrap();
    assert_eq!(registration.name, "test_prices");

    // Create wallet context with token prices
    let mut wallet_context = WalletContext::new("test_wallet".to_string());
    wallet_context.add_token_price(
        "So11111111111111111111111111111111111112".to_string(),
        150.0,
    );
    wallet_context.add_token_price(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        1.0,
    );

    // Render template
    let variables = HashMap::new();
    let result = engine
        .render_template("test_prices", &wallet_context, &variables)
        .await
        .unwrap();

    println!("Rendered template:\n{}", result.rendered);

    // Verify the rendered content contains correct prices
    assert!(result.rendered.contains("Wallet: test_wallet"));

    // This should pass after fixing the helper
    assert!(result.rendered.contains("SOL Price: $150.000000"));
    assert!(result.rendered.contains("USDC Price: $1.000000"));

    // Unknown token should return 0.0
    assert!(result.rendered.contains("Unknown Price: $0.0"));
}

#[tokio::test]
async fn test_token_balance_helper_with_nested_context() {
    // Create temporary directory for templates
    let temp_dir = TempDir::new().unwrap();
    let templates_dir = temp_dir.path();

    // Create base directory
    let base_dir = templates_dir.join("base");
    fs::create_dir_all(&base_dir).await.unwrap();

    // Create test template that uses get_token_balance helper
    let template_content = r#"
Token Balance Test
Wallet: {{wallet.owner}}
SOL Balance: {{get_token_balance "So11111111111111111111111111111111111112"}}
USDC Balance: {{get_token_balance "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"}}
Unknown Balance: {{get_token_balance "UnknownMint"}}
"#;

    let template_path = base_dir.join("test_balances.hbs");
    fs::write(&template_path, template_content).await.unwrap();

    // Create template engine
    let engine = TemplateEngine::new(templates_dir).unwrap();

    // Register template
    let metadata = TemplateMetadata::new(
        "test_balances".to_string(),
        TemplateType::Base,
        "Test token balance helper".to_string(),
        vec![],
        vec![],
    );

    let registration = engine
        .register_template_file(&template_path, metadata)
        .await
        .unwrap();
    assert_eq!(registration.name, "test_balances");

    // Create wallet context with token balances
    let mut wallet_context = WalletContext::new("test_wallet".to_string());

    // Add token balances using the TokenBalance struct
    use reev_types::benchmark::TokenBalance;

    let sol_balance = TokenBalance::new(
        "So11111111111111111111111111111111111112".to_string(),
        1000000000, // 1 SOL in lamports
    )
    .with_decimals(9)
    .with_symbol("SOL".to_string())
    .with_owner("test_wallet".to_string());

    let usdc_balance = TokenBalance::new(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        1000000, // 1 USDC (6 decimals)
    )
    .with_decimals(6)
    .with_symbol("USDC".to_string())
    .with_owner("test_wallet".to_string());

    wallet_context.add_token_balance(
        "So11111111111111111111111111111111111112".to_string(),
        sol_balance,
    );
    wallet_context.add_token_balance(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        usdc_balance,
    );

    // Render template
    let variables = HashMap::new();
    let result = engine
        .render_template("test_balances", &wallet_context, &variables)
        .await
        .unwrap();

    println!("Rendered template:\n{}", result.rendered);

    // Verify the rendered content contains correct balances
    assert!(result.rendered.contains("Wallet: test_wallet"));

    // This should pass after fixing the helper
    assert!(result.rendered.contains("SOL Balance: 1000000000"));
    assert!(result.rendered.contains("USDC Balance: 1000000"));

    // Unknown token should return 0
    assert!(result.rendered.contains("Unknown Balance: 0"));
}
