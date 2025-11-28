//! Recovery Configuration for Error Handling
//!
//! This module defines the configuration for error recovery
//! during flow execution.

/// Configuration for error recovery
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    /// Delay between retry attempts in milliseconds
    pub retry_delay_ms: u64,
    /// Whether to use exponential backoff for retries
    pub exponential_backoff: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            retry_delay_ms: 1000,
            exponential_backoff: false,
        }
    }
}
