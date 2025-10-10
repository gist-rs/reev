//! Process management utilities for dependency services
//!
//! This module provides utilities for managing external processes,
//! including startup, shutdown, monitoring, and lifecycle management.

pub mod lifecycle_manager;
pub mod process_manager;

pub use lifecycle_manager::LifecycleManager;
pub use process_manager::ProcessManager;

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Child;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, info, warn};

/// Process information and status
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: Option<u32>,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub status: ProcessStatus,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub exit_code: Option<i32>,
}

/// Process status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessStatus {
    Running,
    Stopped,
    Failed(String),
    Unknown,
}

/// Process startup configuration
#[derive(Debug, Clone)]
pub struct ProcessConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub env_vars: Vec<(String, String)>,
    pub stdout: Option<PathBuf>,
    pub stderr: Option<PathBuf>,
    pub startup_timeout: Duration,
    pub shutdown_timeout: Duration,
    pub auto_restart: bool,
    pub health_check_url: Option<String>,
    pub health_check_interval: Duration,
}

impl ProcessConfig {
    pub fn new(name: String, command: String) -> Self {
        Self {
            name,
            command,
            args: Vec::new(),
            working_dir: None,
            env_vars: Vec::new(),
            stdout: None,
            stderr: None,
            startup_timeout: Duration::from_secs(60),
            shutdown_timeout: Duration::from_secs(30),
            auto_restart: false,
            health_check_url: None,
            health_check_interval: Duration::from_secs(5),
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    pub fn with_env_var(mut self, key: String, value: String) -> Self {
        self.env_vars.push((key, value));
        self
    }

    pub fn with_stdout(mut self, path: PathBuf) -> Self {
        self.stdout = Some(path);
        self
    }

    pub fn with_stderr(mut self, path: PathBuf) -> Self {
        self.stderr = Some(path);
        self
    }

    pub fn with_startup_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = timeout;
        self
    }

    pub fn with_shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = timeout;
        self
    }

    pub fn with_auto_restart(mut self, auto_restart: bool) -> Self {
        self.auto_restart = auto_restart;
        self
    }

    pub fn with_health_check(mut self, url: String) -> Self {
        self.health_check_url = Some(url);
        self
    }

    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }
}

/// Process startup result
#[derive(Debug)]
pub enum ProcessStartupResult {
    Started { pid: u32 },
    AlreadyRunning { pid: u32 },
    Failed { error: String },
}

/// Process shutdown result
#[derive(Debug)]
pub enum ProcessShutdownResult {
    Stopped,
    AlreadyStopped,
    Failed { error: String },
    Timeout,
}

/// Error types for process management
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("Failed to start process '{name}': {source}")]
    StartupError { name: String, source: anyhow::Error },

    #[error("Failed to stop process '{name}' (pid: {pid}): {source}")]
    StopError {
        name: String,
        pid: u32,
        source: anyhow::Error,
    },

    #[error("Process '{name}' startup timed out after {timeout_ms}ms")]
    StartupTimeout { name: String, timeout_ms: u64 },

    #[error("Process '{name}' shutdown timed out after {timeout_ms}ms")]
    ShutdownTimeout { name: String, timeout_ms: u64 },

    #[error("Process '{name}' not found")]
    NotFound { name: String },

    #[error("Invalid process configuration: {message}")]
    InvalidConfig { message: String },

    #[error("Permission denied for process operation on '{name}'")]
    PermissionError { name: String },
}

/// RAII guard for process lifecycle management
pub struct ProcessGuard {
    process: Child,
    name: String,
    shutdown_timeout: Duration,
}

impl ProcessGuard {
    pub fn new(process: Child, name: String, shutdown_timeout: Duration) -> Self {
        Self {
            process,
            name,
            shutdown_timeout,
        }
    }

    pub fn pid(&self) -> Option<u32> {
        Some(self.process.id())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Attempt graceful shutdown
    pub async fn shutdown(mut self) -> Result<()> {
        info!(process_name = %self.name, pid = ?self.process.id(), "Shutting down process");

        let result = timeout(self.shutdown_timeout, async {
            match self.process.kill() {
                Ok(_) => {
                    // Wait for process to actually exit
                    match self.process.wait() {
                        Ok(status) => {
                            info!(
                                process_name = %self.name,
                                pid = ?self.process.id(),
                                ?status,
                                "Process shutdown completed"
                            );
                            Ok(())
                        }
                        Err(e) => Err(anyhow::anyhow!("Failed to wait for process exit: {e}")),
                    }
                }
                Err(e) => Err(anyhow::anyhow!("Failed to kill process: {e}")),
            }
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                warn!(
                    process_name = %self.name,
                    timeout_ms = self.shutdown_timeout.as_millis(),
                    "Process shutdown timed out"
                );
                Err(ProcessError::ShutdownTimeout {
                    name: self.name.clone(),
                    timeout_ms: self.shutdown_timeout.as_millis() as u64,
                }
                .into())
            }
        }
    }

    /// Force immediate shutdown
    pub fn force_shutdown(mut self) -> Result<()> {
        info!(process_name = %self.name, pid = ?self.process.id(), "Force shutting down process");

        match self.process.kill() {
            Ok(_) => {
                info!(
                    process_name = %self.name,
                    pid = ?self.process.id(),
                    "Process force shutdown completed"
                );
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to force kill process: {e}")),
        }
    }
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        if let Err(e) = self.process.kill() {
            error!(
                process_name = %self.name,
                pid = ?self.process.id(),
                error = %e,
                "Failed to kill process during drop"
            );
        }
    }
}

/// Utility functions for process management
pub struct ProcessUtils;

impl ProcessUtils {
    /// Check if a process with given PID is running
    pub fn is_process_running(pid: u32) -> Result<bool> {
        #[cfg(unix)]
        {
            use std::process::Command;

            let output = Command::new("kill").args(["-0", &pid.to_string()]).output();

            match output {
                Ok(result) => Ok(result.status.success()),
                Err(_) => Ok(false), // Command failed, assume process doesn't exist
            }
        }

        #[cfg(not(unix))]
        {
            // On Windows, we would need to use Windows API
            // For now, return false as a placeholder
            Ok(false)
        }
    }

    /// Send signal to process (Unix only)
    #[cfg(unix)]
    pub fn send_signal(pid: u32, signal: libc::c_int) -> Result<()> {
        use std::process::Command;

        let output = Command::new("kill")
            .args([&format!("-{signal}"), &pid.to_string()])
            .output()
            .context("Failed to execute kill command")?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Failed to send signal: {stderr}"))
        }
    }

    /// Send SIGTERM to process for graceful shutdown
    #[cfg(unix)]
    pub fn send_sigterm(pid: u32) -> Result<()> {
        Self::send_signal(pid, libc::SIGTERM)
    }

    /// Send SIGKILL to process for force shutdown
    #[cfg(unix)]
    pub fn send_sigkill(pid: u32) -> Result<()> {
        Self::send_signal(pid, libc::SIGKILL)
    }

    /// Wait for process to exit with timeout
    pub async fn wait_for_exit(pid: u32, timeout: Duration) -> Result<bool> {
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < timeout {
            if !Self::is_process_running(pid)? {
                return Ok(true);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(false)
    }

    /// Get process command line from PID
    pub fn get_process_command_line(pid: u32) -> Result<String> {
        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("ps")
                .args(["-p", &pid.to_string(), "-o", "command="])
                .output()
                .context("Failed to execute ps command")?;

            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }

        #[cfg(target_os = "linux")]
        {
            let output = std::process::Command::new("cat")
                .args([&format!("/proc/{}/cmdline", pid)])
                .output()
                .context("Failed to read /proc cmdline")?;

            let cmdline = String::from_utf8_lossy(&output.stdout);
            Ok(cmdline.replace('\0', " ").trim().to_string())
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Ok(format!("PID {}", pid))
        }
    }

    /// Find process by name
    pub fn find_process_by_name(name: &str) -> Result<Vec<u32>> {
        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("ps")
                .args(["-axo", "pid,command"])
                .output()
                .context("Failed to execute ps command")?;

            let mut pids = Vec::new();
            let output_str = String::from_utf8_lossy(&output.stdout);

            for line in output_str.lines() {
                if line.contains(name) && !line.contains("grep") {
                    // Extract the command part (after PID)
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let command = parts[1];
                        // Check if the executable name matches exactly
                        // This avoids false positives from log files or arguments
                        if command == name || command.ends_with(&format!("/{name}")) {
                            if let Ok(pid) = parts[0].parse::<u32>() {
                                pids.push(pid);
                            }
                        }
                    }
                }
            }

            Ok(pids)
        }

        #[cfg(target_os = "linux")]
        {
            let output = std::process::Command::new("ps")
                .args(["-eo", "pid,command"])
                .output()
                .context("Failed to execute ps command")?;

            let mut pids = Vec::new();
            let output_str = String::from_utf8_lossy(&output.stdout);

            for line in output_str.lines() {
                if line.contains(name) && !line.contains("grep") {
                    // Extract the command part (after PID)
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let command = parts[1];
                        // Check if the executable name matches exactly
                        // This avoids false positives from log files or arguments
                        if command == name || command.ends_with(&format!("/{}", name)) {
                            if let Ok(pid) = parts[0].parse::<u32>() {
                                pids.push(pid);
                            }
                        }
                    }
                }
            }

            Ok(pids)
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Ok(Vec::new())
        }
    }
}
