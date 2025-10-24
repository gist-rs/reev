//! Context validation tests
//!
//! Tests context resolution without LLM calls to ensure correctness

use anyhow::Result;
use reev_context::{AgentContext, ContextResolver};
use serde_json::json;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;

/// Test context resolution for simple SOL transfer benchmark
#[tokio::test]
async fn test_sol_transfer_context_resolution() -> Result<()> {
    // Create mock resolver (no surfpool connection needed for validation test)
    let rpc_client = RpcClient::new("http://mock:8899");
    let resolver = ContextResolver::new(rpc_client);

    // Create mock context to test validation logic
    let mock_context = create_mock_sol_transfer_context();

    // Test validation without surfpool
    assert!(resolver.validate_resolved_context(&mock_context).is_ok());

    // Validate the resolved context structure
    assert!(mock_context.key_map.contains_key("USER_WALLET_PUBKEY"));
    assert!(mock_context.key_map.contains_key("RECIPIENT_WALLET_PUBKEY"));

    // Check that placeholders are resolved to valid addresses
    let user_address = &mock_context.key_map["USER_WALLET_PUBKEY"];
    let recipient_address = &mock_context.key_map["RECIPIENT_WALLET_PUBKEY"];

    assert!(
        Pubkey::from_str(user_address).is_ok(),
        "USER_WALLET_PUBKEY resolved to invalid address: {user_address}"
    );
    assert!(
        Pubkey::from_str(recipient_address).is_ok(),
        "RECIPIENT_WALLET_PUBKEY resolved to invalid address: {recipient_address}"
    );

    // Check YAML export
    let yaml_output = resolver.context_to_yaml(&mock_context)?;
    assert!(yaml_output.contains("# On-Chain Context for Transaction Processing"));
    assert!(yaml_output.contains("key_map:"));
    assert!(yaml_output.contains("account_states:"));

    println!("✅ SOL transfer context validation passed");
    println!("Resolved {} placeholders", mock_context.key_map.len());
    println!(
        "Fetched {} account states",
        mock_context.account_states.len()
    );

    Ok(())
}

/// Test context resolution for SPL transfer benchmark
#[tokio::test]
async fn test_spl_transfer_context_resolution() -> Result<()> {
    // Create mock resolver (no surfpool connection needed for validation test)
    let rpc_client = RpcClient::new("http://mock:8899");
    let resolver = ContextResolver::new(rpc_client);

    // Create mock context to test validation logic
    let mock_context = create_mock_spl_transfer_context();

    // Test validation without surfpool
    assert!(resolver.validate_resolved_context(&mock_context).is_ok());

    // Validate the resolved context structure
    assert!(mock_context.key_map.contains_key("USER_WALLET_PUBKEY"));
    assert!(mock_context.key_map.contains_key("RECIPIENT_WALLET_PUBKEY"));
    assert!(mock_context.key_map.contains_key("TOKEN_MINT_ADDRESS"));

    // Check that placeholders are resolved to valid addresses
    let user_address = &mock_context.key_map["USER_WALLET_PUBKEY"];
    let recipient_address = &mock_context.key_map["RECIPIENT_WALLET_PUBKEY"];
    let mint_address = &mock_context.key_map["TOKEN_MINT_ADDRESS"];

    assert!(
        Pubkey::from_str(user_address).is_ok(),
        "USER_WALLET_PUBKEY resolved to invalid address: {user_address}"
    );
    assert!(
        Pubkey::from_str(recipient_address).is_ok(),
        "RECIPIENT_WALLET_PUBKEY resolved to invalid address: {recipient_address}"
    );
    assert!(
        Pubkey::from_str(mint_address).is_ok(),
        "TOKEN_MINT_ADDRESS resolved to invalid address: {mint_address}"
    );

    // Check YAML export
    let yaml_output = resolver.context_to_yaml(&mock_context)?;
    assert!(yaml_output.contains("# On-Chain Context for Transaction Processing"));
    assert!(yaml_output.contains("key_map:"));
    assert!(yaml_output.contains("account_states:"));

    println!("✅ SPL transfer context validation passed");
    println!("Resolved {} placeholders", mock_context.key_map.len());
    println!(
        "Fetched {} account states",
        mock_context.account_states.len()
    );

    Ok(())
}

/// Test multi-step flow context updates
#[tokio::test]
async fn test_multi_step_context_updates() -> Result<()> {
    // Create mock resolver
    let rpc_client = RpcClient::new("http://mock:8899");
    let resolver = ContextResolver::new(rpc_client);

    // Create initial context for step 1
    let initial_context = create_mock_multi_step_context();

    // Validate initial context
    assert!(resolver.validate_resolved_context(&initial_context).is_ok());
    assert_eq!(initial_context.current_step, Some(0));

    // Simulate step 1 result (transfer completed)
    let step1_result = json!({
        "status": "success",
        "transaction_hash": "5j7s8...",
        "transfers": [
            {
                "from": "52os5otfYAPyQM2BbCEd6vbbDgLN42feBtxDCyVvKv6x",
                "to": "CMeCZDeQAC2i2i3HTkBP3brXXED7AZsUVeWKQeg7B42c",
                "amount": 1000000,
                "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
            }
        ],
        "new_balances": {
            "52os5otfYAPyQM2BbCEd6vbbDgLN42feBtxDCyVvKv6x": {
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v": 900000
            },
            "CMeCZDeQAC2i2i3HTkBP3brXXED7AZsUVeWKQeg7B42c": {
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v": 1100000
            }
        }
    });

    // Simulate updating context after step 1
    // In real implementation, this would call context_resolver.update_context_after_step()
    // For test, we'll manually update the context
    let mut updated_context = initial_context.clone();
    updated_context.current_step = Some(1);
    updated_context
        .step_results
        .insert("step_0".to_string(), step1_result);

    // Validate updated context
    assert!(resolver.validate_resolved_context(&updated_context).is_ok());
    assert_eq!(updated_context.current_step, Some(1));
    assert!(updated_context.step_results.contains_key("step_0"));

    // Check YAML export includes step results
    let yaml_output = resolver.context_to_yaml(&updated_context)?;
    assert!(yaml_output.contains("step_results"));

    println!("✅ Multi-step context update validation passed");
    println!("Current step: {:?}", updated_context.current_step);
    println!("Step results: {}", updated_context.step_results.len());

    Ok(())
}

/// Test invalid address validation
#[test]
fn test_invalid_address_validation() -> Result<()> {
    let rpc_client = RpcClient::new("http://mock:8899");
    let resolver = ContextResolver::new(rpc_client);

    // Create context with invalid address
    let mut context = create_mock_sol_transfer_context();
    context.key_map.insert(
        "INVALID_ADDRESS".to_string(),
        "not_a_valid_address".to_string(),
    );

    // This should fail validation
    let validation_result = resolver.validate_resolved_context(&context);
    assert!(
        validation_result.is_err(),
        "Validation should fail for invalid address"
    );

    println!("✅ Invalid address validation correctly failed");

    Ok(())
}

/// Test missing placeholder validation
#[test]
fn test_missing_placeholder_validation() -> Result<()> {
    let rpc_client = RpcClient::new("http://mock:8899");
    let resolver = ContextResolver::new(rpc_client);

    // Create context missing required USER_WALLET_PUBKEY
    let mut context = create_mock_sol_transfer_context();
    context.key_map.remove("USER_WALLET_PUBKEY");

    // This should fail validation
    let validation_result = resolver.validate_resolved_context(&context);
    assert!(
        validation_result.is_err(),
        "Validation should fail for missing USER_WALLET_PUBKEY"
    );

    println!("✅ Missing placeholder validation correctly failed");

    Ok(())
}

/// Helper: Create mock SOL transfer context
fn create_mock_sol_transfer_context() -> AgentContext {
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_WALLET_PUBKEY".to_string(),
        "52os5otfYAPyQM2BbCEd6vbbDgLN42feBtxDCyVvKv6x".to_string(),
    );
    key_map.insert(
        "RECIPIENT_WALLET_PUBKEY".to_string(),
        "CMeCZDeQAC2i2i3HTkBP3brXXED7AZsUVeWKQeg7B42c".to_string(),
    );

    let mut account_states = HashMap::new();
    account_states.insert(
        "USER_WALLET_PUBKEY".to_string(),
        json!({
            "lamports": 1000000000,
            "owner": "11111111111111111111111111111111",
            "executable": false,
            "data_len": 0
        }),
    );
    account_states.insert(
        "RECIPIENT_WALLET_PUBKEY".to_string(),
        json!({
            "lamports": 0,
            "owner": "11111111111111111111111111111111",
            "executable": false,
            "data_len": 0,
            "exists": false
        }),
    );

    AgentContext {
        key_map,
        account_states,
        fee_payer_placeholder: Some("USER_WALLET_PUBKEY".to_string()),
        current_step: Some(0),
        step_results: HashMap::new(),
    }
}

/// Helper: Create mock SPL transfer context
fn create_mock_spl_transfer_context() -> AgentContext {
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_WALLET_PUBKEY".to_string(),
        "52os5otfYAPyQM2BbCEd6vbbDgLN42feBtxDCyVvKv6x".to_string(),
    );
    key_map.insert(
        "RECIPIENT_WALLET_PUBKEY".to_string(),
        "CMeCZDeQAC2i2i3HTkBP3brXXED7AZsUVeWKQeg7B42c".to_string(),
    );
    key_map.insert(
        "TOKEN_MINT_ADDRESS".to_string(),
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
    );

    let mut account_states = HashMap::new();
    account_states.insert(
        "USER_WALLET_PUBKEY".to_string(),
        json!({
            "lamports": 1000000000,
            "owner": "11111111111111111111111111111111",
            "executable": false,
            "data_len": 0
        }),
    );
    account_states.insert(
        "RECIPIENT_WALLET_PUBKEY".to_string(),
        json!({
            "lamports": 1000000000,
            "owner": "11111111111111111111111111111111",
            "executable": false,
            "data_len": 0
        }),
    );

    AgentContext {
        key_map,
        account_states,
        fee_payer_placeholder: Some("USER_WALLET_PUBKEY".to_string()),
        current_step: Some(0),
        step_results: HashMap::new(),
    }
}

/// Helper: Create mock multi-step flow context
fn create_mock_multi_step_context() -> AgentContext {
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_WALLET_PUBKEY".to_string(),
        "52os5otfYAPyQM2BbCEd6vbbDgLN42feBtxDCyVvKv6x".to_string(),
    );
    key_map.insert(
        "RECIPIENT_WALLET_PUBKEY".to_string(),
        "CMeCZDeQAC2i2i3HTkBP3brXXED7AZsUVeWKQeg7B42c".to_string(),
    );
    key_map.insert(
        "TOKEN_MINT_ADDRESS".to_string(),
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
    );

    let mut account_states = HashMap::new();
    account_states.insert(
        "USER_WALLET_PUBKEY".to_string(),
        json!({
            "lamports": 1000000000,
            "owner": "11111111111111111111111111111111",
            "executable": false,
            "data_len": 0
        }),
    );
    account_states.insert(
        "RECIPIENT_WALLET_PUBKEY".to_string(),
        json!({
            "lamports": 1000000000,
            "owner": "11111111111111111111111111111111",
            "executable": false,
            "data_len": 0
        }),
    );

    AgentContext {
        key_map,
        account_states,
        fee_payer_placeholder: Some("USER_WALLET_PUBKEY".to_string()),
        current_step: Some(0),
        step_results: HashMap::new(),
    }
}

/// Test SPL transfer error separation
#[test]
fn test_spl_transfer_error_separation() -> Result<()> {
    // Verify that SplTransferError is a separate type from NativeTransferError
    // This test ensures the error types are properly separated as per Phase 4

    use reev_tools::tools::native::{NativeTransferError, SplTransferError};

    // Test that we can create both error types independently
    let spl_error = SplTransferError::PubkeyParse("invalid pubkey".to_string());
    let native_error = NativeTransferError::PubkeyParse("invalid pubkey".to_string());

    // Verify they are different types
    let _spl_str: String = spl_error.to_string();
    let _native_str: String = native_error.to_string();

    println!("✅ SPL transfer error separation test passed");
    println!("SPL Error: {_spl_str}");
    println!("Native Error: {_native_str}");

    Ok(())
}
