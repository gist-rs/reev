//! Common helpers for example files
//!
//! This module provides shared functionality used by multiple example files
//! to reduce code duplication and ensure consistent behavior.

use reqwest::Client;
use std::time::Duration;

/// Default configuration values for examples
pub mod config {
    /// Default reev-agent host
    pub const DEFAULT_HOST: &str = "127.0.0.1";

    /// Default reev-agent port
    pub const DEFAULT_PORT: u16 = 9090;

    /// Health check endpoint
    pub const HEALTH_ENDPOINT: &str = "/health";

    /// Mock parameter for deterministic agent
    pub const MOCK_PARAM: &str = "mock=true";

    /// Timeout for HTTP requests in seconds
    pub const REQUEST_TIMEOUT: u64 = 30;
}

/// Example context and configuration
#[derive(Debug, Clone)]
pub struct ExampleConfig {
    pub agent_url: String,
    pub health_url: String,
    pub client: Client,
}

impl ExampleConfig {
    /// Create a new example configuration
    pub fn new(agent_name: &str) -> Self {
        let base_url = format!("http://{}:{}", config::DEFAULT_HOST, config::DEFAULT_PORT);
        let health_url = format!("{}{}", base_url, config::HEALTH_ENDPOINT);

        Self {
            agent_url: if agent_name == "deterministic" {
                format!("{}?{}", base_url, config::MOCK_PARAM)
            } else {
                base_url.to_string()
            },
            health_url,
            client: Client::builder()
                .timeout(Duration::from_secs(config::REQUEST_TIMEOUT))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Get the transaction generation URL
    pub fn tx_url(&self) -> &str {
        &self.agent_url
    }

    /// Get the health check URL
    pub fn health_check_url(&self) -> &str {
        &self.health_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_config_creation() {
        let config = ExampleConfig::new("deterministic");
        assert!(config.agent_url.contains("mock=true"));

        let config = ExampleConfig::new("ai");
        assert!(!config.agent_url.contains("mock=true"));
    }

    #[test]
    fn test_url_construction() {
        let config = ExampleConfig::new("test");
        assert_eq!(config.health_check_url(), "http://127.0.0.1:9090/health");
        assert_eq!(config.tx_url(), "http://127.0.0.1:9090");
    }
}
