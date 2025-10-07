//! Tests for dependency management functionality

use reev_runner::dependency::{DependencyConfig, DependencyManager, DependencyType};
use tempfile::TempDir;

#[tokio::test]
async fn test_dependency_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config = DependencyConfig {
        cache_dir: temp_dir.path().join("cache").to_string_lossy().to_string(),
        log_dir: temp_dir.path().join("logs").to_string_lossy().to_string(),
        ..Default::default()
    };

    let result = DependencyManager::new(config);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dependency_config_from_env() {
    let config = DependencyConfig::from_env();

    // Should have default values
    assert_eq!(config.reev_agent_port, 9090);
    assert_eq!(config.surfpool_rpc_port, 8899);
    assert!(config.auto_start);
    assert_eq!(config.cache_dir, ".surfpool/cache");
    assert_eq!(config.log_dir, "logs");
}

#[tokio::test]
async fn test_dependency_config_validation() {
    let mut config = DependencyConfig::default();

    // Valid config should pass
    assert!(config.validate().is_ok());

    // Port conflict should fail
    config.reev_agent_port = 8080;
    config.surfpool_rpc_port = 8080;
    assert!(config.validate().is_err());

    // Reset ports
    config.reev_agent_port = 9090;
    config.surfpool_rpc_port = 8899;

    // Empty cache dir should fail
    config.cache_dir = String::new();
    assert!(config.validate().is_err());

    // Empty log dir should fail
    config.cache_dir = ".surfpool/cache".to_string();
    config.log_dir = String::new();
    assert!(config.validate().is_err());
}

#[tokio::test]
async fn test_dependency_urls() {
    let temp_dir = TempDir::new().unwrap();
    let config = DependencyConfig {
        cache_dir: temp_dir.path().join("cache").to_string_lossy().to_string(),
        log_dir: temp_dir.path().join("logs").to_string_lossy().to_string(),
        ..Default::default()
    };

    let manager = DependencyManager::new(config).unwrap();
    let urls = manager.get_dependency_urls().await.unwrap();

    assert_eq!(urls.reev_agent, "http://localhost:9090");
    assert_eq!(urls.surfpool_rpc, "http://localhost:8899");
    assert_eq!(urls.surfpool_ws, Some("ws://localhost:8899/ws".to_string()));
}

#[tokio::test]
async fn test_health_status_before_init() {
    let temp_dir = TempDir::new().unwrap();
    let config = DependencyConfig {
        cache_dir: temp_dir.path().join("cache").to_string_lossy().to_string(),
        log_dir: temp_dir.path().join("logs").to_string_lossy().to_string(),
        ..Default::default()
    };

    let manager = DependencyManager::new(config).unwrap();
    let status = manager.get_health_status().await;

    // Should be empty since no services are initialized
    assert!(status.is_empty());
}

#[tokio::test]
async fn test_are_dependencies_healthy_before_init() {
    let temp_dir = TempDir::new().unwrap();
    let config = DependencyConfig {
        cache_dir: temp_dir.path().join("cache").to_string_lossy().to_string(),
        log_dir: temp_dir.path().join("logs").to_string_lossy().to_string(),
        ..Default::default()
    };

    let manager = DependencyManager::new(config).unwrap();

    // Should return true when no dependencies are initialized
    assert!(manager.are_dependencies_healthy().await);
}

#[tokio::test]
async fn test_dependency_types() {
    assert_eq!(DependencyType::ReevAgent.default_port(), 9090);
    assert_eq!(DependencyType::Surfpool.default_port(), 8899);
    assert_eq!(DependencyType::ReevAgent.process_name(), "reev-agent");
    assert_eq!(DependencyType::Surfpool.process_name(), "surfpool");
    assert_eq!(DependencyType::ReevAgent.health_endpoint(), "/health");
    assert_eq!(DependencyType::Surfpool.health_endpoint(), "/");
}

#[tokio::test]
async fn test_config_update() {
    let temp_dir = TempDir::new().unwrap();
    let config = DependencyConfig {
        cache_dir: temp_dir.path().join("cache").to_string_lossy().to_string(),
        log_dir: temp_dir.path().join("logs").to_string_lossy().to_string(),
        ..Default::default()
    };

    let mut manager = DependencyManager::new(config).unwrap();

    // Update config
    let new_config = DependencyConfig {
        reev_agent_port: 9091,
        surfpool_rpc_port: 8898,
        cache_dir: temp_dir.path().join("cache2").to_string_lossy().to_string(),
        log_dir: temp_dir.path().join("logs2").to_string_lossy().to_string(),
        ..Default::default()
    };

    assert!(manager.update_config(new_config).is_ok());
    assert_eq!(manager.config().reev_agent_port, 9091);
    assert_eq!(manager.config().surfpool_rpc_port, 8898);
}

#[tokio::test]
async fn test_cleanup_before_init() {
    let temp_dir = TempDir::new().unwrap();
    let config = DependencyConfig {
        cache_dir: temp_dir.path().join("cache").to_string_lossy().to_string(),
        log_dir: temp_dir.path().join("logs").to_string_lossy().to_string(),
        ..Default::default()
    };

    let mut manager = DependencyManager::new(config).unwrap();

    // Cleanup should work even before initialization
    assert!(manager.cleanup().await.is_ok());
    assert!(manager.force_cleanup().await.is_ok());
}

#[tokio::test]
async fn test_dependency_config_ports() {
    let mut config = DependencyConfig::default();

    // Test getting ports
    assert_eq!(config.get_port(DependencyType::ReevAgent), 9090);
    assert_eq!(config.get_port(DependencyType::Surfpool), 8899);

    // Test setting ports
    config.set_port(DependencyType::ReevAgent, 9091);
    config.set_port(DependencyType::Surfpool, 8898);

    assert_eq!(config.get_port(DependencyType::ReevAgent), 9091);
    assert_eq!(config.get_port(DependencyType::Surfpool), 8898);
}

// Note: Full integration tests that actually start services would require
// more complex setup and are omitted here to keep tests fast and reliable.
// In a real testing environment, you might want to mock the external services
// or use test fixtures that don't require actual process spawning.
