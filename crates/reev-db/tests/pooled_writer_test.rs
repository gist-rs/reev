use reev_db::{pool::PooledDatabaseWriter, DatabaseConfig};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_pooled_writer_basic() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let config = DatabaseConfig::new(db_path.to_str().unwrap());

    let writer = PooledDatabaseWriter::new(config, 3).await.unwrap();

    // Test basic operations
    let stats = writer.pool_stats().await;
    assert!(stats.current_size >= 1);
    assert!(stats.max_connections == 3);
}

#[tokio::test]
async fn test_concurrent_operations() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let config = DatabaseConfig::new(db_path.to_str().unwrap());

    let writer = Arc::new(PooledDatabaseWriter::new(config, 5).await.unwrap());

    let mut handles = vec![];

    // Spawn concurrent operations
    for i in 0..10 {
        let writer_clone = Arc::clone(&writer);
        let handle = tokio::spawn(async move {
            // Test database stats operation
            let stats = writer_clone.get_database_stats().await;
            assert!(stats.is_ok());
            i
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Check final pool stats
    let stats = writer.pool_stats().await;
    assert!(stats.active_connections <= stats.max_connections);
}
