//! Integration Tests for Turso Test Suite
//!
//! This module provides comprehensive integration tests covering:
//! - Basic database operations
//! - UPSERT functionality
//! - Concurrency behavior
//! - Error handling
//! - Data integrity

use anyhow::Result;
use chrono::Utc;
use std::collections::HashSet;
use tokio::task::JoinSet;
use turso::Builder;

/// Test basic database connection and table creation
#[tokio::test]
async fn test_basic_connection() -> Result<()> {
    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;

    // Create test table
    conn.execute(
        "CREATE TABLE test_table (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            value TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    ).await?;

    // Verify table exists
    let mut rows = conn.query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='test_table'",
        ()
    ).await?;

    assert!(rows.next().await?.is_some(), "Table should exist");
    Ok(())
}

/// Test basic INSERT operations
#[tokio::test]
async fn test_basic_insert() -> Result<()> {
    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;

    // Create test table
    conn.execute(
        "CREATE TABLE test_table (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            value TEXT
        )",
        (),
    ).await?;

    // Insert test data
    let result = conn.execute(
        "INSERT INTO test_table (id, name, value) VALUES (?, ?, ?)",
        ["test-001", "test-name", "test-value"]
    ).await?;

    assert_eq!(result, 1, "Should insert one record");

    // Verify data
    let mut rows = conn.query("SELECT COUNT(*) FROM test_table", ()).await?;
    let row = rows.next().await?.unwrap();
    let count: i64 = row.get(0)?;
    assert_eq!(count, 1, "Should have one record");

    Ok(())
}

/// Test UPSERT operations with ON CONFLICT
#[tokio::test]
async fn test_upsert_functionality() -> Result<()> {
    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;

    // Create test table
    conn.execute(
        "CREATE TABLE test_table (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            value TEXT,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    ).await?;

    let timestamp = Utc::now().to_rfc3339();

    // First insert
    let result1 = conn.execute(
        "INSERT INTO test_table (id, name, value, updated_at) VALUES (?, ?, ?, ?)",
        ["upsert-001", "original-name", "original-value", &timestamp]
    ).await?;
    assert_eq!(result1, 1, "First insert should succeed");

    // Second insert with same ID (should update)
    let result2 = conn.execute(
        "INSERT INTO test_table (id, name, value, updated_at) VALUES (?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             name = excluded.name,
             value = excluded.value,
             updated_at = excluded.updated_at",
        ["upsert-001", "updated-name", "updated-value", &timestamp]
    ).await?;
    assert_eq!(result2, 2, "UPSERT should update existing record");

    // Verify only one record exists
    let mut rows = conn.query("SELECT COUNT(*) FROM test_table WHERE id = 'upsert-001'", ()).await?;
    let row = rows.next().await?.unwrap();
    let count: i64 = row.get(0)?;
    assert_eq!(count, 1, "Should still have only one record");

    // Verify data was updated
    let mut rows = conn.query(
        "SELECT name, value FROM test_table WHERE id = 'upsert-001'",
        ()
    ).await?;
    let row = rows.next().await?.unwrap();
    let name: String = row.get(0)?;
    let value: String = row.get(1)?;
    assert_eq!(name, "updated-name", "Name should be updated");
    assert_eq!(value, "updated-value", "Value should be updated");

    Ok(())
}

/// Test sequential processing of multiple records
#[tokio::test]
async fn test_sequential_processing() -> Result<()> {
    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;

    // Create test table
    conn.execute(
        "CREATE TABLE test_table (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            batch_id TEXT
        )",
        (),
    ).await?;

    let test_data: Vec<_> = (0..50)
        .map(|i| {
            let id = format!("seq-{i:03}");
            let name = format!("Sequential record {i}");
            (id, name)
        })
        .collect();

    // Process sequentially
    for (i, (id, name)) in test_data.iter().enumerate() {
        let result = conn.execute(
            "INSERT INTO test_table (id, name, batch_id) VALUES (?, ?, ?)",
            [id, name, "batch-1"]
        ).await?;
        assert_eq!(result, 1, "Insert {} should succeed", i + 1);
    }

    // Verify all records inserted
    let mut rows = conn.query("SELECT COUNT(*) FROM test_table", ()).await?;
    let row = rows.next().await?.unwrap();
    let count: i64 = row.get(0)?;
    assert_eq!(count, 50, "Should have 50 records");

    // Process same data again (should update)
    for (id, name) in test_data.iter() {
        let result = conn.execute(
            "INSERT INTO test_table (id, name, batch_id) VALUES (?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name,
                 batch_id = excluded.batch_id",
            [id, name, "batch-2"]
        ).await?;
        assert!(result > 0, "UPSERT should succeed");
    }

    // Verify still only 50 records
    let mut rows = conn.query("SELECT COUNT(*) FROM test_table", ()).await?;
    let row = rows.next().await?.unwrap();
    let count: i64 = row.get(0)?;
    assert_eq!(count, 50, "Should still have 50 records");

    // Verify all records have batch-2
    let mut rows = conn.query(
        "SELECT COUNT(*) FROM test_table WHERE batch_id = 'batch-2'",
        ()
    ).await?;
    let row = rows.next().await?.unwrap();
    let count: i64 = row.get(0)?;
    assert_eq!(count, 50, "All records should be updated to batch-2");

    Ok(())
}

/// Test concurrent processing limitations
#[tokio::test]
async fn test_concurrent_limitations() -> Result<()> {
    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;

    // Create test table
    conn.execute(
        "CREATE TABLE test_table (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            task_id INTEGER
        )",
        (),
    ).await?;

    let test_data: Vec<_> = (0..20)
        .map(|i| {
            let id = format!("conc-{i:03}");
            let name = format!("Concurrent record {i}");
            (id, name, i)
        })
        .collect();

    // Test with moderate concurrency
    let mut join_set = JoinSet::new();
    let mut success_count = 0;
    let mut error_count = 0;

    for (i, (id, name, task_id)) in test_data.into_iter().enumerate() {
        let conn_clone = conn.clone();
        join_set.spawn(async move {
            // Add small delay to increase contention
            if i % 3 == 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }

            match conn_clone.execute(
                "INSERT INTO test_table (id, name, task_id) VALUES (?, ?, ?)",
                [id, name, task_id.to_string()]
            ).await {
                Ok(_) => Ok(task_id),
                Err(e) => {
                    eprintln!("Task {task_id} failed: {e}");
                    Err(e)
                }
            }
        });
    }

    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(task_result) => match task_result {
                Ok(task_id) => {
                    results.push(task_id);
                    success_count += 1;
                }
                Err(_) => error_count += 1,
            },
            Err(_) => error_count += 1,
        }
    }

    // This test demonstrates Turso's concurrency limitations
    // We expect some failures due to database locking
    println!("Concurrent test results: {success_count} successes, {error_count} errors");

    // Verify data integrity despite potential failures
    let mut rows = conn.query("SELECT COUNT(*) FROM test_table", ()).await?;
    let row = rows.next().await?.unwrap();
    let count: i64 = row.get(0)?;

    let unique_ids: HashSet<_> = results.iter().collect();
    assert_eq!(count as usize, unique_ids.len(), "Count should match unique successful operations");

    // Test passes even with concurrent errors - this demonstrates the limitation
    Ok(())
}

/// Test error handling and recovery
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;

    // Create test table
    conn.execute(
        "CREATE TABLE test_table (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        )",
        (),
    ).await?;

    // Insert first record
    let result = conn.execute(
        "INSERT INTO test_table (id, name) VALUES (?, ?)",
        ["error-001", "unique-name"]
    ).await?;
    assert_eq!(result, 1, "First insert should succeed");

    // Try to insert duplicate name (should fail)
    let duplicate_result = conn.execute(
        "INSERT INTO test_table (id, name) VALUES (?, ?)",
        ["error-002", "unique-name"]
    ).await;

    assert!(duplicate_result.is_err(), "Duplicate name should fail");

    // Verify first record still exists
    let mut rows = conn.query("SELECT COUNT(*) FROM test_table", ()).await?;
    let row = rows.next().await?.unwrap();
    let count: i64 = row.get(0)?;
    assert_eq!(count, 1, "Should still have only one record");

    Ok(())
}

/// Test MD5 generation and uniqueness
#[tokio::test]
async fn test_md5_generation() -> Result<()> {
    // Test different inputs generate different MD5s
    let input1 = "benchmark-001:prompt one";
    let input2 = "benchmark-001:prompt two";
    let input3 = "benchmark-002:prompt one";

    let md5_1 = format!("{:x}", md5::compute(input1.as_bytes()));
    let md5_2 = format!("{:x}", md5::compute(input2.as_bytes()));
    let md5_3 = format!("{:x}", md5::compute(input3.as_bytes()));

    assert_ne!(md5_1, md5_2, "Different prompts should have different MD5s");
    assert_ne!(md5_1, md5_3, "Different benchmark names should have different MD5s");
    assert_ne!(md5_2, md5_3, "All three should be different");

    // Test same input generates same MD5
    let md5_1_repeat = format!("{:x}", md5::compute(input1.as_bytes()));
    assert_eq!(md5_1, md5_1_repeat, "Same input should generate same MD5");

    Ok(())
}

/// Test database schema integrity
#[tokio::test]
async fn test_schema_integrity() -> Result<()> {
    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;

    // Create production-like schema
    conn.execute(
        "CREATE TABLE benchmarks (
            id TEXT PRIMARY KEY,
            benchmark_name TEXT NOT NULL,
            prompt TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    ).await?;

    // Verify schema
    let mut rows = conn.query(
        "PRAGMA table_info(benchmarks)",
        ()
    ).await?;

    let mut columns = Vec::new();
    while let Some(row) = rows.next().await? {
        let name: String = row.get(1)?;
        let data_type: String = row.get(2)?;
        let not_null: i64 = row.get(3)?;
        let pk: i64 = row.get(5)?;

        columns.push((name, (data_type, not_null == 1, pk == 1)));
    }

    assert_eq!(columns.len(), 6, "Should have 6 columns");

    // Verify specific columns
    let column_map: std::collections::HashMap<String, (String, bool, bool)> = columns.into_iter().collect();

    assert!(column_map.contains_key("id"), "Should have id column");
    assert!(column_map.contains_key("benchmark_name"), "Should have benchmark_name column");
    assert!(column_map.contains_key("prompt"), "Should have prompt column");
    assert!(column_map.contains_key("content"), "Should have content column");
    assert!(column_map.contains_key("created_at"), "Should have created_at column");
    assert!(column_map.contains_key("updated_at"), "Should have updated_at column");

    // Verify id is primary key
    assert!(column_map["id"].2, "id should be primary key");

    // Verify required columns are NOT NULL
    assert!(column_map["benchmark_name"].1, "benchmark_name should be NOT NULL");
    assert!(column_map["prompt"].1, "prompt should be NOT NULL");
    assert!(column_map["content"].1, "content should be NOT NULL");

    Ok(())
}
