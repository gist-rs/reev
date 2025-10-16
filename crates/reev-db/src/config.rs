//! Database configuration module for reev-db
//!
//! Provides configuration options for database connections,
//! including both local SQLite and remote Turso configurations.

use std::path::PathBuf;

/// Configuration for database connections
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database path or URL
    pub path: String,
    /// Authentication token for remote databases (Turso)
    pub auth_token: Option<String>,
    /// Connection timeout in seconds
    pub timeout_secs: u64,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Enable connection pooling for read operations
    pub enable_pooling: bool,
    /// Maximum pool size (when pooling is enabled)
    pub max_pool_size: u32,
}

impl DatabaseConfig {
    /// Create a new database configuration with default settings
    pub fn new<P: Into<String>>(path: P) -> Self {
        Self {
            path: path.into(),
            auth_token: None,
            timeout_secs: 30,
            max_retries: 3,
            enable_pooling: false,
            max_pool_size: 10,
        }
    }

    /// Create configuration for local SQLite database
    pub fn local<P: Into<String>>(path: P) -> Self {
        Self::new(path)
    }

    /// Create configuration for remote Turso database
    pub fn turso<P: Into<String>>(url: P, auth_token: String) -> Self {
        Self {
            path: url.into(),
            auth_token: Some(auth_token),
            timeout_secs: 30,
            max_retries: 3,
            enable_pooling: true,
            max_pool_size: 10,
        }
    }

    /// Set authentication token
    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    /// Set connection timeout
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Set maximum retry attempts
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Enable/disable connection pooling
    pub fn with_pooling(mut self, enabled: bool) -> Self {
        self.enable_pooling = enabled;
        self
    }

    /// Set maximum pool size
    pub fn with_max_pool_size(mut self, size: u32) -> Self {
        self.max_pool_size = size;
        self
    }

    /// Check if this is a remote database configuration
    pub fn is_remote(&self) -> bool {
        self.path.starts_with("libsql://") || self.auth_token.is_some()
    }

    /// Check if this is an in-memory database
    pub fn is_memory(&self) -> bool {
        self.path == ":memory:" || self.path.contains("mode=memory")
    }

    /// Get database type description
    pub fn database_type(&self) -> &'static str {
        if self.is_memory() {
            "in-memory SQLite"
        } else if self.is_remote() {
            "Turso (remote SQLite)"
        } else {
            "local SQLite"
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self::new("reev.db")
    }
}

impl<P: Into<String>> From<P> for DatabaseConfig {
    fn from(path: P) -> Self {
        Self::new(path)
    }
}

/// Builder for database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfigBuilder {
    config: DatabaseConfig,
}

impl DatabaseConfigBuilder {
    /// Create a new configuration builder
    pub fn new<P: Into<String>>(path: P) -> Self {
        Self {
            config: DatabaseConfig::new(path),
        }
    }

    /// Build the configuration
    pub fn build(self) -> DatabaseConfig {
        self.config
    }

    /// Set authentication token
    pub fn auth_token(mut self, token: String) -> Self {
        self.config.auth_token = Some(token);
        self
    }

    /// Set connection timeout
    pub fn timeout(mut self, secs: u64) -> Self {
        self.config.timeout_secs = secs;
        self
    }

    /// Set maximum retry attempts
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// Enable connection pooling
    pub fn enable_pooling(mut self, enabled: bool) -> Self {
        self.config.enable_pooling = enabled;
        self
    }

    /// Set maximum pool size
    pub fn max_pool_size(mut self, size: u32) -> Self {
        self.config.max_pool_size = size;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_config() {
        let config = DatabaseConfig::new("test.db");
        assert_eq!(config.path, "test.db");
        assert!(config.auth_token.is_none());
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
        assert!(!config.enable_pooling);
        assert_eq!(config.database_type(), "local SQLite");
    }

    #[test]
    fn test_memory_config() {
        let config = DatabaseConfig::new(":memory:");
        assert!(config.is_memory());
        assert_eq!(config.database_type(), "in-memory SQLite");
    }

    #[test]
    fn test_turso_config() {
        let config = DatabaseConfig::turso("libsql://my-db.turso.io", "auth-token-123".to_string());
        assert!(config.is_remote());
        assert_eq!(config.database_type(), "Turso (remote SQLite)");
        assert_eq!(config.auth_token, Some("auth-token-123".to_string()));
        assert!(config.enable_pooling);
    }

    #[test]
    fn test_builder_pattern() {
        let config = DatabaseConfigBuilder::new("test.db")
            .auth_token("token123".to_string())
            .timeout(60)
            .max_retries(5)
            .enable_pooling(true)
            .max_pool_size(20)
            .build();

        assert_eq!(config.path, "test.db");
        assert_eq!(config.auth_token, Some("token123".to_string()));
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.max_retries, 5);
        assert!(config.enable_pooling);
        assert_eq!(config.max_pool_size, 20);
    }

    #[test]
    fn test_from_string() {
        let config: DatabaseConfig = "test.db".into();
        assert_eq!(config.path, "test.db");
    }
}
