//! Lifecycle management for dependency services
//!
//! This module provides comprehensive lifecycle management for external services,
//! including graceful shutdown, signal handling, and cleanup procedures.

use super::{ProcessGuard, ProcessUtils};
use crate::dependency::manager::DependencyType;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

/// Lifecycle manager for handling service startup and shutdown
pub struct LifecycleManager {
    /// Managed processes by dependency type
    processes: Arc<RwLock<HashMap<DependencyType, ProcessGuard>>>,

    /// Shutdown signal receivers
    shutdown_signals: Arc<Mutex<Vec<tokio::sync::oneshot::Sender<()>>>>,

    /// Whether graceful shutdown is in progress
    shutting_down: Arc<RwLock<bool>>,

    /// Default timeout for graceful shutdown
    default_shutdown_timeout: Duration,
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new(default_shutdown_timeout: Duration) -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            shutdown_signals: Arc::new(Mutex::new(Vec::new())),
            shutting_down: Arc::new(RwLock::new(false)),
            default_shutdown_timeout,
        }
    }

    /// Register a process for lifecycle management
    pub async fn register_process(
        &self,
        dependency_type: DependencyType,
        process_guard: ProcessGuard,
    ) -> Result<()> {
        let mut processes = self.processes.write().await;
        processes.insert(dependency_type, process_guard);

        info!(
            dependency = ?dependency_type,
            "Registered process for lifecycle management"
        );

        Ok(())
    }

    /// Unregister a process from lifecycle management
    pub async fn unregister_process(&self, dependency_type: &DependencyType) -> Result<bool> {
        let mut processes = self.processes.write().await;
        let removed = processes.remove(dependency_type).is_some();

        if removed {
            info!(
                dependency = ?dependency_type,
                "Unregistered process from lifecycle management"
            );
        }

        Ok(removed)
    }

    /// Perform graceful shutdown of all managed processes
    pub async fn graceful_shutdown(&self) -> Result<()> {
        let mut shutting_down = self.shutting_down.write().await;
        if *shutting_down {
            warn!("Graceful shutdown already in progress");
            return Ok(());
        }
        *shutting_down = true;
        drop(shutting_down);

        info!("Starting graceful shutdown of all managed processes");

        // Send shutdown signals to any registered receivers
        {
            let mut signals = self.shutdown_signals.lock().await;
            for sender in signals.drain(..) {
                let _ = sender.send(());
            }
        }

        // Shutdown processes in reverse order of dependency (surfpool first, then reev-agent)
        let shutdown_order = vec![DependencyType::Surfpool, DependencyType::ReevAgent];
        let mut processes = self.processes.write().await;

        for dependency_type in shutdown_order {
            if let Some(process_guard) = processes.remove(&dependency_type) {
                info!(
                    dependency = ?dependency_type,
                    pid = ?process_guard.pid(),
                    "Shutting down process"
                );

                match process_guard.shutdown().await {
                    Ok(_) => {
                        info!(
                            dependency = ?dependency_type,
                            "Process shutdown completed successfully"
                        );
                    }
                    Err(e) => {
                        error!(
                            dependency = ?dependency_type,
                            error = %e,
                            "Failed to shutdown process gracefully"
                        );
                    }
                }
            }
        }

        info!("Graceful shutdown completed");
        Ok(())
    }

    /// Force immediate shutdown of all managed processes
    pub async fn force_shutdown(&self) -> Result<()> {
        let mut shutting_down = self.shutting_down.write().await;
        *shutting_down = true;
        drop(shutting_down);

        info!("Starting force shutdown of all managed processes");

        // Send shutdown signals
        {
            let mut signals = self.shutdown_signals.lock().await;
            for sender in signals.drain(..) {
                let _ = sender.send(());
            }
        }

        // Force shutdown all processes
        let mut processes = self.processes.write().await;

        for (dependency_type, process_guard) in processes.drain() {
            info!(
                dependency = ?dependency_type,
                pid = ?process_guard.pid(),
                "Force shutting down process"
            );

            if let Err(e) = process_guard.force_shutdown() {
                error!(
                    dependency = ?dependency_type,
                    error = %e,
                    "Failed to force shutdown process"
                );
            }
        }

        info!("Force shutdown completed");
        Ok(())
    }

    /// Shutdown a specific process
    pub async fn shutdown_process(&self, dependency_type: &DependencyType) -> Result<bool> {
        info!(
            dependency = ?dependency_type,
            "Shutting down specific process"
        );

        let mut processes = self.processes.write().await;
        if let Some(process_guard) = processes.remove(dependency_type) {
            match process_guard.shutdown().await {
                Ok(_) => {
                    info!(
                        dependency = ?dependency_type,
                        "Process shutdown completed successfully"
                    );
                    Ok(true)
                }
                Err(e) => {
                    error!(
                        dependency = ?dependency_type,
                        error = %e,
                        "Failed to shutdown process"
                    );
                    Ok(false)
                }
            }
        } else {
            warn!(
                dependency = ?dependency_type,
                "Process not found for shutdown"
            );
            Ok(false)
        }
    }

    /// Force shutdown a specific process
    pub async fn force_shutdown_process(&self, dependency_type: &DependencyType) -> Result<bool> {
        info!(
            dependency = ?dependency_type,
            "Force shutting down specific process"
        );

        let mut processes = self.processes.write().await;
        if let Some(process_guard) = processes.remove(dependency_type) {
            if let Err(e) = process_guard.force_shutdown() {
                error!(
                    dependency = ?dependency_type,
                    error = %e,
                    "Failed to force shutdown process"
                );
                Ok(false)
            } else {
                info!(
                    dependency = ?dependency_type,
                    "Process force shutdown completed"
                );
                Ok(true)
            }
        } else {
            warn!(
                dependency = ?dependency_type,
                "Process not found for force shutdown"
            );
            Ok(false)
        }
    }

    /// Check if a process is managed
    pub async fn is_process_managed(&self, dependency_type: &DependencyType) -> bool {
        let processes = self.processes.read().await;
        processes.contains_key(dependency_type)
    }

    /// Get the number of managed processes
    pub async fn managed_process_count(&self) -> usize {
        let processes = self.processes.read().await;
        processes.len()
    }

    /// Get PIDs of all managed processes
    pub async fn get_managed_pids(&self) -> HashMap<DependencyType, Option<u32>> {
        let processes = self.processes.read().await;
        processes
            .iter()
            .map(|(ty, guard)| (*ty, guard.pid()))
            .collect()
    }

    /// Create a shutdown signal receiver
    pub async fn create_shutdown_signal(&self) -> tokio::sync::oneshot::Receiver<()> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let mut signals = self.shutdown_signals.lock().await;
        signals.push(sender);
        receiver
    }

    /// Check if shutdown is in progress
    pub async fn is_shutting_down(&self) -> bool {
        *self.shutting_down.read().await
    }

    /// Reset shutdown state (for testing)
    pub async fn reset_shutdown_state(&self) {
        let mut shutting_down = self.shutting_down.write().await;
        *shutting_down = false;
    }

    /// Setup signal handlers for graceful shutdown
    pub fn setup_signal_handlers(&self) -> Result<()> {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};

            let processes = self.processes.clone();
            let shutting_down = self.shutting_down.clone();

            // Handle SIGTERM
            let mut sigterm = signal(SignalKind::terminate())?;
            let processes_sigterm = processes.clone();
            let shutting_down_sigterm = shutting_down.clone();
            tokio::spawn(async move {
                if sigterm.recv().await.is_none() {
                    error!("Signal stream for SIGTERM was closed");
                    return;
                }

                warn!("Received SIGTERM, initiating graceful shutdown");

                let mut is_shutting_down = shutting_down_sigterm.write().await;
                if *is_shutting_down {
                    return;
                }
                *is_shutting_down = true;
                drop(is_shutting_down);

                // Perform graceful shutdown
                let mut processes_map = processes_sigterm.write().await;
                for (dependency_type, process_guard) in processes_map.drain() {
                    info!(
                        dependency = ?dependency_type,
                        "Shutting down process due to SIGTERM"
                    );
                    if let Err(e) = process_guard.shutdown().await {
                        error!(
                            dependency = ?dependency_type,
                            error = %e,
                            "Failed to shutdown process during SIGTERM handling"
                        );
                    }
                }
            });

            // Handle SIGINT (Ctrl+C)
            let mut sigint = signal(SignalKind::interrupt())?;
            tokio::spawn(async move {
                if sigint.recv().await.is_none() {
                    error!("Signal stream for SIGINT was closed");
                    return;
                }

                warn!("Received SIGINT, initiating graceful shutdown");

                let mut is_shutting_down = shutting_down.write().await;
                if *is_shutting_down {
                    return;
                }
                *is_shutting_down = true;
                drop(is_shutting_down);

                // Perform graceful shutdown
                let mut processes_map = processes.write().await;
                for (dependency_type, process_guard) in processes_map.drain() {
                    info!(
                        dependency = ?dependency_type,
                        "Shutting down process due to SIGINT"
                    );
                    if let Err(e) = process_guard.shutdown().await {
                        error!(
                            dependency = ?dependency_type,
                            error = %e,
                            "Failed to shutdown process during SIGINT handling"
                        );
                    }
                }
            });

            info!("Signal handlers for SIGTERM and SIGINT installed");
        }

        #[cfg(not(unix))]
        {
            warn!("Signal handling not supported on this platform");
        }

        Ok(())
    }

    /// Perform health check on managed processes
    pub async fn health_check(&self) -> HashMap<DependencyType, ProcessHealth> {
        let processes = self.processes.read().await;
        let mut health_status = HashMap::new();

        for (dependency_type, process_guard) in processes.iter() {
            let health = if let Some(pid) = process_guard.pid() {
                match ProcessUtils::is_process_running(pid) {
                    Ok(true) => ProcessHealth::Healthy,
                    Ok(false) => ProcessHealth::Stopped,
                    Err(_) => ProcessHealth::Unknown,
                }
            } else {
                ProcessHealth::Unknown
            };

            health_status.insert(*dependency_type, health);
        }

        health_status
    }

    /// Get default shutdown timeout
    pub fn default_shutdown_timeout(&self) -> Duration {
        self.default_shutdown_timeout
    }

    /// Set default shutdown timeout
    pub fn set_default_shutdown_timeout(&mut self, timeout: Duration) {
        self.default_shutdown_timeout = timeout;
    }
}

/// Process health status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessHealth {
    Healthy,
    Stopped,
    Unknown,
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}

impl Drop for LifecycleManager {
    fn drop(&mut self) {
        debug!("LifecycleManager dropped");
        // Note: This is a synchronous drop, but cleanup is async
        // In a real implementation, you might want to ensure proper async cleanup
    }
}
