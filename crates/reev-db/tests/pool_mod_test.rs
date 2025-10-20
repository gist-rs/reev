use reev_db::{pool::ConnectionPool, DatabaseConfig};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_connection_pool_basic() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let config = DatabaseConfig::new(db_path.to_str().unwrap());

    let pool = ConnectionPool::new(config, 3).await.unwrap();

    // Get a connection
    let conn1 = pool.get_connection().await.unwrap();
    let _ = conn1.connection();

    // Return connection (implicitly via drop)
    drop(conn1);

    // Should be able to get another connection
    let conn2 = pool.get_connection().await.unwrap();
    let _ = conn2.connection();

    drop(conn2);
}

#[tokio::test]
async fn test_concurrent_connections() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let config = DatabaseConfig::new(db_path.to_str().unwrap());

    let pool = Arc::new(ConnectionPool::new(config, 5).await.unwrap());

    let mut handles = vec![];

    // Spawn concurrent tasks
    for i in 0..10 {
        let pool_clone = Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            let conn = pool_clone.get_connection().await.unwrap();

            // Simulate some work
            let mut rows = conn.connection().query("SELECT 1", ()).await.unwrap();
            let row = rows.next().await.unwrap().unwrap();
            let value: i64 = row.get(0).unwrap();

            assert_eq!(value, 1);

            drop(conn);
            i
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
}
