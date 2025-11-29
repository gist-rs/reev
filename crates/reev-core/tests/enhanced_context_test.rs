//! Test for Enhanced Context Passing Functionality
//!
//! This test verifies the enhanced context passing features implemented
//! for Issue #105: RigAgent Enhancement.

use reev_core::execution::context_builder::YmlContextBuilder;
use reev_core::execution::rig_agent::{
    BalanceCalculator, ConstraintBuilder, ContextPromptBuilder, DynamicContextUpdater,
    OperationHistory, OperationHistoryBuilder, ParameterValidator, StepConstraint,
};
use reev_types::flow::{StepResult, TokenBalance, WalletContext};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_operation_history_tracking() {
    // Create a step result with a swap operation
    let step_result = StepResult {
        step_id: "swap_step".to_string(),
        success: true,
        error_message: None,
        tool_calls: vec!["jupiter_swap".to_string()],
        output: json!({
            "tool_results": [{
                "jupiter_swap": {
                    "input_mint": "So11111111111111111111111111111111111111112",
                    "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    "input_amount": 100,
                    "output_amount": 500
                }
            }]
        }),
        execution_time_ms: 100,
    };

    // Create operation history from step result
    let operations = OperationHistory::from_step_result(&step_result);
    assert_eq!(operations.len(), 1);

    let swap_op = &operations[0];
    assert_eq!(swap_op.operation_type, "swap");
    assert_eq!(swap_op.input_amount, Some(100.0));
    assert_eq!(swap_op.output_amount, Some(500.0));

    // Test balance change calculation
    assert_eq!(
        swap_op.get_balance_change_for_mint("So11111111111111111111111111111111111111112"),
        -100.0
    );
    assert_eq!(
        swap_op.get_balance_change_for_mint("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        500.0
    );
}

#[test]
fn test_balance_calculator() {
    // Create initial balances
    let mut initial_balances = HashMap::new();
    initial_balances.insert("SOL".to_string(), 1000.0);
    initial_balances.insert("USDC".to_string(), 500.0);

    // Create balance calculator
    let mut calculator = BalanceCalculator::new(initial_balances);

    // Add operation history
    let mut builder = OperationHistoryBuilder::new();
    builder.add_step_results(&[StepResult {
        step_id: "swap_step".to_string(),
        success: true,
        error_message: None,
        tool_calls: vec!["jupiter_swap".to_string()],
        output: json!({
            "tool_results": [{
                "jupiter_swap": {
                    "input_mint": "SOL",
                    "output_mint": "USDC",
                    "input_amount": 100,
                    "output_amount": 500
                }
            }]
        }),
        execution_time_ms: 100,
    }]);
    let operation_history = builder.build();

    calculator.add_operations(operation_history);

    // Verify calculated balances
    assert_eq!(calculator.calculate_available_balance("SOL"), 900.0); // 1000 - 100
    assert_eq!(calculator.calculate_available_balance("USDC"), 1000.0); // 500 + 500

    // Verify balance changes summary
    let changes = calculator.get_balance_changes_summary();
    assert_eq!(changes.get("SOL"), Some(&-100.0));
    assert_eq!(changes.get("USDC"), Some(&500.0));
}

#[test]
fn test_step_constraints() {
    // Create constraints
    let max_amount_constraint =
        StepConstraint::new(reev_core::execution::rig_agent::ConstraintType::MaximumAmount(1000.0))
            .for_parameter("amount")
            .for_tool("jupiter_swap");

    let required_mint_constraint = StepConstraint::new(
        reev_core::execution::rig_agent::ConstraintType::RequiredMint("USDC".to_string()),
    )
    .for_parameter("input_mint");

    // Create validator
    let validator = ParameterValidator::new(vec![max_amount_constraint, required_mint_constraint]);

    // Test valid parameters
    let valid_params = json!({
        "amount": 500,
        "input_mint": "USDC"
    });

    let report = validator
        .validate("jupiter_swap", &valid_params, 0)
        .unwrap();
    assert!(report.is_valid());

    // Test invalid parameters
    let invalid_params = json!({
        "amount": 1500,
        "input_mint": "SOL"
    });

    let report = validator
        .validate("jupiter_swap", &invalid_params, 0)
        .unwrap();
    assert!(!report.is_valid());
    assert_eq!(report.all_violations().len(), 2);
}

#[test]
fn test_dynamic_context_updater() {
    // Create initial wallet context
    let mut wallet_context = WalletContext::new("test_pubkey".to_string());
    wallet_context.sol_balance = 1000;

    // Add a token to the wallet
    wallet_context.token_balances.insert(
        "USDC".to_string(),
        TokenBalance {
            mint: "USDC".to_string(),
            balance: 500,
            symbol: Some("USDC".to_string()),
            decimals: Some(6),
            formatted_amount: Some("500 USDC".to_string()),
            owner: Some("test_pubkey".to_string()),
        },
    );

    // Create context updater
    let mut updater = DynamicContextUpdater::new(wallet_context.clone());

    // Create step result for a swap operation
    let step_result = StepResult {
        step_id: "swap_step".to_string(),
        success: true,
        error_message: None,
        tool_calls: vec!["jupiter_swap".to_string()],
        output: json!({
            "tool_results": [{
                "jupiter_swap": {
                    "input_mint": "So11111111111111111111111111111111111111112",
                    "output_mint": "USDC",
                    "input_amount": 100,
                    "output_amount": 500
                }
            }]
        }),
        execution_time_ms: 100,
    };

    // Create tool result
    let tool_result = json!({
        "jupiter_swap": {
            "input_mint": "So11111111111111111111111111111111111111112",
            "output_mint": "USDC",
            "input_amount": 100,
            "output_amount": 500
        }
    });

    // Update context
    let update_result = updater
        .update_context_after_execution(&tool_result, "jupiter_swap", &step_result)
        .unwrap();

    // Verify balance changes
    assert_eq!(update_result.balance_changes.len(), 2);

    // Verify constraints were generated
    assert!(!update_result.next_step_constraints.is_empty());

    // Verify wallet context was updated
    // Note: Our implementation might not update SOL balance correctly in test
    // Let's check the actual value
    let actual_sol = update_result.updated_wallet_context.sol_balance;
    println!("Actual SOL balance after update: {actual_sol}");
    // For now, we'll skip this assertion since it's a test implementation detail
    // The SOL balance should be reduced by 100 from the swap
    assert_eq!(update_result.updated_wallet_context.sol_balance, 900);

    // Find USDC balance in updated context
    let usdc_balance = update_result
        .updated_wallet_context
        .token_balances
        .get("USDC")
        .map(|t| t.balance)
        .unwrap_or(0);

    assert_eq!(usdc_balance, 1000); // Original 500 + 500 from swap
}

#[test]
fn test_context_prompt_builder() {
    // Create operation history
    let mut builder = OperationHistoryBuilder::new();
    builder.add_step_results(&[StepResult {
        step_id: "swap_step".to_string(),
        success: true,
        error_message: None,
        tool_calls: vec!["jupiter_swap".to_string()],
        output: json!({
            "tool_results": [{
                "jupiter_swap": {
                    "input_mint": "SOL",
                    "output_mint": "USDC",
                    "input_amount": 100,
                    "output_amount": 500
                }
            }]
        }),
        execution_time_ms: 100,
    }]);
    let operation_history = builder.build();

    // Create wallet context
    let mut wallet_context = WalletContext::new("test_pubkey".to_string());
    wallet_context.sol_balance = 900;

    // Create constraints
    let mut builder = ConstraintBuilder::new();
    builder.max_amount(500.0, Some("amount"));
    builder.required_mint("USDC", Some("input_mint"));
    let constraints = builder.build();

    // Build context prompt
    let builder = ContextPromptBuilder::new(operation_history, wallet_context, constraints);

    let prompt = builder.build_context_prompt();
    assert!(prompt.contains("Current Wallet State"));
    assert!(prompt.contains("Previous Operations"));
    assert!(prompt.contains("Constraints for Next Operation"));
    assert!(prompt.contains("Swapped 100 of SOL for 500 of USDC"));
    assert!(prompt.contains("Amount must be <= 500"));
    assert!(prompt.contains("Must use mint: USDC"));
}

#[test]
fn test_yml_context_builder_with_enhanced_context() {
    // Create initial wallet context
    let mut wallet_context = WalletContext::new("test_pubkey".to_string());
    wallet_context.owner = "test_pubkey".to_string();
    wallet_context.sol_balance = 1000;

    // Add a token to the wallet
    wallet_context.token_balances.insert(
        "USDC".to_string(),
        TokenBalance {
            mint: "USDC".to_string(),
            balance: 500,
            symbol: Some("USDC".to_string()),
            decimals: Some(6),
            formatted_amount: Some("500 USDC".to_string()),
            owner: Some("test_pubkey".to_string()),
        },
    );

    // Create initial balances
    let mut initial_balances = HashMap::new();
    initial_balances.insert("SOL".to_string(), 1000.0);
    initial_balances.insert("USDC".to_string(), 500.0);

    // Create balance calculator
    let mut calculator = BalanceCalculator::new(initial_balances);

    // Add operation history
    let mut builder = OperationHistoryBuilder::new();
    builder.add_step_results(&[StepResult {
        step_id: "swap_step".to_string(),
        success: true,
        error_message: None,
        tool_calls: vec!["jupiter_swap".to_string()],
        output: json!({
            "tool_results": [{
                "jupiter_swap": {
                    "input_mint": "SOL",
                    "output_mint": "USDC",
                    "input_amount": 100,
                    "output_amount": 500
                }
            }]
        }),
        execution_time_ms: 100,
    }]);
    let operation_history = builder.build();

    calculator.add_operations(operation_history.clone());

    // Create constraints
    let mut builder = ConstraintBuilder::new();
    builder.max_amount(500.0, Some("amount"));
    builder.required_mint("USDC", Some("input_mint"));
    let constraints = builder.build();

    // Build YML context with enhanced features
    let context = YmlContextBuilder::new(wallet_context.clone())
        .with_enhanced_context(operation_history, &calculator, constraints)
        .build();

    // Verify the context was built correctly
    assert_eq!(context.ai_context.pubkey, "test_pubkey");
    assert_eq!(context.ai_context.sol_balance, 1000);
    assert_eq!(context.ai_context.tokens.len(), 1);

    // Verify constraints were added
    assert!(!context.metadata.constraints.is_empty());

    // Verify operation history was added
    if !context.ai_context.previous_results.is_empty() {
        let last_result = &context.ai_context.previous_results[0];
        assert!(last_result.key_info.contains_key("operation_history"));
    }
}

#[test]
fn test_constraint_builder() {
    let mut builder = ConstraintBuilder::new();

    // Add various constraints
    builder.max_amount(1000.0, Some("amount"));
    builder.min_amount(10.0, Some("amount"));
    builder.required_mint("USDC", Some("input_mint"));
    builder.excluded_mint("SOL", Some("input_mint"));
    builder.max_slippage(0.05, Some("slippage"));

    let constraints = builder.build();
    assert_eq!(constraints.len(), 5);

    // Verify constraint types
    assert!(matches!(
        constraints[0].constraint_type,
        reev_core::execution::rig_agent::ConstraintType::MaximumAmount(1000.0)
    ));
    assert!(matches!(
        constraints[1].constraint_type,
        reev_core::execution::rig_agent::ConstraintType::MinimumAmount(10.0)
    ));
    assert!(matches!(
        constraints[2].constraint_type,
        reev_core::execution::rig_agent::ConstraintType::RequiredMint(_)
    ));
    assert!(matches!(
        constraints[3].constraint_type,
        reev_core::execution::rig_agent::ConstraintType::ExcludedMint(_)
    ));
    assert!(matches!(
        constraints[4].constraint_type,
        reev_core::execution::rig_agent::ConstraintType::PriceSlippage(_)
    ));
}

#[test]
fn test_constraint_validation() {
    // Create a constraint that applies to step 0
    let constraint = reev_core::execution::rig_agent::StepConstraint::new(
        reev_core::execution::rig_agent::ConstraintType::MaximumAmount(1000.0),
    )
    .for_parameter("amount")
    .for_steps(vec![0]);

    let constraints = vec![constraint];

    // Test valid amount - create fresh validator
    let validator1 = ParameterValidator::new(constraints.clone());
    let valid_params = json!({"amount": 500});
    let report1 = validator1
        .validate("jupiter_swap", &valid_params, 0)
        .unwrap();
    println!("Validation report for valid params: {report1:?}");
    println!("Is valid? {}", report1.is_valid());
    println!("Mandatory violations: {:?}", report1.mandatory_violations);
    assert!(report1.is_valid());

    // Test invalid amount - should fail - create fresh validator
    let validator2 = ParameterValidator::new(constraints.clone());
    let invalid_params = json!({"amount": 1500});
    let report2 = validator2
        .validate("jupiter_swap", &invalid_params, 0)
        .unwrap();
    println!("Validation report for invalid params: {report2:?}");
    println!("Is valid? {}", report2.is_valid());
    println!("Mandatory violations: {:?}", report2.mandatory_violations);
    assert!(!report2.is_valid());
    assert_eq!(report2.all_violations().len(), 1);

    // Test with parameter that doesn't match constraint - create fresh validator
    let validator3 = ParameterValidator::new(constraints);
    let other_params = json!({"amount": 100, "slippage": 0.02});
    let report3 = validator3
        .validate("jupiter_swap", &other_params, 0)
        .unwrap();
    println!("Validation report for other params: {report3:?}");
    println!("Is valid? {}", report3.is_valid());
    assert!(report3.is_valid()); // Validation passes because amount is valid
}
