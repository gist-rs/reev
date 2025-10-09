//! Process detection utilities for dependency management

use anyhow::{Context, Result};
use std::process::{Command, Output};
use tracing::{debug, info, warn};

/// Utilities for detecting running processes
pub struct ProcessDetector;

impl ProcessDetector {
    /// Check if a process with the given name is running
    pub fn is_process_running(process_name: &str) -> Result<bool> {
        info!(process_name, "Checking if process is running");

        let output = get_process_list_output()?;
        let process_lines = String::from_utf8_lossy(&output.stdout);

        let is_running = process_lines
            .lines()
            .any(|line| line.contains(process_name) && !line.contains("grep"));

        debug!(
            process_name,
            is_running, "Process running status check completed"
        );

        Ok(is_running)
    }

    /// Find the PID of a process by name
    pub fn find_process_pid(process_name: &str) -> Option<u32> {
        info!(process_name, "Searching for process PID");

        match get_process_list_output() {
            Ok(output) => {
                let process_lines = String::from_utf8_lossy(&output.stdout);

                for line in process_lines.lines() {
                    if line.contains(process_name) && !line.contains("grep") {
                        // Extract PID from ps output (first column)
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if let Some(pid_str) = parts.first() {
                            if let Ok(pid) = pid_str.parse::<u32>() {
                                info!(process_name, pid, "Found process PID");
                                return Some(pid);
                            }
                        }
                    }
                }

                warn!(process_name, "Process PID not found");
                None
            }
            Err(e) => {
                warn!(process_name, error = ?e, "Failed to get process list");
                None
            }
        }
    }

    /// Get all ports used by a specific process
    pub fn get_process_ports(pid: u32) -> Result<Vec<u16>> {
        info!(pid, "Getting ports used by process");

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("lsof")
                .args(["-i", "-P", "-n", &format!("{pid}")])
                .output()
                .context("Failed to execute lsof command")?;

            let mut ports = Vec::new();
            let output_str = String::from_utf8_lossy(&output.stdout);

            for line in output_str.lines() {
                if line.contains("LISTEN") {
                    // Extract port from line like: "node    12345 user  4u  IPv6 0x... 0t0 TCP *:3000 (LISTEN)"
                    if let Some(port_part) = line.split(':').nth(1) {
                        if let Some(port_str) = port_part.split_whitespace().next() {
                            if let Ok(port) = port_str.parse::<u16>() {
                                ports.push(port);
                            }
                        }
                    }
                }
            }

            debug!(pid, ports = ?ports, "Found process ports");
            Ok(ports)
        }

        #[cfg(target_os = "linux")]
        {
            let output = Command::new("netstat")
                .args(["-tlnp"])
                .output()
                .context("Failed to execute netstat command")?;

            let mut ports = Vec::new();
            let output_str = String::from_utf8_lossy(&output.stdout);
            let pid_str = format!("{}", pid);

            for line in output_str.lines() {
                if line.contains(&pid_str) && line.contains("LISTEN") {
                    // Extract port from line like: "tcp        0      0 0.0.0.0:3000            0.0.0.0:*               LISTEN      12345/node"
                    if let Some(address_part) = line.split_whitespace().nth(3) {
                        if let Some(port_str) = address_part.split(':').nth(1) {
                            if let Ok(port) = port_str.parse::<u16>() {
                                ports.push(port);
                            }
                        }
                    }
                }
            }

            debug!(pid, ports = ?ports, "Found process ports");
            Ok(ports)
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            warn!("Port detection not supported on this platform");
            Ok(Vec::new())
        }
    }

    /// Check if a specific port is in use
    pub fn is_port_in_use(port: u16) -> Result<bool> {
        info!(port, "Checking if port is in use");

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("lsof")
                .args(["-i", &format!(":{port}")])
                .output()
                .context("Failed to execute lsof command")?;

            let is_in_use = !output.stdout.is_empty();
            debug!(port, is_in_use, "Port usage check completed");
            Ok(is_in_use)
        }

        #[cfg(target_os = "linux")]
        {
            let output = Command::new("netstat")
                .args(["-tln", &format!(":{}", port)])
                .output()
                .context("Failed to execute netstat command")?;

            let is_in_use = !output.stdout.is_empty();
            debug!(port, is_in_use, "Port usage check completed");
            Ok(is_in_use)
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            warn!("Port detection not supported on this platform, assuming port is not in use");
            Ok(false)
        }
    }

    /// Get process command line by PID
    pub fn get_process_command_line(pid: u32) -> Result<String> {
        info!(pid, "Getting process command line");

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("ps")
                .args(["-p", &format!("{pid}"), "-o", "command="])
                .output()
                .context("Failed to execute ps command")?;

            let command_line = String::from_utf8_lossy(&output.stdout).trim().to_string();
            debug!(pid, command_line, "Got process command line");
            Ok(command_line)
        }

        #[cfg(target_os = "linux")]
        {
            let output = Command::new("cat")
                .args([&format!("/proc/{}/cmdline", pid)])
                .output()
                .context("Failed to read /proc cmdline")?;

            let cmdline = String::from_utf8_lossy(&output.stdout);
            // Replace null bytes with spaces for readability
            let command_line = cmdline.replace('\0', " ").trim().to_string();
            debug!(pid, command_line, "Got process command line");
            Ok(command_line)
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            warn!("Process command line detection not supported on this platform");
            Ok(format!("PID {}", pid))
        }
    }

    /// Kill a process by PID
    pub fn kill_process(pid: u32) -> Result<()> {
        info!(pid, "Attempting to kill process");

        #[cfg(unix)]
        {
            let output = Command::new("kill")
                .args([&format!("{pid}")])
                .output()
                .context("Failed to execute kill command")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to kill process {pid}: {stderr}"));
            }

            info!(pid, "Successfully killed process");
            Ok(())
        }

        #[cfg(not(unix))]
        {
            anyhow::bail!("Process killing not supported on this platform");
        }
    }

    /// Wait for a process to terminate
    pub fn wait_for_process_exit(pid: u32, timeout_secs: u64) -> Result<bool> {
        info!(pid, timeout_secs, "Waiting for process to exit");

        let start_time = std::time::Instant::now();
        let timeout_duration = std::time::Duration::from_secs(timeout_secs);

        while start_time.elapsed() < timeout_duration {
            if let Ok(command_line) = Self::get_process_command_line(pid) {
                if command_line.trim().is_empty() {
                    info!(pid, "Process has exited");
                    return Ok(true);
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        warn!(pid, timeout_secs, "Process did not exit within timeout");
        Ok(false)
    }
}

/// Get process list output based on platform
fn get_process_list_output() -> Result<Output> {
    #[cfg(target_os = "macos")]
    {
        Command::new("ps")
            .args(["-axo", "pid,command"])
            .output()
            .context("Failed to execute ps command on macOS")
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("ps")
            .args(["-eo", "pid,command"])
            .output()
            .context("Failed to execute ps command on Linux")
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        anyhow::bail!("Process detection not supported on this platform")
    }
}
