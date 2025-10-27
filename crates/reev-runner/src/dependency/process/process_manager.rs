//! Process manager implementation for dependency services
//!
//! This module provides comprehensive process management capabilities including
//! starting, stopping, monitoring, and managing external processes with proper
//! lifecycle handling and error recovery.

use super::{ProcessConfig, ProcessError, ProcessGuard, ProcessShutdownResult, ProcessUtils};
use anyhow::{Context, Result};
use std::collections::HashMap;

use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};

use tracing::{debug, error, info, warn};

/// Process manager for handling external processes
pub struct ProcessManager {
    /// Managed processes by name
    processes: Arc<RwLock<HashMap<String, ProcessGuard>>>,

    /// Process configurations
    configs: Arc<RwLock<HashMap<String, ProcessConfig>>>,

    /// Process startup history
    startup_history: Arc<Mutex<Vec<ProcessStartupRecord>>>,

    /// Whether auto-restart is enabled globally
    auto_restart_enabled: Arc<RwLock<bool>>,

    /// Default timeouts and settings
    #[allow(dead_code)]
    default_startup_timeout: Duration,
    #[allow(dead_code)]
    default_shutdown_timeout: Duration,
}

/// Record of a process startup event
#[derive(Debug, Clone)]
pub struct ProcessStartupRecord {
    pub process_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub success: bool,
    pub pid: Option<u32>,
    pub startup_time: Duration,
    pub error_message: Option<String>,
}

impl ProcessManager {
    /// Create a new process manager
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            startup_history: Arc::new(Mutex::new(Vec::new())),
            auto_restart_enabled: Arc::new(RwLock::new(true)),
            default_startup_timeout: Duration::from_secs(60),
            default_shutdown_timeout: Duration::from_secs(30),
        }
    }

    /// Create a process manager with custom defaults
    pub fn with_timeouts(startup_timeout: Duration, shutdown_timeout: Duration) -> Self {
        Self {
            default_startup_timeout: startup_timeout,
            default_shutdown_timeout: shutdown_timeout,
            ..Self::new()
        }
    }

    /// Start a process with the given configuration
    pub async fn start_process(&self, config: ProcessConfig) -> Result<u32> {
        let process_name = config.name.clone();
        let start_time = std::time::Instant::now();

        info!(process_name = %process_name, "Starting process");

        // Check if process is already running
        if self.is_process_running(&process_name).await? {
            warn!(process_name = %process_name, "Process is already running");
            return Err(ProcessError::StartupError {
                name: process_name,
                source: anyhow::anyhow!("Process is already running"),
            }
            .into());
        }

        // Store configuration
        {
            let mut configs = self.configs.write().await;
            configs.insert(process_name.clone(), config.clone());
        }

        // Validate configuration
        self.validate_config(&config)?;

        // Start the process
        let result = self.start_process_internal(&config).await;
        let startup_time = start_time.elapsed();

        // If startup failed, return error
        let guard = match result {
            Ok(guard) => guard,
            Err(e) => {
                // Record startup attempt
                let startup_record = ProcessStartupRecord {
                    process_name: process_name.clone(),
                    timestamp: chrono::Utc::now(),
                    success: false,
                    pid: None,
                    startup_time,
                    error_message: Some(e.to_string()),
                };

                // Store startup record
                {
                    let mut history = self.startup_history.lock().await;
                    history.push(startup_record);

                    // Keep only last 100 records
                    if history.len() > 100 {
                        history.remove(0);
                    }
                }

                return Err(e);
            }
        };

        let pid = guard.pid().unwrap_or(0);

        // // If health check is configured, wait for health
        // if let Some(health_url) = &config.health_check_url {
        //     if let Err(e) = self
        //         .wait_for_health_check(
        //             &process_name,
        //             health_url,
        //             config.health_check_interval,
        //             config.startup_timeout,
        //         )
        //         .await
        //     {
        //         error!(process_name = %process_name, error = %e, "Health check failed after startup");

        //         // Shutdown the process since it's not healthy
        //         let _ = guard.shutdown().await;
        //         return Err(ProcessError::StartupError {
        //             name: process_name,
        //             source: anyhow::anyhow!("Health check failed: {e}"),
        //         }
        //         .into());
        //     }
        // }

        // Register the process
        {
            let mut processes = self.processes.write().await;
            processes.insert(process_name.clone(), guard);
        }

        // Record successful startup
        let startup_record = ProcessStartupRecord {
            process_name: process_name.clone(),
            timestamp: chrono::Utc::now(),
            success: true,
            pid: Some(pid),
            startup_time,
            error_message: None,
        };

        // Store startup record
        {
            let mut history = self.startup_history.lock().await;
            history.push(startup_record);

            // Keep only last 100 records
            if history.len() > 100 {
                history.remove(0);
            }
        }

        info!(
            process_name = %process_name,
            pid,
            startup_time_ms = startup_time.as_millis(),
            "Process started successfully"
        );

        Ok(pid)
    }

    /// Internal process startup logic
    async fn start_process_internal(&self, config: &ProcessConfig) -> Result<ProcessGuard> {
        let mut command = Command::new(&config.command);

        // Set arguments
        for arg in &config.args {
            command.arg(arg);
        }

        // Set working directory
        if let Some(working_dir) = &config.working_dir {
            command.current_dir(working_dir);
        }

        // Set environment variables
        for (key, value) in &config.env_vars {
            command.env(key, value);
        }

        // Configure stdout/stderr
        if let Some(stdout_path) = &config.stdout {
            // Debug: Check file existence and properties before opening
            let file_exists = std::path::Path::new(stdout_path).exists();
            let file_metadata = std::path::Path::new(stdout_path).metadata().ok();
            let file_size = file_metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            info!(
                "FILE DEBUG: Opening stdout file: {}, exists: {}, size: {}",
                stdout_path.display(),
                file_exists,
                file_size
            );

            let stdout_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(stdout_path)
                .with_context(|| {
                    format!("Failed to open stdout file: {}", stdout_path.display())
                })?;
            command.stdout(Stdio::from(stdout_file));
        } else {
            command.stdout(Stdio::piped());
        }

        if let Some(stderr_path) = &config.stderr {
            // Debug: Check stderr file existence and size before opening
            let stderr_path_obj = std::path::Path::new(stderr_path);
            let stderr_exists = stderr_path_obj.exists();
            let stderr_metadata = stderr_path_obj.metadata().ok();
            let stderr_size = stderr_metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            info!(
                "FILE DEBUG: Opening stderr file: {}, exists: {}, size: {}",
                stderr_path.display(),
                stderr_exists,
                stderr_size
            );

            let stderr_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(stderr_path)
                .with_context(|| {
                    format!("Failed to open stderr file: {}", stderr_path.display())
                })?;
            command.stderr(Stdio::from(stderr_file));
        } else {
            command.stderr(Stdio::piped());
        }

        // Spawn the process
        let child = command
            .spawn()
            .with_context(|| format!("Failed to spawn process: {}", config.command))?;

        let guard = ProcessGuard::new(child, config.name.clone(), config.shutdown_timeout);

        // Wait a brief moment to ensure the process started
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify the process is still running
        if let Some(pid) = guard.pid() {
            if !ProcessUtils::is_process_running(pid)? {
                return Err(ProcessError::StartupError {
                    name: config.name.clone(),
                    source: anyhow::anyhow!("Process exited immediately after startup"),
                }
                .into());
            }
        }

        Ok(guard)
    }

    /// Stop a process by name
    pub async fn stop_process(&self, process_name: &str) -> Result<ProcessShutdownResult> {
        info!(process_name = %process_name, "Stopping process");

        let mut processes = self.processes.write().await;

        if let Some(guard) = processes.remove(process_name) {
            match guard.shutdown().await {
                Ok(_) => {
                    info!(process_name = %process_name, "Process stopped successfully");
                    Ok(ProcessShutdownResult::Stopped)
                }
                Err(e) => {
                    error!(process_name = %process_name, error = %e, "Failed to stop process gracefully");
                    Ok(ProcessShutdownResult::Failed {
                        error: e.to_string(),
                    })
                }
            }
        } else {
            warn!(process_name = %process_name, "Process not found for stopping");
            Ok(ProcessShutdownResult::AlreadyStopped)
        }
    }

    /// Force stop a process by name
    pub async fn force_stop_process(&self, process_name: &str) -> Result<ProcessShutdownResult> {
        info!(process_name = %process_name, "Force stopping process");

        let mut processes = self.processes.write().await;

        if let Some(guard) = processes.remove(process_name) {
            let _ = guard.force_shutdown();
            info!(process_name = %process_name, "Process force stopped");
            Ok(ProcessShutdownResult::Stopped)
        } else {
            warn!(process_name = %process_name, "Process not found for force stopping");
            Ok(ProcessShutdownResult::AlreadyStopped)
        }
    }

    /// Check if a process is running
    pub async fn is_process_running(&self, process_name: &str) -> Result<bool> {
        let processes = self.processes.read().await;

        if let Some(guard) = processes.get(process_name) {
            if let Some(pid) = guard.pid() {
                ProcessUtils::is_process_running(pid)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// Get process PID by name
    pub async fn get_process_pid(&self, process_name: &str) -> Option<u32> {
        let processes = self.processes.read().await;
        processes.get(process_name).and_then(|g| g.pid())
    }

    /// Get all managed process names
    pub async fn get_managed_processes(&self) -> Vec<String> {
        let processes = self.processes.read().await;
        processes.keys().cloned().collect()
    }

    /// Get the number of managed processes
    pub async fn managed_process_count(&self) -> usize {
        let processes = self.processes.read().await;
        processes.len()
    }

    /// Stop all managed processes
    pub async fn stop_all_processes(&self) -> Result<Vec<(String, ProcessShutdownResult)>> {
        info!("Stopping all managed processes");

        let process_names: Vec<String> = {
            let processes = self.processes.read().await;
            processes.keys().cloned().collect()
        };

        let mut results = Vec::new();

        for process_name in process_names {
            let result = self.stop_process(&process_name).await?;
            results.push((process_name, result));
        }

        info!("All processes stop operation completed");
        Ok(results)
    }

    /// Force stop all managed processes
    pub async fn force_stop_all_processes(&self) -> Result<Vec<(String, ProcessShutdownResult)>> {
        info!("Force stopping all managed processes");

        let process_names: Vec<String> = {
            let processes = self.processes.read().await;
            processes.keys().cloned().collect()
        };

        let mut results = Vec::new();

        for process_name in process_names {
            let result = self.force_stop_process(&process_name).await?;
            results.push((process_name, result));
        }

        info!("All processes force stop operation completed");
        Ok(results)
    }

    /// Restart a process
    pub async fn restart_process(&self, process_name: &str) -> Result<u32> {
        info!(process_name = %process_name, "Restarting process");

        // Get the configuration
        let config = {
            let configs = self.configs.read().await;
            configs
                .get(process_name)
                .cloned()
                .ok_or_else(|| ProcessError::NotFound {
                    name: process_name.to_string(),
                })?
        };

        // Stop the existing process
        let _ = self.stop_process(process_name).await;

        // Start the process again
        self.start_process(config).await
    }

    /// Get startup history for a process
    pub async fn get_startup_history(&self, process_name: &str) -> Vec<ProcessStartupRecord> {
        let history = self.startup_history.lock().await;
        history
            .iter()
            .filter(|record| record.process_name == process_name)
            .cloned()
            .collect()
    }

    /// Get all startup history
    pub async fn get_all_startup_history(&self) -> Vec<ProcessStartupRecord> {
        let history = self.startup_history.lock().await;
        history.clone()
    }

    /// Clear startup history
    pub async fn clear_startup_history(&self) {
        let mut history = self.startup_history.lock().await;
        history.clear();
    }

    /// Enable or disable auto-restart globally
    pub async fn set_auto_restart_enabled(&self, enabled: bool) {
        let mut auto_restart = self.auto_restart_enabled.write().await;
        *auto_restart = enabled;
        info!(enabled, "Auto-restart setting updated");
    }

    /// Check if auto-restart is enabled
    pub async fn is_auto_restart_enabled(&self) -> bool {
        *self.auto_restart_enabled.read().await
    }

    /// Validate process configuration
    fn validate_config(&self, config: &ProcessConfig) -> Result<()> {
        if config.name.is_empty() {
            return Err(ProcessError::InvalidConfig {
                message: "Process name cannot be empty".to_string(),
            }
            .into());
        }

        if config.command.is_empty() {
            return Err(ProcessError::InvalidConfig {
                message: "Process command cannot be empty".to_string(),
            }
            .into());
        }

        // Check if working directory exists
        if let Some(working_dir) = &config.working_dir {
            if !working_dir.exists() {
                return Err(ProcessError::InvalidConfig {
                    message: format!(
                        "Working directory does not exist: {}",
                        working_dir.display()
                    ),
                }
                .into());
            }
        }

        // Check if stdout/stderr directories exist
        if let Some(stdout_path) = &config.stdout {
            if let Some(parent) = stdout_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create stdout directory: {}", parent.display())
                    })?;
                }
            }
        }

        if let Some(stderr_path) = &config.stderr {
            if let Some(parent) = stderr_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create stderr directory: {}", parent.display())
                    })?;
                }
            }
        }

        Ok(())
    }

    /// Wait for health check to pass
    #[allow(dead_code)]
    async fn wait_for_health_check(
        &self,
        process_name: &str,
        health_url: &str,
        check_interval: Duration,
        timeout: Duration,
    ) -> Result<()> {
        info!(
            process_name = %process_name,
            health_url = %health_url,
            timeout_secs = timeout.as_secs(),
            "Waiting for health check"
        );

        let start_time = std::time::Instant::now();
        let client = reqwest::Client::new();

        while start_time.elapsed() < timeout {
            match client.get(health_url).send().await {
                Ok(response) if response.status().is_success() => {
                    info!(
                        process_name = %process_name,
                        elapsed_ms = start_time.elapsed().as_millis(),
                        "Health check passed"
                    );
                    return Ok(());
                }
                Ok(response) => {
                    debug!(
                        process_name = %process_name,
                        status = %response.status(),
                        "Health check failed, retrying..."
                    );
                }
                Err(e) => {
                    debug!(
                        process_name = %process_name,
                        error = %e,
                        "Health check request failed, retrying..."
                    );
                }
            }

            tokio::time::sleep(check_interval).await;
        }

        Err(ProcessError::StartupTimeout {
            name: process_name.to_string(),
            timeout_ms: timeout.as_millis() as u64,
        }
        .into())
    }

    /// Get process configuration
    pub async fn get_process_config(&self, process_name: &str) -> Option<ProcessConfig> {
        let configs = self.configs.read().await;
        configs.get(process_name).cloned()
    }

    /// Update process configuration
    pub async fn update_process_config(
        &self,
        process_name: &str,
        config: ProcessConfig,
    ) -> Result<()> {
        self.validate_config(&config)?;

        let mut configs = self.configs.write().await;
        configs.insert(process_name.to_string(), config);

        info!(process_name = %process_name, "Process configuration updated");
        Ok(())
    }

    /// Remove process configuration
    pub async fn remove_process_config(&self, process_name: &str) -> bool {
        let mut configs = self.configs.write().await;
        configs.remove(process_name).is_some()
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
