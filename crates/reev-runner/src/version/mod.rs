//! Version tracking module for reev-runner and reev-agent
//!
//! This module provides version information for proper logging and
//! identification of runner and agent versions in enhanced OpenTelemetry logs.

use std::env;

/// Get the current reev-runner version
pub fn get_runner_version() -> String {
    // Try to get version from environment first (for runtime injection)
    if let Ok(version) = env::var("REEV_RUNNER_VERSION") {
        return version;
    }

    // Fallback to compile-time version
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get the current reev-agent version
pub fn get_agent_version() -> String {
    // Try to get version from environment first (for runtime injection)
    if let Ok(version) = env::var("REEV_AGENT_VERSION") {
        return version;
    }

    // Default to same version as runner for now
    env!("CARGO_PKG_VERSION").to_string()
}

/// Set agent version at runtime (for agents that know their own version)
pub fn set_agent_version(version: &str) {
    unsafe {
        env::set_var("REEV_AGENT_VERSION", version);
    }
}

/// Version information structure
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub runner_version: String,
    pub agent_version: String,
}

impl VersionInfo {
    /// Create new version info
    pub fn new() -> Self {
        Self {
            runner_version: get_runner_version(),
            agent_version: get_agent_version(),
        }
    }

    /// Create version info with custom agent version
    pub fn with_agent_version(agent_version: String) -> Self {
        Self {
            runner_version: get_runner_version(),
            agent_version,
        }
    }
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize version tracking for the current process
pub fn init_version_tracking() {
    let runner_version = get_runner_version();
    let agent_version = get_agent_version();

    tracing::info!(
        "Version tracking initialized - Runner: {}, Agent: {}",
        runner_version,
        agent_version
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        let version_info = VersionInfo::new();

        assert!(!version_info.runner_version.is_empty());
        assert!(!version_info.agent_version.is_empty());
    }

    #[test]
    fn test_custom_agent_version() {
        let custom_agent = "glm-4.6-custom".to_string();
        let version_info = VersionInfo::with_agent_version(custom_agent.clone());

        assert_eq!(version_info.agent_version, custom_agent);
        assert!(!version_info.runner_version.is_empty());
    }

    #[test]
    fn test_version_setting() {
        set_agent_version("test-agent-version");
        assert_eq!(get_agent_version(), "test-agent-version");
    }
}
