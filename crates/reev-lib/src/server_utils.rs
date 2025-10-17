//! Server utilities for managing reev-agent processes
//!
//! This module provides utilities for managing reev-agent server processes,
//! including cleanup of existing processes on specific ports.

use anyhow::Result;
use std::process::Command;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

/// Kill any existing reev-agent process on the specified port
///
/// This function checks for processes using the specified port and terminates them.
/// It's useful for tests and examples that need to ensure a clean environment.
///
/// # Arguments
/// * `port` - The port number to check and clean up
///
/// # Returns
/// `Ok(())` if cleanup was successful, `Err` if there was an error
///
/// # Examples
/// ```rust
/// use reev_lib::server_utils::kill_existing_reev_agent;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Clean up port 9090 before starting a new server
///     kill_existing_reev_agent(9090).await?;
///     // Now it's safe to start a new server on port 9090
///     Ok(())
/// }
/// ```
pub async fn kill_existing_reev_agent(port: u16) -> Result<()> {
    info!(
        "🧹 Checking for existing reev-agent processes on port {}...",
        port
    );

    // Try to kill any process using the specified port
    match Command::new("lsof")
        .args(["-ti", &format!(":{port}")])
        .output()
    {
        Ok(output) => {
            let pids = String::from_utf8_lossy(&output.stdout);
            if !pids.trim().is_empty() {
                info!("🔪 Found existing reev-agent processes: {}", pids.trim());
                for pid in pids.trim().lines() {
                    match Command::new("kill").args(["-9", pid.trim()]).output() {
                        Ok(_) => {
                            info!("✅ Killed process {}", pid.trim());
                        }
                        Err(e) => {
                            warn!("⚠️  Failed to kill process {}: {}", pid.trim(), e);
                        }
                    }
                }
                // Give processes time to terminate
                sleep(Duration::from_millis(500)).await;
            } else {
                info!("✅ No existing reev-agent processes found on port {}", port);
            }
        }
        Err(e) => {
            warn!(
                "⚠️  Failed to check for existing processes on port {}: {}",
                port, e
            );
        }
    }

    Ok(())
}

/// Kill existing surfpool processes on the specified port
///
/// This function finds and forcefully terminates any processes using the specified port.
/// It's useful for cleaning up leftover surfpool instances before starting new ones.
///
/// # Arguments
/// * `port` - The port to check for surfpool processes (typically 8899)
///
/// # Returns
/// `Ok(())` if cleanup was successful, `Err` if cleanup failed
///
/// # Example
/// ```rust,no_run
/// use reev_lib::server_utils::kill_existing_surfpool;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Kill any existing surfpool processes on port 8899
///     kill_existing_surfpool(8899).await?;
///
///     // Now it's safe to start a new surfpool instance
///     Ok(())
/// }
/// ```
pub async fn kill_existing_surfpool(port: u16) -> Result<()> {
    info!(
        "🧹 Checking for existing surfpool processes on port {}...",
        port
    );

    // Try to kill any process using the specified port
    match Command::new("lsof")
        .args(["-ti", &format!(":{port}")])
        .output()
    {
        Ok(output) => {
            let pids = String::from_utf8_lossy(&output.stdout);
            if !pids.trim().is_empty() {
                info!("🔪 Found existing surfpool processes: {}", pids.trim());
                for pid in pids.trim().lines() {
                    match Command::new("kill").args(["-9", pid.trim()]).output() {
                        Ok(_) => {
                            info!("✅ Killed process {}", pid.trim());
                        }
                        Err(e) => {
                            warn!("⚠️  Failed to kill process {}: {}", pid.trim(), e);
                        }
                    }
                }
                // Give processes time to terminate
                sleep(Duration::from_millis(500)).await;
            } else {
                info!("✅ No existing surfpool processes found on port {}", port);
            }
        }
        Err(e) => {
            warn!(
                "⚠️  Failed to check for existing surfpool processes on port {}: {}",
                port, e
            );
        }
    }

    Ok(())
}

/// Wait for a port to become available
///
/// This function polls the specified port to check if it's available.
/// Useful for waiting for a server to start up.
///
/// # Arguments
/// * `port` - The port to check
/// * `timeout_ms` - Maximum time to wait in milliseconds
///
/// # Returns
/// `Ok(())` if the port becomes available, `Err` if timeout is reached
pub async fn wait_for_port_available(port: u16, timeout_ms: u64) -> Result<()> {
    let start_time = std::time::Instant::now();

    while start_time.elapsed().as_millis() < timeout_ms as u128 {
        match Command::new("lsof")
            .args(["-ti", &format!(":{port}")])
            .output()
        {
            Ok(output) => {
                let pids = String::from_utf8_lossy(&output.stdout);
                if pids.trim().is_empty() {
                    info!("✅ Port {} is now available", port);
                    return Ok(());
                }
            }
            Err(_) => {
                // lsof command failed, assume port is available
                return Ok(());
            }
        }

        sleep(Duration::from_millis(100)).await;
    }

    Err(anyhow::anyhow!(
        "Timeout waiting for port {port} to become available"
    ))
}

/// Check if a port is currently in use
///
/// # Arguments
/// * `port` - The port to check
///
/// # Returns
/// `true` if the port is in use, `false` otherwise
pub async fn is_port_in_use(port: u16) -> bool {
    match Command::new("lsof")
        .args(["-ti", &format!(":{port}")])
        .output()
    {
        Ok(output) => {
            let pids = String::from_utf8_lossy(&output.stdout);
            !pids.trim().is_empty()
        }
        Err(_) => false,
    }
}

/// Kill existing API processes on the specified port
///
/// This function finds and forcefully terminates any processes using the specified port.
/// It's useful for cleaning up leftover API instances before starting new ones.
///
/// # Arguments
/// * `port` - The port to check for API processes (typically 3001)
///
/// # Returns
/// `Ok(())` if cleanup was successful, `Err` if cleanup failed
///
/// # Example
/// ```rust,no_run
/// use reev_lib::server_utils::kill_existing_api;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Kill any existing API processes on port 3001
///     kill_existing_api(3001).await?;
///
///     // Now it's safe to start a new API server
///     Ok(())
/// }
/// ```
pub async fn kill_existing_api(port: u16) -> Result<()> {
    info!("🧹 Checking for existing API processes on port {}...", port);

    // Try to kill any process using the specified port
    match Command::new("lsof")
        .args(["-ti", &format!(":{port}")])
        .output()
    {
        Ok(output) => {
            let pids = String::from_utf8_lossy(&output.stdout);
            if !pids.trim().is_empty() {
                info!("🔪 Found existing API processes: {}", pids.trim());
                for pid in pids.trim().lines() {
                    match Command::new("kill").args(["-9", pid.trim()]).output() {
                        Ok(_) => {
                            info!("✅ Killed process {}", pid.trim());
                        }
                        Err(e) => {
                            warn!("⚠️  Failed to kill process {}: {}", pid.trim(), e);
                        }
                    }
                }
                // Give processes time to terminate
                sleep(Duration::from_millis(500)).await;
            } else {
                info!("✅ No existing API processes found on port {}", port);
            }
        }
        Err(e) => {
            warn!(
                "⚠️  Failed to check for existing API processes on port {}: {}",
                port, e
            );
        }
    }

    Ok(())
}
