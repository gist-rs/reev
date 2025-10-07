//! Configuration for dependency management

use std::time::Duration;

/// Configuration for dependency management system
#[derive(Debug, Clone)]
pub struct DependencyConfig {
    /// Whether to automatically start dependencies when needed
    pub auto_start: bool,

    /// Whether to prefer pre-built binaries over building from source
    pub prefer_binary: bool,

    /// How long to cache binaries before checking for updates
    pub cache_duration: Duration,

    /// How often to perform health checks on running services
    pub health_check_interval: Duration,

    /// Whether to allow multiple runner processes to share the same service instances
    pub shared_instances: bool,

    /// Default ports for services (can be overridden by environment variables)
    pub reev_agent_port: u16,
    pub surfpool_rpc_port: u16,

    /// Timeout for health checks
    pub health_check_timeout: Duration,

    /// Timeout for service startup
    pub startup_timeout: Duration,

    /// Whether to enable detailed logging for dependency management
    pub verbose_logging: bool,

    /// Base directory for storing cached binaries
    pub cache_dir: String,

    /// Base directory for service logs
    pub log_dir: String,
}

impl Default for DependencyConfig {
    fn default() -> Self {
        Self {
            auto_start: true,
            prefer_binary: true,
            cache_duration: Duration::from_secs(24 * 60 * 60), // 24 hours
            health_check_interval: Duration::from_secs(30),
            shared_instances: true,
            reev_agent_port: 9090,
            surfpool_rpc_port: 8899,
            health_check_timeout: Duration::from_secs(5),
            startup_timeout: Duration::from_secs(60),
            verbose_logging: false,
            cache_dir: ".surfpool/cache".to_string(),
            log_dir: "logs".to_string(),
        }
    }
}

impl DependencyConfig {
    /// Load configuration from environment variables with defaults
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Override with environment variables if present
        if let Ok(port) = std::env::var("REEV_AGENT_PORT") {
            if let Ok(port) = port.parse() {
                config.reev_agent_port = port;
            }
        }

        if let Ok(port) = std::env::var("SURFPOOL_RPC_PORT") {
            if let Ok(port) = port.parse() {
                config.surfpool_rpc_port = port;
            }
        }

        if let Ok(val) = std::env::var("REEV_AUTO_START") {
            config.auto_start = val.to_lowercase() != "false" && val != "0";
        }

        if let Ok(val) = std::env::var("REEV_VERBOSE_LOGGING") {
            config.verbose_logging = val.to_lowercase() != "false" && val != "0";
        }

        if let Ok(path) = std::env::var("REEV_CACHE_DIR") {
            config.cache_dir = path;
        }

        if let Ok(path) = std::env::var("REEV_LOG_DIR") {
            config.log_dir = path;
        }

        config
    }

    /// Get port for a specific dependency type
    pub fn get_port(&self, dependency_type: super::DependencyType) -> u16 {
        match dependency_type {
            super::DependencyType::ReevAgent => self.reev_agent_port,
            super::DependencyType::Surfpool => self.surfpool_rpc_port,
        }
    }

    /// Set port for a specific dependency type
    pub fn set_port(&mut self, dependency_type: super::DependencyType, port: u16) {
        match dependency_type {
            super::DependencyType::ReevAgent => self.reev_agent_port = port,
            super::DependencyType::Surfpool => self.surfpool_rpc_port = port,
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), super::DependencyError> {
        // Check for port conflicts
        if self.reev_agent_port == self.surfpool_rpc_port {
            return Err(super::DependencyError::ConfigError {
                message: format!(
                    "Port conflict: reev_agent_port ({}) and surfpool_rpc_port ({}) cannot be the same",
                    self.reev_agent_port, self.surfpool_rpc_port
                ),
            });
        }

        // Check cache directory path
        if self.cache_dir.is_empty() {
            return Err(super::DependencyError::ConfigError {
                message: "cache_dir cannot be empty".to_string(),
            });
        }

        // Check log directory path
        if self.log_dir.is_empty() {
            return Err(super::DependencyError::ConfigError {
                message: "log_dir cannot be empty".to_string(),
            });
        }

        // Check timeout values
        if self.health_check_timeout.as_secs() == 0 {
            return Err(super::DependencyError::ConfigError {
                message: "health_check_timeout must be greater than 0".to_string(),
            });
        }

        if self.startup_timeout.as_secs() == 0 {
            return Err(super::DependencyError::ConfigError {
                message: "startup_timeout must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}
