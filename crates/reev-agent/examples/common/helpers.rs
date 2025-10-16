//! Common helpers for example files
//!
//! This module provides shared functionality used by multiple example files
//! to reduce code duplication and ensure consistent behavior.

use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;
use tracing::info;

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
    pub client: Client,
}

impl ExampleConfig {
    /// Create a new example configuration
    pub fn new(agent_name: &str) -> Self {
        let base_url = format!("http://{}:{}", config::DEFAULT_HOST, config::DEFAULT_PORT);
        let health_url = format!("{}{}", base_url, config::HEALTH_ENDPOINT);
        let tx_url = format!("{}{}", base_url, config::TX_ENDPOINT);

        Self {
            agent_url: if agent_name == "deterministic" {
                format!("{}?{}", tx_url, config::MOCK_PARAM)
            } else {
                tx_url
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

/// Sync benchmarks to database before running examples
///
/// This function ensures the benchmarks table is populated with YML files
/// so that prompt MD5 lookups will work during flow logging.
pub async fn sync_benchmarks_to_database() -> Result<()> {
    info!("🔄 Syncing benchmarks to database...");

    let api_url = "http://127.0.0.1:9090/api/v1/sync";
    let client = Client::new();

    // Wait a bit for the API server to be ready
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Call the sync endpoint
    let response = client
        .post(api_url)
        .send()
        .await
        .context("Failed to call sync endpoint")?;

    if response.status().is_success() {
        let result: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse sync response")?;

        let synced_count = result["synced_count"].as_u64().unwrap_or(0);

        info!(
            "✅ Successfully synced {} benchmarks to database",
            synced_count
        );

        // Assert that benchmarks were actually synced
        assert_benchmarks_exist_in_database().await?;
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        anyhow::bail!("Failed to sync benchmarks: {status} - {error_text}");
    }

    Ok(())
}

/// Verify that benchmarks exist in the database after sync
///
/// This function performs a verification query to ensure the benchmarks
/// table was populated correctly during the sync operation.
async fn assert_benchmarks_exist_in_database() -> Result<()> {
    info!("🔍 Verifying benchmarks exist in database...");

    let api_url = "http://127.0.0.1:9090/api/v1/benchmarks";
    let client = Client::new();

    let response = client
        .get(api_url)
        .send()
        .await
        .context("Failed to query benchmarks")?;

    if response.status().is_success() {
        let benchmarks: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse benchmarks response")?;

        let benchmarks_array = benchmarks
            .as_array()
            .context("Benchmarks response is not an array")?;

        let count = benchmarks_array.len();

        // Assert that we have at least some benchmarks synced
        assert!(
            count > 0,
            "No benchmarks found in database after sync. Expected at least 1 benchmark."
        );

        info!(
            "✅ Database verification passed: {} benchmarks found",
            count
        );
    } else {
        anyhow::bail!("Failed to verify benchmarks in database");
    }

    Ok(())
}
