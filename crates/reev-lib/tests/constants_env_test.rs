//! Tests for environment variable configuration

use reev_lib::constants::env::{agents, network, timeouts};
use serial_test::serial;
use std::env;

#[test]
#[serial]
fn test_default_values() {
    // Clean up any existing env vars first
    env::remove_var("REEV_AGENT_PORT");
    env::remove_var("REEV_AGENT_HOST");
    env::remove_var("SURFPOOL_PORT");

    // Test that defaults work when env vars are not set
    assert_eq!(network::reev_agent_host(), "127.0.0.1");
    assert_eq!(network::reev_agent_port(), 9090);
    assert_eq!(network::surfpool_port(), 8899);
    assert_eq!(timeouts::http_request_seconds(), 30);
    assert_eq!(agents::default_agent(), "deterministic");
}

#[test]
#[serial]
fn test_env_override() {
    // Test that environment variables override defaults
    env::set_var("REEV_AGENT_PORT", "9999");
    assert_eq!(network::reev_agent_port(), 9999);
    env::remove_var("REEV_AGENT_PORT");
}

#[test]
#[serial]
fn test_url_construction() {
    env::set_var("REEV_AGENT_HOST", "example.com");
    env::set_var("REEV_AGENT_PORT", "8080");
    assert_eq!(network::reev_agent_url(), "http://example.com:8080");
    env::remove_var("REEV_AGENT_HOST");
    env::remove_var("REEV_AGENT_PORT");
}

#[test]
#[serial]
fn test_invalid_values() {
    // Test that invalid values fall back to defaults
    env::set_var("REEV_AGENT_PORT", "invalid");
    assert_eq!(network::reev_agent_port(), 9090); // Should fallback to default
    env::remove_var("REEV_AGENT_PORT");
}
