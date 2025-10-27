//! Test for reev-agent restart logic
//!
//! This test ensures that reev-agent processes are properly managed
//! when running multiple benchmarks sequentially.

use anyhow::Result;
use reev_runner::dependency::manager::ProcessDetector;
use reev_runner::dependency::manager::{DependencyConfig, DependencyManager, DependencyType};
use std::time::Duration;
use tokio::time::sleep;

/// Test that reev-agent is not restarted unnecessarily
#[tokio::test]
async fn test_reev_agent_reuse_existing_process() -> Result<()> {
    // Create dependency manager with shared instances enabled
    let mut manager = DependencyManager::new(DependencyConfig {
        shared_instances: true,
        ..Default::default()
    })?;

    // Start surfpool (required for reev-agent)
    manager.ensure_dependencies().await?;

    // Start reev-agent with first config
    manager
        .update_config_and_restart_agent(
            Some("deterministic".to_string()),
            Some("test-1".to_string()),
        )
        .await?;

    // Get the health status
    let health_status = manager.get_health_status().await;
    assert!(health_status.get("reev-agent").map_or(false, |h| matches!(
        h,
        reev_runner::dependency::health::ServiceHealth::Healthy
    )));

    // Try to restart with same config - should not restart
    let port = manager.config().get_port(DependencyType::ReevAgent);
    let is_port_in_use_before = ProcessDetector::is_port_in_use(port)?;

    manager
        .update_config_and_restart_agent(
            Some("deterministic".to_string()),
            Some("test-1".to_string()),
        )
        .await?;

    let is_port_in_use_after = ProcessDetector::is_port_in_use(port)?;

    // Port should still be in use (same process)
    assert!(is_port_in_use_before);
    assert!(is_port_in_use_after);

    // Health should still be good
    let health_status = manager.get_health_status().await;
    assert!(health_status.get("reev-agent").map_or(false, |h| matches!(
        h,
        reev_runner::dependency::health::ServiceHealth::Healthy
    )));

    // Cleanup
    manager.cleanup().await?;

    Ok(())
}

/// Test that reev-agent is restarted when config changes
#[tokio::test]
async fn test_reev_agent_restart_on_config_change() -> Result<()> {
    // Create dependency manager
    let mut manager = DependencyManager::new(DependencyConfig {
        shared_instances: true,
        ..Default::default()
    })?;

    // Start surfpool
    manager.ensure_dependencies().await?;

    // Start reev-agent with first config
    manager
        .update_config_and_restart_agent(
            Some("deterministic".to_string()),
            Some("test-1".to_string()),
        )
        .await?;

    // Get initial process info
    let health_status = manager.get_health_status().await;
    assert!(health_status.get("reev-agent").map_or(false, |h| matches!(
        h,
        reev_runner::dependency::health::ServiceHealth::Healthy
    )));

    // Change config - should restart
    manager
        .update_config_and_restart_agent(Some("glm-4.6".to_string()), Some("test-2".to_string()))
        .await?;

    // Should still be healthy after restart
    let health_status = manager.get_health_status().await;
    assert!(health_status.get("reev-agent").map_or(false, |h| matches!(
        h,
        reev_runner::dependency::health::ServiceHealth::Healthy
    )));

    // Cleanup
    manager.cleanup().await?;

    Ok(())
}

/// Test port release after stop
#[tokio::test]
async fn test_port_released_after_stop() -> Result<()> {
    let mut manager = DependencyManager::new(DependencyConfig {
        shared_instances: false, // Not shared to test proper stop/start
        ..Default::default()
    })?;

    // Start surfpool
    manager.ensure_dependencies().await?;

    // Start reev-agent
    manager
        .update_config_and_restart_agent(
            Some("deterministic".to_string()),
            Some("test-1".to_string()),
        )
        .await?;

    let port = manager.config().get_port(DependencyType::ReevAgent);
    assert!(ProcessDetector::is_port_in_use(port)?);

    // Stop reev-agent
    manager.stop_reev_agent().await?;

    // Wait a bit for process to fully terminate
    sleep(Duration::from_millis(1000)).await;

    // Port should be released (with retries)
    let mut port_released = false;
    for _ in 0..10 {
        if !ProcessDetector::is_port_in_use(port)? {
            port_released = true;
            break;
        }
        sleep(Duration::from_millis(500)).await;
    }

    assert!(port_released, "Port should be released after stop");

    // Cleanup
    manager.cleanup().await?;

    Ok(())
}
