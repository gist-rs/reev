//! Configuration management for reev-agent
//!
//! This module provides configuration for different protocols and services,
//! with environment variable support via dotenvy and sensible defaults.

pub mod jupiter;
pub mod native;

use std::env;

/// Global configuration for the reev-agent
#[derive(Debug, Clone)]
pub struct Config {
    pub jupiter: jupiter::JupiterConfig,
    pub native: native::NativeConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            jupiter: jupiter::JupiterConfig::default(),
            native: native::NativeConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            jupiter: jupiter::JupiterConfig::from_env(),
            native: native::NativeConfig::from_env(),
        }
    }

    /// Load configuration, prioritizing environment variables over defaults
    pub fn load() -> Self {
        dotenvy::dotenv().ok(); // Load .env file if it exists
        Self::from_env()
    }
}

/// Helper function to get environment variable with default value
pub fn get_env_var<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// Helper function to get environment variable as string with default value
pub fn get_env_string(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
