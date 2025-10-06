//! Protocol handlers for various DeFi protocols
//!
//! This module provides real API integration with different DeFi protocols.
//! Each protocol module contains the actual API calls and business logic
//! for interacting with that protocol.

#[cfg(feature = "jupiter")]
pub mod jupiter;
#[cfg(feature = "native")]
pub mod native;

// Common protocol abstractions
pub mod common;

#[cfg(feature = "jupiter")]
pub use jupiter::*;
#[cfg(feature = "native")]
pub use native::*;

// Re-export common protocol utilities
pub use common::*;

/// Re-export configuration for convenience
/// Note: Config is now inline in each protocol module
/// Common error types for protocol operations
pub mod errors {
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum ProtocolError {
        #[error("API request failed: {0}")]
        ApiError(String),
        #[error("HTTP request failed: {0}")]
        HttpError(#[from] reqwest::Error),
        #[error("JSON parsing failed: {0}")]
        JsonError(#[from] serde_json::Error),
        #[error("Invalid response format: {0}")]
        InvalidFormat(String),
        #[error("Rate limit exceeded")]
        RateLimit,
        #[error("Authentication failed")]
        Authentication,
        #[error("Insufficient funds")]
        InsufficientFunds,
        #[error("Invalid parameters: {0}")]
        InvalidParameters(String),
        #[error("Network timeout")]
        Timeout,
        #[error("Configuration error: {0}")]
        ConfigError(String),
    }

    pub type Result<T> = std::result::Result<T, ProtocolError>;
}

/// Common types used across protocols
pub mod types {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    /// Generic API response wrapper
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ApiResponse<T> {
        pub success: bool,
        pub data: Option<T>,
        pub error: Option<String>,
        pub timestamp: Option<String>,
    }

    /// Pagination information
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Pagination {
        pub page: u32,
        pub limit: u32,
        pub total: u32,
        pub has_next: bool,
    }

    /// Rate limit information
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RateLimit {
        pub remaining: u32,
        pub reset: u64,
        pub limit: u32,
    }

    /// Common request parameters
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct RequestParams {
        pub user_pubkey: String,
        pub timeout: Option<u64>,
        pub max_retries: Option<u32>,
        pub extra: HashMap<String, String>,
    }
}

/// HTTP client utilities for protocol implementations
pub mod http {
    use super::errors::{ProtocolError, Result};
    use reqwest::{Client, RequestBuilder};
    use std::time::Duration;

    /// Create a configured HTTP client
    pub fn create_client(user_agent: &str, timeout: Duration) -> Result<Client> {
        Client::builder()
            .user_agent(user_agent)
            .timeout(timeout)
            .build()
            .map_err(ProtocolError::HttpError)
    }

    /// Execute a request with retry logic
    pub async fn execute_with_retry(
        request: RequestBuilder,
        max_retries: u32,
    ) -> Result<reqwest::Response> {
        let mut retries = 0;
        let mut last_error = None;

        while retries <= max_retries {
            match request.try_clone().unwrap().send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    }

                    // Don't retry on client errors (4xx)
                    if response.status().is_client_error() {
                        return Err(ProtocolError::ApiError(format!(
                            "Client error: {}",
                            response.status()
                        )));
                    }

                    last_error = Some(ProtocolError::ApiError(format!(
                        "Server error: {}",
                        response.status()
                    )));
                }
                Err(e) => {
                    last_error = Some(ProtocolError::HttpError(e));
                }
            }

            retries += 1;
            if retries <= max_retries {
                tokio::time::sleep(Duration::from_millis(1000 * retries as u64)).await;
            }
        }

        Err(last_error.unwrap_or(ProtocolError::ApiError("Max retries exceeded".to_string())))
    }
}
