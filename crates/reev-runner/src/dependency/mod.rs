//! Dependency management for reev-runner
//!
//! This module provides intelligent dependency management for external services
//! like reev-agent and surfpool, with automatic startup, health monitoring,
//! binary caching, and lifecycle management.
//!
//! # Features
//!
//! - **Automatic Service Management**: Start, monitor, and stop external services
//! - **Health Checking**: Continuous health monitoring with configurable intervals
//! - **Binary Caching**: Download and cache pre-built binaries for faster startup
//! - **Process Lifecycle**: Graceful shutdown, restart capabilities, and signal handling
//! - **Shared Instances**: Allow multiple runner processes to share service instances
//! - **Zero-Setup Experience**: Automatic dependency resolution and configuration
//!
//! # Architecture
//!
//! The dependency management system is organized into several key components:
//!
//! - [`manager`]: Core dependency management and orchestration
//! - [`binary`]: Binary download, caching, and management
//! - [`health`]: Health checking and monitoring
//! - [`process`]: Process management and lifecycle
//!
//! # Example Usage
//!
//! ```rust
//! use reev_runner::dependency::DependencyManager;
//! use reev_runner::dependency::manager::DependencyConfig;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = DependencyConfig::from_env();
//!     let mut manager = DependencyManager::new(config)?;
//!
//!     // Ensure all dependencies are running
//!     let urls = manager.ensure_dependencies().await?;
//!
//!     println!("reev-agent: {}", urls.reev_agent);
//!     println!("surfpool: {}", urls.surfpool_rpc);
//!
//!     // Use the services...
//!
//!     // Cleanup when done
//!     manager.cleanup().await?;
//!     Ok(())
//! }
//! ```

pub mod binary;
pub mod health;
pub mod manager;
pub mod process;

// re-export main types for convenience
pub use binary::{BinaryAcquisitionResult, BinaryManager, Platform, Version};
pub use health::{HealthCheckConfig, HealthCheckResult, HealthChecker};
pub use manager::{
    DependencyConfig, DependencyManager, DependencyService, DependencyType, DependencyUrls,
    ServiceHealth,
};
pub use process::{LifecycleManager, ProcessConfig, ProcessGuard, ProcessManager};

use anyhow::Result;
use tracing::{info, warn};

/// Initialize dependency management system with default configuration
pub async fn init_default() -> Result<DependencyManager> {
    let config = DependencyConfig::from_env();
    init_with_config(config).await
}

/// Initialize dependency management system with custom configuration
pub async fn init_with_config(config: DependencyConfig) -> Result<DependencyManager> {
    info!("Initializing dependency management system");

    let manager = DependencyManager::new(config)?;

    // Set up signal handlers for graceful shutdown
    manager.setup_signal_handlers()?;

    info!("Dependency management system initialized");
    Ok(manager)
}

/// Quick start function that ensures dependencies are running
pub async fn quick_start() -> Result<DependencyUrls> {
    let mut manager = init_default().await?;
    manager.ensure_dependencies().await
}

/// Check if required dependencies are available and healthy
pub async fn check_dependencies() -> Result<bool> {
    let manager = init_default().await?;
    Ok(manager.are_dependencies_healthy().await)
}

/// Clean up any existing dependency processes
pub async fn cleanup_existing() -> Result<()> {
    warn!("Cleaning up existing dependency processes");

    let config = DependencyConfig::from_env();
    let mut manager = DependencyManager::new(config)?;

    // Force cleanup any existing processes
    manager.force_cleanup().await?;

    info!("Existing dependency processes cleaned up");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_init_default() {
        let result = init_default().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_init_with_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = DependencyConfig {
            cache_dir: temp_dir.path().join("cache").to_string_lossy().to_string(),
            log_dir: temp_dir.path().join("logs").to_string_lossy().to_string(),
            ..Default::default()
        };

        let result = init_with_config(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dependency_types() {
        assert_eq!(DependencyType::ReevAgent.default_port(), 9090);
        assert_eq!(DependencyType::Surfpool.default_port(), 8899);
        assert_eq!(DependencyType::ReevAgent.process_name(), "reev-agent");
        assert_eq!(DependencyType::Surfpool.process_name(), "surfpool");
    }

    #[tokio::test]
    async fn test_dependency_urls() {
        let urls = DependencyUrls {
            reev_agent: "http://localhost:9090".to_string(),
            surfpool_rpc: "http://localhost:8899".to_string(),
            surfpool_ws: Some("ws://localhost:8899/ws".to_string()),
        };

        assert_eq!(urls.reev_agent, "http://localhost:9090");
        assert_eq!(urls.surfpool_rpc, "http://localhost:8899");
        assert_eq!(urls.surfpool_ws, Some("ws://localhost:8899/ws".to_string()));
    }
}
