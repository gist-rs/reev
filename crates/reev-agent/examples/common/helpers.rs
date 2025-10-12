//! Common helpers for example files
//!
//! This module provides shared functionality used by multiple example files
//! to reduce code duplication and ensure consistent behavior.

use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, error, info};

/// Default configuration values for examples
pub mod config {
    /// Default reev-agent host
    pub const DEFAULT_HOST: &str = "127.0.0.1";

    /// Default reev-agent port
    pub const DEFAULT_PORT: u16 = 9090;

    /// Health check endpoint
    pub const HEALTH_ENDPOINT: &str = "/health";

    /// Transaction generation endpoint
    pub const TX_ENDPOINT: &str = "/gen/tx";

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
    pub agent_name: String,
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
            agent_name: agent_name.to_string(),
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

/// Check if the reev-agent service is healthy
pub async fn check_agent_health(config: &ExampleConfig) -> Result<()> {
    info!(
        "🔍 Checking reev-agent health at {}",
        config.health_check_url()
    );

    let response = config
        .client
        .get(config.health_check_url())
        .send()
        .await
        .context("Failed to send health check request")?;

    if response.status().is_success() {
        info!("✅ reev-agent is healthy and ready");
        Ok(())
    } else {
        error!(
            "❌ reev-agent health check failed with status: {}",
            response.status()
        );
        anyhow::bail!("Agent health check failed");
    }
}

/// Generate a transaction request from the agent
pub async fn generate_transaction(
    config: &ExampleConfig,
    benchmark_id: &str,
    prompt: &str,
) -> Result<Value> {
    info!(
        "🚀 Sending transaction generation request to {}",
        config.tx_url()
    );

    let request_body = serde_json::json!({
        "id": benchmark_id,
        "prompt": prompt,
        "context_prompt": format!("You are a helpful blockchain agent assistant. Please help with the following request: {}", prompt)
    });

    debug!(
        "Request body: {}",
        serde_json::to_string_pretty(&request_body)?
    );

    let response = config
        .client
        .post(config.tx_url())
        .json(&request_body)
        .send()
        .await
        .context("Failed to send transaction request")?;

    if response.status().is_success() {
        let response_json: Value = response
            .json()
            .await
            .context("Failed to deserialize agent response")?;

        info!("✅ Agent responded successfully!");
        debug!(
            "Response: {}",
            serde_json::to_string_pretty(&response_json)?
        );

        Ok(response_json)
    } else {
        error!("❌ Agent request failed with status: {}", response.status());
        let error_text = response.text().await.unwrap_or_default();
        error!("Error response: {}", error_text);
        anyhow::bail!("Agent request failed: {}", response.status());
    }
}

/// Run a complete example workflow
pub async fn run_example(agent_name: &str, benchmark_id: &str, prompt: &str) -> Result<Value> {
    let config = ExampleConfig::new(agent_name);

    // Check agent health
    check_agent_health(&config).await?;

    // Generate transaction
    generate_transaction(&config, benchmark_id, prompt).await
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
