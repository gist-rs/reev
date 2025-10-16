//! Tests for database configuration module

use reev_db::{config::DatabaseConfigBuilder, DatabaseConfig};

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
