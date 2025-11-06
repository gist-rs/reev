//! Unit tests for reev-orchestrator components
//!
//! This file contains all unit tests that were previously scattered across
//! source files, now properly organized in the tests folder as per project rules.

use reev_orchestrator::{ContextResolver, OrchestratorGateway, YmlGenerator};
use reev_types::flow::{DynamicStep, WalletContext};
use reev_types::tools::ToolName;

#[tokio::test]
async fn test_context_resolver_creation() {
    let resolver = ContextResolver::new();
    assert_eq!(resolver.get_cache_stats().await, (0, 0));
}

#[tokio::test]
async fn test_wallet_context_resolution() {
    let resolver = ContextResolver::new();
    let pubkey = "test_pubkey";

    let context = resolver.resolve_wallet_context(pubkey).await.unwrap();
    assert_eq!(context.owner, pubkey);
    // Context should be created successfully even with zero balance (mock data)
    // The real wallet query is temporarily disabled, so balance will be 0
}

#[tokio::test]
async fn test_token_price() {
    let resolver = ContextResolver::new();
    let sol_mint = "So11111111111111111111111111111111111111112";
    println!("DEBUG: Token mint bytes: {:?}", sol_mint.as_bytes());
    println!(
        "DEBUG: Expected bytes: {:?}",
        "So11111111111111111111111111111111111111112".as_bytes()
    );

    // Clear cache to ensure fresh price
    resolver.clear_caches().await;

    let price = resolver.get_token_price(sol_mint).await.unwrap();
    assert_eq!(price, 150.0);
}

#[tokio::test]
async fn test_cache_functionality() {
    let resolver = ContextResolver::new();
    let pubkey = "test_cache";

    // First call - should resolve fresh
    let start = std::time::Instant::now();
    let _context1 = resolver.resolve_wallet_context(pubkey).await.unwrap();
    let _first_call_time = start.elapsed();

    // Second call - should use cache
    let start = std::time::Instant::now();
    let _context2 = resolver.resolve_wallet_context(pubkey).await.unwrap();
    let _second_call_time = start.elapsed();

    // Cache should be faster (though this is not guaranteed in tests)
    assert!(resolver.get_cache_stats().await.0 > 0);

    // Clear caches
    resolver.clear_caches().await;
    assert_eq!(resolver.get_cache_stats().await, (0, 0));
}

#[tokio::test]
async fn test_gateway_creation() {
    let gateway = OrchestratorGateway::new().await.unwrap();
    gateway.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_prompt_refinement() {
    let gateway = OrchestratorGateway::new().await.unwrap();
    let mut context = WalletContext::new("test".to_string());
    context.sol_balance = 2_000_000_000; // 2 SOL
    context.total_value_usd = 300.0;

    let refined = gateway.refine_prompt("use my 50% sol", &context);
    // Should include wallet context information
    assert!(refined.contains("2.000000 SOL"));
    assert!(refined.contains("$300.00"));
    assert!(refined.contains("use my 50% sol"));
}

#[tokio::test]
async fn test_swap_flow_generation() {
    let gateway = OrchestratorGateway::new().await.unwrap();
    let context = WalletContext::new("test".to_string());

    // Check if flow fails due to insufficient SOL balance (0 SOL in context)
    let flow = gateway
        .generate_enhanced_flow_plan("swap SOL to USDC using Jupiter", &context, None)
        .await;
    // Flow should now succeed even with 0 SOL balance (new permissive behavior)
    assert!(flow.is_ok());
    let flow_plan = flow.unwrap();

    // Should still generate a proper 3-step flow structure
    assert_eq!(flow_plan.steps.len(), 3);
    assert_eq!(flow_plan.steps[0].step_id, "balance_check");
    assert!(flow_plan.steps[0]
        .required_tools
        .contains(&reev_types::tools::ToolName::GetAccountBalance));
}

#[tokio::test]
async fn test_swap_lend_flow_generation() {
    let gateway = OrchestratorGateway::new().await.unwrap();
    let mut context = WalletContext::new("test".to_string());
    // Give the context some SOL balance to enable the complex flow
    context.sol_balance = 2_000_000_000; // 2 SOL
    context.total_value_usd = 300.0;

    // Flow should succeed with sufficient balance
    let flow = gateway
        .generate_enhanced_flow_plan("swap SOL to USDC then lend using Jupiter", &context, None)
        .await;
    assert!(flow.is_ok());
    let flow = flow.unwrap();
    // Should generate 3 steps for comprehensive flow (balance_check + swap + positions_check)
    assert_eq!(flow.steps.len(), 3);
}

#[test]
fn test_yml_generator_creation() {
    let generator = YmlGenerator::new();
    assert_eq!(
        generator.template_dir,
        std::path::PathBuf::from("templates")
    );
}

#[test]
fn test_yml_generation() {
    let generator = YmlGenerator::new();
    let context = WalletContext::new("test_wallet".to_string());

    let step = DynamicStep::new(
        "swap_1".to_string(),
        "Swap 1 SOL to USDC".to_string(),
        "Swap SOL to USDC".to_string(),
    );

    let flow_plan = reev_types::flow::DynamicFlowPlan::new(
        "test_flow".to_string(),
        "swap SOL to USDC".to_string(),
        context,
    )
    .with_step(step);

    // Test YML content generation without file
    let yml_content = generator.generate_yml_content(&flow_plan).unwrap();
    assert!(!yml_content.is_empty());
    assert!(yml_content.contains("dynamic-test_flow"));

    // Test that generated content is valid YAML
    let parsed: serde_yaml::Value = serde_yaml::from_str(&yml_content).unwrap();
    assert!(parsed.is_mapping());
}

#[test]
fn test_system_prompt_generation() {
    let generator = YmlGenerator::new();
    let mut context = WalletContext::new("test".to_string());
    context.sol_balance = 2_000_000_000; // 2 SOL
    context.total_value_usd = 300.0;

    let prompt = generator.generate_system_prompt(&context);
    assert!(prompt.contains("2.00 SOL"));
    assert!(prompt.contains("$300.00"));
    assert!(prompt.contains("test"));
}

#[test]
fn test_tools_config_generation() {
    let generator = YmlGenerator::new();
    let steps = vec![
        DynamicStep::new("1".to_string(), "test".to_string(), "test".to_string())
            .with_tool(ToolName::SolTransfer),
        DynamicStep::new("2".to_string(), "test".to_string(), "test".to_string())
            .with_tool(ToolName::GetJupiterLendEarnPosition),
    ];

    let tools = generator.generate_tools_config(&steps);
    assert_eq!(tools.len(), 2);
}

#[test]
fn test_steps_config_generation() {
    let generator = YmlGenerator::new();
    let steps = vec![DynamicStep::new(
        "step_1".to_string(),
        "prompt".to_string(),
        "desc".to_string(),
    )
    .with_critical(true)];

    let steps_config = generator.generate_steps_config(&steps);
    assert_eq!(steps_config.len(), 1);
}

#[test]
fn test_dynamic_step_creation() {
    let step = DynamicStep::new(
        "test_step".to_string(),
        "Test prompt template".to_string(),
        "Test description".to_string(),
    )
    .with_critical(false)
    .with_tool(ToolName::GetAccountBalance)
    .with_estimated_time(60);

    assert_eq!(step.step_id, "test_step");
    assert_eq!(step.prompt_template, "Test prompt template");
    assert_eq!(step.description, "Test description");
    assert!(!step.critical);
    assert!(step
        .required_tools
        .contains(&reev_types::tools::ToolName::GetAccountBalance));
    assert_eq!(step.estimated_time_seconds, 60);
}

#[test]
fn test_wallet_context_calculation() {
    let mut context = WalletContext::new("test".to_string());
    context.sol_balance = 2_000_000_000; // 2 SOL
    context.add_token_price(
        "So11111111111111111111111111111111111111111112".to_string(),
        150.0,
    );

    context.calculate_total_value();
    assert_eq!(context.total_value_usd, 300.0); // 2 SOL * $150
}
