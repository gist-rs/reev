//! Tests for core protocol interface

use reev_core::protocols::{
    JupiterEarnResult, JupiterLendResult, JupiterResult, JupiterSwapResult, OperationType,
    ProtocolError, ProtocolExecutor, ProtocolOperation, ProtocolRegistry, ProtocolResult,
};
use serde_json::json;
use std::collections::HashMap;

// Mock protocol implementation for testing
struct MockProtocol {
    _name: String,
}

#[async_trait::async_trait]
impl ProtocolExecutor for MockProtocol {
    type Error = ProtocolError;
    type Result = ProtocolResult;

    async fn execute(&self, operation: &ProtocolOperation) -> Result<Self::Result, Self::Error> {
        match operation.operation_type {
            OperationType::Swap => {
                let input_amount = operation
                    .parameters
                    .get("input_amount")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| ProtocolError::MissingParameter("input_amount".to_string()))?;

                let output_amount = operation
                    .parameters
                    .get("output_amount")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(input_amount * 2); // Default 2x multiplier

                let result = JupiterSwapResult {
                    success: true,
                    signature: Some("mock_swap_signature".to_string()),
                    input_amount,
                    output_amount,
                    price_impact: Some(0.05),
                };

                Ok(ProtocolResult::Jupiter(JupiterResult::Swap(result)))
            }
            OperationType::Lend => {
                let amount = operation
                    .parameters
                    .get("amount")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| ProtocolError::MissingParameter("amount".to_string()))?;

                let mint = operation
                    .parameters
                    .get("mint")
                    .and_then(|v| v.as_str())
                    .unwrap_or("USDC")
                    .to_string();

                let result = JupiterLendResult {
                    success: true,
                    signature: Some("mock_lend_signature".to_string()),
                    amount,
                    mint,
                };

                Ok(ProtocolResult::Jupiter(JupiterResult::Lend(result)))
            }
            OperationType::Earn => {
                let amount = operation
                    .parameters
                    .get("amount")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| ProtocolError::MissingParameter("amount".to_string()))?;

                let mint = operation
                    .parameters
                    .get("mint")
                    .and_then(|v| v.as_str())
                    .unwrap_or("USDC")
                    .to_string();

                let result = JupiterEarnResult {
                    success: true,
                    signature: Some("mock_earn_signature".to_string()),
                    amount,
                    mint,
                };

                Ok(ProtocolResult::Jupiter(JupiterResult::Earn(result)))
            }
            _ => Err(ProtocolError::UnsupportedOperation(
                operation.operation_type.clone(),
            )),
        }
    }
}

#[tokio::test]
async fn test_protocol_executor_trait() {
    let protocol = MockProtocol {
        _name: "TestProtocol".to_string(),
    };

    // Test swap operation
    let mut parameters = HashMap::new();
    parameters.insert("input_amount".to_string(), json!(1000000)); // 1 SOL in lamports
    parameters.insert("output_amount".to_string(), json!(20000000)); // 20 USDC

    let operation = ProtocolOperation {
        operation_type: OperationType::Swap,
        parameters,
    };

    let result = protocol.execute(&operation).await.unwrap();

    match result {
        ProtocolResult::Jupiter(JupiterResult::Swap(swap_result)) => {
            assert!(swap_result.success);
            assert_eq!(swap_result.input_amount, 1000000);
            assert_eq!(swap_result.output_amount, 20000000);
            assert_eq!(
                swap_result.signature,
                Some("mock_swap_signature".to_string())
            );
            assert_eq!(swap_result.price_impact, Some(0.05));
        }
        _ => panic!("Expected Jupiter swap result"),
    }

    // Test lend operation
    let mut parameters = HashMap::new();
    parameters.insert("amount".to_string(), json!(5000000)); // 5 USDC
    parameters.insert("mint".to_string(), json!("USDC"));

    let operation = ProtocolOperation {
        operation_type: OperationType::Lend,
        parameters,
    };

    let result = protocol.execute(&operation).await.unwrap();

    match result {
        ProtocolResult::Jupiter(JupiterResult::Lend(lend_result)) => {
            assert!(lend_result.success);
            assert_eq!(lend_result.amount, 5000000);
            assert_eq!(lend_result.mint, "USDC");
            assert_eq!(
                lend_result.signature,
                Some("mock_lend_signature".to_string())
            );
        }
        _ => panic!("Expected Jupiter lend result"),
    }

    // Test earn operation
    let mut parameters = HashMap::new();
    parameters.insert("amount".to_string(), json!(10000000)); // 10 USDC
                                                              // Omit mint parameter to test default value

    let operation = ProtocolOperation {
        operation_type: OperationType::Earn,
        parameters,
    };

    let result = protocol.execute(&operation).await.unwrap();

    match result {
        ProtocolResult::Jupiter(JupiterResult::Earn(earn_result)) => {
            assert!(earn_result.success);
            assert_eq!(earn_result.amount, 10000000);
            assert_eq!(earn_result.mint, "USDC"); // Default value
            assert_eq!(
                earn_result.signature,
                Some("mock_earn_signature".to_string())
            );
        }
        _ => panic!("Expected Jupiter earn result"),
    }

    // Test unsupported operation
    let operation = ProtocolOperation {
        operation_type: OperationType::Stake,
        parameters: HashMap::new(),
    };

    let result = protocol.execute(&operation).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ProtocolError::UnsupportedOperation(op_type) => {
            assert_eq!(op_type, OperationType::Stake);
        }
        _ => panic!("Expected UnsupportedOperation error"),
    }
}

#[tokio::test]
async fn test_protocol_operation() {
    // Test serialization and deserialization
    let mut parameters = HashMap::new();
    parameters.insert("input_amount".to_string(), json!(1000000));
    parameters.insert("output_amount".to_string(), json!(20000000));

    let operation = ProtocolOperation {
        operation_type: OperationType::Swap,
        parameters,
    };

    let serialized = serde_json::to_string(&operation).unwrap();
    let deserialized: ProtocolOperation = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.operation_type, OperationType::Swap);
    assert_eq!(
        deserialized.parameters.get("input_amount"),
        Some(&json!(1000000))
    );
    assert_eq!(
        deserialized.parameters.get("output_amount"),
        Some(&json!(20000000))
    );
}

#[tokio::test]
async fn test_protocol_registry() {
    let mut registry = ProtocolRegistry::new();

    // Register a protocol
    let protocol = MockProtocol {
        _name: "TestProtocol".to_string(),
    };

    registry
        .register_with_operations(
            "test_protocol",
            protocol,
            &[
                OperationType::Swap,
                OperationType::Lend,
                OperationType::Earn,
            ],
        )
        .unwrap();

    // Test protocol retrieval by name
    let retrieved = registry.get("test_protocol");
    assert!(retrieved.is_some());

    // Test protocol retrieval by operation type
    let swap_protocol = registry.get_for_operation(&OperationType::Swap);
    assert!(swap_protocol.is_some());

    let lend_protocol = registry.get_for_operation(&OperationType::Lend);
    assert!(lend_protocol.is_some());

    let earn_protocol = registry.get_for_operation(&OperationType::Earn);
    assert!(earn_protocol.is_some());

    // Test unsupported operation (falls back to first protocol)
    let stake_protocol = registry.get_for_operation(&OperationType::Stake);
    assert!(stake_protocol.is_some());

    // Test protocol list
    let protocols = registry.list_protocols();
    assert_eq!(protocols.len(), 1);
    assert_eq!(protocols[0], "test_protocol");

    // Test operation support
    assert!(registry.supports_operation(&OperationType::Swap));
    assert!(registry.supports_operation(&OperationType::Lend));
    assert!(registry.supports_operation(&OperationType::Earn));
    assert!(!registry.supports_operation(&OperationType::Stake));

    // Test protocol execution through registry
    let mut parameters = HashMap::new();
    parameters.insert("input_amount".to_string(), json!(1000000));
    parameters.insert("output_amount".to_string(), json!(20000000));

    let operation = ProtocolOperation {
        operation_type: OperationType::Swap,
        parameters,
    };

    if let Some(protocol) = registry.get_for_operation(&OperationType::Swap) {
        let result = protocol.execute(&operation).await.unwrap();

        match result {
            ProtocolResult::Jupiter(JupiterResult::Swap(swap_result)) => {
                assert!(swap_result.success);
                assert_eq!(swap_result.input_amount, 1000000);
                assert_eq!(swap_result.output_amount, 20000000);
            }
            _ => panic!("Expected Jupiter swap result"),
        }
    } else {
        panic!("Expected protocol for swap operation");
    }
}
