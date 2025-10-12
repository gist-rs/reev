//! Environment variable configuration for reev services
//!
//! This module provides centralized configuration management through environment variables
//! with sensible defaults and validation.

use anyhow::{Context, Result};
use std::env;
use std::str::FromStr;

/// Network configuration from environment variables
pub mod network {
    use super::*;

    /// Get the reev-agent host from environment or use default
    pub fn reev_agent_host() -> String {
        env::var("REEV_AGENT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
    }

    /// Get the reev-agent port from environment or use default
    pub fn reev_agent_port() -> u16 {
        env::var("REEV_AGENT_PORT")
            .ok()
            .and_then(|s| u16::from_str(&s).ok())
            .unwrap_or(9090)
    }

    /// Get the surfpool host from environment or use default
    pub fn surfpool_host() -> String {
        env::var("SURFPOOL_HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
    }

    /// Get the surfpool port from environment or use default
    pub fn surfpool_port() -> u16 {
        env::var("SURFPOOL_PORT")
            .ok()
            .and_then(|s| u16::from_str(&s).ok())
            .unwrap_or(8899)
    }

    /// Get the reev-agent URL constructed from host and port
    pub fn reev_agent_url() -> String {
        format!("http://{}:{}", reev_agent_host(), reev_agent_port())
    }

    /// Get the surfpool RPC URL constructed from host and port
    pub fn surfpool_url() -> String {
        format!("http://{}:{}", surfpool_host(), surfpool_port())
    }
}

/// Timeout configuration from environment variables
pub mod timeouts {
    use super::*;

    /// Get HTTP request timeout in seconds
    pub fn http_request_seconds() -> u64 {
        env::var("HTTP_REQUEST_TIMEOUT_SECONDS")
            .ok()
            .and_then(|s| u64::from_str(&s).ok())
            .unwrap_or(30)
    }

    /// Get health check timeout in seconds
    pub fn health_check_seconds() -> u64 {
        env::var("HEALTH_CHECK_TIMEOUT_SECONDS")
            .ok()
            .and_then(|s| u64::from_str(&s).ok())
            .unwrap_or(5)
    }

    /// Get benchmark timeout in seconds
    pub fn benchmark_timeout_seconds() -> u64 {
        env::var("BENCHMARK_TIMEOUT_SECONDS")
            .ok()
            .and_then(|s| u64::from_str(&s).ok())
            .unwrap_or(300) // 5 minutes
    }
}

/// Agent configuration from environment variables
pub mod agents {
    use super::*;

    /// Get the default agent type
    pub fn default_agent() -> String {
        env::var("DEFAULT_AGENT").unwrap_or_else(|_| "deterministic".to_string())
    }

    /// Get mock parameter for deterministic agent
    pub fn mock_param() -> String {
        env::var("MOCK_PARAM").unwrap_or_else(|_| "mock=true".to_string())
    }

    /// Get whether to enable mock mode
    pub fn enable_mock() -> bool {
        env::var("ENABLE_MOCK")
            .ok()
            .and_then(|s| bool::from_str(&s).ok())
            .unwrap_or(true)
    }
}

/// Logging configuration from environment variables
pub mod logging {
    use super::*;

    /// Get the RUST_LOG filter level
    pub fn rust_log_filter() -> String {
        env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
    }

    /// Get whether to enable debug logging
    pub fn enable_debug() -> bool {
        env::var("DEBUG")
            .ok()
            .and_then(|s| bool::from_str(&s).ok())
            .unwrap_or(false)
    }
}

/// Database configuration from environment variables
pub mod database {
    use super::*;

    /// Get the database file path
    pub fn database_path() -> String {
        env::var("DATABASE_PATH").unwrap_or_else(|_| "db/reev_results.db".to_string())
    }

    /// Get whether to enable database connection pooling
    pub fn enable_connection_pooling() -> bool {
        env::var("DB_ENABLE_POOLING")
            .ok()
            .and_then(|s| bool::from_str(&s).ok())
            .unwrap_or(true)
    }

    /// Get maximum database connections
    pub fn max_connections() -> u32 {
        env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| u32::from_str(&s).ok())
            .unwrap_or(10)
    }
}

/// Solana configuration from environment variables
pub mod solana {
    use super::*;

    /// Get the Solana RPC URL for mainnet
    pub fn mainnet_rpc_url() -> String {
        env::var("SOLANA_MAINNET_RPC")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
    }

    /// Get the Solana RPC URL for devnet
    pub fn devnet_rpc_url() -> String {
        env::var("SOLANA_DEVNET_RPC")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string())
    }

    /// Get commitment level for transactions
    pub fn commitment_level() -> String {
        env::var("SOLANA_COMMITMENT").unwrap_or_else(|_| "confirmed".to_string())
    }

    /// Get preflight commitment level
    pub fn preflight_commitment() -> String {
        env::var("SOLANA_PREFLIGHT_COMMITMENT").unwrap_or_else(|_| "confirmed".to_string())
    }
}

/// LLM configuration from environment variables
pub mod llm {
    use super::*;

    /// Get the Google API key for Gemini
    pub fn google_api_key() -> Result<String> {
        env::var("GOOGLE_API_KEY").context(
            "GOOGLE_API_KEY environment variable not set. \
             Please set it to use Gemini agents.",
        )
    }

    /// Get the local LLM server URL
    pub fn local_server_url() -> String {
        env::var("LOCAL_LLM_URL").unwrap_or_else(|_| "http://localhost:1234".to_string())
    }

    /// Get the maximum number of turns for multi-turn conversations
    pub fn max_turns() -> u32 {
        env::var("MAX_TURNS")
            .ok()
            .and_then(|s| u32::from_str(&s).ok())
            .unwrap_or(5)
    }

    /// Get the temperature for AI model responses
    pub fn temperature() -> f32 {
        env::var("LLM_TEMPERATURE")
            .ok()
            .and_then(|s| f32::from_str(&s).ok())
            .unwrap_or(0.7)
    }
}

/// Load and validate all configuration
pub fn load_config() -> Result<()> {
    // Validate critical configuration
    agents::default_agent();
    network::reev_agent_port();
    network::surfpool_port();

    // Log configuration summary
    tracing::info!("🔧 Configuration loaded:");
    tracing::info!("  reev-agent: {}", network::reev_agent_url());
    tracing::info!("  surfpool: {}", network::surfpool_url());
    tracing::info!("  default agent: {}", agents::default_agent());
    tracing::info!("  log level: {}", logging::rust_log_filter());

    Ok(())
}

/// Validate that required environment variables are set
pub fn validate_required_env() -> Result<()> {
    let mut missing_vars = Vec::new();

    // Check for LLM API keys if using those agents
    if agents::default_agent().contains("gemini") && env::var("GOOGLE_API_KEY").is_err() {
        missing_vars.push("GOOGLE_API_KEY");
    }

    if !missing_vars.is_empty() {
        anyhow::bail!(
            "Missing required environment variables: {}. \
             Please set these variables and try again.",
            missing_vars.join(", ")
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_values() {
        // Test that defaults work when env vars are not set
        assert_eq!(network::reev_agent_host(), "127.0.0.1");
        assert_eq!(network::reev_agent_port(), 9090);
        assert_eq!(network::surfpool_port(), 8899);
        assert_eq!(timeouts::http_request_seconds(), 30);
        assert_eq!(agents::default_agent(), "deterministic");
    }

    #[test]
    fn test_env_override() {
        // Test that environment variables override defaults
        env::set_var("REEV_AGENT_PORT", "9999");
        assert_eq!(network::reev_agent_port(), 9999);
        env::remove_var("REEV_AGENT_PORT");
    }

    #[test]
    fn test_url_construction() {
        env::set_var("REEV_AGENT_HOST", "example.com");
        env::set_var("REEV_AGENT_PORT", "8080");
        assert_eq!(network::reev_agent_url(), "http://example.com:8080");
        env::remove_var("REEV_AGENT_HOST");
        env::remove_var("REEV_AGENT_PORT");
    }

    #[test]
    fn test_invalid_values() {
        // Test that invalid values fall back to defaults
        env::set_var("REEV_AGENT_PORT", "invalid");
        assert_eq!(network::reev_agent_port(), 9090); // Should fallback to default
        env::remove_var("REEV_AGENT_PORT");
    }
}
