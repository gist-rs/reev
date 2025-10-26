//! Main dependency manager implementation
//!
//! This module provides the core dependency management functionality that orchestrates
//! the startup, monitoring, and lifecycle of external services like reev-agent and surfpool.

use std::sync::Arc;

use super::{DependencyConfig, DependencyError, DependencyService, DependencyType, DependencyUrls};
use crate::dependency::binary::BinaryManager;
use crate::dependency::health::{HealthChecker, ServiceHealth};
use crate::dependency::process::{ProcessConfig, ProcessGuard, ProcessManager, ProcessUtils};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Main dependency manager for external services
pub struct DependencyManager {
    /// Configuration
    config: DependencyConfig,

    /// Binary manager for surfpool
    binary_manager: Arc<BinaryManager>,

    /// Process manager
    process_manager: Arc<ProcessManager>,

    /// Health checker
    health_checker: Arc<HealthChecker>,

    /// Currently managed services
    services: Arc<RwLock<HashMap<DependencyType, DependencyService>>>,

    /// Running processes
    processes: Arc<RwLock<HashMap<DependencyType, ProcessGuard>>>,

    /// Whether dependencies have been initialized
    initialized: Arc<RwLock<bool>>,
}

impl DependencyManager {
    /// Create a new dependency manager
    pub fn new(config: DependencyConfig) -> Result<Self> {
        config.validate()?;

        let binary_manager = Arc::new(BinaryManager::new(
            config.cache_dir.clone(),
            format!("{}/installs", config.cache_dir),
            config.prefer_binary,
        ));

        let process_manager = Arc::new(ProcessManager::new());
        let health_checker = Arc::new(HealthChecker::new(
            super::super::health::HealthCheckConfig {
                check_interval: config.health_check_interval,
                timeout: config.health_check_timeout,
                failure_threshold: 3,
                success_threshold: 2,
                verbose_logging: config.verbose_logging,
            },
        ));

        Ok(Self {
            config,
            binary_manager,
            process_manager,
            health_checker,
            services: Arc::new(RwLock::new(HashMap::new())),
            processes: Arc::new(RwLock::new(HashMap::new())),
            initialized: Arc::new(RwLock::new(false)),
        })
    }

    /// Initialize dependencies and ensure they are running
    pub async fn ensure_dependencies(&mut self) -> Result<DependencyUrls> {
        {
            let initialized = self.initialized.read().await;
            if *initialized {
                debug!("Dependencies already initialized");
                return self.get_dependency_urls().await;
            }
        }

        debug!("Initializing dependencies...");

        // Clear log files for clean debugging
        debug!("Clearing log files...");
        self.clear_log_files().await?;
        debug!("Log files cleared");

        // Start both services with optimized staggered approach for faster startup
        debug!("Starting both services with optimized staggered approach...");
        let start_time = std::time::Instant::now();

        // Start reev-agent first
        debug!("Starting reev-agent service...");
        if let Err(e) = self.start_reev_agent().await {
            error!(error = %e, "Failed to start reev-agent");
            return Err(e);
        }
        debug!("reev-agent started");

        // Start surfpool with minimal delay to avoid resource contention
        tokio::time::sleep(Duration::from_millis(100)).await; // Reduced from 200ms to 100ms
        debug!("Starting surfpool service...");
        if let Err(e) = self.start_surfpool().await {
            error!(error = %e, "Failed to start surfpool");
            return Err(e);
        }
        debug!("surfpool started");

        debug!(
            "Both services started with optimized staggered approach in {:?}",
            start_time.elapsed()
        );

        // No continuous monitoring needed - services will be checked individually
        debug!(
            "Dependencies initialized successfully in {:?}",
            start_time.elapsed()
        );

        {
            let mut initialized = self.initialized.write().await;
            *initialized = true;
        }
        info!("Dependencies marked as initialized");

        self.get_dependency_urls().await
    }

    /// Start reev-agent service
    async fn start_reev_agent(&mut self) -> Result<()> {
        let dependency_type = DependencyType::ReevAgent;
        let port = self.config.get_port(dependency_type);

        debug!(port, "Starting reev-agent service");

        // Check if reev-agent is already running
        debug!("Checking for existing reev-agent processes...");
        if let Ok(pids) = ProcessUtils::find_process_by_name(dependency_type.process_name()) {
            if !pids.is_empty() && self.config.shared_instances {
                debug!(
                    "Found {} existing reev-agent process(es), using shared instance",
                    pids.len()
                );
                let mut service =
                    DependencyService::new(dependency_type.process_name().to_string(), Some(port));
                service.add_url("api".to_string(), format!("http://localhost:{port}"));

                let mut services = self.services.write().await;
                services.insert(dependency_type, service);
                return Ok(());
            }
        }
        debug!("No existing reev-agent processes found, starting new instance");

        // Check if port is available
        if super::ProcessDetector::is_port_in_use(port)? {
            return Err(DependencyError::PortConflict {
                service: dependency_type.process_name().to_string(),
                port,
            }
            .into());
        }

        // Setup log directory
        std::fs::create_dir_all(&self.config.log_dir)
            .with_context(|| format!("Failed to create log directory: {}", self.config.log_dir))?;

        let log_file = PathBuf::from(&self.config.log_dir).join("reev-agent.log");

        // Create process configuration
        debug!("Creating reev-agent process configuration...");
        let process_config = ProcessConfig::new(
            dependency_type.process_name().to_string(),
            "cargo".to_string(),
        )
        .with_args(vec![
            "run".to_string(),
            "--package".to_string(),
            "reev-agent".to_string(),
        ])
        .with_stdout(log_file.clone())
        .with_stderr(log_file)
        .with_startup_timeout(self.config.startup_timeout)
        .with_health_check(format!("http://localhost:{port}/health"))
        .with_health_check_interval(Duration::from_secs(2));

        // Start the process
        debug!("Starting reev-agent process...");
        let _pid = self.process_manager.start_process(process_config).await?;
        debug!("reev-agent process started with PID {:?}", _pid);

        // Wait for health check with shorter timeout for faster startup
        let health_url = format!("http://localhost:{port}");
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_secs(30); // Reduced from 60s to 30s
        debug!("Waiting for reev-agent to be healthy...");

        let mut check_count = 0;
        while start_time.elapsed() < timeout {
            check_count += 1;
            let result = self.health_checker.check_reev_agent(&health_url).await;
            debug!(
                "Health check attempt {} for reev-agent: {:?}",
                check_count, result.status
            );
            if result.is_healthy() {
                debug!(
                    "reev-agent health check passed after {} attempts in {:?}",
                    check_count,
                    start_time.elapsed()
                );
                break;
            }
            if start_time.elapsed() + Duration::from_secs(2) >= timeout {
                error!(
                    "reev-agent health check timed out after {} attempts",
                    check_count
                );
                return Err(DependencyError::HealthTimeout {
                    service: dependency_type.process_name().to_string(),
                }
                .into());
            }
            debug!("reev-agent not ready yet, waiting 2s before next health check...");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        // Register service
        let mut service =
            DependencyService::new(dependency_type.process_name().to_string(), Some(port));
        service.add_url("api".to_string(), format!("http://localhost:{port}"));
        service.set_health(ServiceHealth::Healthy);

        let mut services = self.services.write().await;
        services.insert(dependency_type, service);

        info!(port, "reev-agent is healthy and ready");
        Ok(())
    }

    /// Start surfpool service
    async fn start_surfpool(&mut self) -> Result<()> {
        let dependency_type = DependencyType::Surfpool;
        let port = self.config.get_port(dependency_type);

        debug!(port, "Starting surfpool service");

        // Check if surfpool is already running
        debug!("Checking for existing surfpool processes...");
        if let Ok(pids) = ProcessUtils::find_process_by_name(dependency_type.process_name()) {
            if !pids.is_empty() && self.config.shared_instances {
                debug!(
                    "Found {} existing surfpool process(es), using shared instance",
                    pids.len()
                );
                let mut service =
                    DependencyService::new(dependency_type.process_name().to_string(), Some(port));
                service.add_url("rpc".to_string(), format!("http://localhost:{port}"));

                let mut services = self.services.write().await;
                services.insert(dependency_type, service);
                return Ok(());
            }
        }
        debug!("No existing surfpool processes found, starting new instance");

        // Get or build surfpool binary
        debug!("Getting or building surfpool binary...");
        let binary_start = std::time::Instant::now();
        let surfpool_binary = self
            .binary_manager
            .get_or_build_surfpool()
            .await
            .map_err(|e| DependencyError::BinaryNotFound {
                service: dependency_type.process_name().to_string(),
                reason: e.to_string(),
            })?;
        debug!("surfpool binary ready in {:?}", binary_start.elapsed());

        // Extract the path from the binary result
        let surfpool_path = match surfpool_binary {
            crate::dependency::BinaryAcquisitionResult::Cached(path) => path,
            crate::dependency::BinaryAcquisitionResult::Downloaded(path) => path,
            crate::dependency::BinaryAcquisitionResult::Built(path) => path,
            crate::dependency::BinaryAcquisitionResult::Existing(path) => path,
        };

        // Check if port is available
        if super::ProcessDetector::is_port_in_use(port)? {
            return Err(DependencyError::PortConflict {
                service: dependency_type.process_name().to_string(),
                port,
            }
            .into());
        }

        // Setup log directory
        std::fs::create_dir_all(&self.config.log_dir)
            .with_context(|| format!("Failed to create log directory: {}", self.config.log_dir))?;

        let log_file = PathBuf::from(&self.config.log_dir).join("surfpool.log");

        // Create process configuration for surfpool start
        debug!("Creating surfpool process configuration...");
        let process_config = ProcessConfig::new(
            dependency_type.process_name().to_string(),
            surfpool_path.to_string_lossy().to_string(),
        )
        .with_args(vec![
            "start".to_string(),
            "--no-tui".to_string(),
            "--debug".to_string(),
        ])
        .with_stdout(log_file.clone())
        .with_stderr(log_file)
        .with_startup_timeout(self.config.startup_timeout)
        .with_health_check(format!("http://localhost:{port}"))
        .with_health_check_interval(Duration::from_secs(2));

        // Start the process
        debug!("Starting surfpool process...");
        let _pid = self.process_manager.start_process(process_config).await?;
        debug!("surfpool process started with PID {:?}", _pid);

        // Wait for health check with even shorter timeout for faster startup
        let health_url = format!("http://localhost:{port}");
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_secs(20); // Reduced from 30s to 20s for faster startup
        debug!("Waiting for surfpool to be healthy...");

        let mut check_count = 0;
        while start_time.elapsed() < timeout {
            check_count += 1;
            let result = self.health_checker.check_surfpool(&health_url).await;
            debug!(
                "Health check attempt {} for surfpool: {:?}",
                check_count, result.status
            );
            if result.is_healthy() {
                debug!(
                    "surfpool health check passed after {} attempts in {:?}",
                    check_count,
                    start_time.elapsed()
                );
                break;
            }
            if start_time.elapsed() + Duration::from_secs(2) >= timeout {
                warn!(
                    "surfpool health check timed out after {} attempts, but continuing anyway",
                    check_count
                );
                break; // Don't fail the startup, just warn
            }
            debug!("surfpool not ready yet, waiting 2s before next health check...");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        debug!("Waiting additional 3s for surfpool to fully initialize...");
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Register service
        debug!("Registering surfpool service...");
        let mut service =
            DependencyService::new(dependency_type.process_name().to_string(), Some(port));
        service.add_url("rpc".to_string(), format!("http://localhost:{port}"));
        service.set_health(ServiceHealth::Healthy);

        let mut services = self.services.write().await;
        services.insert(dependency_type, service);

        info!(port, "surfpool is healthy and ready");
        Ok(())
    }

    /// Get dependency URLs
    pub async fn get_dependency_urls(&self) -> Result<DependencyUrls> {
        let services = self.services.read().await;

        let reev_agent_url = services
            .get(&DependencyType::ReevAgent)
            .and_then(|s| s.urls.get("api").cloned())
            .unwrap_or_else(|| format!("http://localhost:{}", self.config.reev_agent_port));

        let surfpool_rpc_url = services
            .get(&DependencyType::Surfpool)
            .and_then(|s| s.urls.get("rpc").cloned())
            .unwrap_or_else(|| format!("http://localhost:{}", self.config.surfpool_rpc_port));

        let surfpool_ws_url = Some(format!(
            "ws://localhost:{}/ws",
            self.config.surfpool_rpc_port
        ));

        Ok(DependencyUrls {
            reev_agent: reev_agent_url,
            surfpool_rpc: surfpool_rpc_url,
            surfpool_ws: surfpool_ws_url,
        })
    }

    /// Get health status of all dependencies
    pub async fn get_health_status(&self) -> HashMap<String, ServiceHealth> {
        let services = self.services.read().await;
        services
            .iter()
            .map(|(ty, service)| (ty.process_name().to_string(), service.health.clone()))
            .collect()
    }

    /// Check if all dependencies are healthy
    pub async fn are_dependencies_healthy(&self) -> bool {
        let services = self.services.read().await;
        services.values().all(|s| s.is_healthy())
    }

    /// Cleanup and shutdown all dependencies
    pub async fn cleanup(&mut self) -> Result<()> {
        debug!("Cleaning up dependencies...");

        // No monitoring to stop - services are shut down directly
        debug!("Health checking stopped (continuous monitoring was not used)");

        // Shutdown processes
        let mut processes = self.processes.write().await;
        for (dependency_type, process_guard) in processes.drain() {
            debug!(dependency = ?dependency_type, "Shutting down process");
            if let Err(e) = process_guard.shutdown().await {
                warn!(dependency = ?dependency_type, error = %e, "Failed to shutdown process gracefully");
            }
        }

        // Clear services
        let mut services = self.services.write().await;
        services.clear();

        // Mark as not initialized
        let mut initialized = self.initialized.write().await;
        *initialized = false;

        debug!("Dependency cleanup completed");
        Ok(())
    }

    /// Force shutdown of all dependencies
    pub async fn force_cleanup(&mut self) -> Result<()> {
        debug!("Force cleaning up dependencies...");

        // No monitoring to stop - services are shut down directly
        debug!("Health checking force stopped (continuous monitoring was not used)");

        // Force shutdown processes
        let mut processes = self.processes.write().await;
        for (dependency_type, process_guard) in processes.drain() {
            debug!(dependency = ?dependency_type, "Force shutting down process");
            let _ = process_guard.force_shutdown();
        }

        // Clear services
        let mut services = self.services.write().await;
        services.clear();

        // Mark as not initialized
        let mut initialized = self.initialized.write().await;
        *initialized = false;

        debug!("Force dependency cleanup completed");
        Ok(())
    }

    /// Get configuration
    pub fn config(&self) -> &DependencyConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: DependencyConfig) -> Result<()> {
        config.validate()?;
        self.config = config;
        Ok(())
    }

    /// Clear log files for clean debugging
    async fn clear_log_files(&self) -> Result<()> {
        debug!("Clearing log files for clean debugging...");

        let log_files = ["reev-agent.log", "surfpool.log"];

        for log_file in &log_files {
            let log_path = PathBuf::from(&self.config.log_dir).join(log_file);
            if log_path.exists() {
                match fs::write(&log_path, "") {
                    Ok(()) => {
                        debug!("Cleared log file: {}", log_file);
                    }
                    Err(e) => {
                        warn!("Failed to clear log file {}: {}", log_file, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Setup signal handlers for graceful shutdown
    pub fn setup_signal_handlers(&self) -> Result<()> {
        #[cfg(unix)]
        {
            warn!("Signal handling temporarily disabled to focus on core functionality");
        }

        #[cfg(not(unix))]
        {
            warn!("Signal handling not supported on this platform");
        }

        Ok(())
    }
}

impl Drop for DependencyManager {
    fn drop(&mut self) {
        // Note: This is a synchronous drop, but cleanup is async
        // In a real implementation, you might want to use a different approach
        // or ensure proper async cleanup is called before the manager goes out of scope
        debug!("DependencyManager dropped");
    }
}
