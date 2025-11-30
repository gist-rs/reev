//! Protocol abstraction layer for Reev
//!
//! This module provides a unified interface for interacting with various DeFi protocols.
//! It implements a two-stage approach:
//! 1. Stage 1: Minimal protocol interface for immediate needs
//! 2. Stage 2: Full protocol abstraction for future extensibility

pub mod executor;
pub mod jupiter;
pub mod registry;

// Re-export commonly used types
pub use executor::{
    JupiterEarnResult, JupiterLendResult, JupiterResult, JupiterSwapResult, OperationType,
    ProtocolError, ProtocolExecutor, ProtocolOperation, ProtocolResult,
};
pub use registry::ProtocolRegistry;
