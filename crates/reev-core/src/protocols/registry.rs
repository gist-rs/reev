//! Protocol registry for protocol discovery and management
//!
//! This module provides a simple protocol registry for Stage 1 implementation.
//! It allows for protocol registration, discovery, and selection based on operation types.

use super::{ProtocolError, ProtocolExecutor, ProtocolResult};
use std::collections::HashMap;

/// Simple protocol registry for Stage 1
///
/// This registry manages protocol instances and allows for protocol selection
/// based on operation types. It is designed to be simple and efficient for
/// immediate needs while being extensible for future enhancements.
pub struct ProtocolRegistry {
    /// Map of protocol name to protocol instance
    protocols:
        HashMap<String, Box<dyn ProtocolExecutor<Error = ProtocolError, Result = ProtocolResult>>>,

    /// Map of operation type to protocol name for efficient lookup
    operation_to_protocol: HashMap<super::OperationType, String>,
}

impl Default for ProtocolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolRegistry {
    /// Create a new protocol registry
    pub fn new() -> Self {
        Self {
            protocols: HashMap::new(),
            operation_to_protocol: HashMap::new(),
        }
    }

    /// Register a protocol with the registry
    ///
    /// # Arguments
    /// * `name` - The name of the protocol
    /// * `protocol` - The protocol instance
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn register<E>(&mut self, name: &str, protocol: E) -> Result<(), ProtocolError>
    where
        E: ProtocolExecutor<Error = ProtocolError, Result = ProtocolResult> + Send + Sync + 'static,
    {
        // Store the protocol with type erasure
        self.protocols.insert(name.to_string(), Box::new(protocol));

        Ok(())
    }

    /// Register a protocol with supported operation types
    ///
    /// # Arguments
    /// * `name` - The name of the protocol
    /// * `protocol` - The protocol instance
    /// * `supported_operations` - List of operation types supported by this protocol
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn register_with_operations<E>(
        &mut self,
        name: &str,
        protocol: E,
        supported_operations: &[super::OperationType],
    ) -> Result<(), ProtocolError>
    where
        E: ProtocolExecutor<Error = ProtocolError, Result = ProtocolResult> + Send + Sync + 'static,
    {
        // Register the protocol
        self.register(name, protocol)?;

        // Register the operation mappings
        for operation in supported_operations {
            self.operation_to_protocol
                .insert(operation.clone(), name.to_string());
        }

        Ok(())
    }

    /// Get a protocol by name
    ///
    /// # Arguments
    /// * `name` - The name of the protocol
    ///
    /// # Returns
    /// Option containing the protocol if found
    pub fn get(
        &self,
        name: &str,
    ) -> Option<&dyn ProtocolExecutor<Error = ProtocolError, Result = ProtocolResult>> {
        self.protocols.get(name).map(|p| p.as_ref())
    }

    /// Get a protocol by operation type
    ///
    /// # Arguments
    /// * `operation_type` - The operation type
    ///
    /// # Returns
    /// Option containing the protocol if found
    pub fn get_for_operation(
        &self,
        operation_type: &super::OperationType,
    ) -> Option<&dyn ProtocolExecutor<Error = ProtocolError, Result = ProtocolResult>> {
        if let Some(protocol_name) = self.operation_to_protocol.get(operation_type) {
            return self.get(protocol_name);
        }

        // Fallback to first protocol for unsupported operations
        self.protocols.values().next().map(|p| p.as_ref())
    }

    /// Get all registered protocol names
    ///
    /// # Returns
    /// A vector of protocol names
    pub fn list_protocols(&self) -> Vec<String> {
        self.protocols.keys().cloned().collect()
    }

    /// Check if an operation type is supported by any protocol
    ///
    /// # Arguments
    /// * `operation_type` - The operation type to check
    ///
    /// # Returns
    /// True if the operation is supported, false otherwise
    pub fn supports_operation(&self, operation_type: &super::OperationType) -> bool {
        self.operation_to_protocol.contains_key(operation_type)
    }
}

// Adapter removed since we simplified the type bounds

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::executor::{JupiterResult, JupiterSwapResult, OperationType};

    // Mock protocol for testing
    struct MockProtocol {
        _name: String,
    }

    #[async_trait::async_trait]
    impl ProtocolExecutor for MockProtocol {
        type Error = ProtocolError;
        type Result = ProtocolResult;

        async fn execute(
            &self,
            operation: &super::super::ProtocolOperation,
        ) -> Result<Self::Result, Self::Error> {
            match operation.operation_type {
                OperationType::Swap => {
                    let result = JupiterSwapResult {
                        success: true,
                        signature: Some("signature".to_string()),
                        input_amount: 100,
                        output_amount: 200,
                        price_impact: Some(0.1),
                    };
                    Ok(ProtocolResult::Jupiter(JupiterResult::Swap(result)))
                }
                _ => Err(ProtocolError::UnsupportedOperation(
                    operation.operation_type.clone(),
                )),
            }
        }
    }

    #[tokio::test]
    async fn test_protocol_registry() {
        let mut registry = ProtocolRegistry::new();

        // Register a protocol
        let protocol = MockProtocol {
            _name: "TestProtocol".to_string(),
        };

        registry
            .register_with_operations("test_protocol", protocol, &[OperationType::Swap])
            .unwrap();

        // Test protocol retrieval by name
        let retrieved = registry.get("test_protocol");
        assert!(retrieved.is_some());

        // Test protocol retrieval by operation type
        let retrieved_op = registry.get_for_operation(&OperationType::Swap);
        assert!(retrieved_op.is_some());

        // Test unsupported operation
        let retrieved_unsupported = registry.get_for_operation(&OperationType::Lend);
        assert!(retrieved_unsupported.is_some()); // Falls back to first protocol

        // Test protocol list
        let protocols = registry.list_protocols();
        assert_eq!(protocols.len(), 1);
        assert_eq!(protocols[0], "test_protocol");

        // Test operation support
        assert!(registry.supports_operation(&OperationType::Swap));
        assert!(!registry.supports_operation(&OperationType::Lend));
    }
}
