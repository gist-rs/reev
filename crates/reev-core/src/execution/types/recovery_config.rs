//! Recovery configuration for tool execution
//!
//! This module defines the configuration used for error recovery during
//! tool execution and transaction processing.

/// Configuration for error recovery strategies
pub struct RecoveryConfig {
    /// Base delay between retries in milliseconds
    pub base_retry_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_retry_delay_ms: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum total recovery time per step in milliseconds
    pub max_recovery_time_ms: u64,
    /// Whether to enable alternative flow recovery
    pub enable_alternative_flows: bool,
    /// Whether to enable user fulfillment recovery
    pub enable_user_fulfillment: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            base_retry_delay_ms: 1000,
            max_retry_delay_ms: 30000,
            backoff_multiplier: 1.5,
            max_recovery_time_ms: 300000,
            enable_alternative_flows: true,
            enable_user_fulfillment: true,
        }
    }
}
