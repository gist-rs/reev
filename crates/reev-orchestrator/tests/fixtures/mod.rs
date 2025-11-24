//! Test Fixtures for Orchestrator Tests
//!
//! This module provides test fixtures and utilities to handle database
//! isolation and other common test setup requirements.

use reev_db::writer::DatabaseWriter;
use reev_db::DatabaseConfig;
use reev_orchestrator::{ContextResolver, OrchestratorGateway};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::OnceCell;

// Global test fixture for shared setup
static TEST_FIXTURE: OnceCell<TestFixture> = OnceCell::const_new();

/// Test fixture with isolated database and shared components
pub struct TestFixture {
    #[allow(dead_code)]
    pub temp_dir: TempDir,
    pub db_path: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture with isolated database
    async fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test.db");

        Self { temp_dir, db_path }
    }

    /// Get or create the global test fixture
    #[allow(dead_code)]
    pub async fn get() -> &'static TestFixture {
        TEST_FIXTURE
            .get_or_init(|| async { Self::new().await })
            .await
    }

    /// Create a database writer for testing
    pub async fn create_db_writer(&self) -> Arc<DatabaseWriter> {
        let db_config = DatabaseConfig::local(self.db_path.to_str().unwrap());
        Arc::new(
            DatabaseWriter::new(db_config)
                .await
                .expect("Failed to create database writer"),
        )
    }

    /// Create an orchestrator gateway with isolated database
    pub async fn create_gateway(&self) -> OrchestratorGateway {
        let db = self.create_db_writer().await;

        // Set mock environment for testing
        std::env::set_var("REEV_TEST_MODE", "true");

        OrchestratorGateway::with_database(db)
            .await
            .expect("Failed to create orchestrator gateway")
    }

    /// Create a context resolver for testing
    pub async fn create_context_resolver(&self) -> ContextResolver {
        ContextResolver::new()
    }

    /// Clean up resources
    pub fn cleanup(&self) {
        // Nothing to do - TempDir will be cleaned up when dropped
    }
}

/// Test helper to run a test with isolated database
#[allow(dead_code)]
pub async fn with_isolated_db<F, Fut, R>(test_fn: F) -> R
where
    F: FnOnce(Arc<DatabaseWriter>) -> Fut,
    Fut: std::future::Future<Output = R>,
{
    let fixture = TestFixture::new().await;
    let db = fixture.create_db_writer().await;

    let result = test_fn(db).await;

    // Explicit cleanup to ensure database connections are closed
    fixture.cleanup();

    result
}

/// Test helper to run a test with isolated gateway
pub async fn with_isolated_gateway<F, Fut, R>(test_fn: F) -> R
where
    F: FnOnce(OrchestratorGateway) -> Fut,
    Fut: std::future::Future<Output = R>,
{
    let fixture = TestFixture::new().await;
    let gateway = fixture.create_gateway().await;

    let result = test_fn(gateway).await;

    // Explicit cleanup to ensure database connections are closed
    fixture.cleanup();

    result
}

/// Test helper to run a test with isolated context resolver
pub async fn with_isolated_context_resolver<F, Fut, R>(test_fn: F) -> R
where
    F: FnOnce(ContextResolver) -> Fut,
    Fut: std::future::Future<Output = R>,
{
    let fixture = TestFixture::new().await;
    let resolver = fixture.create_context_resolver().await;

    let result = test_fn(resolver).await;

    // Explicit cleanup to ensure database connections are closed
    fixture.cleanup();

    result
}
