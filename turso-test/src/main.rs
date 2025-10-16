//! # Turso Test - Basic Usage Examples
//!
//! This file demonstrates how to use Turso database for basic operations:
//! - Database connection
//! - Table creation
//! - Basic INSERT operations
//! - UPSERT operations with ON CONFLICT
//! - Reading data

use anyhow::Result;
use chrono::Utc;
use turso::Builder;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Turso Basic Usage Examples");
    println!("============================");

    // Create in-memory database for demonstration
    let conn = Builder::new_local(":memory:").build().await?.connect()?;
    println!("✅ Connected to in-memory database");

    // Example 1: Create a table
    println!("\n📝 Example 1: Create table");
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
    println!("✅ Created benchmarks table");

    // Example 2: Basic INSERT
    println!("\n📝 Example 2: Basic INSERT");
    let result = conn.execute(
        "INSERT INTO benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
        [
            "test-001".to_string(),
            "basic-test".to_string(),
            "Test prompt".to_string(),
            "Test content".to_string(),
            Utc::now().to_rfc3339(),
            Utc::now().to_rfc3339(),
        ],
    ).await?;
    println!("✅ Inserted record, result: {result}");

    // Example 3: Read data
    println!("\n📝 Example 3: Read data");
    let mut rows = conn.query("SELECT COUNT(*) FROM benchmarks", ()).await?;
    if let Some(row) = rows.next().await? {
        let count: i64 = row.get(0)?;
        println!("📊 Total records: {count}");
    }

    // Example 4: UPSERT with ON CONFLICT
    println!("\n📝 Example 4: UPSERT with ON CONFLICT");
    let timestamp = Utc::now().to_rfc3339();

    // First insert
    let result1 = conn.execute(
        "INSERT INTO benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             benchmark_name = excluded.benchmark_name,
             prompt = excluded.prompt,
             content = excluded.content,
             updated_at = excluded.updated_at",
        [
            "upsert-001".to_string(),
            "original-name".to_string(),
            "Original prompt".to_string(),
            "Original content".to_string(),
            timestamp.clone(),
            timestamp.clone(),
        ],
    ).await?;
    println!("✅ First UPSERT, result: {result1}");

    // Second insert with same ID (should update)
    let result2 = conn.execute(
        "INSERT INTO benchmarks (id, benchmark_name, prompt, content, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             benchmark_name = excluded.benchmark_name,
             prompt = excluded.prompt,
             content = excluded.content,
             updated_at = excluded.updated_at",
        [
            "upsert-001".to_string(),
            "updated-name".to_string(),
            "Updated prompt".to_string(),
            "Updated content".to_string(),
            timestamp.clone(),
            timestamp.clone(),
        ],
    ).await?;
    println!("✅ Second UPSERT (update), result: {result2}");

    // Example 5: Verify UPSERT worked correctly
    println!("\n📝 Example 5: Verify UPSERT results");
    let mut rows = conn.query("SELECT COUNT(*) FROM benchmarks WHERE id = 'upsert-001'", ()).await?;
    if let Some(row) = rows.next().await? {
        let count: i64 = row.get(0)?;
        if count == 1 {
            println!("✅ UPSERT worked correctly - only 1 record exists");

            // Show the actual record
            let mut rows = conn.query(
                "SELECT id, benchmark_name, prompt FROM benchmarks WHERE id = 'upsert-001'",
                ()
            ).await?;
            if let Some(row) = rows.next().await? {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let prompt: String = row.get(2)?;
                println!("📋 Record: {id} -> {name} | {prompt}");
            }
        } else {
            println!("❌ UPSERT failed - {count} records exist (should be 1)");
        }
    }

    // Example 6: Show all records
    println!("\n📝 Example 6: Show all records");
    let mut rows = conn.query(
        "SELECT id, benchmark_name, SUBSTR(prompt, 1, 30) as prompt_preview FROM benchmarks ORDER BY id",
        ()
    ).await?;
    while let Some(row) = rows.next().await? {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let prompt_preview: String = row.get(2)?;
        println!("   {id} | {name} | {prompt_preview}...");
    }

    println!("\n🎯 Summary:");
    println!("✅ Database connection: Working");
    println!("✅ Table creation: Working");
    println!("✅ Basic INSERT: Working");
    println!("✅ Data reading: Working");
    println!("✅ UPSERT with ON CONFLICT: Working");
    println!("\n💡 For more advanced tests and concurrency examples, see the tests/ directory");

    Ok(())
}
