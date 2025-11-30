//! Tests for Jupiter protocol implementation

use reev_core::protocols::{
    jupiter::JupiterProtocol, JupiterResult, OperationType, ProtocolExecutor, ProtocolOperation,
    ProtocolRegistry, ProtocolResult,
};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_jupiter_protocol_swap() {
    let protocol = JupiterProtocol::new();

    let mut parameters = HashMap::new();
    parameters.insert(
        "input_mint".to_string(),
        json!("So11111111111111111111111111111111111111112"),
    );
    parameters.insert(
        "output_mint".to_string(),
        json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    );
    parameters.insert("amount".to_string(), json!(1000000));

    let operation = ProtocolOperation {
        operation_type: OperationType::Swap,
        parameters,
    };

    let result = protocol.execute(&operation).await.unwrap();

    match result {
        ProtocolResult::Jupiter(JupiterResult::Swap(swap_result)) => {
            assert!(swap_result.success);
            assert_eq!(swap_result.input_amount, 1000000);
            assert_eq!(swap_result.output_amount, 2000000);
            assert_eq!(
                swap_result.signature,
                Some("jupiter_swap_placeholder".to_string())
            );
            assert_eq!(swap_result.price_impact, Some(0.05));
        }
        _ => panic!("Expected Jupiter swap result"),
    }
}

#[tokio::test]
async fn test_jupiter_protocol_lend() {
    let protocol = JupiterProtocol::new();

    let mut parameters = HashMap::new();
    parameters.insert(
        "mint".to_string(),
        json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    );
    parameters.insert("amount".to_string(), json!(10000000));

    let operation = ProtocolOperation {
        operation_type: OperationType::Lend,
        parameters,
    };

    let result = protocol.execute(&operation).await.unwrap();

    match result {
        ProtocolResult::Jupiter(JupiterResult::Lend(lend_result)) => {
            assert!(lend_result.success);
            assert_eq!(lend_result.amount, 10000000);
            assert_eq!(
                lend_result.mint,
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
            );
            assert_eq!(
                lend_result.signature,
                Some("jupiter_lend_placeholder".to_string())
            );
        }
        _ => panic!("Expected Jupiter lend result"),
    }
}

#[tokio::test]
async fn test_jupiter_protocol_earn() {
    let protocol = JupiterProtocol::new();

    let mut parameters = HashMap::new();
    parameters.insert(
        "mint".to_string(),
        json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    );
    parameters.insert("amount".to_string(), json!(5000000));

    let operation = ProtocolOperation {
        operation_type: OperationType::Earn,
        parameters,
    };

    let result = protocol.execute(&operation).await.unwrap();

    match result {
        ProtocolResult::Jupiter(JupiterResult::Earn(earn_result)) => {
            assert!(earn_result.success);
            assert_eq!(earn_result.amount, 5000000);
            assert_eq!(
                earn_result.mint,
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
            );
            assert_eq!(
                earn_result.signature,
                Some("jupiter_earn_placeholder".to_string())
            );
        }
        _ => panic!("Expected Jupiter earn result"),
    }
}

#[tokio::test]
async fn test_jupiter_protocol_unsupported_operation() {
    let protocol = JupiterProtocol::new();

    let operation = ProtocolOperation {
        operation_type: OperationType::Stake,
        parameters: HashMap::new(),
    };

    let result = protocol.execute(&operation).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jupiter_protocol_in_registry() {
    let mut registry = ProtocolRegistry::new();

    // Register Jupiter protocol
    let jupiter_protocol = JupiterProtocol::new();
    registry
        .register_with_operations(
            "jupiter",
            jupiter_protocol,
            &[
                OperationType::Swap,
                OperationType::Lend,
                OperationType::Earn,
            ],
        )
        .unwrap();

    // Test protocol retrieval
    let protocol = registry.get("jupiter");
    assert!(protocol.is_some());

    // Test swap operation through registry
    let mut parameters = HashMap::new();
    parameters.insert(
        "input_mint".to_string(),
        json!("So11111111111111111111111111111111111111112"),
    );
    parameters.insert(
        "output_mint".to_string(),
        json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    );
    parameters.insert("amount".to_string(), json!(1000000));

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
                assert_eq!(swap_result.output_amount, 2000000);
            }
            _ => panic!("Expected Jupiter swap result"),
        }
    } else {
        panic!("Expected protocol for swap operation");
    }
}

#[tokio::test]
async fn test_jupiter_protocol_error_handling() {
    let protocol = JupiterProtocol::new();

    // Test swap with missing parameters
    let operation = ProtocolOperation {
        operation_type: OperationType::Swap,
        parameters: HashMap::new(), // Missing required parameters
    };

    let result = protocol.execute(&operation).await;
    assert!(result.is_err());

    // Test lend with missing parameters
    let operation = ProtocolOperation {
        operation_type: OperationType::Lend,
        parameters: HashMap::new(), // Missing required parameters
    };

    let result = protocol.execute(&operation).await;
    assert!(result.is_err());

    // Test earn with missing parameters
    let operation = ProtocolOperation {
        operation_type: OperationType::Earn,
        parameters: HashMap::new(), // Missing required parameters
    };

    let result = protocol.execute(&operation).await;
    assert!(result.is_err());
}
