//! Dependency management for reev-runner
//!
//! This module provides intelligent dependency management for external services
//! like reev-agent and surfpool, with automatic startup, health monitoring,
//! and lifecycle management.

// pub mod binary_manager; // Not implemented yet
pub mod config;
pub mod dependency_manager;
// pub mod lifecycle_manager; // Moved to process module
pub mod process_detector;

// pub use binary_manager::BinaryManager; // Not implemented yet
pub use config::DependencyConfig;
pub use dependency_manager::DependencyManager;
// pub use lifecycle_manager::LifecycleManager; // Moved to process module
pub use process_detector::ProcessDetector;

use std::collections::HashMap;

/// URLs for running dependencies
#[derive(Debug, Clone)]
pub struct DependencyUrls {
    pub reev_agent: String,
    pub surfpool_rpc: String,
    pub surfpool_ws: Option<String>,
}

// Re-export ServiceHealth from health module to avoid conflicts
pub use crate::dependency::health::ServiceHealth;

/// Information about a running dependency
#[derive(Debug, Clone)]
pub struct DependencyService {
    pub name: String,
    pub pid: Option<u32>,
    pub port: Option<u16>,
    pub health: ServiceHealth,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub urls: HashMap<String, String>,
}

impl DependencyService {
    pub fn new(name: String, port: Option<u16>) -> Self {
        Self {
            name,
            pid: None,
            port,
            health: ServiceHealth::Unknown,
            start_time: chrono::Utc::now(),
            urls: HashMap::new(),
        }
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.health, ServiceHealth::Healthy)
    }

    pub fn set_health(&mut self, health: ServiceHealth) {
        self.health = health;
    }

    pub fn add_url(&mut self, key: String, url: String) {
        self.urls.insert(key, url);
    }
}

/// Types of dependencies managed by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DependencyType {
    ReevAgent,
    Surfpool,
}

impl DependencyType {
    pub fn default_port(&self) -> u16 {
        match self {
            DependencyType::ReevAgent => 9090,
            DependencyType::Surfpool => 8899,
        }
    }

    pub fn process_name(&self) -> &'static str {
        match self {
            DependencyType::ReevAgent => "reev-agent",
            DependencyType::Surfpool => "surfpool",
        }
    }

    pub fn health_endpoint(&self) -> &'static str {
        match self {
            DependencyType::ReevAgent => "/health",
            DependencyType::Surfpool => "/", // Root endpoint for surfpool
        }
    }
}

/// Error types for dependency management
#[derive(Debug, thiserror::Error)]
pub enum DependencyError {
    #[error("Failed to start {service}: {source}")]
    StartError {
        service: String,
        source: anyhow::Error,
    },

    #[error("Failed to stop {service}: {source}")]
    StopError {
        service: String,
        source: anyhow::Error,
    },

    #[error("Health check failed for {service}: {reason}")]
    HealthCheckError { service: String, reason: String },

    #[error("Binary not found for {service}: {reason}")]
    BinaryNotFound { service: String, reason: String },

    #[error("Port conflict for {service}: port {port} is in use")]
    PortConflict { service: String, port: u16 },

    #[error("Process already running for {service} (pid: {pid})")]
    ProcessAlreadyRunning { service: String, pid: u32 },

    #[error("Timeout waiting for {service} to become healthy")]
    HealthTimeout { service: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },
}
