//! Core protocol interface for Stage 1 implementation
//!
//! This module defines the minimal protocol interface needed for immediate
//! protocol consolidation while maintaining flexibility for future extensions.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Minimal protocol interface for Stage 1
///
/// This trait provides a simple, unified interface for executing protocol operations
/// while keeping the implementation flexible enough for future enhancements.
#[async_trait]
pub trait ProtocolExecutor: Send + Sync {
    /// Error type returned by the protocol
    type Error: std::error::Error + Send + Sync + 'static;

    /// Result type returned by the protocol
    type Result: Send + Sync;

    /// Execute a protocol operation
    ///
    /// # Arguments
    /// * `operation` - The operation to execute with its parameters
    ///
    /// # Returns
    /// The result of the operation or an error if execution fails
    async fn execute(&self, operation: &ProtocolOperation) -> Result<Self::Result, Self::Error>;
}

/// Protocol operation definition
///
/// This structure represents a protocol operation with its type and parameters.
/// It is designed to be protocol-agnostic while providing all necessary information
/// for protocol execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolOperation {
    /// Type of operation to perform
    pub operation_type: OperationType,

    /// Parameters specific to the operation
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Protocol operation types
///
/// This enum defines the types of operations supported by the protocol system.
/// It includes common DeFi operations and allows for custom operation types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperationType {
    /// Swap operation between tokens
    Swap,

    /// Lend operation to deposit tokens
    Lend,

    /// Earn operation for yield farming
    Earn,

    /// Stake operation for staking tokens
    Stake,

    /// Transfer operation to send tokens to another address
    Transfer,

    /// Custom operation type for protocol-specific operations
    Custom(String),
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::Swap => write!(f, "Swap"),
            OperationType::Lend => write!(f, "Lend"),
            OperationType::Earn => write!(f, "Earn"),
            OperationType::Stake => write!(f, "Stake"),
            OperationType::Transfer => write!(f, "Transfer"),
            OperationType::Custom(name) => write!(f, "Custom({name})"),
        }
    }
}

/// Protocol error types
///
/// This enum defines common error types that can occur during protocol operations.
/// It provides a standardized way to handle errors across different protocols.
#[derive(Error, Debug)]
pub enum ProtocolError {
    /// Unsupported operation type for this protocol
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(OperationType),

    /// Invalid parameter provided
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Missing required parameter
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    /// Protocol-specific error
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Network or RPC error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Transaction failed
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}

impl From<anyhow::Error> for ProtocolError {
    fn from(err: anyhow::Error) -> Self {
        ProtocolError::ProtocolError(err.to_string())
    }
}

impl From<solana_sdk::transaction::TransactionError> for ProtocolError {
    fn from(err: solana_sdk::transaction::TransactionError) -> Self {
        ProtocolError::TransactionFailed(format!("{err:?}"))
    }
}

/// Protocol operation result
///
/// This enum represents the result of a protocol operation.
/// It is designed to be extended with protocol-specific result types.
#[derive(Debug, Clone)]
pub enum ProtocolResult {
    /// Jupiter protocol result
    Jupiter(JupiterResult),

    /// Placeholder for future protocols
    Placeholder(String),
}

/// Jupiter protocol result types
///
/// This enum represents the different types of results from Jupiter operations.
#[derive(Debug, Clone)]
pub enum JupiterResult {
    /// Swap operation result
    Swap(JupiterSwapResult),

    /// Lend operation result
    Lend(JupiterLendResult),

    /// Earn operation result
    Earn(JupiterEarnResult),
}

/// Jupiter swap result
///
/// This structure represents the result of a Jupiter swap operation.
#[derive(Debug, Clone)]
pub struct JupiterSwapResult {
    /// Whether the swap was successful
    pub success: bool,

    /// Transaction signature
    pub signature: Option<String>,

    /// Input amount
    pub input_amount: u64,

    /// Output amount
    pub output_amount: u64,

    /// Price impact
    pub price_impact: Option<f64>,
}

/// Jupiter lend result
///
/// This structure represents the result of a Jupiter lend operation.
#[derive(Debug, Clone)]
pub struct JupiterLendResult {
    /// Whether the lend operation was successful
    pub success: bool,

    /// Transaction signature
    pub signature: Option<String>,

    /// Amount lent
    pub amount: u64,

    /// Token mint address
    pub mint: String,
}

/// Jupiter earn result
///
/// This structure represents the result of a Jupiter earn operation.
#[derive(Debug, Clone)]
pub struct JupiterEarnResult {
    /// Whether the earn operation was successful
    pub success: bool,

    /// Transaction signature
    pub signature: Option<String>,

    /// Amount deposited
    pub amount: u64,

    /// Token mint address
    pub mint: String,
}
