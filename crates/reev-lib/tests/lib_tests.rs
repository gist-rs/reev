//! Tests for reev-lib library

use reev_lib::*;

#[test]
fn test_core_types_creation() {
    // Test that core types can be created without errors
    let wallet_state = WalletState::new();
    assert_eq!(wallet_state.sol_amount, 0);
    assert_eq!(wallet_state.usdc_amount, 0);

    let api_service = CachedApiService::new("./cache".to_string(), true, false);
    assert!(!api_service.mock_mode);
    assert!(api_service.real_jupiter_client);

    let refined_prompt = RefinedPrompt::new(1, "test".to_string(), "reasoning".to_string());
    assert_eq!(refined_prompt.step, 1);
    assert_eq!(refined_prompt.prompt, "test");

    let execution_result = ExecutionResult::new("test_id".to_string(), "test_tool".to_string());
    assert_eq!(execution_result.execution_id, "test_id");
    assert_eq!(execution_result.tool_name, "test_tool");
    assert!(!execution_result.success);
}

#[test]
fn test_constants() {
    // Verify constants are valid Solana addresses (SOL_MINT is 43 chars, USDC_MINT is 44 chars)
    assert_eq!(SOL_MINT.len(), 43, "SOL_MINT should be 43 characters");
    assert_eq!(USDC_MINT.len(), 44, "USDC_MINT should be 44 characters");
    assert_ne!(
        SOL_MINT, USDC_MINT,
        "SOL_MINT and USDC_MINT should be different"
    );

    // Verify they're valid base58 format (don't panic)
    bs58::decode(SOL_MINT)
        .into_vec()
        .expect("SOL_MINT should be valid base58");
    bs58::decode(USDC_MINT)
        .into_vec()
        .expect("USDC_MINT should be valid base58");
}
